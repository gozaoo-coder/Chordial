//! 音乐源管理模組
//!
//! 管理本地文件夾和網盤源的添加、刪除、啟用/禁用

pub mod source;
pub mod source_manager;

pub use source::{MusicSource, SourceConfig, SourceType, WebDiskAuth};
pub use source_manager::{SourceManager, MusicLibrary, TrackMetadata};
