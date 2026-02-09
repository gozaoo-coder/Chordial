//! 音频格式读取器模块
//!
//! 提供各种音频格式的元数据读取功能

pub mod flac;
pub mod mp3;
pub mod m4a;
pub mod ogg;
pub mod wav;

pub use flac::read_flac_metadata;
pub use mp3::read_mp3_metadata;
pub use m4a::read_m4a_metadata;
pub use ogg::read_ogg_metadata;
pub use wav::read_wav_metadata;

use crate::audio_metadata::{
    core::{AudioFormat, AudioMetadata},
    MetadataError,
};
use std::path::Path;

/// 根据文件扩展名自动选择读取器
pub fn read_metadata_by_extension(path: &Path) -> Result<AudioMetadata, MetadataError> {
    let format = AudioFormat::from_extension(path);

    match format {
        AudioFormat::Flac => read_flac_metadata(path),
        AudioFormat::Mp3 => read_mp3_metadata(path),
        AudioFormat::M4a => read_m4a_metadata(path),
        AudioFormat::Ogg => read_ogg_metadata(path),
        AudioFormat::Wav => read_wav_metadata(path),
        AudioFormat::Unknown => Err(MetadataError::UnsupportedFormat(
            format!("不支持的文件格式: {:?}", path.extension())
        )),
    }
}

/// 根据文件内容（魔数）检测格式并读取元数据
pub fn read_metadata_by_content(path: &Path) -> Result<AudioMetadata, MetadataError> {
    use std::fs::File;
    use std::io::Read;

    let mut file = File::open(path)
        .map_err(|e| MetadataError::IoError(e.to_string()))?;

    // 读取前 12 字节用于检测格式
    let mut magic = [0u8; 12];
    file.read_exact(&mut magic)
        .map_err(|e| MetadataError::IoError(e.to_string()))?;

    let format = AudioFormat::from_magic_bytes(&magic);

    match format {
        AudioFormat::Flac => read_flac_metadata(path),
        AudioFormat::Mp3 => read_mp3_metadata(path),
        AudioFormat::M4a => read_m4a_metadata(path),
        AudioFormat::Ogg => read_ogg_metadata(path),
        AudioFormat::Wav => read_wav_metadata(path),
        AudioFormat::Unknown => {
            // 如果魔数检测失败，尝试使用扩展名
            read_metadata_by_extension(path)
        }
    }
}

/// 支持的音频格式扩展名列表
pub const SUPPORTED_EXTENSIONS: &[&str] = &[
    "flac", "mp3", "m4a", "mp4", "aac", "ogg", "oga", "wav", "wave",
];

/// 检查文件扩展名是否受支持
pub fn is_supported_extension(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext = ext.to_string_lossy().to_lowercase();
        SUPPORTED_EXTENSIONS.contains(&ext.as_str())
    } else {
        false
    }
}

/// 获取音频格式的 MIME 类型
pub fn get_mime_type(format: AudioFormat) -> &'static str {
    match format {
        AudioFormat::Flac => "audio/flac",
        AudioFormat::Mp3 => "audio/mpeg",
        AudioFormat::M4a => "audio/mp4",
        AudioFormat::Ogg => "audio/ogg",
        AudioFormat::Wav => "audio/wav",
        AudioFormat::Unknown => "application/octet-stream",
    }
}

/// 获取音频格式的文件扩展名
pub fn get_file_extension(format: AudioFormat) -> &'static str {
    match format {
        AudioFormat::Flac => "flac",
        AudioFormat::Mp3 => "mp3",
        AudioFormat::M4a => "m4a",
        AudioFormat::Ogg => "ogg",
        AudioFormat::Wav => "wav",
        AudioFormat::Unknown => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_is_supported_extension() {
        assert!(is_supported_extension(Path::new("test.mp3")));
        assert!(is_supported_extension(Path::new("test.flac")));
        assert!(is_supported_extension(Path::new("test.m4a")));
        assert!(!is_supported_extension(Path::new("test.txt")));
        assert!(!is_supported_extension(Path::new("test")));
    }

    #[test]
    fn test_get_mime_type() {
        assert_eq!(get_mime_type(AudioFormat::Mp3), "audio/mpeg");
        assert_eq!(get_mime_type(AudioFormat::Flac), "audio/flac");
        assert_eq!(get_mime_type(AudioFormat::Unknown), "application/octet-stream");
    }

    #[test]
    fn test_get_file_extension() {
        assert_eq!(get_file_extension(AudioFormat::Mp3), "mp3");
        assert_eq!(get_file_extension(AudioFormat::Flac), "flac");
        assert_eq!(get_file_extension(AudioFormat::Unknown), "");
    }

    #[test]
    fn test_supported_extensions() {
        assert!(SUPPORTED_EXTENSIONS.contains(&"mp3"));
        assert!(SUPPORTED_EXTENSIONS.contains(&"flac"));
        assert!(SUPPORTED_EXTENSIONS.contains(&"m4a"));
        assert!(SUPPORTED_EXTENSIONS.contains(&"ogg"));
        assert!(SUPPORTED_EXTENSIONS.contains(&"wav"));
    }
}
