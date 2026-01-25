//! OGG格式解析器
//! 
//! 支持OGG容器和Vorbis comments元数据解析

use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;
use byteorder::{LittleEndian, ReadBytesExt};

use super::super::core::{AudioMetadata, AudioFormat, MetadataReader, MetadataError, Result};
use super::super::utils::read_bytes;

/// OGG捕获模式
const OGG_CAPTURE_PATTERN: &[u8] = b"OggS";
/// Vorbis捕获模式
const VORBIS_CAPTURE_PATTERN: &[u8] = b"vorbis";

/// OGG解析器
pub struct OggParser;

impl MetadataReader for OggParser {
    fn read_metadata<P: AsRef<Path>>(path: P) -> Result<AudioMetadata> {
        let path = path.as_ref();
        let file = File::open(path)
            .map_err(|e| MetadataError::IoError(e))?;
        
        let mut reader = BufReader::new(file);
        let mut metadata = AudioMetadata::new(AudioFormat::Ogg);
        
        // 解析OGG pages
        parse_ogg_pages(&mut reader, &mut metadata)?;
        
        Ok(metadata)
    }

    fn can_parse<P: AsRef<Path>>(path: P) -> bool {
        if let Ok(header) = super::super::utils::peek_file_header(path.as_ref(), 4) {
            header == OGG_CAPTURE_PATTERN
        } else {
            false
        }
    }

    fn supported_extensions() -> &'static [&'static str] {
        &["ogg", "oga", "ogv"]
    }
}

/// OGG Page结构
#[allow(dead_code)]
struct OggPage {
    capture_pattern: [u8; 4],
    version: u8,
    header_type: u8,
    granule_position: u64,
    bitstream_serial: u32,
    page_sequence: u32,
    crc_checksum: u32,
    num_segments: u8,
    segment_table: Vec<u8>,
}

/// 解析OGG pages
fn parse_ogg_pages<R: Read + Seek>(reader: &mut R, metadata: &mut AudioMetadata) -> Result<()> {
    loop {
        // 检查是否到达文件末尾
        let pos = reader.seek(SeekFrom::Current(0))
            .map_err(|e| MetadataError::IoError(e))?;
        
        let mut capture = [0u8; 4];
        match reader.read_exact(&mut capture) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(MetadataError::IoError(e)),
        }
        
        // 回到开头
        reader.seek(SeekFrom::Start(pos))
            .map_err(|e| MetadataError::IoError(e))?;
        
        // 读取page头
        let page = match read_ogg_page_header(reader) {
            Ok(Some(page)) => page,
            Ok(None) => break,
            Err(e) => return Err(e),
        };
        
        // 解析page数据
        match parse_ogg_page_data(reader, &page, metadata) {
            Ok(true) => {
                // 找到元数据，继续查找
            }
            Ok(false) => {
                // 所有必要数据已读取
            }
            Err(e) => return Err(e),
        }
        
        // 跳到下一页
        let next_pos = pos + 27 + page.num_segments as u64 + page.segment_table.iter().sum::<u8>() as u64;
        reader.seek(SeekFrom::Start(next_pos))
            .map_err(|e| MetadataError::IoError(e))?;
    }
    
    Ok(())
}

/// 读取OGG page头
fn read_ogg_page_header<R: Read>(reader: &mut R) -> Result<Option<OggPage>> {
    let mut capture = [0u8; 4];
    match reader.read_exact(&mut capture) {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(MetadataError::IoError(e)),
    }
    
    if &capture != OGG_CAPTURE_PATTERN {
        return Ok(None);
    }
    
    let version = reader.read_u8()
        .map_err(|e| MetadataError::IoError(e))?;
    
    let header_type = reader.read_u8()
        .map_err(|e| MetadataError::IoError(e))?;
    
    let granule_position = reader.read_u64::<LittleEndian>()
        .map_err(|e| MetadataError::IoError(e))?;
    
    let bitstream_serial = reader.read_u32::<LittleEndian>()
        .map_err(|e| MetadataError::IoError(e))?;
    
    let page_sequence = reader.read_u32::<LittleEndian>()
        .map_err(|e| MetadataError::IoError(e))?;
    
    let crc_checksum = reader.read_u32::<LittleEndian>()
        .map_err(|e| MetadataError::IoError(e))?;
    
    let num_segments = reader.read_u8()
        .map_err(|e| MetadataError::IoError(e))?;
    
    let mut segment_table = vec![0u8; num_segments as usize];
    reader.read_exact(&mut segment_table)
        .map_err(|e| MetadataError::IoError(e))?;
    
    Ok(Some(OggPage {
        capture_pattern: capture,
        version,
        header_type,
        granule_position,
        bitstream_serial,
        page_sequence,
        crc_checksum,
        num_segments,
        segment_table,
    }))
}

/// 解析OGG page数据
fn parse_ogg_page_data<R: Read>(reader: &mut R, page: &OggPage, metadata: &mut AudioMetadata) -> Result<bool> {
    let segment_size = page.segment_table.iter().sum::<u8>() as usize;
    let page_data = read_bytes(reader, segment_size)?;
    
    // 检查是否是vorbis数据
    if page_data.starts_with(VORBIS_CAPTURE_PATTERN) {
        // 解析Vorbis包
        parse_vorbis_packet(&page_data, metadata)?;
    }
    
    Ok(true)
}

/// 解析Vorbis包
fn parse_vorbis_packet(data: &[u8], metadata: &mut AudioMetadata) -> Result<()> {
    if data.len() < 7 {
        return Ok(());
    }
    
    let packet_type = data[0];
    
    // 0x01: Identification header
    // 0x03: Comment header
    // 0x05: Book header
    
    if packet_type == 0x03 {
        // Comment header
        parse_vorbis_comments(&data[7..], metadata)?;
    }
    
    Ok(())
}

/// 解析Vorbis comments
fn parse_vorbis_comments(data: &[u8], metadata: &mut AudioMetadata) -> Result<()> {
    let mut offset = 0;
    
    // 读取vendor string长度
    if offset + 4 > data.len() {
        return Ok(());
    }
    let vendor_length = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    offset += 4;
    
    // 读取vendor string
    if offset + vendor_length > data.len() {
        return Ok(());
    }
    let _vendor = std::str::from_utf8(&data[offset..offset + vendor_length])
        .unwrap_or("unknown");
    offset += vendor_length;
    
    // 读取comment数量
    if offset + 4 > data.len() {
        return Ok(());
    }
    let num_comments = u32::from_le_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3]]) as usize;
    offset += 4;
    
    // 读取每个comment
    for _ in 0..num_comments {
        if offset + 4 > data.len() {
            break;
        }
        
        let comment_length = u32::from_le_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3]]) as usize;
        offset += 4;
        
        if offset + comment_length > data.len() {
            break;
        }
        
        let comment = std::str::from_utf8(&data[offset..offset + comment_length])
            .unwrap_or("");
        offset += comment_length;
        
        // 解析comment: FIELD=value
        if let Some(eq_pos) = comment.find('=') {
            let field = &comment[..eq_pos].to_uppercase();
            let value = &comment[eq_pos + 1..];
            
            match field.as_str() {
                "TITLE" | "TRACKNAME" => {
                    if metadata.title.is_none() {
                        metadata.title = Some(value.to_string());
                    }
                }
                "ARTIST" => {
                    if metadata.artist.is_none() {
                        metadata.artist = Some(value.to_string());
                    }
                }
                "ALBUM" | "ALBUMTITLE" => {
                    if metadata.album.is_none() {
                        metadata.album = Some(value.to_string());
                    }
                }
                "ALBUMARTIST" => {
                    if metadata.album_artist.is_none() {
                        metadata.album_artist = Some(value.to_string());
                    }
                }
                "COMPOSER" | "AUTHOR" => {
                    if metadata.composer.is_none() {
                        metadata.composer = Some(value.to_string());
                    }
                }
                "DATE" | "YEAR" => {
                    if let Ok(year) = value.parse::<u32>() {
                        if metadata.year.is_none() {
                            metadata.year = Some(year);
                        }
                    }
                }
                "TRACKNUMBER" | "TRACK" => {
                    if let Ok(track) = value.parse::<u32>() {
                        if metadata.track_number.is_none() {
                            metadata.track_number = Some(track);
                        }
                    }
                }
                "TRACKTOTAL" | "TRACKS" => {
                    if let Ok(total) = value.parse::<u32>() {
                        if metadata.total_tracks.is_none() {
                            metadata.total_tracks = Some(total);
                        }
                    }
                }
                "DISCNUMBER" | "DISC" => {
                    if let Ok(disc) = value.parse::<u32>() {
                        if metadata.disc_number.is_none() {
                            metadata.disc_number = Some(disc);
                        }
                    }
                }
                "DISCTOTAL" | "DISCS" => {
                    if let Ok(total) = value.parse::<u32>() {
                        if metadata.total_discs.is_none() {
                            metadata.total_discs = Some(total);
                        }
                    }
                }
                "GENRE" => {
                    if metadata.genre.is_none() {
                        metadata.genre = Some(value.to_string());
                    }
                }
                "COMMENT" | "DESCRIPTION" => {
                    if metadata.comment.is_none() {
                        metadata.comment = Some(value.to_string());
                    }
                }
                "LYRICS" => {
                    if metadata.lyrics.is_none() {
                        metadata.lyrics = Some(value.to_string());
                    }
                }
                _ => {
                    // 保存到extra_tags
                    if !value.is_empty() {
                        metadata.extra_tags.insert(field.to_string(), value.to_string());
                    }
                }
            }
        }
    }
    
    Ok(())
}