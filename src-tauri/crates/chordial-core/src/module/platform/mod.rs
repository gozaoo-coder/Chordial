//! 平台文件访问抽象层 — 为 Windows/Android 提供统一的文件 I/O 接口。
//!
//! # 设计
//!
//! 通过 `#[cfg]` 条件编译选择底层实现：
//! - **桌面端**（Windows/macOS/Linux）：薄封装 `std::fs`，零开销。
//! - **Android**：通过 Tauri mobile plugin 桥接 Kotlin 层的
//!   `MediaStore` / `ContentResolver` / SAF API。
//!
//! # PlatformPath
//!
//! - 桌面端：`std::path::PathBuf`（完整文件路径）
//! - Android：`String`（可以是文件路径或 `content://` URI）
//!
//! # PlatformFile
//!
//! 实现 `std::io::Read + std::io::Seek`，可直接传入 symphonia 等音频库：
//! - 桌面端：包装 `std::fs::File`
//! - Android：包装 `std::io::Cursor<Vec<u8>>`（预读全部字节到内存）

#[cfg(not(target_os = "android"))]
mod desktop;
#[cfg(not(target_os = "android"))]
pub use desktop::*;

#[cfg(target_os = "android")]
mod android;
#[cfg(target_os = "android")]
pub use android::*;

#[cfg(target_os = "android")]
pub mod android_bridge;
#[cfg(target_os = "android")]
pub use android_bridge as bridge;

// ══════════════════════════════════════════════════════════════════════════════
// 平台路径类型
// ══════════════════════════════════════════════════════════════════════════════

/// 平台路径类型。
///
/// - 桌面端：`std::path::PathBuf`
/// - Android：`String`（文件路径或 `content://` URI）
#[cfg(not(target_os = "android"))]
pub type PlatformPath = std::path::PathBuf;

#[cfg(target_os = "android")]
pub type PlatformPath = String;

// ══════════════════════════════════════════════════════════════════════════════
// 平台文件句柄（Read + Seek）
// ══════════════════════════════════════════════════════════════════════════════

/// 跨平台文件句柄，实现 `Read + Seek`。
///
/// 可直接传入 symphonia 的 `MediaSourceStream`。
#[cfg(not(target_os = "android"))]
pub type PlatformFile = std::fs::File;

#[cfg(target_os = "android")]
pub type PlatformFile = std::io::Cursor<Vec<u8>>;

// ══════════════════════════════════════════════════════════════════════════════
// 辅助函数（平台无关的路径操作）
// ══════════════════════════════════════════════════════════════════════════════

/// 将 `PlatformPath` 转为可显示的字符串。
#[cfg(not(target_os = "android"))]
pub fn path_to_string(path: &PlatformPath) -> String {
    path.to_string_lossy().to_string()
}

#[cfg(target_os = "android")]
pub fn path_to_string(path: &PlatformPath) -> String {
    path.clone()
}

/// 获取路径的文件名（不含扩展名）。
#[cfg(not(target_os = "android"))]
pub fn path_file_stem(path: &PlatformPath) -> Option<String> {
    path.file_stem().and_then(|s| s.to_str()).map(|s| s.to_string())
}

#[cfg(target_os = "android")]
pub fn path_file_stem(path: &PlatformPath) -> Option<String> {
    // 从路径或 URI 中提取文件名
    let name = path.rsplit('/').next().unwrap_or(path);
    let stem = name.rsplit('.').next().unwrap_or(name);
    if stem.is_empty() { None } else { Some(stem.to_string()) }
}

/// 获取路径的扩展名。
#[cfg(not(target_os = "android"))]
pub fn path_extension(path: &PlatformPath) -> Option<String> {
    path.extension().and_then(|s| s.to_str()).map(|s| s.to_lowercase())
}

#[cfg(target_os = "android")]
pub fn path_extension(path: &PlatformPath) -> Option<String> {
    let name = path.rsplit('/').next().unwrap_or(path);
    let ext = name.rsplit('.').next().unwrap_or("");
    if ext == name || ext.is_empty() { None } else { Some(ext.to_lowercase()) }
}

/// 用新扩展名替换路径的扩展名。
#[cfg(not(target_os = "android"))]
pub fn path_with_extension(path: &PlatformPath, ext: &str) -> PlatformPath {
    path.with_extension(ext)
}

#[cfg(target_os = "android")]
pub fn path_with_extension(path: &PlatformPath, ext: &str) -> PlatformPath {
    // 简单字符串操作：去掉旧扩展名 + 新扩展名
    if let Some(dot_pos) = path.rfind('.') {
        let slash_pos = path.rfind('/').unwrap_or(0);
        if dot_pos > slash_pos {
            format!("{}.{}", &path[..dot_pos], ext)
        } else {
            format!("{}.{}", path, ext)
        }
    } else {
        format!("{}.{}", path, ext)
    }
}

/// 获取路径的父目录。
#[cfg(not(target_os = "android"))]
pub fn path_parent(path: &PlatformPath) -> Option<PlatformPath> {
    path.parent().map(|p| p.to_path_buf())
}

#[cfg(target_os = "android")]
pub fn path_parent(path: &PlatformPath) -> Option<PlatformPath> {
    path.rfind('/').map(|pos| path[..pos].to_string())
}

/// 拼接路径。
#[cfg(not(target_os = "android"))]
pub fn path_join(base: &PlatformPath, child: &str) -> PlatformPath {
    base.join(child)
}

#[cfg(target_os = "android")]
pub fn path_join(base: &PlatformPath, child: &str) -> PlatformPath {
    if base.ends_with('/') {
        format!("{}{}", base, child)
    } else {
        format!("{}/{}", base, child)
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Note: 所有平台 I/O 函数（read_bytes, open_file, exists, is_dir, read_dir,
// file_size, mime_from_path）由 desktop.rs / android.rs 定义，
// 并通过文件顶部的 `pub use desktop::*` / `pub use android::*` 重新导出。
// ══════════════════════════════════════════════════════════════════════════════
