//! FLAC格式解析器
//! 
//! FLAC (Free Lossless Audio Codec) 元数据解析
//! FLAC文件结构：fLaC标记 + metadata blocks + audio frames

use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;
use std::time::Duration;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};

use super::super::core::{AudioMetadata, AudioFormat, MetadataReader, MetadataError, Result};
use super::super::core::types::{Picture, PictureType};
use super::super::utils::{read_bytes, auto_decode_text};

/// FLAC文件签名
const FLAC_SIGNATURE: &[u8] = b"fLaC";

/// FLAC metadata block类型
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockType {
    StreamInfo = 0,
    Padding = 1,
    Application = 2,
    SeekTable = 3,
    VorbisComment = 4,
    CueSheet = 5,
    Picture = 6,
    Unknown(u8),
}

impl From<u8> for BlockType {
    fn from(value: u8) -> Self {
        match value & 0x7F {
            0 => BlockType::StreamInfo,
            1 => BlockType::Padding,
            2 => BlockType::Application,
            3 => BlockType::SeekTable,
            4 => BlockType::VorbisComment,
            5 => BlockType::CueSheet,
            6 => BlockType::Picture,
            n => BlockType::Unknown(n),
        }
    }
}

/// FLAC解析器
pub struct FlacParser;

impl MetadataReader for FlacParser {
    fn read_metadata<P: AsRef<Path>>(path: P) -> Result<AudioMetadata> {
        let path = path.as_ref();
        let file = File::open(path)
            .map_err(|e| MetadataError::IoError(e))?;
        
        let mut reader = BufReader::new(file);
        let mut metadata = AudioMetadata::new(AudioFormat::Flac);
        
        // 检查FLAC签名
        let mut signature = [0u8; 4];
        reader.read_exact(&mut signature)
            .map_err(|e| MetadataError::IoError(e))?;
        
        if &signature != FLAC_SIGNATURE {
            return Err(MetadataError::invalid_format("不是有效的FLAC文件"));
        }
        
        // 读取metadata blocks
        loop {
            let block_header = reader.read_u8()
                .map_err(|e| MetadataError::IoError(e))?;
            
            let block_type = BlockType::from(block_header);
            let is_last_block = (block_header & 0x80) != 0;
            
            let block_size = reader.read_u24::<BigEndian>()
                .map_err(|e| MetadataError::IoError(e))? as usize;
            
            // 处理不同类型的metadata block
            match block_type {
                BlockType::StreamInfo => {
                    parse_stream_info(&mut reader, &mut metadata, block_size)?;
                }
                BlockType::VorbisComment => {
                    parse_vorbis_comment(&mut reader, &mut metadata, block_size)?;
                }
                BlockType::Picture => {
                    parse_picture(&mut reader, &mut metadata, block_size)?;
                }
                BlockType::Padding => {
                    // 跳过padding block
                    reader.seek(SeekFrom::Current(block_size as i64))
                        .map_err(|e| MetadataError::IoError(e))?;
                }
                _ => {
                    // 其他类型的block，跳过
                    reader.seek(SeekFrom::Current(block_size as i64))
                        .map_err(|e| MetadataError::IoError(e))?;
                }
            }
            
            if is_last_block {
                break;
            }
        }
        
        Ok(metadata)
    }

    fn can_parse<P: AsRef<Path>>(path: P) -> bool {
        if let Ok(header) = super::super::utils::peek_file_header(path.as_ref(), 4) {
            header == FLAC_SIGNATURE
        } else {
            false
        }
    }

    fn supported_extensions() -> &'static [&'static str] {
        &["flac"]
    }
}

/// 解析StreamInfo block
fn parse_stream_info<R: Read>(reader: &mut R, metadata: &mut AudioMetadata, block_size: usize) -> Result<()> {
    if block_size < 34 {
        return Err(MetadataError::invalid_format("StreamInfo block大小不正确"));
    }

    // 读取音频参数
    let _min_block_size = reader.read_u16::<BigEndian>()
        .map_err(|e| MetadataError::IoError(e))?;
    
    let _max_block_size = reader.read_u16::<BigEndian>()
        .map_err(|e| MetadataError::IoError(e))?;
    
    let _min_frame_size = reader.read_u24::<BigEndian>()
        .map_err(|e| MetadataError::IoError(e))?;
    
    let _max_frame_size = reader.read_u24::<BigEndian>()
        .map_err(|e| MetadataError::IoError(e))?;
    
    // 读取采样率、声道数、采样位数
    let sample_info = reader.read_u32::<BigEndian>()
        .map_err(|e| MetadataError::IoError(e))?;
    
    let sample_rate = (sample_info >> 12) & 0xFFFFF;
    let channels = ((sample_info >> 9) & 0x7) + 1;
    let _bits_per_sample = ((sample_info >> 4) & 0x1F) + 1;
    
    // 读取总采样数
    let total_samples = ((sample_info & 0xF) as u64) << 32;
    let lower_samples = reader.read_u32::<BigEndian>()
        .map_err(|e| MetadataError::IoError(e))? as u64;
    let total_samples = total_samples | lower_samples;
    
    // 计算持续时间
    if sample_rate > 0 {
        let duration_secs = total_samples / (sample_rate as u64);
        metadata.duration = Some(Duration::from_secs(duration_secs));
    }
    
    metadata.sample_rate = Some(sample_rate);
    metadata.channels = Some(channels);
    
    // 计算比特率（估算）
    if let Some(duration) = metadata.duration {
        if duration.as_secs() > 0 {
            let bitrate = ((block_size as u64 * 8) / duration.as_secs()) / 1000;
            metadata.bitrate = Some(bitrate as u32);
        }
    }
    
    // 跳过MD5签名（16字节）
    let mut md5_buffer = [0u8; 16];
    reader.read_exact(&mut md5_buffer)
        .map_err(|e| MetadataError::IoError(e))?;
    
    Ok(())
}

/// 解析Vorbis Comment block
fn parse_vorbis_comment<R: Read>(reader: &mut R, metadata: &mut AudioMetadata, _block_size: usize) -> Result<()> {
    // 读取vendor string长度
    let vendor_length = reader.read_u32::<LittleEndian>()
        .map_err(|e| MetadataError::IoError(e))? as usize;
    
    // 跳过vendor string
    let _vendor_string = read_bytes(reader, vendor_length)?;
    
    // 读取comment数量
    let comment_count = reader.read_u32::<LittleEndian>()
        .map_err(|e| MetadataError::IoError(e))? as usize;
    
    // 读取每个comment
    for _ in 0..comment_count {
        let comment_length = reader.read_u32::<LittleEndian>()
            .map_err(|e| MetadataError::IoError(e))? as usize;
        
        let comment_bytes = read_bytes(reader, comment_length)?;
        let comment = auto_decode_text(&comment_bytes)?;
        
        // 解析comment格式：FIELD=value
        if let Some(eq_pos) = comment.find('=') {
            let field = &comment[..eq_pos].to_uppercase();
            let value = &comment[eq_pos + 1..];
            
            match field.as_str() {
                "TITLE" => metadata.title = Some(value.to_string()),
                "ARTIST" => metadata.artist = Some(value.to_string()),
                "ALBUM" => metadata.album = Some(value.to_string()),
                "ALBUMARTIST" | "ALBUM_ARTIST" => metadata.album_artist = Some(value.to_string()),
                "COMPOSER" => metadata.composer = Some(value.to_string()),
                "DATE" | "YEAR" => {
                    if let Ok(year) = value.parse::<u32>() {
                        metadata.year = Some(year);
                    }
                }
                "TRACKNUMBER" | "TRACK" => {
                    if let Ok(track) = value.parse::<u32>() {
                        metadata.track_number = Some(track);
                    }
                }
                "TRACKTOTAL" | "TRACKSTOTAL" => {
                    if let Ok(total) = value.parse::<u32>() {
                        metadata.total_tracks = Some(total);
                    }
                }
                "DISCNUMBER" | "DISC" => {
                    if let Ok(disc) = value.parse::<u32>() {
                        metadata.disc_number = Some(disc);
                    }
                }
                "DISCTOTAL" | "DISCSTOTAL" => {
                    if let Ok(total) = value.parse::<u32>() {
                        metadata.total_discs = Some(total);
                    }
                }
                "GENRE" => metadata.genre = Some(value.to_string()),
                "COMMENT" | "DESCRIPTION" => metadata.comment = Some(value.to_string()),
                "LYRICS" => metadata.lyrics = Some(value.to_string()),
                _ => {
                    // 保存到extra_tags中
                    metadata.extra_tags.insert(field.to_string(), value.to_string());
                }
            }
        }
    }
    
    Ok(())
}

/// 解析Picture block
fn parse_picture<R: Read>(reader: &mut R, metadata: &mut AudioMetadata, _block_size: usize) -> Result<()> {
    // 读取图片类型
    let picture_type_code = reader.read_u32::<BigEndian>()
        .map_err(|e| MetadataError::IoError(e))?;
    let picture_type = PictureType::from_id3_code(picture_type_code as u8);
    
    // 读取MIME类型长度和内容
    let mime_length = reader.read_u32::<BigEndian>()
        .map_err(|e| MetadataError::IoError(e))? as usize;
    let mime_bytes = read_bytes(reader, mime_length)?;
    let mime_type = auto_decode_text(&mime_bytes)?;
    
    // 读取描述长度和内容
    let desc_length = reader.read_u32::<BigEndian>()
        .map_err(|e| MetadataError::IoError(e))? as usize;
    let desc_bytes = read_bytes(reader, desc_length)?;
    let description = if desc_bytes.is_empty() {
        None
    } else {
        Some(auto_decode_text(&desc_bytes)?)
    };
    
    // 读取图片尺寸
    let width = reader.read_u32::<BigEndian>()
        .map_err(|e| MetadataError::IoError(e))?;
    let height = reader.read_u32::<BigEndian>()
        .map_err(|e| MetadataError::IoError(e))?;
    
    // 读取色深和颜色数
    let _bits_per_pixel = reader.read_u32::<BigEndian>()
        .map_err(|e| MetadataError::IoError(e))?;
    let _colors = reader.read_u32::<BigEndian>()
        .map_err(|e| MetadataError::IoError(e))?;
    
    // 读取图片数据
    let data_length = reader.read_u32::<BigEndian>()
        .map_err(|e| MetadataError::IoError(e))? as usize;
    let data = read_bytes(reader, data_length)?;
    
    // 创建图片对象
    let mut picture = Picture::new(picture_type, mime_type, data);
    picture.description = description;
    picture.width = if width > 0 { Some(width) } else { None };
    picture.height = if height > 0 { Some(height) } else { None };
    
    metadata.pictures.push(picture);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flac_signature() {
        assert_eq!(FLAC_SIGNATURE, b"fLaC");
    }

    #[test]
    fn test_block_type_conversion() {
        assert_eq!(BlockType::from(0), BlockType::StreamInfo);
        assert_eq!(BlockType::from(4), BlockType::VorbisComment);
        assert_eq!(BlockType::from(6), BlockType::Picture);
        assert_eq!(BlockType::from(0x84), BlockType::VorbisComment); // 带last block标记
    }
}