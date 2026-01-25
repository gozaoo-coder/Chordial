//! 工具模块

pub mod encoding;
pub mod io;

pub use encoding::{detect_encoding, decode_text, safe_bytes_to_string, auto_decode_text};
pub use io::{read_bytes, read_u8, read_u16_be, read_u16_le, read_u32_be, read_u32_le, read_string, peek_file_header, crc32};