//! 元数据读取器trait定义

use std::path::Path;
use super::{AudioMetadata, Result};

/// 元数据读取器trait
/// 
/// 所有音频格式解析器都需要实现这个trait
pub trait MetadataReader {
    /// 从文件路径读取元数据
    fn read_metadata<P: AsRef<Path>>(path: P) -> Result<AudioMetadata>;

    /// 检查文件是否支持该解析器
    fn can_parse<P: AsRef<Path>>(path: P) -> bool;

    /// 获取支持的文件扩展名列表
    fn supported_extensions() -> &'static [&'static str];
}

/// 可扩展的元数据读取器
/// 
/// 允许注册自定义解析器
pub trait ExtensibleMetadataReader: MetadataReader {
    /// 注册新的解析器
    fn register_parser<T: MetadataReader + 'static>();
}