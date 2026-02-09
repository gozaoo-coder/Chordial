//! 音频元数据工具模块
//!
//! 提供编码处理、字节操作等通用工具

pub mod encoding;

pub use encoding::{
    auto_decode_text,
    decode_id3v2_text,
    safe_decode,
    is_valid_utf8,
    encode_text,
    to_utf8_bom,
    EncodingError,
};

use std::io::{Read, Seek, SeekFrom};

/// 读取指定长度的字节
pub fn read_bytes<R: Read>(reader: &mut R, len: usize) -> std::io::Result<Vec<u8>> {
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)?;
    Ok(buf)
}

/// 读取直到遇到分隔符
pub fn read_until<R: Read>(reader: &mut R, delimiter: u8) -> std::io::Result<Vec<u8>> {
    let mut result = Vec::new();
    let mut buf = [0u8; 1];

    loop {
        match reader.read_exact(&mut buf) {
            Ok(_) => {
                if buf[0] == delimiter {
                    break;
                }
                result.push(buf[0]);
            }
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e),
        }
    }

    Ok(result)
}

/// 读取 null 结尾的字符串
pub fn read_c_string<R: Read>(reader: &mut R) -> std::io::Result<String> {
    let bytes = read_until(reader, 0)?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

/// 安全地读取指定长度，如果不足则返回已读取的内容
pub fn read_bytes_safe<R: Read>(reader: &mut R, len: usize) -> std::io::Result<Vec<u8>> {
    let mut buf = vec![0u8; len];
    let mut total_read = 0;

    while total_read < len {
        match reader.read(&mut buf[total_read..])? {
            0 => break,
            n => total_read += n,
        }
    }

    buf.truncate(total_read);
    Ok(buf)
}

/// 跳过指定字节数
pub fn skip_bytes<R: Read + Seek>(reader: &mut R, count: u64) -> std::io::Result<()> {
    reader.seek(SeekFrom::Current(count as i64))?;
    Ok(())
}

/// 查找字节模式在数据中的位置
pub fn find_pattern(data: &[u8], pattern: &[u8]) -> Option<usize> {
    if pattern.is_empty() || data.len() < pattern.len() {
        return None;
    }

    data.windows(pattern.len()).position(|window| window == pattern)
}

/// 将大端字节序的 4 字节转换为 u32
pub fn be_u32(bytes: &[u8]) -> u32 {
    if bytes.len() >= 4 {
        u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
    } else {
        0
    }
}

/// 将小端字节序的 4 字节转换为 u32
pub fn le_u32(bytes: &[u8]) -> u32 {
    if bytes.len() >= 4 {
        u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
    } else {
        0
    }
}

/// 将大端字节序的 2 字节转换为 u16
pub fn be_u16(bytes: &[u8]) -> u16 {
    if bytes.len() >= 2 {
        u16::from_be_bytes([bytes[0], bytes[1]])
    } else {
        0
    }
}

/// 将小端字节序的 2 字节转换为 u16
pub fn le_u16(bytes: &[u8]) -> u16 {
    if bytes.len() >= 2 {
        u16::from_le_bytes([bytes[0], bytes[1]])
    } else {
        0
    }
}

/// 同步安全整数解码（用于 MP3 等格式）
pub fn syncsafe_to_u32(data: &[u8]) -> u32 {
    let mut result: u32 = 0;
    for &byte in data {
        result = (result << 7) | (byte as u32 & 0x7F);
    }
    result
}

/// 将 u32 转换为同步安全格式
pub fn u32_to_syncsafe(value: u32) -> [u8; 4] {
    [
        ((value >> 21) & 0x7F) as u8,
        ((value >> 14) & 0x7F) as u8,
        ((value >> 7) & 0x7F) as u8,
        (value & 0x7F) as u8,
    ]
}

/// 计算 MP3 帧的持续时间（毫秒）
pub fn mp3_frame_duration_ms(sample_rate: u32, mpeg_version: u8, layer: u8) -> u32 {
    let samples_per_frame = match (mpeg_version, layer) {
        (3, 3) => 384,   // MPEG 1, Layer 3
        (3, 2) => 1152,  // MPEG 1, Layer 2
        (3, 1) => 1152,  // MPEG 1, Layer 1
        (_, 3) => 384,   // MPEG 2/2.5, Layer 3
        (_, _) => 1152,  // MPEG 2/2.5, Layer 1/2
    };

    (samples_per_frame * 1000) / sample_rate
}

/// 解析音轨号字符串（支持 "1/10" 格式）
pub fn parse_track_number(s: &str) -> (Option<u32>, Option<u32>) {
    let parts: Vec<&str> = s.split('/').collect();
    let track = parts.get(0).and_then(|p| p.trim().parse().ok());
    let total = parts.get(1).and_then(|p| p.trim().parse().ok());
    (track, total)
}

/// 解析年份字符串
pub fn parse_year(s: &str) -> Option<u32> {
    s.trim()
        .chars()
        .take(4)
        .collect::<String>()
        .parse()
        .ok()
}

/// 去除字符串末尾的 null 字符
pub fn trim_nulls(s: &str) -> &str {
    s.trim_end_matches('\0')
}

/// 检查字节数组是否以指定前缀开头
pub fn starts_with(data: &[u8], prefix: &[u8]) -> bool {
    data.len() >= prefix.len() && &data[..prefix.len()] == prefix
}

/// 检查字节数组是否以指定后缀结尾
pub fn ends_with(data: &[u8], suffix: &[u8]) -> bool {
    data.len() >= suffix.len() && &data[data.len() - suffix.len()..] == suffix
}

/// 将字节转换为十六进制字符串（用于调试）
pub fn to_hex(data: &[u8]) -> String {
    data.iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_track_number() {
        assert_eq!(parse_track_number("5"), (Some(5), None));
        assert_eq!(parse_track_number("3/12"), (Some(3), Some(12)));
        assert_eq!(parse_track_number("01/10"), (Some(1), Some(10)));
    }

    #[test]
    fn test_syncsafe() {
        let value: u32 = 255;
        let encoded = u32_to_syncsafe(value);
        let decoded = syncsafe_to_u32(&encoded);
        assert_eq!(value, decoded);
    }

    #[test]
    fn test_be_u32() {
        let bytes = [0x00, 0x00, 0x01, 0x00];
        assert_eq!(be_u32(&bytes), 256);
    }

    #[test]
    fn test_find_pattern() {
        let data = b"Hello, World!";
        assert_eq!(find_pattern(data, b"World"), Some(7));
        assert_eq!(find_pattern(data, b"xyz"), None);
    }
}
