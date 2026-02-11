//! WAV 音频格式读取器
//!
//! 支持标准 WAV 文件和带有 INFO 列表的扩展元数据

use crate::audio_metadata::{
    core::{AudioFormat, AudioMetadata},
    utils::encoding::auto_decode_text,
};
use crate::audio_metadata::MetadataError;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::time::Duration;

/// WAV 元数据读取器
pub struct WavReader;

impl WavReader {
    /// 从任意实现了 Read + Seek 的读取器中读取 WAV 元数据
    pub fn read_from<R: Read + Seek>(mut reader: R) -> Result<AudioMetadata, MetadataError> {
        // 检查 RIFF 标记
        let mut riff_marker = [0u8; 4];
        reader.read_exact(&mut riff_marker)
            .map_err(|e| MetadataError::IoError(e.to_string()))?;

        if &riff_marker != b"RIFF" {
            return Err(MetadataError::InvalidFormat("不是有效的 RIFF 文件".to_string()));
        }

        // 读取文件大小
        let mut file_size = [0u8; 4];
        reader.read_exact(&mut file_size)
            .map_err(|e| MetadataError::IoError(e.to_string()))?;
        let _file_size = u32::from_le_bytes(file_size);

        // 检查 WAVE 标记
        let mut wave_marker = [0u8; 4];
        reader.read_exact(&mut wave_marker)
            .map_err(|e| MetadataError::IoError(e.to_string()))?;

        if &wave_marker != b"WAVE" {
            return Err(MetadataError::InvalidFormat("不是有效的 WAV 文件".to_string()));
        }

        let mut metadata = AudioMetadata::new(AudioFormat::Wav);

        // 解析各个块
        loop {
            let chunk = match read_chunk_header(&mut reader) {
                Ok(Some(chunk)) => chunk,
                Ok(None) => break,
                Err(e) => {
                    log::warn!("读取 WAV 块失败: {}", e);
                    break;
                }
            };

            match chunk.id.as_str() {
                "fmt " => {
                    parse_fmt_chunk(&mut reader, &mut metadata, chunk.size)?;
                }
                "data" => {
                    // 计算时长
                    calculate_duration(&mut metadata, chunk.size);
                    // 跳过数据块
                    reader.seek(SeekFrom::Current(chunk.size as i64))
                        .map_err(|e| MetadataError::IoError(e.to_string()))?;
                }
                "LIST" => {
                    parse_list_chunk(&mut reader, &mut metadata, chunk.size)?;
                }
                "id3 " | "ID3 " => {
                    parse_id3_chunk(&mut reader, &mut metadata, chunk.size)?;
                }
                _ => {
                    // 跳过未知块
                    let padding = if chunk.size % 2 == 1 { 1 } else { 0 };
                    reader.seek(SeekFrom::Current(chunk.size as i64 + padding))
                        .map_err(|e| MetadataError::IoError(e.to_string()))?;
                }
            }
        }

        Ok(metadata)
    }
}

/// RIFF 块头部
#[derive(Debug)]
struct RiffChunk {
    id: String,
    size: u32,
}

/// 读取 WAV 文件元数据
pub fn read_wav_metadata(path: &Path) -> Result<AudioMetadata, MetadataError> {
    let file = File::open(path)
        .map_err(|e| MetadataError::IoError(e.to_string()))?;
    WavReader::read_from(file)
}

/// 读取块头部
fn read_chunk_header<R: Read>(reader: &mut R) -> Result<Option<RiffChunk>, MetadataError> {
    let mut id = [0u8; 4];
    match reader.read_exact(&mut id) {
        Ok(_) => {}
        Err(_) => return Ok(None),
    }

    let mut size_bytes = [0u8; 4];
    reader.read_exact(&mut size_bytes)
        .map_err(|e| MetadataError::IoError(e.to_string()))?;
    let size = u32::from_le_bytes(size_bytes);

    Ok(Some(RiffChunk {
        id: String::from_utf8_lossy(&id).to_string(),
        size,
    }))
}

/// 解析 fmt 块
fn parse_fmt_chunk<R: Read>(
    reader: &mut R,
    metadata: &mut AudioMetadata,
    size: u32,
) -> Result<(), MetadataError> {
    let mut data = vec![0u8; size as usize];
    reader.read_exact(&mut data)
        .map_err(|e| MetadataError::IoError(e.to_string()))?;

    if data.len() < 16 {
        return Ok(());
    }

    // 音频格式（2 字节）
    let format_tag = u16::from_le_bytes([data[0], data[1]]);

    // 声道数（2 字节）
    let channels = u16::from_le_bytes([data[2], data[3]]);
    metadata.channels = Some(channels as u8);

    // 采样率（4 字节）
    let sample_rate = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
    metadata.sample_rate = Some(sample_rate);

    // 字节率（4 字节）
    let byte_rate = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);

    // 块对齐（2 字节）
    // let block_align = u16::from_le_bytes([data[12], data[13]]);

    // 位深度（2 字节）
    let bits_per_sample = u16::from_le_bytes([data[14], data[15]]);

    // 计算比特率
    if format_tag == 1 {
        // PCM
        metadata.bitrate = Some((byte_rate * 8) / 1000);
    } else if data.len() >= 18 {
        // 扩展格式
        let extra_size = u16::from_le_bytes([data[16], data[17]]);
        if extra_size >= 2 && data.len() >= 20 {
            // 对于压缩格式，使用采样率 * 位深度 * 声道数估算
            let estimated_bitrate = (sample_rate * bits_per_sample as u32 * channels as u32) / 1000;
            metadata.bitrate = Some(estimated_bitrate);
        }
    }

    Ok(())
}

/// 计算时长
fn calculate_duration(metadata: &mut AudioMetadata, data_size: u32) {
    if let (Some(sample_rate), Some(channels)) = (metadata.sample_rate, metadata.channels) {
        // 假设 16 位采样（常见情况）
        let bytes_per_sample = 2u32;
        let bytes_per_second = sample_rate * channels as u32 * bytes_per_sample;

        if bytes_per_second > 0 {
            let duration_secs = data_size as f64 / bytes_per_second as f64;
            metadata.duration = Some(Duration::from_secs_f64(duration_secs));
        }
    }
}

/// 解析 LIST 块
fn parse_list_chunk<R: Read + Seek>(
    reader: &mut R,
    metadata: &mut AudioMetadata,
    size: u32,
) -> Result<(), MetadataError> {
    let start_pos = reader.stream_position()
        .map_err(|e| MetadataError::IoError(e.to_string()))?;
    let end_pos = start_pos + size as u64;

    // 读取列表类型 ID
    let mut list_type = [0u8; 4];
    reader.read_exact(&mut list_type)
        .map_err(|e| MetadataError::IoError(e.to_string()))?;

    match &list_type {
        b"INFO" => {
            parse_info_list(reader, metadata, end_pos)?;
        }
        b"adtl" => {
            // 标签列表，跳过
            reader.seek(SeekFrom::Start(end_pos))
                .map_err(|e| MetadataError::IoError(e.to_string()))?;
        }
        _ => {
            // 跳过未知列表类型
            reader.seek(SeekFrom::Start(end_pos))
                .map_err(|e| MetadataError::IoError(e.to_string()))?;
        }
    }

    Ok(())
}

/// 解析 INFO 列表
fn parse_info_list<R: Read + Seek>(
    reader: &mut R,
    metadata: &mut AudioMetadata,
    end_pos: u64,
) -> Result<(), MetadataError> {
    while reader.stream_position()
        .map_err(|e| MetadataError::IoError(e.to_string()))? < end_pos {
        let chunk = match read_chunk_header(reader) {
            Ok(Some(chunk)) => chunk,
            Ok(None) => break,
            Err(_) => break,
        };

        let mut data = vec![0u8; chunk.size as usize];
        if reader.read_exact(&mut data).is_err() {
            break;
        }

        // 跳过填充字节
        if chunk.size % 2 == 1 {
            let mut padding = [0u8; 1];
            let _ = reader.read_exact(&mut padding);
        }

        // 解码字符串（去除 null 结尾）
        let text = auto_decode_text(&data).unwrap_or_default();
        let text = text.trim_end_matches('\0');

        match chunk.id.as_str() {
            "INAM" => metadata.title = Some(text.to_string()),
            "IART" => metadata.artist = Some(text.to_string()),
            "IPRD" => metadata.album = Some(text.to_string()),
            "ICMT" => metadata.comment = Some(text.to_string()),
            "ICRD" => {
                metadata.year = text.chars().take(4).collect::<String>().parse().ok();
            }
            "IGNR" => metadata.genre = Some(text.to_string()),
            "ICOP" => {
                // 版权信息
            }
            "ISFT" => {
                // 软件信息
            }
            "IENG" => {
                // 工程师
            }
            "ITCH" => {
                // 技术员
            }
            "ISRC" => {
                // 来源
            }
            _ => {}
        }
    }

    Ok(())
}

/// 解析 ID3 块
fn parse_id3_chunk<R: Read + Seek>(
    reader: &mut R,
    metadata: &mut AudioMetadata,
    size: u32,
) -> Result<(), MetadataError> {
    // 读取 ID3 数据
    let mut data = vec![0u8; size as usize];
    reader.read_exact(&mut data)
        .map_err(|e| MetadataError::IoError(e.to_string()))?;

    // 检查 ID3 标记
    if data.len() < 10 || &data[0..3] != b"ID3" {
        return Ok(());
    }

    let version = data[3];
    let tag_size = syncsafe_to_u32(&data[6..10]) as usize;

    if data.len() < 10 + tag_size {
        return Ok(());
    }

    let tag_data = &data[10..10 + tag_size];

    // 简化的 ID3 解析
    parse_id3_frames(tag_data, metadata, version)?;

    Ok(())
}

/// 解析 ID3 帧
fn parse_id3_frames(
    data: &[u8],
    metadata: &mut AudioMetadata,
    version: u8,
) -> Result<(), MetadataError> {
    let mut pos = 0;

    // 跳过扩展头部（ID3v2.3/2.4）
    if version >= 3 {
        if pos + 4 <= data.len() {
            let ext_size = syncsafe_to_u32(&data[pos..pos+4]) as usize;
            if ext_size > 0 {
                pos += ext_size;
            }
        }
    }

    while pos < data.len() {
        // 检查是否到达填充区域
        if data[pos] == 0 {
            break;
        }

        // 读取帧 ID
        let frame_id_len = if version == 2 { 3 } else { 4 };
        if pos + frame_id_len > data.len() {
            break;
        }

        let frame_id = String::from_utf8_lossy(&data[pos..pos + frame_id_len]);
        pos += frame_id_len;

        // 读取帧大小
        let frame_size = if version == 2 {
            if pos + 3 > data.len() {
                break;
            }
            ((data[pos] as usize) << 16) | ((data[pos + 1] as usize) << 8) | (data[pos + 2] as usize)
        } else if version == 3 {
            if pos + 4 > data.len() {
                break;
            }
            u32::from_be_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]) as usize
        } else {
            if pos + 4 > data.len() {
                break;
            }
            syncsafe_to_u32(&data[pos..pos+4]) as usize
        };

        let header_size = if version == 2 { 3 } else { 4 + 2 }; // +2 for flags in v2.3+
        pos += header_size;

        if pos + frame_size > data.len() {
            break;
        }

        let frame_data = &data[pos..pos + frame_size];
        pos += frame_size;

        // 解析帧内容
        parse_id3_frame(metadata, &frame_id, frame_data)?;
    }

    Ok(())
}

/// 解析单个 ID3 帧
fn parse_id3_frame(
    metadata: &mut AudioMetadata,
    frame_id: &str,
    data: &[u8],
) -> Result<(), MetadataError> {
    if data.is_empty() {
        return Ok(());
    }

    // 尝试解码文本帧
    let text = decode_id3_text(data).unwrap_or_default();

    match frame_id {
        "TIT2" | "TT2" => metadata.title = Some(text),
        "TPE1" | "TP1" => metadata.artist = Some(text),
        "TPE2" | "TP2" => metadata.album_artist = Some(text),
        "TALB" | "TAL" => metadata.album = Some(text),
        "TYER" | "TDRC" | "TYE" => {
            metadata.year = text.chars().take(4).collect::<String>().parse().ok();
        }
        "TCON" | "TCO" => metadata.genre = Some(text),
        "TCOM" | "TCM" => metadata.composer = Some(text),
        "COMM" | "COM" => metadata.comment = Some(text),
        _ => {}
    }

    Ok(())
}

/// 解码 ID3 文本
fn decode_id3_text(data: &[u8]) -> Option<String> {
    if data.is_empty() {
        return None;
    }

    let encoding = data[0];
    let text_data = &data[1..];

    match encoding {
        0x00 => {
            // ISO-8859-1
            Some(text_data.iter().map(|&b| b as char).collect())
        }
        0x01 => {
            // UTF-16 with BOM
            decode_utf16_with_bom(text_data)
        }
        0x02 => {
            // UTF-16 BE without BOM
            decode_utf16_be(text_data)
        }
        0x03 => {
            // UTF-8
            String::from_utf8(text_data.to_vec()).ok()
        }
        _ => {
            // 未知编码，尝试 UTF-8
            String::from_utf8(data.to_vec()).ok()
        }
    }
}

/// 解码带 BOM 的 UTF-16
fn decode_utf16_with_bom(data: &[u8]) -> Option<String> {
    if data.len() < 2 {
        return None;
    }

    let (is_be, data_start) = if &data[0..2] == b"\xFE\xFF" {
        (true, 2)
    } else if &data[0..2] == b"\xFF\xFE" {
        (false, 2)
    } else {
        // 默认小端
        (false, 0)
    };

    let u16_data: Vec<u16> = data[data_start..]
        .chunks_exact(2)
        .map(|chunk| {
            if is_be {
                u16::from_be_bytes([chunk[0], chunk[1]])
            } else {
                u16::from_le_bytes([chunk[0], chunk[1]])
            }
        })
        .collect();

    char::decode_utf16(u16_data)
        .collect::<Result<String, _>>()
        .ok()
}

/// 解码 UTF-16 BE
fn decode_utf16_be(data: &[u8]) -> Option<String> {
    if data.len() % 2 != 0 {
        return None;
    }

    let u16_data: Vec<u16> = data
        .chunks_exact(2)
        .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
        .collect();

    char::decode_utf16(u16_data)
        .collect::<Result<String, _>>()
        .ok()
}

/// 同步安全整数转换
fn syncsafe_to_u32(data: &[u8]) -> u32 {
    let mut result: u32 = 0;
    for &byte in data.iter().take(4) {
        result = (result << 7) | (byte as u32 & 0x7F);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syncsafe() {
        let data = [0x00, 0x00, 0x01, 0x7F];
        assert_eq!(syncsafe_to_u32(&data), 255);
    }

    #[test]
    fn test_decode_utf16_be() {
        let data = vec![0x00, 0x48, 0x00, 0x65, 0x00, 0x6C, 0x00, 0x6C, 0x00, 0x6F];
        assert_eq!(decode_utf16_be(&data), Some("Hello".to_string()));
    }

    #[test]
    fn test_calculate_duration() {
        let mut metadata = AudioMetadata::new(AudioFormat::Wav);
        metadata.sample_rate = Some(44100);
        metadata.channels = Some(2);

        // 4 秒的数据（假设 16 位采样）
        // 44100 * 2 * 2 * 4 = 705600 字节
        calculate_duration(&mut metadata, 705600);

        assert_eq!(metadata.duration, Some(Duration::from_secs(4)));
    }
}
