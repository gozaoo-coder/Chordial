//! M4A/MP4格式解析器
//! 
//! 支持MP4容器的atom结构和iTunes/QuickTime元数据解析

use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

use super::super::core::{AudioMetadata, AudioFormat, MetadataReader, MetadataError, Result};
use super::super::core::types::{Picture, PictureType};

/// MP4文件签名
const MP4_SIGNATURE: &[u8] = b"ftyp";

/// M4A解析器
pub struct M4aParser;

impl MetadataReader for M4aParser {
    fn read_metadata<P: AsRef<Path>>(path: P) -> Result<AudioMetadata> {
        let path = path.as_ref();
        let file = File::open(path)
            .map_err(|e| MetadataError::IoError(e))?;
        
        let mut reader = BufReader::new(file);
        let mut metadata = AudioMetadata::new(AudioFormat::M4a);
        
        // 解析MP4 boxes
        parse_mp4_boxes(&mut reader, &mut metadata, u64::MAX)?;
        
        Ok(metadata)
    }

    fn can_parse<P: AsRef<Path>>(path: P) -> bool {
        if let Ok(header) = super::super::utils::peek_file_header(path.as_ref(), 12) {
            // 检查ftyp box
            &header[4..8] == MP4_SIGNATURE
        } else {
            false
        }
    }

    fn supported_extensions() -> &'static [&'static str] {
        &["m4a", "mp4", "aac", "m4b", "m4p"]
    }
}

/// MP4 box类型
#[derive(Debug, Clone)]
struct BoxInfo {
    box_type: [u8; 4],
    size: u64,
}

/// 解析MP4 boxes
fn parse_mp4_boxes<R: Read + Seek>(reader: &mut R, metadata: &mut AudioMetadata, max_offset: u64) -> Result<()> {
    let mut offset = 0u64;
    
    loop {
        // 检查是否到达限制
        if offset >= max_offset {
            break;
        }
        
        // 读取box头
        let box_info = match read_box_header(reader) {
            Ok(Some(info)) => info,
            Ok(None) => break,
            Err(e) => return Err(e),
        };
        
        let next_offset = offset + box_info.size;
        
        // 解析特定类型的box
        match &box_info.box_type {
            b"ftyp" => {
                // 文件类型，跳过
            }
            b"moov" => {
                // 电影盒，包含主要元数据
                parse_moov_box(reader, metadata, next_offset)?;
            }
            b"udta" => {
                // 用户数据
                parse_udta_box(reader, metadata, next_offset)?;
            }
            b"meta" => {
                // 元数据
                parse_meta_box(reader, metadata, next_offset)?;
            }
            b"ilst" => {
                // 项目列表（iTunes元数据）
                parse_ilst_box(reader, metadata, next_offset - offset - 8)?;
            }
            b"covr" => {
                // 封面图片
                parse_covr_box(reader, metadata, next_offset - offset - 8)?;
            }
            b"free" | b"skip" | b"wide" => {
                // 跳过空闲空间
            }
            _ => {}
        }
        
        // 跳到下一个box
        if next_offset > offset + 8 {
            reader.seek(SeekFrom::Start(offset + box_info.size))
                .map_err(|e| MetadataError::IoError(e))?;
        }
        
        offset = next_offset;
    }
    
    Ok(())
}

/// 读取box头
fn read_box_header<R: Read>(reader: &mut R) -> Result<Option<BoxInfo>> {
    let mut size_bytes = [0u8; 4];
    match reader.read_exact(&mut size_bytes) {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(MetadataError::IoError(e)),
    }
    
    let mut box_type = [0u8; 4];
    reader.read_exact(&mut box_type)
        .map_err(|e| MetadataError::IoError(e))?;
    
    let mut size = u32::from_be_bytes(size_bytes) as u64;
    
    // 大小为0表示到文件末尾
    if size == 0 {
        return Ok(Some(BoxInfo { box_type, size: 0 }));
    }
    
    // 大小为1表示使用64位扩展大小
    if size == 1 {
        let mut ext_size = [0u8; 8];
        reader.read_exact(&mut ext_size)
            .map_err(|e| MetadataError::IoError(e))?;
        size = u64::from_be_bytes(ext_size);
    }
    
    Ok(Some(BoxInfo { box_type, size }))
}

/// 解析moov box
fn parse_moov_box<R: Read + Seek>(reader: &mut R, metadata: &mut AudioMetadata, end_offset: u64) -> Result<()> {
    let _current_offset = reader.seek(SeekFrom::Current(0))
        .map_err(|e| MetadataError::IoError(e))?;
    
    parse_mp4_boxes(reader, metadata, end_offset)?;
    
    reader.seek(SeekFrom::Start(end_offset))
        .map_err(|e| MetadataError::IoError(e))?;
    
    Ok(())
}

/// 解析udta box
fn parse_udta_box<R: Read + Seek>(reader: &mut R, metadata: &mut AudioMetadata, end_offset: u64) -> Result<()> {
    let _current_offset = reader.seek(SeekFrom::Current(0))
        .map_err(|e| MetadataError::IoError(e))?;
    
    parse_mp4_boxes(reader, metadata, end_offset)?;
    
    reader.seek(SeekFrom::Start(end_offset))
        .map_err(|e| MetadataError::IoError(e))?;
    
    Ok(())
}

/// 解析meta box
fn parse_meta_box<R: Read + Seek>(reader: &mut R, metadata: &mut AudioMetadata, end_offset: u64) -> Result<()> {
    // meta box后有4字节版本/标志，跳过
    reader.seek(SeekFrom::Current(4))
        .map_err(|e| MetadataError::IoError(e))?;
    
    let _current_offset = reader.seek(SeekFrom::Current(0))
        .map_err(|e| MetadataError::IoError(e))?;
    
    parse_mp4_boxes(reader, metadata, end_offset)?;
    
    reader.seek(SeekFrom::Start(end_offset))
        .map_err(|e| MetadataError::IoError(e))?;
    
    Ok(())
}

/// 解析ilst box（iTunes元数据）
fn parse_ilst_box<R: Read + Seek>(reader: &mut R, metadata: &mut AudioMetadata, size: u64) -> Result<()> {
    let mut remaining = size;
    
    while remaining > 8 {
        // 读取子box
        let mut size_bytes = [0u8; 4];
        if reader.read_exact(&mut size_bytes).is_err() {
            break;
        }
        
        let mut box_type = [0u8; 4];
        if reader.read_exact(&mut box_type).is_err() {
            break;
        }
        
        let box_size = u32::from_be_bytes(size_bytes) as u64;
        if box_size == 0 || box_size > remaining {
            break;
        }
        
        let data_size = box_size - 8;
        
        // 解析具体的数据项
        match &box_type {
            b"\xa9nam" => {
                if let Some(text) = read_mp4_text_data(reader, data_size)? {
                    if metadata.title.is_none() {
                        metadata.title = Some(text);
                    }
                }
            }
            b"\xa9ART" | b"aART" => {
                if let Some(text) = read_mp4_text_data(reader, data_size)? {
                    if box_type == *b"aART" {
                        metadata.album_artist = Some(text);
                    } else {
                        metadata.artist = Some(text);
                    }
                }
            }
            b"\xa9alb" => {
                // 专辑
                if let Some(text) = read_mp4_text_data(reader, data_size)? {
                    if metadata.album.is_none() {
                        metadata.album = Some(text);
                    }
                }
            }
            b"\xa9day" | b"IYER" => {
                if let Some(text) = read_mp4_text_data(reader, data_size)? {
                    if let Ok(year) = text.parse::<u32>() {
                        if metadata.year.is_none() {
                            metadata.year = Some(year);
                        }
                    }
                }
            }
            b"\xa9cmt" => {
                if let Some(text) = read_mp4_text_data(reader, data_size)? {
                    if metadata.comment.is_none() {
                        metadata.comment = Some(text);
                    }
                }
            }
            b"\xa9gen" | b"gen " => {
                if let Some(text) = read_mp4_text_data(reader, data_size)? {
                    if metadata.genre.is_none() {
                        metadata.genre = Some(text);
                    }
                }
            }
            b"\xa9wrt" | b"wcop" => {
                if let Some(text) = read_mp4_text_data(reader, data_size)? {
                    if box_type == *b"\xa9wrt" {
                        if metadata.composer.is_none() {
                            metadata.composer = Some(text);
                        }
                    }
                }
            }
            b"trkn" | b"tkhd" => {
                read_track_number(reader, data_size, metadata)?;
            }
            b"disk" | b"DISK" => {
                read_disc_number(reader, data_size, metadata)?;
            }
            b"tmpo" => {
                if let Some(_text) = read_mp4_text_data(reader, data_size)? {
                }
            }
            b"gnre" | b"GEID" => {
                if let Some(text) = read_mp4_text_data(reader, data_size)? {
                    if metadata.genre.is_none() {
                        metadata.genre = Some(text);
                    }
                }
            }
            b"\xa9lyr" => {
                if let Some(text) = read_mp4_text_data(reader, data_size)? {
                    if metadata.lyrics.is_none() {
                        metadata.lyrics = Some(text);
                    }
                }
            }
            _ => {
                // 跳过未知数据
                reader.seek(SeekFrom::Current(data_size as i64))
                    .map_err(|e| MetadataError::IoError(e))?;
            }
        }
        
        remaining -= box_size;
    }
    
    Ok(())
}

/// 读取MP4文本数据
fn read_mp4_text_data<R: Read + Seek>(reader: &mut R, size: u64) -> Result<Option<String>> {
    if size < 8 {
        return Ok(None);
    }
    
    // 跳过content version box（4字节）
    reader.seek(SeekFrom::Current(4))
        .map_err(|e| MetadataError::IoError(e))?;
    
    let text_size = size - 8;
    let mut text_bytes = vec![0u8; text_size as usize];
    reader.read_exact(&mut text_bytes)
        .map_err(|e| MetadataError::IoError(e))?;
    
    // MP4文本通常是UTF-8或UTF-16BE
    let text = if text_bytes.len() >= 2 && text_bytes[0] == 0xFE && text_bytes[1] == 0xFF {
        // UTF-16BE
        let text_u16: Vec<u16> = text_bytes.chunks(2)
            .filter_map(|c| {
                if c.len() == 2 {
                    Some(u16::from_be_bytes([c[0], c[1]]))
                } else {
                    None
                }
            })
            .collect();
        String::from_utf16(&text_u16).unwrap_or_default()
    } else {
        // UTF-8
        String::from_utf8_lossy(&text_bytes).trim().to_string()
    };
    
    if text.is_empty() {
        Ok(None)
    } else {
        Ok(Some(text))
    }
}

/// 读取音轨号
fn read_track_number<R: Read + Seek>(reader: &mut R, size: u64, metadata: &mut AudioMetadata) -> Result<()> {
    if size < 8 {
        reader.seek(SeekFrom::Current(size as i64))
            .map_err(|e| MetadataError::IoError(e))?;
        return Ok(());
    }
    
    // 跳过version (4 bytes)
    reader.seek(SeekFrom::Current(4))
        .map_err(|e| MetadataError::IoError(e))?;
    
    // 读取音轨数据
    let mut data = [0u8; 4];
    reader.read_exact(&mut data)
        .map_err(|e| MetadataError::IoError(e))?;
    
    let track = u16::from_be_bytes([data[0], data[1]]);
    let total = u16::from_be_bytes([data[2], data[3]]);
    
    if track > 0 {
        metadata.track_number = Some(track as u32);
    }
    if total > 0 {
        metadata.total_tracks = Some(total as u32);
    }
    
    Ok(())
}

/// 读取光盘号
fn read_disc_number<R: Read + Seek>(reader: &mut R, size: u64, metadata: &mut AudioMetadata) -> Result<()> {
    if size < 8 {
        reader.seek(SeekFrom::Current(size as i64))
            .map_err(|e| MetadataError::IoError(e))?;
        return Ok(());
    }
    
    // 跳过version (4 bytes)
    reader.seek(SeekFrom::Current(4))
        .map_err(|e| MetadataError::IoError(e))?;
    
    // 读取光盘数据
    let mut data = [0u8; 4];
    reader.read_exact(&mut data)
        .map_err(|e| MetadataError::IoError(e))?;
    
    let disc = u16::from_be_bytes([data[0], data[1]]);
    let total = u16::from_be_bytes([data[2], data[3]]);
    
    if disc > 0 {
        metadata.disc_number = Some(disc as u32);
    }
    if total > 0 {
        metadata.total_discs = Some(total as u32);
    }
    
    Ok(())
}

/// 解析covr box（封面图片）
fn parse_covr_box<R: Read>(reader: &mut R, metadata: &mut AudioMetadata, size: u64) -> Result<()> {
    let mut remaining = size;
    
    while remaining > 8 {
        let mut size_bytes = [0u8; 4];
        if reader.read_exact(&mut size_bytes).is_err() {
            break;
        }
        
        let mut box_type = [0u8; 4];
        if reader.read_exact(&mut box_type).is_err() {
            break;
        }
        
        let box_size = u32::from_be_bytes(size_bytes) as u64;
        if box_size == 0 || box_size > remaining {
            break;
        }
        
        let data_size = box_size - 8;
        
        // 读取图片数据
        let mut picture_data = vec![0u8; data_size as usize];
        reader.read_exact(&mut picture_data)
            .map_err(|e| MetadataError::IoError(e))?;
        
        // 确定MIME类型
        let mime_type = if picture_data.starts_with(&[0xFF, 0xD8, 0xFF]) {
            "image/jpeg".to_string()
        } else if picture_data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
            "image/png".to_string()
        } else {
            "image/unknown".to_string()
        };
        
        let picture = Picture::new(
            PictureType::CoverFront,
            mime_type,
            picture_data,
        );
        
        metadata.pictures.push(picture);
        
        remaining -= box_size;
    }
    
    Ok(())
}