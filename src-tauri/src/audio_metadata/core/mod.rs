//! 核心模块 - 定义通用接口和数据结构

pub mod error;
pub mod traits;
pub mod types;

pub use error::{MetadataError, Result};
pub use traits::MetadataReader;
pub use types::{AudioMetadata, LyricLine, Picture, PictureType};

/// 音频格式枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFormat {
    Flac,
    Mp3,
    M4a,
    Ogg,
    Wav,
    Unknown,
}

impl AudioFormat {
    /// 从文件扩展名识别格式
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "flac" => AudioFormat::Flac,
            "mp3" | "mp2" | "mpeg" => AudioFormat::Mp3,
            "m4a" | "mp4" | "aac" => AudioFormat::M4a,
            "ogg" | "oga" => AudioFormat::Ogg,
            "wav" | "wave" => AudioFormat::Wav,
            _ => AudioFormat::Unknown,
        }
    }

    /// 获取格式的MIME类型
    pub fn mime_type(&self) -> &'static str {
        match self {
            AudioFormat::Flac => "audio/flac",
            AudioFormat::Mp3 => "audio/mpeg",
            AudioFormat::M4a => "audio/mp4",
            AudioFormat::Ogg => "audio/ogg",
            AudioFormat::Wav => "audio/wav",
            AudioFormat::Unknown => "application/octet-stream",
        }
    }
}

impl Default for AudioFormat {
    fn default() -> Self {
        AudioFormat::Unknown
    }
}

impl std::fmt::Display for AudioFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioFormat::Flac => write!(f, "FLAC"),
            AudioFormat::Mp3 => write!(f, "MP3"),
            AudioFormat::M4a => write!(f, "M4A"),
            AudioFormat::Ogg => write!(f, "OGG"),
            AudioFormat::Wav => write!(f, "WAV"),
            AudioFormat::Unknown => write!(f, "Unknown"),
        }
    }
}