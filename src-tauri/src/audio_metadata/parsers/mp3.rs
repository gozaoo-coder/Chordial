//! MP3格式解析器
//! 
//! 支持ID3v1.0/1.1和ID3v2.2/2.3/2.4标签解析

use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

use super::super::core::{AudioMetadata, AudioFormat, MetadataReader, MetadataError, Result};
use super::super::core::types::{Picture, PictureType, LyricLine};
use super::super::utils::read_bytes;

/// ID3v2签名
const ID3V2_SIGNATURE: &[u8] = b"ID3";
/// ID3v1签名
const ID3V1_SIGNATURE: &[u8] = b"TAG";

/// MP3解析器
pub struct Mp3Parser;

impl MetadataReader for Mp3Parser {
    fn read_metadata<P: AsRef<Path>>(path: P) -> Result<AudioMetadata> {
        let path = path.as_ref();
        let file = File::open(path)
            .map_err(|e| MetadataError::IoError(e))?;
        
        let mut reader = BufReader::new(file);
        let mut metadata = AudioMetadata::new(AudioFormat::Mp3);
        
        // 先尝试读取ID3v2标签
        let id3v2_size = read_id3v2_tags(&mut reader, &mut metadata)?;
        
        // 记录文件大小用于ID3v1查找
        let file_size = reader.seek(SeekFrom::End(0))
            .map_err(|e| MetadataError::IoError(e))?;
        
        // 尝试读取ID3v1标签
        read_id3v1_tag(&mut reader, file_size, &mut metadata)?;
        
        // 尝试读取音频帧信息以获取时长等
        reader.seek(SeekFrom::Start(id3v2_size as u64))
            .map_err(|e| MetadataError::IoError(e))?;
        
        Ok(metadata)
    }

    fn can_parse<P: AsRef<Path>>(path: P) -> bool {
        if let Ok(header) = super::super::utils::peek_file_header(path.as_ref(), 10) {
            // 检查ID3v2或音频帧签名
            &header[0..3] == ID3V2_SIGNATURE || is_mp3_frame_header(&header[0..4])
        } else {
            false
        }
    }

    fn supported_extensions() -> &'static [&'static str] {
        &["mp3", "mp2", "mpeg"]
    }
}

/// 检查是否为MP3帧头
fn is_mp3_frame_header(bytes: &[u8]) -> bool {
    if bytes.len() < 4 {
        return false;
    }
    // MP3帧头同步字：11个1位 (0xFF 0xE0 ~ 0xFF 0xEF)
    bytes[0] == 0xFF && (bytes[1] & 0xE0) == 0xE0
}

/// 读取ID3v2标签
fn read_id3v2_tags<R: Read + Seek>(reader: &mut R, metadata: &mut AudioMetadata) -> Result<usize> {
    let mut header = [0u8; 10];
    reader.read_exact(&mut header)
        .map_err(|e| MetadataError::IoError(e))?;
    
    // 检查是否是ID3v2标签
    if &header[0..3] != ID3V2_SIGNATURE {
        return Ok(0);
    }
    
    // 读取ID3v2版本
    let major = header[3];
    let _minor = header[4];
    
    // 计算标签大小（同步安全整数 - 每个字节只使用低7位）
    let size = ((header[6] as u32) << 21) | 
               ((header[7] as u32) << 14) | 
               ((header[8] as u32) << 7) | 
               (header[9] as u32);
    
    let total_size = size as usize + 10;
    
    // 读取扩展头部（如果存在）
    let ext_header_size = if major >= 4 && (header[5] & 0x10) != 0 {
        // ID3v2.4: 扩展头部大小是同步安全整数
        let mut ext_header = [0u8; 4];
        reader.read_exact(&mut ext_header)
            .map_err(|e| MetadataError::IoError(e))?;
        let ext_size = ((ext_header[0] as u32) << 21) | 
                      ((ext_header[1] as u32) << 14) | 
                      ((ext_header[2] as u32) << 7) | 
                      (ext_header[3] as u32);
        (ext_size as usize) + 4
    } else if major == 3 && (header[5] & 0x40) != 0 {
        // ID3v2.3: 扩展头部大小是普通的32位整数，不包括长度字段本身
        let mut ext_header = [0u8; 4];
        reader.read_exact(&mut ext_header)
            .map_err(|e| MetadataError::IoError(e))?;
        u32::from_be_bytes(ext_header) as usize
    } else {
        0
    };
    
    let mut pos = 10 + ext_header_size;
    let end_pos = total_size;
    
    // 读取帧
    while pos < end_pos {
        let frame_header = peek_frame_header(reader, major)?;
        if frame_header.is_none() {
            break;
        }
        let (frame_id, frame_size, flags) = frame_header.unwrap();
        
        if frame_id == [0u8; 4] || (major == 4 && frame_id[0] == 0) {
            break;
        }
        
        pos += 10;
        
        if pos + frame_size > end_pos {
            break;
        }
        
        let frame_data = read_bytes(reader, frame_size)?;
        
        parse_id3v2_frame(major, &frame_id, &frame_data, flags, metadata)?;
        
        pos += frame_size;
    }
    
    Ok(total_size)
}

/// 偷看帧头
fn peek_frame_header<R: Read + Seek>(reader: &mut R, id3_version: u8) -> Result<Option<([u8; 4], usize, u16)>> {
    let mut header = [0u8; 10];
    match reader.read_exact(&mut header) {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(MetadataError::IoError(e)),
    }
    
    reader.seek(SeekFrom::Current(-10))
        .map_err(|e| MetadataError::IoError(e))?;
    
    // 检查是否有效的帧ID（字母数字）
    if !header[0].is_ascii_alphanumeric() {
        return Ok(None);
    }
    
    let frameid = [header[0], header[1], header[2], header[3]];
    
    // ID3v2.4 使用同步安全整数作为帧大小，ID3v2.3 使用普通32位整数
    let frame_size = if id3_version >= 4 {
        ((header[4] as u32) << 21) |
        ((header[5] as u32) << 14) |
        ((header[6] as u32) << 7) |
        (header[7] as u32)
    } else {
        u32::from_be_bytes([header[4], header[5], header[6], header[7]])
    };
    
    let flags = u16::from_be_bytes([header[8], header[9]]);
    
    Ok(Some((frameid, frame_size as usize, flags)))
}

/// 解析ID3v2帧
fn parse_id3v2_frame(major: u8, frame_id: &[u8; 4], data: &[u8], _flags: u16, 
                     metadata: &mut AudioMetadata) -> Result<()> {
    let frame_id_str = std::str::from_utf8(frame_id).unwrap_or("");
    
    // 检查是否是文本帧（T开头的4字节ID）
    if frame_id_str.starts_with('T') && frame_id_str != "TXXX" && !data.is_empty() {
        let encoding = data[0];
        let text_data = &data[1..];
        let text = decode_id3v2_text(encoding, text_data)?;
        
        match frame_id_str {
            "TIT1" => {}, // 内容组描述
            "TIT2" => metadata.title = Some(text),
            "TIT3" => {}, // 副标题
            "TPE1" => metadata.artist = Some(text),
            "TPE2" => metadata.album_artist = Some(text),
            "TPE3" => {}, // 指挥
            "TPE4" => {}, // 翻译者
            "TALB" => metadata.album = Some(text),
            "TYER" | "YEAR" => {
                if let Ok(year) = text.parse::<u32>() {
                    metadata.year = Some(year);
                }
            }
            "TDAT" => {}, // 日期 (DDMM)
            "TRCK" | "TRK" => parse_track_number(&text, metadata),
            "TPOS" | "TPA" => parse_disc_number(&text, metadata),
            "TCON" => metadata.genre = Some(text),
            "COMM" | "COM" => {
                if let Some(desc_end) = data[1..].iter().position(|&b| b == 0x00) {
                    let desc = std::str::from_utf8(&data[1..1+desc_end]).unwrap_or("");
                    if desc.is_empty() || desc == "ENG" {
                        if metadata.comment.is_none() {
                            metadata.comment = Some(text[desc.len()+1..].to_string());
                        }
                    }
                }
            }
            "TCOP" => {}, // 版权
            "TPRO" => {}, // 制作
            "TPUB" => {}, // 发行商
            "TENC" => {}, // 编码者
            "TLEN" => {
                if let Ok(len) = text.parse::<u64>() {
                    metadata.duration = Some(std::time::Duration::from_millis(len));
                }
            }
            _ => {
                // 未知文本帧，保存到extra_tags
                if !text.is_empty() {
                    metadata.extra_tags.insert(frame_id_str.to_string(), text);
                }
            }
        }
    }
    // 图片帧
    else if (major >= 4 && *frame_id == [b'P', b'I', b'C', b'\0']) || 
            (major == 3 && *frame_id == *b"APIC") ||
            (major == 2 && *frame_id == [b'P', b'I', b'C', b'\0']) {
        parse_picture_frame(major, data, metadata)?;
    }
    // 用户定义文本帧
    else if frame_id_str == "TXXX" {
        parse_txxx_frame(data, metadata)?;
    }
    // 流行度计量
    else if frame_id_str == "POPM" || frame_id_str == "POP" {
        parse_popm_frame(data, metadata)?;
    }
    // 非同步歌词 (USLT)
    else if frame_id_str == "USLT" {
        parse_uslt_frame(data, metadata)?;
    }
    // 同步歌词 (SYLT)
    else if frame_id_str == "SYLT" {
        parse_sylt_frame(data, metadata)?;
    }
    
    Ok(())
}

/// 解析ID3v2文本
fn decode_id3v2_text(encoding: u8, data: &[u8]) -> Result<String> {
    use super::super::utils::encoding::TextEncoding;
    
    let text_encoding = TextEncoding::from_id3v2_byte(encoding);
    let (text, _, _) = text_encoding.to_encoding().decode(data);
    Ok(text.trim().to_string())
}

/// 解析音轨号
fn parse_track_number(text: &str, metadata: &mut AudioMetadata) {
    let parts: Vec<&str> = text.split('/').collect();
    if let Ok(num) = parts[0].parse::<u32>() {
        metadata.track_number = Some(num);
    }
    if parts.len() > 1 {
        if let Ok(total) = parts[1].parse::<u32>() {
            metadata.total_tracks = Some(total);
        }
    }
}

/// 解析光盘号
fn parse_disc_number(text: &str, metadata: &mut AudioMetadata) {
    let parts: Vec<&str> = text.split('/').collect();
    if let Ok(num) = parts[0].parse::<u32>() {
        metadata.disc_number = Some(num);
    }
    if parts.len() > 1 {
        if let Ok(total) = parts[1].parse::<u32>() {
            metadata.total_discs = Some(total);
        }
    }
}

/// 解析图片帧
fn parse_picture_frame(major: u8, data: &[u8], metadata: &mut AudioMetadata) -> Result<()> {
    let (mime_type, desc, picture_data) = if major >= 4 {
        // ID3v2.4 APIC
        let encoding = data[0];
        let mime_end = memchr::memchr(b'\0', &data[1..]).unwrap_or(data.len() - 1);
        let mime_type = std::str::from_utf8(&data[1..1+mime_end]).unwrap_or("image/unknown");
        let desc_start = 1 + mime_end + 1;
        let desc_end = memchr::memchr(b'\0', &data[desc_start..]).unwrap_or(data.len() - desc_start);
        let description = if desc_end > 0 {
            Some(decode_id3v2_text(encoding, &data[desc_start..desc_start+desc_end])?)
        } else {
            None
        };
        let _pic_type = data[desc_start + desc_end + 1];
        let picture_data = &data[desc_start + desc_end + 2..];
        
        (mime_type, description, picture_data)
    } else {
        // ID3v2.3 PIC
        let encoding = data[0];
        let format_end = memchr::memchr(b'\0', &data[1..]).unwrap_or(data.len() - 1);
        let format = std::str::from_utf8(&data[1..1+format_end]).unwrap_or("image/unknown");
        let _pic_type = data[1 + format_end + 1];
        let desc_start = 1 + format_end + 2;
        let desc_end = memchr::memchr(b'\0', &data[desc_start..]).unwrap_or(data.len() - desc_start);
        let description = if desc_end > 0 {
            Some(decode_id3v2_text(encoding, &data[desc_start..desc_start+desc_end])?)
        } else {
            None
        };
        let picture_data = &data[desc_start + desc_end + 1..];
        
        (format, description, picture_data)
    };
    
    let picture_type = PictureType::from_id3_code(data[0]);
    let mut picture = Picture::new(picture_type, mime_type.to_string(), picture_data.to_vec());
    picture.description = desc;
    
    metadata.pictures.push(picture);
    
    Ok(())
}

/// 解析TXXX帧
fn parse_txxx_frame(data: &[u8], metadata: &mut AudioMetadata) -> Result<()> {
    if data.len() < 2 {
        return Ok(());
    }
    
    let encoding = data[0];
    let content = &data[1..];
    
    if let Some(null_pos) = memchr::memchr(b'\0', content) {
        let description = std::str::from_utf8(&content[..null_pos]).unwrap_or("");
        let value = decode_id3v2_text(encoding, &content[null_pos+1..])?;
        
        match description.to_uppercase().as_str() {
            "ALBUM ARTIST" | "ALBUMARTIST" => metadata.album_artist = Some(value),
            "COMPOSER" => metadata.composer = Some(value),
            "COMMENT" => metadata.comment = Some(value),
            _ => {
                metadata.extra_tags.insert(description.to_string(), value);
            }
        }
    }
    
    Ok(())
}

/// 解析POPM帧
fn parse_popm_frame(data: &[u8], metadata: &mut AudioMetadata) -> Result<()> {
    let email_end = memchr::memchr(b'\0', data).unwrap_or(data.len());
    let _email = std::str::from_utf8(&data[..email_end]).unwrap_or("");
    
    if email_end < data.len() - 1 {
        let rating = data[email_end + 1];
        let rating = (rating as f64 / 255.0 * 5.0) as u32;
        metadata.extra_tags.insert("rating".to_string(), rating.to_string());
    }
    
    Ok(())
}

/// 解析USLT帧（非同步歌词）
fn parse_uslt_frame(data: &[u8], metadata: &mut AudioMetadata) -> Result<()> {
    if data.len() < 4 {
        return Ok(());
    }
    
    let encoding = data[0];
    let lang_start = 1;
    let lang_end = lang_start + 3;
    
    if lang_end > data.len() {
        return Ok(());
    }
    
    let _language = std::str::from_utf8(&data[lang_start..lang_end]).unwrap_or("");
    
    let desc_end = memchr::memchr(b'\0', &data[lang_end..]).map(|p| lang_end + p).unwrap_or(data.len());
    let _description = std::str::from_utf8(&data[lang_end..desc_end]).unwrap_or("");
    
    let lyrics_text = if desc_end + 1 < data.len() {
        decode_id3v2_text(encoding, &data[desc_end + 1..])?
    } else {
        String::new()
    };
    
    if !lyrics_text.is_empty() && metadata.lyrics.is_none() {
        metadata.lyrics = Some(lyrics_text);
    }
    
    Ok(())
}

/// 解析SYLT帧（同步歌词）
fn parse_sylt_frame(data: &[u8], metadata: &mut AudioMetadata) -> Result<()> {
    if data.len() < 6 {
        return Ok(());
    }
    
    let encoding = data[0];
    let lang_start = 1;
    let lang_end = lang_start + 3;
    
    if lang_end > data.len() {
        return Ok(());
    }
    
    let _language = std::str::from_utf8(&data[lang_start..lang_end]).unwrap_or("");
    
    let _format_byte = data[lang_end];
    let _content_type = data[lang_end + 1];
    
    let desc_end = memchr::memchr(b'\0', &data[lang_end + 2..]).map(|p| lang_end + 2 + p).unwrap_or(data.len());
    let _description = std::str::from_utf8(&data[lang_end + 2..desc_end]).unwrap_or("");
    
    let sylt_data = if desc_end + 1 < data.len() {
        &data[desc_end + 1..]
    } else {
        return Ok(());
    };
    
    let mut lyrics_lines = Vec::new();
    let mut offset = 0;
    
    while offset + 4 <= sylt_data.len() {
        let timestamp = u32::from_be_bytes([sylt_data[offset], sylt_data[offset + 1], sylt_data[offset + 2], sylt_data[offset + 3]]);
        let text_start = offset + 4;
        let text_end = memchr::memchr(b'\0', sylt_data).map(|p| text_start + p).unwrap_or(sylt_data.len());
        
        if text_start >= sylt_data.len() {
            break;
        }
        
        let text = decode_id3v2_text(encoding, &sylt_data[text_start..text_end.min(sylt_data.len())])?;
        
        if !text.is_empty() {
            let duration = std::time::Duration::from_millis(timestamp as u64);
            lyrics_lines.push(LyricLine::new(duration, text));
        }
        
        if text_end >= sylt_data.len() {
            break;
        }
        offset = text_end + 1;
    }
    
    if !lyrics_lines.is_empty() {
        metadata.synced_lyrics = Some(lyrics_lines);
    }
    
    Ok(())
}

/// 读取ID3v1标签
fn read_id3v1_tag<R: Read + Seek>(reader: &mut R, file_size: u64, metadata: &mut AudioMetadata) -> Result<()> {
    if file_size < 128 {
        return Ok(());
    }
    
    reader.seek(SeekFrom::End(-128))
        .map_err(|e| MetadataError::IoError(e))?;
    
    let mut tag = [0u8; 128];
    reader.read_exact(&mut tag)
        .map_err(|e| MetadataError::IoError(e))?;
    
    if &tag[0..3] != ID3V1_SIGNATURE {
        return Ok(());
    }
    
    // ID3v1标签 (ISO-8859-1编码)
    let decode_single_byte = |data: &[u8; 30]| -> String {
        let s = std::str::from_utf8(data).unwrap_or("");
        s.trim_end_matches('\0').trim().to_string()
    };
    
    let title = decode_single_byte(&tag[3..33].try_into().unwrap());
    let artist = decode_single_byte(&tag[33..63].try_into().unwrap());
    let album = decode_single_byte(&tag[63..93].try_into().unwrap());
    
    let year = std::str::from_utf8(&tag[93..97])
        .unwrap_or("")
        .trim()
        .to_string();
    
    if !title.is_empty() && metadata.title.is_none() {
        metadata.title = Some(title);
    }
    if !artist.is_empty() && metadata.artist.is_none() {
        metadata.artist = Some(artist);
    }
    if !album.is_empty() && metadata.album.is_none() {
        metadata.album = Some(album);
    }
    if !year.is_empty() {
        if let Ok(y) = year.parse::<u32>() {
            if metadata.year.is_none() {
                metadata.year = Some(y);
            }
        }
    }
    
    // ID3v1.1 音轨号和流派
    let track = tag[125];
    let genre = tag[127];
    
    if track > 0 && metadata.track_number.is_none() {
        metadata.track_number = Some(track as u32);
    }
    
    if genre < 148 {
        let genres = [
            "Blues", "Classic Rock", "Country", "Dance", "Disco", "Funk", "Grunge",
            "Hip-Hop", "Jazz", "Metal", "New Age", "Oldies", "Other", "Pop", "R&B",
            "Rap", "Reggae", "Rock", "Techno", "Industrial", "Alternative", "Ska",
            "Death Metal", "Pranks", "Soundtrack", "Euro-Techno", "Ambient", "Trip-Hop",
            "Vocal", "Jazz+Funk", "Fusion", "Trance", "Classical", "Instrumental",
            "Acid", "House", "Game", "Sound Clip", "Gospel", "Noise", "AlternRock",
            "Bass", "Soul", "Punk", "Space", "Medieval", "Bluegrass", "Celtic",
            "Bluegrass", "Folk", "Folk-Rock", "National Folk", "Swing", "Fast Fusion",
            "Bebob", "Latin", "Revival", "Celtic", "Pop/Funk", "Jungle", "Native American",
            "Cabaret", "New Wave", "Psychadelic", "Rave", "Showtunes", "Trailer", "Lo-Fi",
            "Tribal", "Acid Punk", "Acid Jazz", "Polka", "Retro", "Rock'n'Roll", "Hard Rock",
        ];
        
        if (genre as usize) < genres.len() {
            if metadata.genre.is_none() {
                metadata.genre = Some(genres[genre as usize].to_string());
            }
        }
    }
    
    Ok(())
}