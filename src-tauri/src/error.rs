//! 统一错误处理模块
//!
//! 为整个应用提供一致的错误类型和错误转换

use thiserror::Error;
use serde::Serialize;

/// 应用主错误类型
#[derive(Error, Debug, Clone, Serialize)]
#[serde(tag = "type", content = "message")]
pub enum AppError {
    /// IO 错误
    #[error("IO 错误: {0}")]
    Io(String),

    /// 文件未找到
    #[error("文件未找到: {0}")]
    FileNotFound(String),

    /// 无效的文件格式
    #[error("无效的文件格式: {0}")]
    InvalidFormat(String),

    /// 不支持的格式
    #[error("不支持的格式: {0}")]
    UnsupportedFormat(String),

    /// 元数据读取错误
    #[error("元数据读取错误: {0}")]
    MetadataError(String),

    /// 解析错误
    #[error("解析错误: {0}")]
    ParseError(String),

    /// 缓存错误
    #[error("缓存错误: {0}")]
    CacheError(String),

    /// WebDAV 错误
    #[error("WebDAV 错误: {0}")]
    WebDavError(String),

    /// 网络错误
    #[error("网络错误: {0}")]
    NetworkError(String),

    /// 配置错误
    #[error("配置错误: {0}")]
    ConfigError(String),

    /// 序列化/反序列化错误
    #[error("数据序列化错误: {0}")]
    SerializationError(String),

    /// 任务被取消
    #[error("操作被取消")]
    Cancelled,

    /// 未知错误
    #[error("未知错误: {0}")]
    Unknown(String),
}

impl AppError {
    /// 创建 IO 错误
    pub fn io<S: Into<String>>(msg: S) -> Self {
        AppError::Io(msg.into())
    }

    /// 创建文件未找到错误
    pub fn not_found<S: Into<String>>(path: S) -> Self {
        AppError::FileNotFound(path.into())
    }

    /// 创建无效格式错误
    pub fn invalid_format<S: Into<String>>(msg: S) -> Self {
        AppError::InvalidFormat(msg.into())
    }

    /// 创建不支持格式错误
    pub fn unsupported<S: Into<String>>(msg: S) -> Self {
        AppError::UnsupportedFormat(msg.into())
    }

    /// 创建元数据错误
    pub fn metadata<S: Into<String>>(msg: S) -> Self {
        AppError::MetadataError(msg.into())
    }

    /// 创建解析错误
    pub fn parse<S: Into<String>>(msg: S) -> Self {
        AppError::ParseError(msg.into())
    }

    /// 创建缓存错误
    pub fn cache<S: Into<String>>(msg: S) -> Self {
        AppError::CacheError(msg.into())
    }

    /// 创建 WebDAV 错误
    pub fn webdav<S: Into<String>>(msg: S) -> Self {
        AppError::WebDavError(msg.into())
    }

    /// 创建网络错误
    pub fn network<S: Into<String>>(msg: S) -> Self {
        AppError::NetworkError(msg.into())
    }

    /// 创建配置错误
    pub fn config<S: Into<String>>(msg: S) -> Self {
        AppError::ConfigError(msg.into())
    }

    /// 创建序列化错误
    pub fn serialization<S: Into<String>>(msg: S) -> Self {
        AppError::SerializationError(msg.into())
    }

    /// 创建未知错误
    pub fn unknown<S: Into<String>>(msg: S) -> Self {
        AppError::Unknown(msg.into())
    }

    /// 获取错误代码（用于前端识别）
    pub fn code(&self) -> &'static str {
        match self {
            AppError::Io(_) => "IO_ERROR",
            AppError::FileNotFound(_) => "FILE_NOT_FOUND",
            AppError::InvalidFormat(_) => "INVALID_FORMAT",
            AppError::UnsupportedFormat(_) => "UNSUPPORTED_FORMAT",
            AppError::MetadataError(_) => "METADATA_ERROR",
            AppError::ParseError(_) => "PARSE_ERROR",
            AppError::CacheError(_) => "CACHE_ERROR",
            AppError::WebDavError(_) => "WEBDAV_ERROR",
            AppError::NetworkError(_) => "NETWORK_ERROR",
            AppError::ConfigError(_) => "CONFIG_ERROR",
            AppError::SerializationError(_) => "SERIALIZATION_ERROR",
            AppError::Cancelled => "CANCELLED",
            AppError::Unknown(_) => "UNKNOWN",
        }
    }

    /// 检查是否为可恢复错误
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            AppError::FileNotFound(_) | AppError::Cancelled | AppError::NetworkError(_)
        )
    }
}

// 标准 IO 错误转换
impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        match e.kind() {
            std::io::ErrorKind::NotFound => AppError::FileNotFound(e.to_string()),
            _ => AppError::Io(e.to_string()),
        }
    }
}

// 序列化错误转换
impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::SerializationError(e.to_string())
    }
}

// 正则表达式错误转换
impl From<regex::Error> for AppError {
    fn from(e: regex::Error) -> Self {
        AppError::ParseError(e.to_string())
    }
}

// 编码错误转换
impl From<encoding_rs::CoderResult> for AppError {
    fn from(_: encoding_rs::CoderResult) -> Self {
        AppError::ParseError("文本编码错误".to_string())
    }
}

/// 应用结果类型别名
pub type AppResult<T> = std::result::Result<T, AppError>;

/// 错误上下文扩展 trait
pub trait ResultExt<T> {
    /// 添加错误上下文
    fn context<C: Into<String>>(self, context: C) -> AppResult<T>;

    /// 使用闭包添加上下文
    fn with_context<F, C>(self, f: F) -> AppResult<T>
    where
        F: FnOnce() -> C,
        C: Into<String>;
}

impl<T> ResultExt<T> for std::result::Result<T, AppError> {
    fn context<C: Into<String>>(self, context: C) -> AppResult<T> {
        self.map_err(|e| {
            AppError::Unknown(format!("{}: {}", context.into(), e))
        })
    }

    fn with_context<F, C>(self, f: F) -> AppResult<T>
    where
        F: FnOnce() -> C,
        C: Into<String>,
    {
        self.map_err(|e| {
            AppError::Unknown(format!("{}: {}", f().into(), e))
        })
    }
}

impl<T> ResultExt<T> for std::result::Result<T, std::io::Error> {
    fn context<C: Into<String>>(self, context: C) -> AppResult<T> {
        self.map_err(|e| {
            let app_err: AppError = e.into();
            AppError::Unknown(format!("{}: {}", context.into(), app_err))
        })
    }

    fn with_context<F, C>(self, f: F) -> AppResult<T>
    where
        F: FnOnce() -> C,
        C: Into<String>,
    {
        self.map_err(|e| {
            let app_err: AppError = e.into();
            AppError::Unknown(format!("{}: {}", f().into(), app_err))
        })
    }
}

/// 错误日志宏
#[macro_export]
macro_rules! log_error {
    ($err:expr) => {
        log::error!("[{}] {}", $err.code(), $err)
    };
    ($err:expr, $context:expr) => {
        log::error!("[{}] {} - {}", $err.code(), $context, $err)
    };
}

/// 错误转换宏
#[macro_export]
macro_rules! map_err {
    ($result:expr, $variant:ident) => {
        $result.map_err(|e| crate::error::AppError::$variant(e.to_string()))
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        assert_eq!(AppError::io("test").code(), "IO_ERROR");
        assert_eq!(AppError::not_found("test").code(), "FILE_NOT_FOUND");
        assert_eq!(AppError::Cancelled.code(), "CANCELLED");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let app_err: AppError = io_err.into();
        assert!(matches!(app_err, AppError::FileNotFound(_)));
    }

    #[test]
    fn test_recoverable() {
        assert!(AppError::not_found("test").is_recoverable());
        assert!(AppError::Cancelled.is_recoverable());
        assert!(!AppError::io("test").is_recoverable());
    }
}
