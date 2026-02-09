//! MP3 音频格式读取器
//!
//! 支持 ID3v1、ID3v2.2/2.3/2.4 标签解析和 MP3 帧信息读取

use crate::audio_metadata::{
    core::{AudioFormat, AudioMetadata, Picture, PictureType},
};
use crate::audio_metadata::MetadataError;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::time::Duration;

/// MP3 元数据读取器
pub struct Mp3Reader;

impl Mp3Reader {
    /// 从任意实现了 Read + Seek 的读取器中读取 MP3 元数据
    pub fn read_from<R: Read + Seek>(mut reader: R) -> Result<AudioMetadata, MetadataError> {
        let mut metadata = AudioMetadata::new(AudioFormat::Mp3);

        // 首先尝试读取 ID3v2 标签（位于文件开头）
        let id3v2_size = read_id3v2_tag(&mut reader, &mut metadata)?;

        // 保存 ID3v2 标签后的位置
        let audio_start = id3v2_size;

        // 解析 MP3 帧以获取技术信息
        parse_mp3_frames(&mut reader, &mut metadata, audio_start)?;

        // 尝试读取 ID3v1 标签（位于文件末尾）
        let _ = read_id3v1_tag(&mut reader, &mut metadata);

        Ok(metadata)
    }
}

/// 读取 MP3 文件元数据
pub fn read_mp3_metadata(path: &Path) -> Result<AudioMetadata, MetadataError> {
    let file = File::open(path)
        .map_err(|e| MetadataError::IoError(e.to_string()))?;
    Mp3Reader::read_from(file)
}

/// 读取 ID3v2 标签
fn read_id3v2_tag<R: Read + Seek>(
    reader: &mut R,
    metadata: &mut AudioMetadata,
) -> Result<u64, MetadataError> {
    let start_pos = reader.stream_position()
        .map_err(|e| MetadataError::IoError(e.to_string()))?;

    // 检查 ID3 标记
    let mut marker = [0u8; 3];
    if reader.read_exact(&mut marker).is_err() || &marker != b"ID3" {
        // 没有 ID3v2 标签，重置位置
        reader.seek(SeekFrom::Start(start_pos))
            .map_err(|e| MetadataError::IoError(e.to_string()))?;
        return Ok(0);
    }

    // 读取版本
    let mut version = [0u8; 2];
    reader.read_exact(&mut version)
        .map_err(|e| MetadataError::IoError(e.to_string()))?;
    let major_version = version[0];

    // 跳过标志字节
    let mut flags = [0u8; 1];
    reader.read_exact(&mut flags)
        .map_err(|e| MetadataError::IoError(e.to_string()))?;
    let flags = flags[0];

    // 读取标签大小（同步安全整数）
    let mut size_bytes = [0u8; 4];
    reader.read_exact(&mut size_bytes)
        .map_err(|e| MetadataError::IoError(e.to_string()))?;
    let tag_size = syncsafe_to_u32(&size_bytes) as u64;

    // 计算总标签大小（包括头部）
    let total_size = 10 + tag_size;

    // 检查是否有扩展头部
    let has_extended_header = (flags & 0x40) != 0;

    // 读取标签数据
    let mut tag_data = vec![0u8; tag_size as usize];
    reader.read_exact(&mut tag_data)
        .map_err(|e| MetadataError::IoError(e.to_string()))?;

    // 解析帧
    parse_id3v2_frames(&tag_data, metadata, major_version, has_extended_header)?;

    Ok(total_size)
}

/// 解析 ID3v2 帧
fn parse_id3v2_frames(
    data: &[u8],
    metadata: &mut AudioMetadata,
    version: u8,
    has_extended_header: bool,
) -> Result<(), MetadataError> {
    let mut pos = 0;

    // 跳过扩展头部
    if has_extended_header && version >= 3 {
        if pos + 4 <= data.len() {
            let ext_size = syncsafe_to_u32(&data[pos..pos+4]) as usize;
            pos += ext_size;
        }
    }

    // 根据版本确定帧 ID 长度和大小格式
    let (id_len, size_len, has_flags) = match version {
        2 => (3, 3, false),  // ID3v2.2
        3 => (4, 4, true),   // ID3v2.3
        4 => (4, 4, true),   // ID3v2.4
        _ => return Ok(()),
    };

    while pos < data.len() {
        // 检查是否到达填充区域
        if data[pos] == 0 {
            break;
        }

        // 读取帧 ID
        if pos + id_len > data.len() {
            break;
        }
        let frame_id = String::from_utf8_lossy(&data[pos..pos + id_len]);
        pos += id_len;

        // 读取帧大小
        if pos + size_len > data.len() {
            break;
        }
        let frame_size = if version == 2 {
            // ID3v2.2: 3 字节大端
            ((data[pos] as usize) << 16) |
            ((data[pos + 1] as usize) << 8) |
            (data[pos + 2] as usize)
        } else if version == 3 {
            // ID3v2.3: 4 字节大端
            u32::from_be_bytes([
                data[pos], data[pos+1], data[pos+2], data[pos+3]
            ]) as usize
        } else {
            // ID3v2.4: 4 字节同步安全
            syncsafe_to_u32(&data[pos..pos+4]) as usize
        };
        pos += size_len;

        // 跳过标志字节
        if has_flags {
            if pos + 2 > data.len() {
                break;
            }
            pos += 2;
        }

        // 检查帧数据是否完整
        if pos + frame_size > data.len() {
            break;
        }

        let frame_data = &data[pos..pos + frame_size];
        pos += frame_size;

        // 解析帧内容
        parse_id3v2_frame(metadata, &frame_id, frame_data, version)?;
    }

    Ok(())
}

/// 解析单个 ID3v2 帧
fn parse_id3v2_frame(
    metadata: &mut AudioMetadata,
    frame_id: &str,
    data: &[u8],
    version: u8,
) -> Result<(), MetadataError> {
    if data.is_empty() {
        return Ok(());
    }

    // 将不同版本的帧 ID 映射到统一格式
    let normalized_id = normalize_frame_id(frame_id, version);

    match normalized_id.as_str() {
        "TIT2" => {
            if let Some(text) = decode_id3v2_text(data) {
                metadata.title = Some(text);
            }
        }
        "TPE1" => {
            if let Some(text) = decode_id3v2_text(data) {
                metadata.artist = Some(text);
            }
        }
        "TPE2" => {
            if let Some(text) = decode_id3v2_text(data) {
                metadata.album_artist = Some(text);
            }
        }
        "TALB" => {
            if let Some(text) = decode_id3v2_text(data) {
                metadata.album = Some(text);
            }
        }
        "TYER" | "TDRC" => {
            if let Some(text) = decode_id3v2_text(data) {
                metadata.year = text.chars().take(4).collect::<String>().parse().ok();
            }
        }
        "TRCK" => {
            if let Some(text) = decode_id3v2_text(data) {
                let parts: Vec<&str> = text.split('/').collect();
                metadata.track_number = parts[0].trim().parse().ok();
                if parts.len() > 1 {
                    metadata.total_tracks = parts[1].trim().parse().ok();
                }
            }
        }
        "TPOS" => {
            if let Some(text) = decode_id3v2_text(data) {
                let parts: Vec<&str> = text.split('/').collect();
                metadata.disc_number = parts[0].trim().parse().ok();
                if parts.len() > 1 {
                    metadata.total_discs = parts[1].trim().parse().ok();
                }
            }
        }
        "TCON" => {
            if let Some(text) = decode_id3v2_text(data) {
                metadata.genre = Some(parse_genre(&text));
            }
        }
        "TCOM" => {
            if let Some(text) = decode_id3v2_text(data) {
                metadata.composer = Some(text);
            }
        }
        "TEXT" => {
            if let Some(text) = decode_id3v2_text(data) {
                metadata.lyricist = Some(text);
            }
        }
        "COMM" => {
            if let Some(text) = decode_id3v2_comment(data) {
                metadata.comment = Some(text);
            }
        }
        "USLT" => {
            if let Some(text) = decode_id3v2_unsync_lyrics(data) {
                metadata.lyrics = Some(text);
            }
        }
        "APIC" => {
            if let Ok(picture) = decode_id3v2_picture(data) {
                metadata.add_picture(picture);
            }
        }
        "TLEN" => {
            // 时长（毫秒）
            if let Some(text) = decode_id3v2_text(data) {
                if let Ok(ms) = text.parse::<u64>() {
                    metadata.duration = Some(Duration::from_millis(ms));
                }
            }
        }
        "TBPM" => {
            // BPM，可选存储
        }
        _ => {}
    }

    Ok(())
}

/// 标准化帧 ID（将不同版本映射到 ID3v2.4 格式）
fn normalize_frame_id(frame_id: &str, version: u8) -> String {
    if version == 2 {
        // ID3v2.2 到 ID3v2.3/2.4 的映射
        match frame_id {
            "TT2" => "TIT2".to_string(),
            "TP1" => "TPE1".to_string(),
            "TP2" => "TPE2".to_string(),
            "TAL" => "TALB".to_string(),
            "TYE" => "TYER".to_string(),
            "TRK" => "TRCK".to_string(),
            "TPA" => "TPOS".to_string(),
            "TCO" => "TCON".to_string(),
            "TCM" => "TCOM".to_string(),
            "TXT" => "TEXT".to_string(),
            "COM" => "COMM".to_string(),
            "ULT" => "USLT".to_string(),
            "PIC" => "APIC".to_string(),
            "TLE" => "TLEN".to_string(),
            _ => frame_id.to_string(),
        }
    } else {
        frame_id.to_string()
    }
}

/// 解码 ID3v2 文本帧
fn decode_id3v2_text(data: &[u8]) -> Option<String> {
    if data.is_empty() {
        return None;
    }

    let encoding = data[0];
    let text_data = &data[1..];

    match encoding {
        0x00 => {
            // ISO-8859-1
            Some(text_data.iter().map(|&b| b as char).collect::<String>().trim_end_matches('\0').to_string())
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
        _ => String::from_utf8(data.to_vec()).ok(),
    }
}

/// 解码 ID3v2 注释帧
fn decode_id3v2_comment(data: &[u8]) -> Option<String> {
    if data.len() < 4 {
        return None;
    }

    let encoding = data[0];
    let lang = &data[1..4];
    let _ = lang; // 语言代码，暂时不使用

    let content_data = &data[4..];

    // 找到描述和实际内容的分离点（null 字节）
    let content_start = match encoding {
        0x00 | 0x03 => {
            // 单字节编码
            content_data.iter().position(|&b| b == 0).map(|p| p + 1).unwrap_or(0)
        }
        0x01 | 0x02 => {
            // UTF-16
            content_data.windows(2).position(|w| w == [0x00, 0x00]).map(|p| p + 2).unwrap_or(0)
        }
        _ => 0,
    };

    let actual_content = &content_data[content_start.min(content_data.len())..];

    match encoding {
        0x00 => Some(actual_content.iter().map(|&b| b as char).collect::<String>().trim_end_matches('\0').to_string()),
        0x01 => decode_utf16_with_bom(actual_content),
        0x02 => decode_utf16_be(actual_content),
        0x03 => String::from_utf8(actual_content.to_vec()).ok(),
        _ => None,
    }
}

/// 解码非同步歌词帧
fn decode_id3v2_unsync_lyrics(data: &[u8]) -> Option<String> {
    // 格式与注释帧相同
    decode_id3v2_comment(data)
}

/// 解码 APIC 图片帧
fn decode_id3v2_picture(data: &[u8]) -> Result<Picture, MetadataError> {
    if data.len() < 4 {
        return Err(MetadataError::InvalidFormat("APIC 帧太小".to_string()));
    }

    let encoding = data[0];
    let mut pos = 1;

    // 读取 MIME 类型（以 null 结尾）
    let mime_end = data[pos..].iter().position(|&b| b == 0)
        .ok_or_else(|| MetadataError::InvalidFormat("无效的 MIME 类型".to_string()))?;
    let mime_type = String::from_utf8_lossy(&data[pos..pos + mime_end]).to_string();
    pos += mime_end + 1;

    if pos >= data.len() {
        return Err(MetadataError::InvalidFormat("APIC 帧数据不足".to_string()));
    }

    // 图片类型
    let picture_type = data[pos];
    pos += 1;

    // 读取描述（以 null 结尾）
    let desc_end = match encoding {
        0x00 | 0x03 => {
            data[pos..].iter().position(|&b| b == 0).unwrap_or(data.len() - pos)
        }
        0x01 | 0x02 => {
            data[pos..].windows(2).position(|w| w == [0x00, 0x00]).unwrap_or(data.len() - pos)
        }
        _ => data[pos..].iter().position(|&b| b == 0).unwrap_or(data.len() - pos),
    };

    let description = match encoding {
        0x00 => String::from_utf8_lossy(&data[pos..pos + desc_end]).to_string(),
        0x01 => decode_utf16_with_bom(&data[pos..pos + desc_end]).unwrap_or_default(),
        0x02 => decode_utf16_be(&data[pos..pos + desc_end]).unwrap_or_default(),
        0x03 => String::from_utf8_lossy(&data[pos..pos + desc_end]).to_string(),
        _ => String::new(),
    };

    pos += desc_end + if encoding == 0x01 || encoding == 0x02 { 2 } else { 1 };

    // 剩余数据是图片
    let picture_data = data[pos.min(data.len())..].to_vec();

    Ok(Picture::new(
        PictureType::from_id3v2_type(picture_type),
        mime_type,
        description,
        picture_data,
    ))
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

/// 解析流派（处理 ID3v1 数字流派）
fn parse_genre(text: &str) -> String {
    // 处理 (数字) 格式的流派
    if text.starts_with('(') && text.ends_with(')') {
        if let Ok(id) = text[1..text.len()-1].parse::<usize>() {
            return get_id3_genre(id);
        }
    }
    // 处理纯数字
    if let Ok(id) = text.parse::<usize>() {
        return get_id3_genre(id);
    }
    text.to_string()
}

/// 获取 ID3 流派名称
fn get_id3_genre(id: usize) -> String {
    let genres = [
        "Blues", "Classic Rock", "Country", "Dance", "Disco", "Funk", "Grunge",
        "Hip-Hop", "Jazz", "Metal", "New Age", "Oldies", "Other", "Pop", "R&B",
        "Rap", "Reggae", "Rock", "Techno", "Industrial", "Alternative", "Ska",
        "Death Metal", "Pranks", "Soundtrack", "Euro-Techno", "Ambient",
        "Trip-Hop", "Vocal", "Jazz+Funk", "Fusion", "Trance", "Classical",
        "Instrumental", "Acid", "House", "Game", "Sound Clip", "Gospel", "Noise",
        "Alt Rock", "Bass", "Soul", "Punk", "Space", "Meditative", "Instrumental Pop",
        "Instrumental Rock", "Ethnic", "Gothic", "Darkwave", "Techno-Industrial",
        "Electronic", "Pop-Folk", "Eurodance", "Dream", "Southern Rock", "Comedy",
        "Cult", "Gangsta", "Top 40", "Christian Rap", "Pop/Funk", "Jungle",
        "Native American", "Cabaret", "New Wave", "Psychedelic", "Rave", "Showtunes",
        "Trailer", "Lo-Fi", "Tribal", "Acid Punk", "Acid Jazz", "Polka", "Retro",
        "Musical", "Rock & Roll", "Hard Rock",
    ];

    genres.get(id).map(|&s| s.to_string()).unwrap_or_else(|| format!("Unknown({})", id))
}

/// 读取 ID3v1 标签
fn read_id3v1_tag<R: Read + Seek>(
    reader: &mut R,
    metadata: &mut AudioMetadata,
) -> Result<(), MetadataError> {
    // 移动到文件末尾
    let file_size = reader.seek(SeekFrom::End(0))
        .map_err(|e| MetadataError::IoError(e.to_string()))?;

    if file_size < 128 {
        return Ok(());
    }

    // 读取最后 128 字节
    reader.seek(SeekFrom::End(-128))
        .map_err(|e| MetadataError::IoError(e.to_string()))?;

    let mut tag_data = [0u8; 128];
    reader.read_exact(&mut tag_data)
        .map_err(|e| MetadataError::IoError(e.to_string()))?;

    // 检查 TAG 标记
    if &tag_data[0..3] != b"TAG" {
        return Ok(());
    }

    // 解析字段（只在元数据为空时填充）
    if metadata.title.is_none() {
        metadata.title = Some(trim_null_bytes(&tag_data[3..33]));
    }
    if metadata.artist.is_none() {
        metadata.artist = Some(trim_null_bytes(&tag_data[33..63]));
    }
    if metadata.album.is_none() {
        metadata.album = Some(trim_null_bytes(&tag_data[63..93]));
    }
    if metadata.year.is_none() {
        let year_str = trim_null_bytes(&tag_data[93..97]);
        metadata.year = year_str.parse().ok();
    }
    if metadata.comment.is_none() {
        metadata.comment = Some(trim_null_bytes(&tag_data[97..127]));
    }
    if metadata.genre.is_none() {
        let genre_id = tag_data[127] as usize;
        if genre_id < 255 {
            metadata.genre = Some(get_id3_genre(genre_id));
        }
    }

    Ok(())
}

/// 去除 null 字节并修剪空白
fn trim_null_bytes(data: &[u8]) -> String {
    data.iter()
        .take_while(|&&b| b != 0)
        .map(|&b| b as char)
        .collect::<String>()
        .trim()
        .to_string()
}

/// 解析 MP3 帧以获取技术信息
fn parse_mp3_frames<R: Read + Seek>(
    reader: &mut R,
    metadata: &mut AudioMetadata,
    start_pos: u64,
) -> Result<(), MetadataError> {
    reader.seek(SeekFrom::Start(start_pos))
        .map_err(|e| MetadataError::IoError(e.to_string()))?;

    // 尝试找到第一个有效的 MP3 帧
    let mut buffer = [0u8; 4];
    let mut attempts = 0;
    const MAX_ATTEMPTS: usize = 10000;

    while attempts < MAX_ATTEMPTS {
        if reader.read_exact(&mut buffer).is_err() {
            break;
        }

        // 检查同步字 (11 个 1)
        let sync_word = u16::from_be_bytes([buffer[0], buffer[1]]);
        if sync_word & 0xFFE0 == 0xFFE0 {
            // 找到同步字，解析帧头
            if let Some((sample_rate, channels, bitrate)) = parse_mp3_frame_header(&buffer) {
                metadata.sample_rate = Some(sample_rate);
                metadata.channels = Some(channels);
                metadata.bitrate = Some(bitrate);

                // 计算时长（如果有文件大小信息）
                // 这里简化处理，实际应该根据文件大小计算
                break;
            }
        }

        // 向后移动 1 字节继续搜索
        reader.seek(SeekFrom::Current(-3))
            .map_err(|e| MetadataError::IoError(e.to_string()))?;
        attempts += 1;
    }

    Ok(())
}

/// 解析 MP3 帧头部
fn parse_mp3_frame_header(header: &[u8; 4]) -> Option<(u32, u8, u32)> {
    // MPEG 音频版本 ID (bits 19-20)
    let mpeg_version = (header[1] >> 3) & 0x03;
    // Layer 描述 (bits 17-18)
    let layer = (header[1] >> 1) & 0x03;
    // 位率索引 (bits 12-15)
    let bitrate_index = (header[2] >> 4) & 0x0F;
    // 采样率频率索引 (bits 10-11)
    let sample_rate_index = (header[2] >> 2) & 0x03;
    // 声道模式 (bits 6-7)
    let channel_mode = (header[3] >> 6) & 0x03;

    // 跳过无效值
    if mpeg_version == 1 || layer == 0 || bitrate_index == 0 || bitrate_index == 15 || sample_rate_index == 3 {
        return None;
    }

    // 采样率表 (Hz)
    let sample_rates: [[u32; 3]; 4] = [
        [44100, 48000, 32000], // MPEG 1
        [22050, 24000, 16000], // MPEG 2
        [11025, 12000, 8000],  // MPEG 2.5
        [0, 0, 0],
    ];

    let version_idx = match mpeg_version {
        0b11 => 0, // MPEG 1
        0b10 => 1, // MPEG 2
        0b00 => 2, // MPEG 2.5
        _ => 3,
    };

    let sample_rate = sample_rates[version_idx][sample_rate_index as usize];

    // 位率表 (kbps)
    let bitrates: [[u32; 15]; 5] = [
        // MPEG 1, Layer I
        [0, 32, 64, 96, 128, 160, 192, 224, 256, 288, 320, 352, 384, 416, 448],
        // MPEG 1, Layer II
        [0, 32, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 384],
        // MPEG 1, Layer III
        [0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320],
        // MPEG 2, Layer I
        [0, 32, 48, 56, 64, 80, 96, 112, 128, 144, 160, 176, 192, 224, 256],
        // MPEG 2, Layer II & III
        [0, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160],
    ];

    let layer_idx = match layer {
        0b11 => 0, // Layer I
        0b10 => 1, // Layer II
        0b01 => 2, // Layer III
        _ => 0,
    };

    let bitrate_row = if version_idx == 0 {
        layer_idx
    } else {
        if layer_idx == 0 { 3 } else { 4 }
    };

    let bitrate = bitrates[bitrate_row][bitrate_index as usize];

    // 声道数
    let channels = if channel_mode == 0b11 { 1 } else { 2 };

    Some((sample_rate, channels, bitrate))
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
    fn test_syncsafe_to_u32() {
        assert_eq!(syncsafe_to_u32(&[0x00, 0x00, 0x01, 0x7F]), 255);
        assert_eq!(syncsafe_to_u32(&[0x00, 0x00, 0x02, 0x00]), 256);
    }

    #[test]
    fn test_normalize_frame_id() {
        assert_eq!(normalize_frame_id("TT2", 2), "TIT2");
        assert_eq!(normalize_frame_id("TP1", 2), "TPE1");
        assert_eq!(normalize_frame_id("TAL", 2), "TALB");
        assert_eq!(normalize_frame_id("TIT2", 3), "TIT2");
    }

    #[test]
    fn test_parse_genre() {
        assert_eq!(parse_genre("(17)"), "Rock");
        assert_eq!(parse_genre("17"), "Rock");
        assert_eq!(parse_genre("Custom Genre"), "Custom Genre");
    }

    #[test]
    fn test_get_id3_genre() {
        assert_eq!(get_id3_genre(0), "Blues");
        assert_eq!(get_id3_genre(17), "Rock");
        assert_eq!(get_id3_genre(999), "Unknown(999)");
    }

    #[test]
    fn test_trim_null_bytes() {
        let data = b"Hello\0World";
        assert_eq!(trim_null_bytes(data), "Hello");
    }

    #[test]
    fn test_parse_mp3_frame_header() {
        // MPEG 1, Layer III, 128kbps, 44100Hz, Stereo
        let header = [0xFF, 0xFB, 0x92, 0x00];
        let result = parse_mp3_frame_header(&header);
        assert!(result.is_some());
        let (sample_rate, channels, bitrate) = result.unwrap();
        assert_eq!(sample_rate, 44100);
        assert_eq!(channels, 2);
        assert_eq!(bitrate, 128);
    }
}
