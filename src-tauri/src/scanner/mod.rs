//! 音樂掃描模組
//!
//! 提供多線程音樂掃描服務，支持本地文件夾和網盤源

pub mod music_scanner;
pub mod webdav;

pub use music_scanner::{MusicScanner, ScanProgress, ScanOptions, ScanResult, ScanError};
pub use webdav::{WebDavClient, WebDavError, DavItem};
