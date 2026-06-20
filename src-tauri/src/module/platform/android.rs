//! Android 端文件 I/O — 混合 std::fs（内部存储）和 JNI 桥接（ContentResolver）。
//!
//! Android 有两类路径：
//! 1. **内部存储路径**（如 `/data/data/<app>/files/`）— `std::fs` 可直接访问
//! 2. **共享存储 / content URI**（如 `content://media/external/audio/media/123`）—
//!    通过 `android_bridge.rs` → JNI → Kotlin `ContentResolver` 访问
//!
//! # 回退策略
//!
//! 所有函数先尝试 `std::fs`。若路径为 content URI 或 `std::fs` 失败，
//! 则回退到 JNI 桥接层。

use super::{PlatformFile, PlatformPath};

/// 判断路径是否为 content URI（`content://` 开头）。
fn is_content_uri(path: &str) -> bool {
    path.starts_with("content://")
}

/// 读取文件全部字节。
pub fn read_bytes(path: &PlatformPath) -> Result<Vec<u8>, String> {
    if is_content_uri(path) {
        return super::bridge::read_content_uri_bytes(path)
            .map_err(|e| format!("Content URI 读取失败 '{}': {}", path, e));
    }
    std::fs::read(path).or_else(|e| {
        // 回退：尝试 JNI 桥接
        super::bridge::read_content_uri_bytes(path)
            .map_err(|e2| format!("读取文件失败 '{}': {} (JNI 回退: {})", path, e, e2))
    })
}

/// 打开文件，返回 `PlatformFile`（Android 端为 `Cursor<Vec<u8>>`）。
pub fn open_file(path: &PlatformPath) -> Result<PlatformFile, String> {
    let data = read_bytes(path)?;
    Ok(std::io::Cursor::new(data))
}

/// 检查路径是否存在。
pub fn exists(path: &PlatformPath) -> bool {
    if is_content_uri(path) {
        return super::bridge::check_content_uri_exists(path).unwrap_or(false);
    }
    if std::path::Path::new(path).exists() {
        return true;
    }
    // 回退：JNI 检查
    super::bridge::check_content_uri_exists(path).unwrap_or(false)
}

/// 检查路径是否为目录。
pub fn is_dir(path: &PlatformPath) -> bool {
    if is_content_uri(path) {
        return false; // content URI 无法通过此 API 判断类型
    }
    std::path::Path::new(path).is_dir()
}

/// 检查路径是否为文件。
pub fn is_file(path: &PlatformPath) -> bool {
    if is_content_uri(path) {
        return exists(path); // content URI 存在即可视为文件
    }
    std::path::Path::new(path).is_file()
}

/// 列出目录下所有条目的路径（非递归）。
pub fn read_dir_entries(path: &PlatformPath) -> Result<Vec<PlatformPath>, String> {
    if is_content_uri(path) {
        let json = super::bridge::query_audio_files(path)
            .map_err(|e| format!("MediaStore 查询失败: {}", e))?;
        // 解析 JSON 数组，提取 contentUri 字段
        return parse_audio_json(&json);
    }
    let mut entries = Vec::new();
    let dir = std::fs::read_dir(path)
        .map_err(|e| format!("读取目录失败 '{}': {}", path, e))?;
    for entry in dir {
        let entry = entry.map_err(|e| format!("读取目录条目失败: {}", e))?;
        entries.push(entry.path().to_string_lossy().to_string());
    }
    Ok(entries)
}

/// 解析 `queryAudioFiles` 返回的 JSON 数组，提取 content URI。
fn parse_audio_json(json: &str) -> Result<Vec<String>, String> {
    let parsed: serde_json::Value = serde_json::from_str(json)
        .map_err(|e| format!("解析 JSON 失败: {}", e))?;
    let arr = parsed.as_array()
        .ok_or("JSON 不是数组")?;
    let mut uris = Vec::new();
    for item in arr {
        if let Some(uri) = item.get("contentUri").and_then(|v| v.as_str()) {
            uris.push(uri.to_string());
        }
    }
    Ok(uris)
}

/// 获取文件大小（字节）。
pub fn file_size(path: &PlatformPath) -> Result<u64, String> {
    if is_content_uri(path) {
        return super::bridge::read_content_uri_size(path)
            .map_err(|e| format!("Content URI 大小查询失败 '{}': {}", path, e));
    }
    let meta = std::fs::metadata(path)
        .map_err(|e| format!("获取文件元数据失败 '{}': {}", path, e))?;
    Ok(meta.len())
}

/// 规范化路径。
pub fn canonicalize(path: &PlatformPath) -> Result<PlatformPath, String> {
    if is_content_uri(path) {
        return Ok(path.clone()); // Content URI 本身就是规范形式
    }
    let p = std::path::Path::new(path);
    match p.canonicalize() {
        Ok(canon) => Ok(canon.to_string_lossy().to_string()),
        Err(e) => {
            // 回退：尝试作为 content URI 检查存在性
            if super::bridge::check_content_uri_exists(path).unwrap_or(false) {
                Ok(path.clone())
            } else {
                Err(format!("规范化路径失败 '{}': {}", path, e))
            }
        }
    }
}

/// 根据文件路径推断 MIME 类型。
pub fn mime_from_path(path_str: &str) -> &'static str {
    let lower = path_str.to_lowercase();
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
