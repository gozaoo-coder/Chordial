//! 字符编码处理工具

use encoding_rs::{Encoding, UTF_8, UTF_16BE, UTF_16LE, WINDOWS_1252};
use crate::audio_metadata::core::MetadataError;

/// 文本编码类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextEncoding {
    Utf8,
    Utf16Be,
    Utf16Le,
    Latin1,
    Unknown,
}

impl TextEncoding {
    /// 从字节序标记(BOM)检测编码
    pub fn from_bom(bytes: &[u8]) -> Option<Self> {
        if bytes.len() >= 3 && &bytes[0..3] == b"\xEF\xBB\xBF" {
            Some(TextEncoding::Utf8)
        } else if bytes.len() >= 2 {
            match &bytes[0..2] {
                b"\xFE\xFF" => Some(TextEncoding::Utf16Be),
                b"\xFF\xFE" => Some(TextEncoding::Utf16Le),
                _ => None,
            }
        } else {
            None
        }
    }

    /// 从ID3v2的文本编码字节检测
    pub fn from_id3v2_byte(byte: u8) -> Self {
        match byte {
            0 => TextEncoding::Latin1,
            1 => TextEncoding::Utf16Be,
            2 => TextEncoding::Utf16Le,
            3 => TextEncoding::Utf8,
            _ => TextEncoding::Unknown,
        }
    }

    /// 转换为encoding_rs的Encoding引用
    pub fn to_encoding(&self) -> &'static Encoding {
        match self {
            TextEncoding::Utf8 => UTF_8,
            TextEncoding::Utf16Be => UTF_16BE,
            TextEncoding::Utf16Le => UTF_16LE,
            TextEncoding::Latin1 => WINDOWS_1252,
            TextEncoding::Unknown => UTF_8,
        }
    }
}

/// 自动检测文本编码
pub fn detect_encoding(bytes: &[u8]) -> TextEncoding {
    // 首先检查BOM
    if let Some(encoding) = TextEncoding::from_bom(bytes) {
        return encoding;
    }

    // 尝试UTF-8检测
    if is_valid_utf8(bytes) {
        return TextEncoding::Utf8;
    }

    // 检查是否有UTF-16的特征
    if bytes.len() >= 2 && bytes.len() % 2 == 0 {
        if is_valid_utf16be(bytes) {
            return TextEncoding::Utf16Be;
        }
        if is_valid_utf16le(bytes) {
            return TextEncoding::Utf16Le;
        }
    }

    // 默认返回Latin1
    TextEncoding::Latin1
}

/// 解码文本数据
pub fn decode_text(bytes: &[u8], encoding: TextEncoding) -> Result<String, MetadataError> {
    let encoding_ref = encoding.to_encoding();
    let (result, _, had_errors) = encoding_ref.decode(bytes);
    
    if had_errors {
        // 如果有错误，尝试用UTF-8再解码一次
        if encoding != TextEncoding::Utf8 && is_valid_utf8(bytes) {
            return Ok(String::from_utf8_lossy(bytes).into_owned());
        }
        return Err(MetadataError::encoding_error("文本解码失败"));
    }

    Ok(result.into_owned())
}

/// 自动检测并解码文本
pub fn auto_decode_text(bytes: &[u8]) -> Result<String, MetadataError> {
    let encoding = detect_encoding(bytes);
    
    // 如果有BOM，跳过BOM部分
    let data_to_decode = if let Some(bom_len) = get_bom_length(bytes) {
        &bytes[bom_len..]
    } else {
        bytes
    };

    decode_text(data_to_decode, encoding)
}

/// 检查是否为有效的UTF-8
fn is_valid_utf8(bytes: &[u8]) -> bool {
    std::str::from_utf8(bytes).is_ok()
}

/// 检查是否为有效的UTF-16BE
fn is_valid_utf16be(bytes: &[u8]) -> bool {
    if bytes.len() % 2 != 0 {
        return false;
    }
    
    for chunk in bytes.chunks(2) {
        if chunk.len() != 2 {
            return false;
        }
        // 检查是否有代理对
        let code_unit = u16::from_be_bytes([chunk[0], chunk[1]]);
        if is_surrogate(code_unit) && !is_valid_surrogate_pair(bytes, code_unit) {
            return false;
        }
    }
    true
}

/// 检查是否为有效的UTF-16LE
fn is_valid_utf16le(bytes: &[u8]) -> bool {
    if bytes.len() % 2 != 0 {
        return false;
    }
    
    for chunk in bytes.chunks(2) {
        if chunk.len() != 2 {
            return false;
        }
        // 检查是否有代理对
        let code_unit = u16::from_le_bytes([chunk[0], chunk[1]]);
        if is_surrogate(code_unit) && !is_valid_surrogate_pair(bytes, code_unit) {
            return false;
        }
    }
    true
}

/// 检查是否为代理对
fn is_surrogate(code_unit: u16) -> bool {
    (0xD800..=0xDFFF).contains(&code_unit)
}

/// 检查是否为有效的代理对（简化版本）
fn is_valid_surrogate_pair(_bytes: &[u8], _code_unit: u16) -> bool {
    // 这里简化处理，实际应该检查完整的代理对
    true
}

/// 获取BOM长度
fn get_bom_length(bytes: &[u8]) -> Option<usize> {
    if bytes.len() >= 3 && &bytes[0..3] == b"\xEF\xBB\xBF" {
        Some(3)
    } else if bytes.len() >= 2 {
        match &bytes[0..2] {
            b"\xFE\xFF" | b"\xFF\xFE" => Some(2),
            _ => None,
        }
    } else {
        None
    }
}

/// 去除字符串末尾的空字节
pub fn trim_null_bytes(s: &str) -> &str {
    s.trim_end_matches('\0')
}

/// 安全转换字节到字符串
pub fn safe_bytes_to_string(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return String::new();
    }

    match auto_decode_text(bytes) {
        Ok(s) => s.trim().to_string(),
        Err(_) => {
            // 如果所有方法都失败，使用有损转换
            String::from_utf8_lossy(bytes).trim().to_string()
        }
    }
}