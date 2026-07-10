//! 媒体资源协议核心 — `chordial://` 的逻辑内核。
//!
//! 本模块提供**与传输层无关**的资源响应构建逻辑，输出标准的
//! [`http::Response<Vec<u8>>`]。两种消费者复用同一逻辑：
//!
//! | 消费者 | 传输方式 |
//! |--------|---------|
//! | `chordial-tauri` | Tauri URI scheme protocol（`chordial://`，进程内） |
//! | `chordial-server` | axum HTTP 端点（`GET /media/{type}/{sn}/{eid}`，网络） |
//!
//! # URL 格式
//!
//! ```text
//! /audio/<base64url(source_name)>/<base64url(entity_id)>
//! /image/<base64url(source_name)>/<base64url(entity_id)>
//! /lyric/<base64url(source_name)>/<base64url(entity_id)>
//! ```
//!
//! 所有参数使用 base64url 编码（`+` → `-`, `/` → `_`, 无填充），
//! 避免文件路径中的特殊字符（如 Windows 的 `\`、`:`）破坏 URL 解析。
//!
//! audio 端点完整支持 HTTP Range 请求（`206 Partial Content`），

use crate::module::music_source::registrar::SourceRegistrar;
use crate::module::music_source::resource;
use crate::module::music_source::types::{EntityType, SourceId, SourceType};
use crate::module::perf;
use crate::module::platform::{self, PlatformPath};
use http::{header, Method, Request, Response, StatusCode};
use std::io::{Read, Seek, SeekFrom};

/// base64url 解码。
/// 标准 base64url：`-` → `+`, `_` → `/`，补回缺失的 padding。
fn base64url_decode(input: &str) -> Result<Vec<u8>, String> {
    let mut encoded = input.replace('-', "+").replace('_', "/");
    // 补齐 padding
    let rem = encoded.len() % 4;
    if rem > 0 {
        encoded.push_str(&"=".repeat(4 - rem));
    }
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(&encoded)
        .map_err(|e| format!("base64 解码失败: {}", e))
}

/// 提取路径段：`/<type>/<sn_b64>/<eid_b64>`
pub struct ParsedUrl {
    pub resource_type: String, // "audio" | "image" | "lyric"
    pub source_name: String,
    pub entity_id: String,
}

/// 解析 chordial URL 路径。
pub fn parse_url(path: &str) -> Result<ParsedUrl, String> {
    // 去掉开头的 `/`
    let path = path.trim_start_matches('/');
    let segments: Vec<&str> = path.splitn(3, '/').collect();

    if segments.len() != 3 {
        return Err(format!("无效的 chordial URL 路径 (期望 3 段): {}", path));
    }

    let resource_type = segments[0].to_lowercase();

    let source_name_bytes = base64url_decode(segments[1])?;
    let source_name =
        String::from_utf8(source_name_bytes).map_err(|e| format!("来源名称解码失败: {}", e))?;

    let entity_id_bytes = base64url_decode(segments[2])?;
    let entity_id =
        String::from_utf8(entity_id_bytes).map_err(|e| format!("entity_id 解码失败: {}", e))?;

    Ok(ParsedUrl {
        resource_type,
        source_name,
        entity_id,
    })
}

/// 快速错误响应。
pub fn error_response(status: StatusCode, msg: &str) -> Response<Vec<u8>> {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(msg.as_bytes().to_vec())
        .unwrap()
}

/// 为音频文件提供流式响应，支持 Range 请求。
///
/// 使用 [`platform::open_file`] 和 [`platform::file_size`] 实现跨平台文件访问。
pub fn serve_audio_file(path_str: &str, request: &Request<Vec<u8>>) -> Response<Vec<u8>> {
    let _token = perf::start("media.serve_audio_file");
    let path = PlatformPath::from(path_str);

    // 获取文件大小（先于 open 以减少内存分配）
    let file_size = match platform::file_size(&path) {
        Ok(s) => s,
        Err(e) => return error_response(StatusCode::NOT_FOUND, &e),
    };

    // 打开文件
    let mut file = match platform::open_file(&path) {
        Ok(f) => f,
        Err(e) => return error_response(StatusCode::NOT_FOUND, &format!("文件未找到: {}", e)),
    };

    let mime = platform::mime_from_path(path_str);

    // 处理 HEAD 请求 — 只返回头信息
    if request.method() == Method::HEAD {
        return Response::builder()
            .header(header::CONTENT_TYPE, mime)
            .header(header::CONTENT_LENGTH, file_size.to_string())
            .header(header::ACCEPT_RANGES, "bytes")
            .body(Vec::new())
            .unwrap();
    }

    // 解析 Range 头
    if let Some(range_header) = request.headers().get(header::RANGE) {
        let range_str = match range_header.to_str() {
            Ok(s) => s,
            Err(_) => return error_response(StatusCode::BAD_REQUEST, "无效的 Range 头"),
        };

        // 形如 "bytes=0-1023" 或 "bytes=1024-"
        if let Some(range_value) = range_str.strip_prefix("bytes=") {
            let parts: Vec<&str> = range_value.split('-').collect();
            if parts.len() == 2 {
                let start: u64 = match parts[0].parse() {
                    Ok(s) => s,
                    Err(_) => return error_response(StatusCode::BAD_REQUEST, "无效的 Range 起始"),
                };

                let end: u64 = if parts[1].is_empty() {
                    // "bytes=1024-" → 从 start 到文件末尾
                    file_size.saturating_sub(1)
                } else {
                    match parts[1].parse::<u64>() {
                        Ok(e) => e.min(file_size.saturating_sub(1)),
                        Err(_) => {
                            return error_response(StatusCode::BAD_REQUEST, "无效的 Range 结束")
                        }
                    }
                };

                if start > end || start >= file_size {
                    return Response::builder()
                        .status(StatusCode::RANGE_NOT_SATISFIABLE)
                        .header(header::CONTENT_RANGE, format!("bytes */{}", file_size))
                        .body(Vec::new())
                        .unwrap();
                }

                let length = end - start + 1;

                if let Err(e) = file.seek(SeekFrom::Start(start)) {
                    return error_response(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        &format!("Seek 失败: {}", e),
                    );
                }

                let mut buf = Vec::with_capacity(length as usize);
                if let Err(e) = file.take(length).read_to_end(&mut buf) {
                    return error_response(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        &format!("读取文件失败: {}", e),
                    );
                }

                // 仅在 perf 启用时构建 meta 字符串，避免 release 中无谓分配
                let meta = if perf::enabled() {
                    Some(format!("bytes={}", length))
                } else {
                    None
                };
                perf::end(&_token, meta.as_deref());
                return Response::builder()
                    .status(StatusCode::PARTIAL_CONTENT)
                    .header(header::CONTENT_TYPE, mime)
                    .header(header::CONTENT_LENGTH, length.to_string())
                    .header(
                        header::CONTENT_RANGE,
                        format!("bytes {}-{}/{}", start, end, file_size),
                    )
                    .header(header::ACCEPT_RANGES, "bytes")
                    .body(buf)
                    .unwrap();
            }
        }

        return error_response(StatusCode::BAD_REQUEST, "无法解析 Range 头");
    }

    // 完整的文件响应（无 Range）
    let mut buf = Vec::with_capacity(file_size as usize);
    if let Err(e) = file.read_to_end(&mut buf) {
        return error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("读取文件失败: {}", e),
        );
    }

    // 仅在 perf 启用时构建 meta 字符串
    let meta = if perf::enabled() {
        Some(format!("bytes={}", file_size))
    } else {
        None
    };
    perf::end(&_token, meta.as_deref());
    Response::builder()
        .header(header::CONTENT_TYPE, mime)
        .header(header::CONTENT_LENGTH, file_size.to_string())
        .header(header::ACCEPT_RANGES, "bytes")
        .body(buf)
        .unwrap()
}

/// 处理一个 chordial 媒体请求，返回标准 HTTP 响应。
///
/// 这是与传输层无关的核心入口：
/// - Tauri 端：将返回的 `Response` 交给 `UriSchemeResponder::respond`
/// - server 端：直接作为 axum 响应返回
///
/// `path` 为请求 URL 的路径部分（`/audio/<sn>/<eid>` 等）。
pub fn handle(registrar: &SourceRegistrar, path: &str, request: &Request<Vec<u8>>) -> Response<Vec<u8>> {
    let _scope = perf::scope("media.handle");
    match parse_url(path) {
        Ok(parsed) => match parsed.resource_type.as_str() {
            "audio" => {
                let source_id = SourceId {
                    source_name: parsed.source_name.clone(),
                    source_type: SourceType::Local,
                    entity_type: EntityType::Song,
                    entity_id: parsed.entity_id.clone(),
                };

                match resource::get_song_file_path(registrar, &source_id) {
                    Some(file_path) => serve_audio_file(&file_path, request),
                    None => {
                        // 回退：通过 trait 方法获取完整数据
                        match resource::get_song_file(registrar, &source_id) {
                            Ok(data) => Response::builder()
                                .header(header::CONTENT_TYPE, platform::mime_from_path(&parsed.entity_id))
                                .header(header::CONTENT_LENGTH, data.len().to_string())
                                .body(data)
                                .unwrap(),
                            Err(e) => error_response(StatusCode::NOT_FOUND, &e),
                        }
                    }
                }
            }
            "image" => {
                let source_id = SourceId {
                    source_name: parsed.source_name.clone(),
                    source_type: SourceType::Local,
                    entity_type: EntityType::Album,
                    entity_id: parsed.entity_id.clone(),
                };

                match resource::get_album_picture(registrar, &source_id) {
                    Ok(data) => {
                        let mime = platform::mime_from_path(&parsed.entity_id);
                        Response::builder()
                            .header(header::CONTENT_TYPE, mime)
                            .header(header::CONTENT_LENGTH, data.len().to_string())
                            .body(data)
                            .unwrap()
                    }
                    Err(e) => error_response(StatusCode::NOT_FOUND, &e),
                }
            }
            "lyric" => {
                let source_id = SourceId {
                    source_name: parsed.source_name,
                    source_type: SourceType::Local,
                    entity_type: EntityType::Lyric,
                    entity_id: parsed.entity_id,
                };

                match resource::get_lyric_text(registrar, &source_id) {
                    Ok(text) => Response::builder()
                        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
                        .header(header::CONTENT_LENGTH, text.len().to_string())
                        .body(text.into_bytes())
                        .unwrap(),
                    Err(e) => error_response(StatusCode::NOT_FOUND, &e),
                }
            }
            _ => error_response(
                StatusCode::BAD_REQUEST,
                &format!("未知的资源类型: {}", parsed.resource_type),
            ),
        },
        Err(e) => error_response(StatusCode::BAD_REQUEST, &e),
    }
}
