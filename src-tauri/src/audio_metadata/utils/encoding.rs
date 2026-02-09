//! 文本编码处理工具
//!
//! 提供自动编码检测和转换功能

use encoding_rs::{Encoding, UTF_8, GBK, GB18030, BIG5, EUC_JP, SHIFT_JIS, WINDOWS_1252};

/// 支持的编码列表（按优先级排序）
const ENCODINGS: &[&'static Encoding] = &[
    &UTF_8,
    &GB18030,
    &GBK,
    &BIG5,
    &EUC_JP,
    &SHIFT_JIS,
    &WINDOWS_1252,
];

/// 编码检测错误
#[derive(Debug, Clone)]
pub enum EncodingError {
    /// 无法检测有效编码
    DetectionFailed,
    /// 解码失败
    DecodeFailed(String),
    /// 无效的输入
    InvalidInput,
}

impl std::fmt::Display for EncodingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EncodingError::DetectionFailed => write!(f, "无法检测文本编码"),
            EncodingError::DecodeFailed(msg) => write!(f, "解码失败: {}", msg),
            EncodingError::InvalidInput => write!(f, "无效的输入数据"),
        }
    }
}

impl std::error::Error for EncodingError {}

/// 自动检测并解码文本
///
/// 尝试多种编码进行解码，返回第一个成功解码的结果
pub fn auto_decode_text(data: &[u8]) -> Result<String, EncodingError> {
    if data.is_empty() {
        return Ok(String::new());
    }

    // 首先尝试 UTF-8 BOM 检测
    if data.len() >= 3 && &data[0..3] == b"\xEF\xBB\xBF" {
        let (decoded, had_errors) = UTF_8.decode_without_bom_handling(&data[3..]);
        if had_errors {
            return Err(EncodingError::DecodeFailed("UTF-8 BOM".to_string()));
        }
        return Ok(decoded.into_owned());
    }

    // 尝试 UTF-16 LE BOM
    if data.len() >= 2 && &data[0..2] == b"\xFF\xFE" {
        return decode_utf16le(&data[2..]);
    }

    // 尝试 UTF-16 BE BOM
    if data.len() >= 2 && &data[0..2] == b"\xFE\xFF" {
        return decode_utf16be(&data[2..]);
    }

    // 尝试各种编码
    for encoding in ENCODINGS {
        // 检查是否为有效的该编码
        if let Some(decoded) = try_decode(data, encoding) {
            // 额外验证：解码后再编码应该能得到相似的结果
            let (re_encoded, _, had_errors) = encoding.encode(&decoded);
            if !had_errors && is_valid_text(&decoded) {
                return Ok(decoded);
            }
        }
    }

    // 最后的尝试：使用 UTF-8 有损解码
    let lossy = String::from_utf8_lossy(data);
    if !lossy.chars().all(|c| c == '\u{FFFD}') {
        return Ok(lossy.into_owned());
    }

    Err(EncodingError::DetectionFailed)
}

/// 尝试使用指定编码解码
fn try_decode(data: &[u8], encoding: &'static Encoding) -> Option<String> {
    let (decoded, had_errors) = encoding.decode_without_bom_handling(data);
    if had_errors {
        return None;
    }
    Some(decoded.into_owned())
}

/// 解码 UTF-16 LE
fn decode_utf16le(data: &[u8]) -> Result<String, EncodingError> {
    if data.len() % 2 != 0 {
        return Err(EncodingError::InvalidInput);
    }

    let u16_iter = data.chunks_exact(2).map(|chunk| {
        u16::from_le_bytes([chunk[0], chunk[1]])
    });

    char::decode_utf16(u16_iter)
        .collect::<Result<String, _>>()
        .map_err(|e| EncodingError::DecodeFailed(format!("UTF-16 LE: {}", e)))
}

/// 解码 UTF-16 BE
fn decode_utf16be(data: &[u8]) -> Result<String, EncodingError> {
    if data.len() % 2 != 0 {
        return Err(EncodingError::InvalidInput);
    }

    let u16_iter = data.chunks_exact(2).map(|chunk| {
        u16::from_be_bytes([chunk[0], chunk[1]])
    });

    char::decode_utf16(u16_iter)
        .collect::<Result<String, _>>()
        .map_err(|e| EncodingError::DecodeFailed(format!("UTF-16 BE: {}", e)))
}

/// 检查文本是否有效（不包含过多替换字符）
fn is_valid_text(text: &str) -> bool {
    if text.is_empty() {
        return true;
    }

    let replacement_count = text.chars().filter(|&c| c == '\u{FFFD}').count();
    let total_chars = text.chars().count();

    // 替换字符比例小于 5% 认为是有效文本
    (replacement_count as f64 / total_chars as f64) < 0.05
}

/// 解码 ID3v2 文本帧
///
/// ID3v2 文本帧的第一个字节表示编码：
/// - 0x00: ISO-8859-1
/// - 0x01: UTF-16 with BOM
/// - 0x02: UTF-16 BE without BOM
/// - 0x03: UTF-8
pub fn decode_id3v2_text(data: &[u8]) -> Result<String, EncodingError> {
    if data.is_empty() {
        return Ok(String::new());
    }

    let encoding_byte = data[0];
    let text_data = &data[1..];

    match encoding_byte {
        0x00 => {
            // ISO-8859-1 (Latin-1)
            Ok(text_data.iter().map(|&b| b as char).collect())
        }
        0x01 => {
            // UTF-16 with BOM
            if text_data.len() < 2 {
                return Ok(String::new());
            }
            if &text_data[0..2] == b"\xFE\xFF" {
                decode_utf16be(&text_data[2..])
            } else if &text_data[0..2] == b"\xFF\xFE" {
                decode_utf16le(&text_data[2..])
            } else {
                // 默认小端
                decode_utf16le(text_data)
            }
        }
        0x02 => {
            // UTF-16 BE without BOM
            decode_utf16be(text_data)
        }
        0x03 => {
            // UTF-8
            String::from_utf8(text_data.to_vec())
                .map_err(|e| EncodingError::DecodeFailed(format!("UTF-8: {}", e)))
        }
        _ => {
            // 未知编码，尝试自动检测
            auto_decode_text(data)
        }
    }
}

/// 安全地将字节转换为字符串，自动检测编码
///
/// 这是 `auto_decode_text` 的便捷包装，永远不会失败
pub fn safe_decode(data: &[u8]) -> String {
    auto_decode_text(data).unwrap_or_else(|_| {
        String::from_utf8_lossy(data).into_owned()
    })
}

/// 检测文本是否可能是 UTF-8
pub fn is_valid_utf8(data: &[u8]) -> bool {
    std::str::from_utf8(data).is_ok()
}

/// 尝试将文本编码为指定编码
pub fn encode_text(text: &str, encoding: &'static Encoding) -> Vec<u8> {
    let (encoded, _, _) = encoding.encode(text);
    encoded.into_owned()
}

/// 将字符串转换为 UTF-8 字节（带 BOM）
pub fn to_utf8_bom(text: &str) -> Vec<u8> {
    let mut result = vec![0xEF, 0xBB, 0xBF];
    result.extend_from_slice(text.as_bytes());
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utf8_decode() {
        let text = "Hello, 世界!";
        let bytes = text.as_bytes();
        assert_eq!(auto_decode_text(bytes).unwrap(), text);
    }

    #[test]
    fn test_utf8_bom_decode() {
        let text = "Hello";
        let mut bytes = vec![0xEF, 0xBB, 0xBF];
        bytes.extend_from_slice(text.as_bytes());
        assert_eq!(auto_decode_text(&bytes).unwrap(), text);
    }

    #[test]
    fn test_gbk_decode() {
        // GBK 编码的"你好"
        let gbk_bytes = vec![0xC4, 0xE3, 0xBA, 0xC3];
        let result = auto_decode_text(&gbk_bytes).unwrap();
        assert_eq!(result, "你好");
    }

    #[test]
    fn test_id3v2_utf8() {
        // ID3v2 UTF-8 编码
        let data = vec![0x03, 0x48, 0x65, 0x6C, 0x6C, 0x6F]; // 0x03 = UTF-8, "Hello"
        assert_eq!(decode_id3v2_text(&data).unwrap(), "Hello");
    }

    #[test]
    fn test_safe_decode() {
        let bytes = vec![0x80, 0x81, 0x82]; // 无效 UTF-8
        let result = safe_decode(&bytes);
        assert!(!result.is_empty());
    }
}
