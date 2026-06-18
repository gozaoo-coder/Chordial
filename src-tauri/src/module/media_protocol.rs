//! 自定义 URI scheme 协议 — `chordial://` 流式传输。
//!
//! 注册 `chordial` 协议后，前端可直接将 `<audio>` / `<img>` 的 `src`
//! 设置为 `chordial://` URL，浏览器原生处理流式传输和 Range 请求，
//! 完全绕过 Tauri IPC 序列化开销。
//!
//! # URL 格式
//!
//! ```text
//! chordial://localhost/audio/<base64url(source_name)>/<base64url(entity_id)>
//! chordial://localhost/image/<base64url(source_name)>/<base64url(entity_id)>
//! chordial://localhost/lyric/<base64url(source_name)>/<base64url(entity_id)>
//! ```
//!
//! 所有参数使用 base64url 编码（`+` → `-`, `/` → `_`, 无填充），
//! 避免文件路径中的特殊字符（如 Windows 的 `\`、`:`）破坏 URL 解析。

use crate::module::commands;
use crate::module::music_source::types::SourceId;
use http::{header, Method, Request, Response, StatusCode};
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use tauri::UriSchemeResponder;

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
struct ParsedUrl {
    resource_type: String,   // "audio" | "image" | "lyric"
    source_name: String,
    entity_id: String,
}

fn parse_url(path: &str) -> Result<ParsedUrl, String> {
    // 去掉开头的 `/`
    let path = path.trim_start_matches('/');
    let segments: Vec<&str> = path.splitn(3, '/').collect();

    if segments.len() != 3 {
        return Err(format!(
            "无效的 chordial URL 路径 (期望 3 段): {}",
            path
        ));
    }

    let resource_type = segments[0].to_lowercase();

    let source_name_bytes = base64url_decode(segments[1])?;
    let source_name = String::from_utf8(source_name_bytes)
        .map_err(|e| format!("来源名称解码失败: {}", e))?;

    let entity_id_bytes = base64url_decode(segments[2])?;
    let entity_id = String::from_utf8(entity_id_bytes)
        .map_err(|e| format!("entity_id 解码失败: {}", e))?;

    Ok(ParsedUrl {
        resource_type,
        source_name,
        entity_id,
    })
}

/// 根据文件扩展名推断 MIME 类型
fn mime_from_path(path: &str) -> &'static str {
    let lower = path.to_lowercase();
    // 音频
    if lower.ends_with(".mp3") {
        "audio/mpeg"
    } else if lower.ends_with(".flac") {
        "audio/flac"
    } else if lower.ends_with(".wav") || lower.ends_with(".wave") {
        "audio/wav"
    } else if lower.ends_with(".ogg") || lower.ends_with(".oga") {
        "audio/ogg"
    } else if lower.ends_with(".m4a") || lower.ends_with(".aac") {
        "audio/mp4"
    } else if lower.ends_with(".wma") {
        "audio/x-ms-wma"
    } else if lower.ends_with(".opus") {
        "audio/opus"
    }
    // 图片
    else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        "image/jpeg"
    } else if lower.ends_with(".png") {
        "image/png"
    } else if lower.ends_with(".webp") {
        "image/webp"
    } else if lower.ends_with(".bmp") {
        "image/bmp"
    } else if lower.ends_with(".gif") {
        "image/gif"
    }
    // 歌词
    else if lower.ends_with(".lrc") || lower.ends_with(".txt") {
        "text/plain; charset=utf-8"
    } else {
        "application/octet-stream"
    }
}

/// 快速错误响应
fn error_response(status: StatusCode, msg: &str) -> Response<Vec<u8>> {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(msg.as_bytes().to_vec())
        .unwrap()
}

/// 为音频文件提供流式响应，支持 Range 请求。
fn serve_audio_file(path: &str, request: &Request<Vec<u8>>) -> Response<Vec<u8>> {
    let path_buf = PathBuf::from(path);

    // 打开文件获取元数据
    let mut file = match fs::File::open(&path_buf) {
        Ok(f) => f,
        Err(e) => return error_response(StatusCode::NOT_FOUND, &format!("文件未找到: {}", e)),
    };

    let file_size = match file.metadata() {
        Ok(m) => m.len(),
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("读取文件元数据失败: {}", e)),
    };

    let mime = mime_from_path(path);

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
                        Err(_) => return error_response(StatusCode::BAD_REQUEST, "无效的 Range 结束"),
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
                    return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Seek 失败: {}", e));
                }

                let mut buf = vec![0u8; length as usize];
                if let Err(e) = file.read_exact(&mut buf) {
                    return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("读取文件失败: {}", e));
                }

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
        return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("读取文件失败: {}", e));
    }

    Response::builder()
        .header(header::CONTENT_TYPE, mime)
        .header(header::CONTENT_LENGTH, file_size.to_string())
        .header(header::ACCEPT_RANGES, "bytes")
        .body(buf)
        .unwrap()
}

/// 处理 chordial 协议请求。
///
/// 此函数被 Tauri 的 URI scheme 协议处理器调用，
/// 在独立线程中执行，可以安全地进行阻塞 I/O。
pub fn handle_protocol(request: Request<Vec<u8>>, responder: UriSchemeResponder) {
    let path = request.uri().path().to_string();

    let response = match parse_url(&path) {
        Ok(parsed) => {
            let registrar = commands::source_registrar();

            match parsed.resource_type.as_str() {
                "audio" => {
                    // 获取文件路径，流式传输
                    let source_id = SourceId {
                        source_name: parsed.source_name.clone(),
                        source_type: crate::module::music_source::types::SourceType::Local,
                        entity_type: crate::module::music_source::types::EntityType::Song,
                        entity_id: parsed.entity_id.clone(),
                    };

                    match crate::module::music_source::resource::get_song_file_path(
                        registrar,
                        &source_id,
                    ) {
                        Some(file_path) => serve_audio_file(&file_path, &request),
                        None => {
                            // 回退：通过 trait 方法获取完整数据
                            match crate::module::music_source::resource::get_song_file(
                                registrar,
                                &source_id,
                            ) {
                                Ok(data) => Response::builder()
                                    .header(
                                        header::CONTENT_TYPE,
                                        mime_from_path(&parsed.entity_id),
                                    )
                                    .header(header::CONTENT_LENGTH, data.len().to_string())
                                    .body(data)
                                    .unwrap(),
                                Err(e) => error_response(StatusCode::NOT_FOUND, &e),
                            }
                        }
                    }
                }
                "image" => {
                    // 图片：通过 trait 方法获取
                    let source_id = SourceId {
                        source_name: parsed.source_name.clone(),
                        source_type: crate::module::music_source::types::SourceType::Local,
                        entity_type: crate::module::music_source::types::EntityType::Album,
                        entity_id: parsed.entity_id.clone(),
                    };

                    match crate::module::music_source::resource::get_album_picture(
                        registrar,
                        &source_id,
                    ) {
                        Ok(data) => {
                            let mime = mime_from_path(&parsed.entity_id);
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
                    // 歌词：通过 trait 方法获取
                    let source_id = SourceId {
                        source_name: parsed.source_name,
                        source_type: crate::module::music_source::types::SourceType::Local,
                        entity_type: crate::module::music_source::types::EntityType::Lyric,
                        entity_id: parsed.entity_id,
                    };

                    match crate::module::music_source::resource::get_lyric_text(
                        registrar,
                        &source_id,
                    ) {
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
            }
        }
        Err(e) => error_response(StatusCode::BAD_REQUEST, &e),
    };

    responder.respond(response);
}
