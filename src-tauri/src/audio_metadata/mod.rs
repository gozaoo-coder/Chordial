//! 音频元数据读取模块
//!
//! 支持多种音频格式的元数据解析，包括 FLAC、MP3、M4A、OGG、WAV 等格式。
//! 提供统一的接口来读取音频文件的各种元数据信息。

pub mod core;
pub mod utils;
pub mod readers;

pub use core::{
    AudioMetadata,
    AudioFormat,
    Picture,
    PictureType,
    LyricLine,
};

pub use utils::encoding::{
    auto_decode_text,
    EncodingError,
};

pub use readers::{
    read_flac_metadata,
    read_mp3_metadata,
    read_m4a_metadata,
    read_ogg_metadata,
    read_wav_metadata,
    read_metadata_by_extension,
    read_metadata_by_content,
    is_supported_extension,
    SUPPORTED_EXTENSIONS,
};

use std::path::{Path, PathBuf};
use std::sync::Arc;
use rayon::prelude::*;

/// 元数据读取错误类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetadataError {
    /// IO 错误
    IoError(String),
    /// 无效的格式
    InvalidFormat(String),
    /// 不支持的格式
    UnsupportedFormat(String),
    /// 解析错误
    ParseError(String),
    /// 文件过大
    FileTooLarge,
    /// 未知错误
    Unknown,
}

impl std::fmt::Display for MetadataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetadataError::IoError(msg) => write!(f, "IO 错误: {}", msg),
            MetadataError::InvalidFormat(msg) => write!(f, "无效的格式: {}", msg),
            MetadataError::UnsupportedFormat(msg) => write!(f, "不支持的格式: {}", msg),
            MetadataError::ParseError(msg) => write!(f, "解析错误: {}", msg),
            MetadataError::FileTooLarge => write!(f, "文件过大"),
            MetadataError::Unknown => write!(f, "未知错误"),
        }
    }
}

impl std::error::Error for MetadataError {}

impl From<std::io::Error> for MetadataError {
    fn from(e: std::io::Error) -> Self {
        MetadataError::IoError(e.to_string())
    }
}

/// 元数据读取器 Trait
///
/// 所有格式特定的读取器都应实现此 Trait
pub trait MetadataReader {
    /// 读取文件路径的元数据
    fn read_metadata(path: &Path) -> Result<AudioMetadata, MetadataError>;

    /// 检查文件是否支持此读取器
    fn supports_file(path: &Path) -> bool {
        Self::format() == core::AudioFormat::from_extension(path)
            || Self::format() == core::AudioFormat::from_magic_bytes(&[])
    }

    /// 获取此读取器支持的格式
    fn format() -> core::AudioFormat;
}

/// 读取单个音频文件的元数据
///
/// 根据文件格式自动选择合适的读取器
pub fn read_metadata(path: &Path) -> Result<AudioMetadata, MetadataError> {
    use core::AudioFormat;

    // 首先检查文件是否存在且可读
    if !path.exists() {
        return Err(MetadataError::IoError("文件不存在".to_string()));
    }

    if !path.is_file() {
        return Err(MetadataError::IoError("不是文件".to_string()));
    }

    // 尝试通过魔数检测格式
    let format = detect_format(path)?;

    match format {
        AudioFormat::Flac => readers::flac::read_flac_metadata(path),
        AudioFormat::Mp3 => readers::mp3::read_mp3_metadata(path),
        AudioFormat::M4a => readers::m4a::read_m4a_metadata(path),
        AudioFormat::Ogg => readers::ogg::read_ogg_metadata(path),
        AudioFormat::Wav => readers::wav::read_wav_metadata(path),
        AudioFormat::Unknown => {
            // 尝试通过扩展名检测
            let ext_format = AudioFormat::from_extension(path);

            match ext_format {
                AudioFormat::Flac => readers::flac::read_flac_metadata(path),
                AudioFormat::Mp3 => readers::mp3::read_mp3_metadata(path),
                AudioFormat::M4a => readers::m4a::read_m4a_metadata(path),
                AudioFormat::Ogg => readers::ogg::read_ogg_metadata(path),
                AudioFormat::Wav => readers::wav::read_wav_metadata(path),
                _ => Err(MetadataError::UnsupportedFormat(
                    format!("不支持的文件格式: {:?}", path.extension())
                )),
            }
        }
    }
}

/// 检测文件格式（通过魔数）
fn detect_format(path: &Path) -> Result<AudioFormat, MetadataError> {
    use std::fs::File;
    use std::io::Read;

    let mut file = File::open(path)?;
    let mut magic = [0u8; MAGIC_BYTES_SIZE];
    file.read_exact(&mut magic)?;
    Ok(AudioFormat::from_magic_bytes(&magic))
}

/// 批量读取多个音频文件的元数据
///
/// 使用并行处理提高效率
pub fn batch_read_metadata(paths: &[PathBuf]) -> Vec<Result<AudioMetadata, MetadataError>> {
    paths
        .par_iter()
        .map(|path| read_metadata(path))
        .collect()
}

/// 批量读取多个音频文件的元数据（带进度回调）
///
/// - `paths`: 文件路径列表
/// - `progress_callback`: 进度回调函数
pub fn batch_read_metadata_with_progress(
    paths: &[PathBuf],
    progress_callback: Arc<dyn Fn(usize, usize) + Send + Sync>,
) -> Vec<Result<AudioMetadata, MetadataError>> {
    let total = paths.len();
    let results: Vec<_> = paths
        .par_iter()
        .enumerate()
        .map(|(index, path)| {
            let result = read_metadata(path);
            progress_callback(index + 1, total);
            result
        })
        .collect();

    results
}

/// 从文件路径列表中过滤支持的文件
pub fn filter_supported_files(paths: &[PathBuf]) -> Vec<PathBuf> {
    paths
        .iter()
        .filter(|p| is_supported_extension(p))
        .cloned()
        .collect()
}

/// 获取文件扩展名对应的 AudioFormat
pub fn get_format_from_path(path: &Path) -> AudioFormat {
    AudioFormat::from_extension(path)
}

/// 最大支持的元数据文件大小（100MB）
const MAX_METADATA_SIZE: u64 = 100 * 1024 * 1024; // 100MB

/// 魔数检测所需的字节数
const MAGIC_BYTES_SIZE: usize = 12;

/// 检查文件是否太大而不适合完整读取
pub fn is_file_too_large(path: &Path) -> Result<bool, MetadataError> {
    let metadata = std::fs::metadata(path)?;
    Ok(metadata.len() > MAX_METADATA_SIZE)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_read_metadata_nonexistent() {
        let path = PathBuf::from("/nonexistent/file.mp3");
        assert!(read_metadata(&path).is_err());
    }

    #[test]
    fn test_filter_supported_files() {
        let paths = vec![
            PathBuf::from("test.mp3"),
            PathBuf::from("test.flac"),
            PathBuf::from("test.txt"),
            PathBuf::from("test.mp4"),
            PathBuf::from("test.doc"),
        ];

        let supported = filter_supported_files(&paths);
        assert_eq!(supported.len(), 3);
    }

    #[test]
    fn test_error_display() {
        let error = MetadataError::IoError("test error".to_string());
        assert!(error.to_string().contains("IO 错误"));

        let error = MetadataError::InvalidFormat("invalid".to_string());
        assert!(error.to_string().contains("无效的格式"));
    }
}
