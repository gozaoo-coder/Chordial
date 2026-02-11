//! OGG Vorbis 音频格式读取器
//!
//! 支持 OGG 容器的 Vorbis 评论和 Opus 标签

use crate::audio_metadata::{
    core::{AudioFormat, AudioMetadata, Picture, PictureType},
    utils::encoding::auto_decode_text,
};
use crate::audio_metadata::MetadataError;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// OGG 元数据读取器
pub struct OggReader;

impl OggReader {
    /// 从任意实现了 Read + Seek 的读取器中读取 OGG 元数据
    pub fn read_from<R: Read + Seek>(mut reader: R) -> Result<AudioMetadata, MetadataError> {
        // 检查 OGG 标记
        let mut marker = [0u8; 4];
        reader.read_exact(&mut marker)
            .map_err(|e| MetadataError::IoError(e.to_string()))?;

        if &marker != b"OggS" {
            return Err(MetadataError::InvalidFormat("不是有效的 OGG 文件".to_string()));
        }

        // 重置文件位置
        reader.seek(SeekFrom::Start(0))
            .map_err(|e| MetadataError::IoError(e.to_string()))?;

        let mut metadata = AudioMetadata::new(AudioFormat::Ogg);

        // 读取第一个页（识别码）
        let first_page = read_ogg_page(&mut reader)?;
        if first_page.is_none() {
            return Err(MetadataError::InvalidFormat("无法读取 OGG 页".to_string()));
        }

        // 读取识别包数据
        let identification_packet = read_packet_data(&mut reader, &first_page.unwrap())?;

        // 检测编解码器类型
        let codec_type = detect_codec(&identification_packet);

        // 读取第二个页（评论）
        let second_page = read_ogg_page(&mut reader)?;
        if let Some(page) = second_page {
            let comment_packet = read_packet_data(&mut reader, &page)?;

            match codec_type {
                CodecType::Vorbis => {
                    parse_vorbis_comment(&comment_packet, &mut metadata)?;
                }
                CodecType::Opus => {
                    parse_opus_tags(&comment_packet, &mut metadata)?;
                }
                CodecType::Flac => {
                    parse_flac_in_ogg(&comment_packet, &mut metadata)?;
                }
                _ => {}
            }
        }

        // 解析技术信息
        parse_technical_info(&identification_packet, &mut metadata, codec_type)?;

        Ok(metadata)
    }
}

/// OGG 页头部
#[derive(Debug)]
struct OggPage {
    version: u8,
    header_type: u8,
    granule_position: i64,
    bitstream_serial: u32,
    page_sequence: u32,
    crc_checksum: u32,
    page_segments: u8,
    segment_table: Vec<u8>,
}

/// 读取 OGG 文件元数据
pub fn read_ogg_metadata(path: &Path) -> Result<AudioMetadata, MetadataError> {
    let file = File::open(path)
        .map_err(|e| MetadataError::IoError(e.to_string()))?;
    OggReader::read_from(file)
}

/// 编解码器类型
#[derive(Debug, Clone, Copy)]
enum CodecType {
    Vorbis,
    Opus,
    Flac,
    Unknown,
}

/// 检测编解码器类型
fn detect_codec(packet: &[u8]) -> CodecType {
    if packet.len() < 7 {
        return CodecType::Unknown;
    }

    // Vorbis: 包类型 (1) + "vorbis"
    if packet[0] == 1 && &packet[1..7] == b"vorbis" {
        return CodecType::Vorbis;
    }

    // Opus: "OpusHead"
    if packet.len() >= 8 && &packet[0..8] == b"OpusHead" {
        return CodecType::Opus;
    }

    // FLAC: "fLaC"
    if packet.len() >= 4 && &packet[0..4] == b"fLaC" {
        return CodecType::Flac;
    }

    CodecType::Unknown
}

/// 读取 OGG 页
fn read_ogg_page<R: Read + Seek>(reader: &mut R) -> Result<Option<OggPage>, MetadataError> {
    let mut capture_pattern = [0u8; 4];
    if reader.read_exact(&mut capture_pattern).is_err() {
        return Ok(None);
    }

    if &capture_pattern != b"OggS" {
        // 不是 OGG 页开始，返回 None
        return Ok(None);
    }

    let mut header = [0u8; 23];
    reader.read_exact(&mut header)
        .map_err(|e| MetadataError::IoError(e.to_string()))?;

    let version = header[0];
    let header_type = header[1];
    let granule_position = i64::from_le_bytes([
        header[2], header[3], header[4], header[5],
        header[6], header[7], header[8], header[9],
    ]);
    let bitstream_serial = u32::from_le_bytes([header[10], header[11], header[12], header[13]]);
    let page_sequence = u32::from_le_bytes([header[14], header[15], header[16], header[17]]);
    let crc_checksum = u32::from_le_bytes([header[18], header[19], header[20], header[21]]);
    let page_segments = header[22];

    // 读取段表
    let mut segment_table = vec![0u8; page_segments as usize];
    reader.read_exact(&mut segment_table)
        .map_err(|e| MetadataError::IoError(e.to_string()))?;

    Ok(Some(OggPage {
        version,
        header_type,
        granule_position,
        bitstream_serial,
        page_sequence,
        crc_checksum,
        page_segments,
        segment_table,
    }))
}

/// 读取包数据
fn read_packet_data<R: Read>(
    reader: &mut R,
    page: &OggPage,
) -> Result<Vec<u8>, MetadataError> {
    let total_size: usize = page.segment_table.iter().map(|&s| s as usize).sum();
    let mut data = vec![0u8; total_size];
    reader.read_exact(&mut data)
        .map_err(|e| MetadataError::IoError(e.to_string()))?;
    Ok(data)
}

/// 解析 Vorbis 评论
fn parse_vorbis_comment(
    packet: &[u8],
    metadata: &mut AudioMetadata,
) -> Result<(), MetadataError> {
    if packet.len() < 7 {
        return Ok(());
    }

    // 检查包类型 (3 = comment)
    if packet[0] != 3 {
        return Ok(());
    }

    // 检查 "vorbis" 标记
    if &packet[1..7] != b"vorbis" {
        return Ok(());
    }

    let mut pos = 7;

    // 读取 vendor string 长度（小端）
    if pos + 4 > packet.len() {
        return Ok(());
    }
    let vendor_len = u32::from_le_bytes([packet[pos], packet[pos+1], packet[pos+2], packet[pos+3]]);
    pos += 4;

    // 跳过 vendor string
    pos += vendor_len as usize;

    if pos + 4 > packet.len() {
        return Ok(());
    }

    // 读取用户评论数量
    let comment_count = u32::from_le_bytes([packet[pos], packet[pos+1], packet[pos+2], packet[pos+3]]);
    pos += 4;

    // 解析每个评论
    for _ in 0..comment_count {
        if pos + 4 > packet.len() {
            break;
        }

        let comment_len = u32::from_le_bytes([packet[pos], packet[pos+1], packet[pos+2], packet[pos+3]]);
        pos += 4;

        if pos + comment_len as usize > packet.len() {
            break;
        }

        let comment = &packet[pos..pos + comment_len as usize];
        pos += comment_len as usize;

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
        "ALBUMARTIST" => metadata.album_artist = Some(value.to_string()),
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
        "TRACKTOTAL" | "TOTALTRACKS" => {
            metadata.total_tracks = value.parse().ok();
        }
        "DISCNUMBER" => {
            let parts: Vec<&str> = value.split('/').collect();
            metadata.disc_number = parts[0].trim().parse().ok();
            if parts.len() > 1 {
                metadata.total_discs = parts[1].trim().parse().ok();
            }
        }
        "DISCTOTAL" | "TOTALDISCS" => {
            metadata.total_discs = value.parse().ok();
        }
        "GENRE" => metadata.genre = Some(value.to_string()),
        "COMPOSER" => metadata.composer = Some(value.to_string()),
        "LYRICIST" => metadata.lyricist = Some(value.to_string()),
        "COMMENT" | "DESCRIPTION" => metadata.comment = Some(value.to_string()),
        "LYRICS" | "UNSYNCEDLYRICS" => metadata.lyrics = Some(value.to_string()),
        "METADATA_BLOCK_PICTURE" => {
            // Base64 编码的 FLAC 风格图片
            if let Ok(picture_data) = base64_decode(value) {
                parse_flac_picture(&picture_data, metadata);
            }
        }
        _ => {}
    }
}

/// 解析 Opus 标签
fn parse_opus_tags(
    packet: &[u8],
    metadata: &mut AudioMetadata,
) -> Result<(), MetadataError> {
    if packet.len() < 8 {
        return Ok(());
    }

    // 检查 "OpusTags" 标记
    if &packet[0..8] != b"OpusTags" {
        return Ok(());
    }

    let mut pos = 8;

    // 读取 vendor string 长度（小端）
    if pos + 4 > packet.len() {
        return Ok(());
    }
    let vendor_len = u32::from_le_bytes([packet[pos], packet[pos+1], packet[pos+2], packet[pos+3]]);
    pos += 4;

    // 跳过 vendor string
    pos += vendor_len as usize;

    if pos + 4 > packet.len() {
        return Ok(());
    }

    // 读取用户评论数量
    let comment_count = u32::from_le_bytes([packet[pos], packet[pos+1], packet[pos+2], packet[pos+3]]);
    pos += 4;

    // 解析每个评论（与 Vorbis 相同格式）
    for _ in 0..comment_count {
        if pos + 4 > packet.len() {
            break;
        }

        let comment_len = u32::from_le_bytes([packet[pos], packet[pos+1], packet[pos+2], packet[pos+3]]);
        pos += 4;

        if pos + comment_len as usize > packet.len() {
            break;
        }

        let comment = &packet[pos..pos + comment_len as usize];
        pos += comment_len as usize;

        if let Ok(comment_str) = auto_decode_text(comment) {
            parse_vorbis_field(metadata, &comment_str);
        }
    }

    Ok(())
}

/// 解析 FLAC-in-OGG
fn parse_flac_in_ogg(
    _packet: &[u8],
    _metadata: &mut AudioMetadata,
) -> Result<(), MetadataError> {
    // FLAC in OGG 使用不同的封装方式
    // 这里简化处理，实际实现需要解析 FLAC 的元数据块
    Ok(())
}

/// 解析 FLAC 风格的图片
fn parse_flac_picture(data: &[u8], metadata: &mut AudioMetadata) {
    if data.len() < 32 {
        return;
    }

    let mut pos = 0;

    // 图片类型（4 字节，大端）
    let picture_type = u32::from_be_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]);
    pos += 4;

    // MIME 类型长度
    let mime_len = u32::from_be_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]) as usize;
    pos += 4;

    if pos + mime_len > data.len() {
        return;
    }
    let mime_type = String::from_utf8_lossy(&data[pos..pos + mime_len]).to_string();
    pos += mime_len;

    // 描述长度
    if pos + 4 > data.len() {
        return;
    }
    let desc_len = u32::from_be_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]) as usize;
    pos += 4;

    if pos + desc_len > data.len() {
        return;
    }
    let description = String::from_utf8_lossy(&data[pos..pos + desc_len]).to_string();
    pos += desc_len;

    // 跳过宽度、高度、颜色深度、索引颜色数（各 4 字节）
    pos += 16;

    if pos + 4 > data.len() {
        return;
    }

    // 图片数据长度
    let data_len = u32::from_be_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]) as usize;
    pos += 4;

    if pos + data_len > data.len() {
        return;
    }

    let picture_data = data[pos..pos + data_len].to_vec();

    let picture = Picture::new(
        PictureType::from_id3v2_type(picture_type as u8),
        mime_type,
        description,
        picture_data,
    );
    metadata.add_picture(picture);
}

/// 解析技术信息
fn parse_technical_info(
    packet: &[u8],
    metadata: &mut AudioMetadata,
    codec_type: CodecType,
) -> Result<(), MetadataError> {
    match codec_type {
        CodecType::Vorbis => {
            // Vorbis 识别包格式:
            // [包类型(1)] ["vorbis"(6)] [版本(4)] [声道(1)] [采样率(4)]
            // [比特率最大(4)] [比特率标称(4)] [比特率最小(4)] [块大小(1)]
            if packet.len() >= 29 {
                let channels = packet[11];
                let sample_rate = u32::from_le_bytes([packet[12], packet[13], packet[14], packet[15]]);
                let bitrate_nominal = u32::from_le_bytes([packet[20], packet[21], packet[22], packet[23]]);

                metadata.channels = Some(channels);
                metadata.sample_rate = Some(sample_rate);
                if bitrate_nominal > 0 {
                    metadata.bitrate = Some(bitrate_nominal / 1000);
                }
            }
        }
        CodecType::Opus => {
            // OpusHead 格式:
            // ["OpusHead"(8)] [版本(1)] [声道(1)] [预跳过(2)] [采样率(4)] [输出增益(2)] [映射族(1)]
            if packet.len() >= 19 {
                let channels = packet[9];
                // Opus 总是解码为 48kHz
                metadata.channels = Some(channels);
                metadata.sample_rate = Some(48000);
            }
        }
        _ => {}
    }

    Ok(())
}

/// 简单的 Base64 解码
fn base64_decode(input: &str) -> Result<Vec<u8>, ()> {
    const BASE64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut result = Vec::new();
    let mut buffer: u32 = 0;
    let mut bits_collected = 0;

    for c in input.chars() {
        if c == '=' {
            break;
        }

        let value = BASE64_CHARS.iter().position(|&b| b == c as u8);
        if let Some(val) = value {
            buffer = (buffer << 6) | val as u32;
            bits_collected += 6;

            if bits_collected >= 8 {
                bits_collected -= 8;
                result.push((buffer >> bits_collected) as u8);
                buffer &= (1 << bits_collected) - 1;
            }
        } else if c.is_whitespace() {
            continue;
        } else {
            return Err(());
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_decode() {
        assert_eq!(base64_decode("SGVsbG8=").unwrap(), b"Hello");
        assert_eq!(base64_decode("V29ybGQ=").unwrap(), b"World");
        assert_eq!(base64_decode("").unwrap(), Vec::<u8>::new());
    }

    #[test]
    fn test_parse_vorbis_field() {
        let mut metadata = AudioMetadata::new(AudioFormat::Ogg);

        parse_vorbis_field(&mut metadata, "TITLE=Test Song");
        assert_eq!(metadata.title, Some("Test Song".to_string()));

        parse_vorbis_field(&mut metadata, "ARTIST=Test Artist");
        assert_eq!(metadata.artist, Some("Test Artist".to_string()));

        parse_vorbis_field(&mut metadata, "TRACKNUMBER=3/10");
        assert_eq!(metadata.track_number, Some(3));
        assert_eq!(metadata.total_tracks, Some(10));
    }

    #[test]
    fn test_detect_codec() {
        let vorbis = vec![0x01, b'v', b'o', b'r', b'b', b'i', b's', 0x00];
        assert!(matches!(detect_codec(&vorbis), CodecType::Vorbis));

        let opus = vec![b'O', b'p', b'u', b's', b'H', b'e', b'a', b'd'];
        assert!(matches!(detect_codec(&opus), CodecType::Opus));
    }
}
