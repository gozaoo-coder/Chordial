//! FLAC 音频格式读取器
//!
//! 支持 FLAC 文件的 Vorbis Comment 和 Picture 元数据解析

use crate::audio_metadata::{
    core::{AudioFormat, AudioMetadata, Picture, PictureType, LyricLine},
    utils::encoding::auto_decode_text,
};
use crate::audio_metadata::MetadataError;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::time::Duration;

/// FLAC 元数据读取器
pub struct FlacReader;

impl FlacReader {
    /// 从任意实现了 Read + Seek 的读取器中读取 FLAC 元数据
    pub fn read_from<R: Read + Seek>(mut reader: R) -> Result<AudioMetadata, MetadataError> {
        // 检查 FLAC 标记
        let mut marker = [0u8; 4];
        reader.read_exact(&mut marker)
            .map_err(|e| MetadataError::IoError(e.to_string()))?;

        if &marker != b"fLaC" {
            return Err(MetadataError::InvalidFormat("不是有效的 FLAC 文件".to_string()));
        }

        let mut metadata = AudioMetadata::new(AudioFormat::Flac);

        // 读取元数据块
        loop {
            let header = read_block_header(&mut reader)?;
            let last_block = header.last_block;

            match header.block_type {
                FlacBlockType::StreamInfo => {
                    parse_stream_info(&mut reader, &mut metadata, header.size)?;
                }
                FlacBlockType::VorbisComment => {
                    parse_vorbis_comment(&mut reader, &mut metadata, header.size)?;
                }
                FlacBlockType::Picture => {
                    parse_picture(&mut reader, &mut metadata, header.size)?;
                }
                _ => {
                    // 跳过其他块
                    reader.seek(SeekFrom::Current(header.size as i64))
                        .map_err(|e| MetadataError::IoError(e.to_string()))?;
                }
            }

            if last_block {
                break;
            }
        }

        Ok(metadata)
    }
}

/// FLAC 块类型
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum FlacBlockType {
    StreamInfo = 0,
    Padding = 1,
    Application = 2,
    SeekTable = 3,
    VorbisComment = 4,
    CueSheet = 5,
    Picture = 6,
    Invalid = 127,
}

impl FlacBlockType {
    fn from_u8(value: u8) -> Self {
        match value {
            0 => FlacBlockType::StreamInfo,
            1 => FlacBlockType::Padding,
            2 => FlacBlockType::Application,
            3 => FlacBlockType::SeekTable,
            4 => FlacBlockType::VorbisComment,
            5 => FlacBlockType::CueSheet,
            6 => FlacBlockType::Picture,
            127 => FlacBlockType::Invalid,
            _ => FlacBlockType::Invalid,
        }
    }
}

/// FLAC 元数据块头部
#[derive(Debug)]
struct FlacBlockHeader {
    block_type: FlacBlockType,
    last_block: bool,
    size: u32,
}

/// 读取 FLAC 文件元数据
pub fn read_flac_metadata(path: &Path) -> Result<AudioMetadata, MetadataError> {
    let file = File::open(path)
        .map_err(|e| MetadataError::IoError(e.to_string()))?;
    FlacReader::read_from(file)
}

/// 读取块头部
fn read_block_header<R: Read>(reader: &mut R) -> Result<FlacBlockHeader, MetadataError> {
    let mut header = [0u8; 4];
    reader.read_exact(&mut header)
        .map_err(|e| MetadataError::IoError(e.to_string()))?;

    let block_type_byte = header[0] & 0x7F;
    let last_block = (header[0] & 0x80) != 0;
    let size = u32::from_be_bytes([0, header[1], header[2], header[3]]);

    Ok(FlacBlockHeader {
        block_type: FlacBlockType::from_u8(block_type_byte),
        last_block,
        size,
    })
}

/// 解析 STREAMINFO 块
fn parse_stream_info<R: Read>(
    reader: &mut R,
    metadata: &mut AudioMetadata,
    size: u32,
) -> Result<(), MetadataError> {
    if size < 34 {
        return Err(MetadataError::InvalidFormat("STREAMINFO 块太小".to_string()));
    }

    let mut data = vec![0u8; size as usize];
    reader.read_exact(&mut data)
        .map_err(|e| MetadataError::IoError(e.to_string()))?;

    // 最小块大小（2 字节）
    // 最大块大小（2 字节）
    // 最小帧大小（3 字节）
    // 最大帧大小（3 字节）

    // 采样率、声道数、位深度、总采样数（8 字节）
    let sample_info = u64::from_be_bytes([
        data[10], data[11], data[12], data[13],
        data[14], data[15], data[16], data[17],
    ]);

    // 解析采样率（20 位）
    let sample_rate = ((sample_info >> 44) & 0xFFFFF) as u32;
    // 解析声道数（3 位）- 1
    let channels = ((sample_info >> 41) & 0x7) as u8 + 1;
    // 解析位深度（5 位）- 1
    let bits_per_sample = ((sample_info >> 36) & 0x1F) as u8 + 1;
    // 解析总采样数（36 位）
    let total_samples = (sample_info & 0xFFFFFFFFF) as u64;

    // 计算时长
    if sample_rate > 0 && total_samples > 0 {
        let duration_secs = total_samples as f64 / sample_rate as f64;
        metadata.duration = Some(Duration::from_secs_f64(duration_secs));
    }

    metadata.sample_rate = Some(sample_rate);
    metadata.channels = Some(channels);

    // 计算比特率（粗略估计）
    // MD5 签名（16 字节）

    Ok(())
}

/// 解析 Vorbis Comment 块
fn parse_vorbis_comment<R: Read>(
    reader: &mut R,
    metadata: &mut AudioMetadata,
    size: u32,
) -> Result<(), MetadataError> {
    let mut data = vec![0u8; size as usize];
    reader.read_exact(&mut data)
        .map_err(|e| MetadataError::IoError(e.to_string()))?;

    let mut pos = 0;

    // 读取 vendor string 长度（小端）
    let vendor_len = read_le_u32(&data, pos)?;
    pos += 4;

    // 跳过 vendor string
    pos += vendor_len as usize;

    // 读取用户评论数量（小端）
    let comment_count = read_le_u32(&data, pos)?;
    pos += 4;

    // 解析每个评论
    for _ in 0..comment_count {
        if pos + 4 > data.len() {
            break;
        }

        let comment_len = read_le_u32(&data, pos)?;
        pos += 4;

        if pos + comment_len as usize > data.len() {
            break;
        }

        let comment = &data[pos..pos + comment_len as usize];
        pos += comment_len as usize;

        // 解码评论字符串
        if let Ok(comment_str) = auto_decode_text(comment) {
            parse_vorbis_field(metadata, &comment_str);
        }
    }

    Ok(())
}

/// 解析 Vorbis 字段
fn parse_vorbis_field(metadata: &mut AudioMetadata, field: &str) {
    let parts: Vec<&str> = field.splitn(2, '=').collect();
    if parts.len() != 2 {
        return;
    }

    let key = parts[0].to_uppercase();
    let value = parts[1].trim();

    match key.as_str() {
        "TITLE" => metadata.title = Some(value.to_string()),
        "ARTIST" => metadata.artist = Some(value.to_string()),
        "ALBUM" => metadata.album = Some(value.to_string()),
        "ALBUMARTIST" | "ALBUM ARTIST" => metadata.album_artist = Some(value.to_string()),
        "DATE" | "YEAR" => {
            metadata.year = value.chars().take(4).collect::<String>().parse().ok();
        }
        "TRACKNUMBER" => {
            let parts: Vec<&str> = value.split('/').collect();
            metadata.track_number = parts[0].trim().parse().ok();
            if parts.len() > 1 {
                metadata.total_tracks = parts[1].trim().parse().ok();
            }
        }
        "DISCNUMBER" => {
            let parts: Vec<&str> = value.split('/').collect();
            metadata.disc_number = parts[0].trim().parse().ok();
            if parts.len() > 1 {
                metadata.total_discs = parts[1].trim().parse().ok();
            }
        }
        "GENRE" => metadata.genre = Some(value.to_string()),
        "COMPOSER" => metadata.composer = Some(value.to_string()),
        "LYRICIST" => metadata.lyricist = Some(value.to_string()),
        "COMMENT" | "DESCRIPTION" => metadata.comment = Some(value.to_string()),
        "LYRICS" => metadata.lyrics = Some(value.to_string()),
        _ => {}
    }
}

/// 解析 Picture 块
fn parse_picture<R: Read>(
    reader: &mut R,
    metadata: &mut AudioMetadata,
    size: u32,
) -> Result<(), MetadataError> {
    let mut data = vec![0u8; size as usize];
    reader.read_exact(&mut data)
        .map_err(|e| MetadataError::IoError(e.to_string()))?;

    let mut pos = 0;

    // 图片类型（4 字节，大端）
    if pos + 4 > data.len() {
        return Ok(());
    }
    let picture_type = u32::from_be_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]);
    pos += 4;

    // MIME 类型长度（4 字节，大端）
    let mime_len = read_be_u32(&data, pos)?;
    pos += 4;

    // MIME 类型
    if pos + mime_len as usize > data.len() {
        return Ok(());
    }
    let mime_type = String::from_utf8_lossy(&data[pos..pos + mime_len as usize]).to_string();
    pos += mime_len as usize;

    // 描述长度（4 字节，大端）
    let desc_len = read_be_u32(&data, pos)?;
    pos += 4;

    // 描述
    if pos + desc_len as usize > data.len() {
        return Ok(());
    }
    let description = String::from_utf8_lossy(&data[pos..pos + desc_len as usize]).to_string();
    pos += desc_len as usize;

    // 宽度（4 字节，大端）
    let width = read_be_u32(&data, pos)?;
    pos += 4;

    // 高度（4 字节，大端）
    let height = read_be_u32(&data, pos)?;
    pos += 4;

    // 颜色深度（4 字节，大端）
    let color_depth = read_be_u32(&data, pos)?;
    pos += 4;

    // 索引颜色数（4 字节，大端）
    let indexed_colors = read_be_u32(&data, pos)?;
    pos += 4;

    // 图片数据长度（4 字节，大端）
    let data_len = read_be_u32(&data, pos)?;
    pos += 4;

    // 图片数据
    if pos + data_len as usize > data.len() {
        return Ok(());
    }
    let picture_data = data[pos..pos + data_len as usize].to_vec();

    let mut picture = Picture::new(
        PictureType::from_id3v2_type(picture_type as u8),
        mime_type,
        description,
        picture_data,
    );
    picture.width = Some(width);
    picture.height = Some(height);
    picture.color_depth = Some(color_depth);
    picture.indexed_colors = Some(indexed_colors);

    metadata.add_picture(picture);

    Ok(())
}

/// 读取小端 u32
fn read_le_u32(data: &[u8], pos: usize) -> Result<u32, MetadataError> {
    if pos + 4 > data.len() {
        return Err(MetadataError::InvalidFormat("数据不足".to_string()));
    }
    Ok(u32::from_le_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]))
}

/// 读取大端 u32
fn read_be_u32(data: &[u8], pos: usize) -> Result<u32, MetadataError> {
    if pos + 4 > data.len() {
        return Err(MetadataError::InvalidFormat("数据不足".to_string()));
    }
    Ok(u32::from_be_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_vorbis_field() {
        let mut metadata = AudioMetadata::new(AudioFormat::Flac);

        parse_vorbis_field(&mut metadata, "TITLE=Test Song");
        assert_eq!(metadata.title, Some("Test Song".to_string()));

        parse_vorbis_field(&mut metadata, "ARTIST=Test Artist");
        assert_eq!(metadata.artist, Some("Test Artist".to_string()));

        parse_vorbis_field(&mut metadata, "TRACKNUMBER=3/10");
        assert_eq!(metadata.track_number, Some(3));
        assert_eq!(metadata.total_tracks, Some(10));
    }
}
