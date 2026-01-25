//! 音乐元数据读取器模块
//! 
//! 支持多种音频格式的元数据解析，包括 FLAC、MP3、M4A、OGG、WAV 等格式。

pub mod core;
pub mod parsers;
pub mod utils;

#[cfg(test)]
mod tests;

pub use core::{
    AudioMetadata,
    AudioFormat,
    LyricLine,
    MetadataReader,
    MetadataError,
    Result,
};

use std::path::Path;

/// 统一的元数据读取函数
/// 
/// 根据文件扩展名自动选择合适的解析器
pub fn read_metadata<P: AsRef<Path>>(path: P) -> Result<AudioMetadata> {
    let path = path.as_ref();
    let extension = path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase());

    match extension.as_deref() {
        Some("flac") => parsers::flac::FlacParser::read_metadata(path),
        Some("mp3") | Some("mp2") | Some("mpeg") => parsers::mp3::Mp3Parser::read_metadata(path),
        Some("m4a") | Some("mp4") | Some("aac") => parsers::m4a::M4aParser::read_metadata(path),
        Some("ogg") | Some("oga") => parsers::ogg::OggParser::read_metadata(path),
        Some("wav") | Some("wave") => parsers::wav::WavParser::read_metadata(path),
        _ => Err(MetadataError::UnsupportedFormat(
            extension.as_deref().unwrap_or("unknown").to_string()
        )),
    }
}

/// 批量读取元数据
pub fn batch_read_metadata<P: AsRef<Path>>(paths: &[P]) -> Vec<Result<AudioMetadata>> {
    paths.iter().map(|path| read_metadata(path)).collect()
}