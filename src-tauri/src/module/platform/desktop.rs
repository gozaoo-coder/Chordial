//! 桌面端文件 I/O — 薄封装 `std::fs`，零开销。
//!
//! 所有函数签名与 `android.rs` 保持一致，
//! 通过 `mod.rs` 中的 `pub use` 统一导出。

use super::{PlatformFile, PlatformPath};

/// 读取文件全部字节。
pub fn read_bytes(path: &PlatformPath) -> Result<Vec<u8>, String> {
    std::fs::read(path).map_err(|e| format!("读取文件失败 '{}': {}", path.display(), e))
}

/// 打开文件，返回 `PlatformFile`（桌面端即 `std::fs::File`）。
pub fn open_file(path: &PlatformPath) -> Result<PlatformFile, String> {
    std::fs::File::open(path).map_err(|e| format!("打开文件失败 '{}': {}", path.display(), e))
}

/// 检查路径是否存在。
pub fn exists(path: &PlatformPath) -> bool {
    path.exists()
}

/// 检查路径是否为目录。
pub fn is_dir(path: &PlatformPath) -> bool {
    path.is_dir()
}

/// 检查路径是否为文件。
pub fn is_file(path: &PlatformPath) -> bool {
    path.is_file()
}

/// 列出目录下所有条目的路径（非递归）。
pub fn read_dir_entries(path: &PlatformPath) -> Result<Vec<PlatformPath>, String> {
    let mut entries = Vec::new();
    let dir = std::fs::read_dir(path)
        .map_err(|e| format!("读取目录失败 '{}': {}", path.display(), e))?;
    for entry in dir {
        let entry = entry.map_err(|e| format!("读取目录条目失败: {}", e))?;
        entries.push(entry.path());
    }
    Ok(entries)
}

/// 获取文件大小（字节）。
pub fn file_size(path: &PlatformPath) -> Result<u64, String> {
    let meta = std::fs::metadata(path)
        .map_err(|e| format!("获取文件元数据失败 '{}': {}", path.display(), e))?;
    Ok(meta.len())
}

/// 规范化路径（解析符号链接、相对路径等）。
pub fn canonicalize(path: &PlatformPath) -> Result<PlatformPath, String> {
    path.canonicalize()
        .map_err(|e| format!("规范化路径失败 '{}': {}", path.display(), e))
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
