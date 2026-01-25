//! WAV格式解析器
//! 
//! 支持RIFF容器和INFO chunks元数据解析

use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;
use byteorder::{LittleEndian, ReadBytesExt};

use super::super::core::{AudioMetadata, AudioFormat, MetadataReader, MetadataError, Result};

/// RIFF签名
const RIFF_SIGNATURE: &[u8] = b"RIFF";
/// WAVE格式
const WAVE_FORMAT: &[u8] = b"WAVE";
/// LIST签名
#[allow(dead_code)]
const LIST_SIGNATURE: &[u8] = b"LIST";
/// INFO签名
const INFO_SIGNATURE: &[u8] = b"INFO";

/// WAV解析器
pub struct WavParser;

impl MetadataReader for WavParser {
    fn read_metadata<P: AsRef<Path>>(path: P) -> Result<AudioMetadata> {
        let path = path.as_ref();
        let file = File::open(path)
            .map_err(|e| MetadataError::IoError(e))?;
        
        let mut reader = BufReader::new(file);
        let mut metadata = AudioMetadata::new(AudioFormat::Wav);
        
        // 读取RIFF头
        let file_size = read_riff_header(&mut reader, &mut metadata)?;
        
        // 解析chunks
        parse_wave_chunks(&mut reader, &mut metadata, file_size)?;
        
        Ok(metadata)
    }

    fn can_parse<P: AsRef<Path>>(path: P) -> bool {
        if let Ok(header) = super::super::utils::peek_file_header(path.as_ref(), 12) {
            &header[0..4] == RIFF_SIGNATURE && &header[8..12] == WAVE_FORMAT
        } else {
            false
        }
    }

    fn supported_extensions() -> &'static [&'static str] {
        &["wav", "wave", "aiff", "aif"]
    }
}

/// 读取RIFF头
fn read_riff_header<R: Read + Seek>(reader: &mut R, _metadata: &mut AudioMetadata) -> Result<u64> {
    let mut riff = [0u8; 4];
    reader.read_exact(&mut riff)
        .map_err(|e| MetadataError::IoError(e))?;
    
    if &riff != RIFF_SIGNATURE {
        return Err(MetadataError::invalid_format("不是有效的RIFF文件"));
    }
    
    let file_size = reader.read_u32::<LittleEndian>()
        .map_err(|e| MetadataError::IoError(e))? as u64;
    
    let mut wave = [0u8; 4];
    reader.read_exact(&mut wave)
        .map_err(|e| MetadataError::IoError(e))?;
    
    if &wave != WAVE_FORMAT {
        return Err(MetadataError::invalid_format("不是有效的WAVE文件"));
    }
    
    Ok(file_size)
}

/// 解析WAVE chunks
fn parse_wave_chunks<R: Read + Seek>(reader: &mut R, metadata: &mut AudioMetadata, max_offset: u64) -> Result<()> {
    let mut offset = 12u64; // 跳过RIFF头
    
    while offset < max_offset {
        // 读取chunk头
        let mut chunk_id = [0u8; 4];
        if reader.read_exact(&mut chunk_id).is_err() {
            break;
        }
        
        let chunk_size = reader.read_u32::<LittleEndian>()
            .map_err(|e| MetadataError::IoError(e))? as u64;
        
        let next_offset = offset + 8 + chunk_size;
        
        // 对齐到2字节边界（chunk size已经是字节对齐的）
        
        // 解析特定chunk
        match &chunk_id {
            b"fmt " => {
                // 音频格式信息
                parse_format_chunk(reader, chunk_size, metadata)?;
            }
            b"LIST" => {
                // LIST容器
                parse_list_chunk(reader, chunk_size, metadata)?;
            }
            b"data" => {
                // 音频数据，读取时长估算
                if let Some(duration) = estimate_duration(reader, chunk_size, metadata) {
                    metadata.duration = Some(duration);
                }
                // 跳过data chunk
                reader.seek(SeekFrom::Current(chunk_size as i64))
                    .map_err(|e| MetadataError::IoError(e))?;
            }
            _ => {
                // 跳过未知chunk
                reader.seek(SeekFrom::Current(chunk_size as i64))
                    .map_err(|e| MetadataError::IoError(e))?;
            }
        }
        
        offset = next_offset;
    }
    
    Ok(())
}

/// 解析fmt chunk
fn parse_format_chunk<R: Read + Seek>(reader: &mut R, size: u64, metadata: &mut AudioMetadata) -> Result<()> {
    let _audio_format = reader.read_u16::<LittleEndian>()
        .map_err(|e| MetadataError::IoError(e))?;
    
    let num_channels = reader.read_u16::<LittleEndian>()
        .map_err(|e| MetadataError::IoError(e))?;
    
    let sample_rate = reader.read_u32::<LittleEndian>()
        .map_err(|e| MetadataError::IoError(e))?;
    
    let _byte_rate = reader.read_u32::<LittleEndian>()
        .map_err(|e| MetadataError::IoError(e))?;
    
    let _block_align = reader.read_u16::<LittleEndian>()
        .map_err(|e| MetadataError::IoError(e))?;
    
    let bits_per_sample = reader.read_u16::<LittleEndian>()
        .map_err(|e| MetadataError::IoError(e))?;
    
    // 跳过额外格式信息
    if size > 16 {
        let extra_size = size - 16;
        reader.seek(SeekFrom::Current(extra_size as i64))
            .map_err(|e| MetadataError::IoError(e))?;
    }
    
    metadata.sample_rate = Some(sample_rate);
    metadata.channels = Some(num_channels as u32);
    
    // 计算比特率
    let bitrate = sample_rate * num_channels as u32 * bits_per_sample as u32;
    metadata.bitrate = Some(bitrate / 1000);
    
    Ok(())
}

/// 估算音频时长
fn estimate_duration<R: Read>(_reader: &mut R, data_size: u64, metadata: &mut AudioMetadata) -> Option<std::time::Duration> {
    if let (Some(sample_rate), Some(channels), _) = 
        (metadata.sample_rate, metadata.channels, metadata.bitrate) {
        let bytes_per_sample = 2; // 假设16位
        let samples = data_size / (bytes_per_sample * channels as u64);
        let duration_secs = samples / sample_rate as u64;
        
        Some(std::time::Duration::from_secs(duration_secs))
    } else {
        None
    }
}

/// 解析LIST chunk
fn parse_list_chunk<R: Read + Seek>(reader: &mut R, size: u64, metadata: &mut AudioMetadata) -> Result<()> {
    let mut list_type = [0u8; 4];
    reader.read_exact(&mut list_type)
        .map_err(|e| MetadataError::IoError(e))?;
    
    let remaining_size = size - 4;
    
    if &list_type == INFO_SIGNATURE {
        // INFO sub-chunks
        parse_info_chunks(reader, metadata, remaining_size)?;
    } else {
        // 跳过未知LIST类型
        reader.seek(SeekFrom::Current(remaining_size as i64))
            .map_err(|e| MetadataError::IoError(e))?;
    }
    
    Ok(())
}

/// 解析INFO chunks
fn parse_info_chunks<R: Read + Seek>(reader: &mut R, metadata: &mut AudioMetadata, size: u64) -> Result<()> {
    let mut offset = 0u64;
    
    while offset < size {
        let mut chunk_id = [0u8; 4];
        if reader.read_exact(&mut chunk_id).is_err() {
            break;
        }
        
        let chunk_size = reader.read_u32::<LittleEndian>()
            .map_err(|e| MetadataError::IoError(e))? as u64;
        
        // 解析text chunk
        let text_size = chunk_size.min(256); // 限制读取大小
        let mut text_data = vec![0u8; text_size as usize];
        reader.read_exact(&mut text_data)
            .map_err(|e| MetadataError::IoError(e))?;
        
        // 跳过填充字节
        if chunk_size % 2 != 0 {
            reader.seek(SeekFrom::Current(1))
                .map_err(|e| MetadataError::IoError(e))?;
        }
        
        // 解码文本
        let text = decode_wav_text(&text_data);
        
        // 设置元数据
        match &chunk_id {
            b"INAM" => {
                if metadata.title.is_none() {
                    metadata.title = Some(text);
                }
            }
            b"IART" => {
                if metadata.artist.is_none() {
                    metadata.artist = Some(text);
                }
            }
            b"IALB" => {
                if metadata.album.is_none() {
                    metadata.album = Some(text);
                }
            }
            b"ICRD" => {
                if let Ok(year) = text.parse::<u32>() {
                    if metadata.year.is_none() {
                        metadata.year = Some(year);
                    }
                }
            }
            b"ITRK" => {
                if let Ok(track) = text.parse::<u32>() {
                    if metadata.track_number.is_none() {
                        metadata.track_number = Some(track);
                    }
                }
            }
            b"IGNR" => {
                if metadata.genre.is_none() {
                    metadata.genre = Some(text);
                }
            }
            b"ICMT" => {
                if metadata.comment.is_none() {
                    metadata.comment = Some(text);
                }
            }
            b"IPRO" => {
            }
            b"IENG" => {
            }
            b"ISFT" => {
            }
            b"ICOP" => {
            }
            _ => {}
        }
        
        offset += 8 + chunk_size;
    }
    
    Ok(())
}

/// 解码WAV文本
fn decode_wav_text(data: &[u8]) -> String {
    // 尝试UTF-8
    if let Ok(s) = std::str::from_utf8(data) {
        return s.trim_end_matches('\0').trim().to_string();
    }
    
    // 尝试UTF-16LE
    if data.len() >= 2 && data.len() % 2 == 0 {
        let chars: Vec<u16> = data.chunks(2)
            .filter_map(|c| {
                if c.len() == 2 {
                    Some(u16::from_le_bytes([c[0], c[1]]))
                } else {
                    None
                }
            })
            .collect();
        
        if let Ok(s) = String::from_utf16(&chars) {
            return s.trim_end_matches('\0').trim().to_string();
        }
    }
    
    // 回退到Latin1
    let latin1: String = data.iter()
        .map(|&c| c as char)
        .collect();
    latin1.trim_end_matches('\0').trim().to_string()
}