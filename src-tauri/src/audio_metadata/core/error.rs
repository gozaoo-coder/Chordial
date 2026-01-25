//! 错误处理模块

use std::io;
use std::string::FromUtf8Error;

pub type Result<T> = std::result::Result<T, MetadataError>;

/// 元数据解析错误类型
#[derive(Debug, thiserror::Error)]
pub enum MetadataError {
    /// 不支持的音频格式
    #[error("不支持的音频格式: {0}")]
    UnsupportedFormat(String),

    /// 文件I/O错误
    #[error("文件读取错误: {0}")]
    IoError(#[from] io::Error),

    /// 文件格式错误
    #[error("文件格式错误: {0}")]
    InvalidFormat(String),

    /// 标签解析错误
    #[error("标签解析错误: {0}")]
    TagError(String),

    /// 编码错误
    #[error("文本编码错误: {0}")]
    EncodingError(String),

    /// UTF-8转换错误
    #[error("UTF-8转换错误: {0}")]
    Utf8Error(#[from] FromUtf8Error),

    /// 数据不足
    #[error("数据不足，需要 {needed} 字节但只有 {available} 字节")]
    InsufficientData { needed: usize, available: usize },

    /// 无效的标签版本
    #[error("无效的标签版本: {0}")]
    InvalidVersion(String),

    /// CRC校验失败
    #[error("CRC校验失败")]
    CrcError,

    /// 其他错误
    #[error("其他错误: {0}")]
    Other(String),
}

impl MetadataError {
    /// 创建格式错误
    pub fn invalid_format(msg: impl Into<String>) -> Self {
        MetadataError::InvalidFormat(msg.into())
    }

    /// 创建标签错误
    pub fn tag_error(msg: impl Into<String>) -> Self {
        MetadataError::TagError(msg.into())
    }

    /// 创建编码错误
    pub fn encoding_error(msg: impl Into<String>) -> Self {
        MetadataError::EncodingError(msg.into())
    }

    /// 创建数据不足错误
    pub fn insufficient_data(needed: usize, available: usize) -> Self {
        MetadataError::InsufficientData { needed, available }
    }
}

impl From<MetadataError> for String {
    fn from(error: MetadataError) -> Self {
        error.to_string()
    }
}