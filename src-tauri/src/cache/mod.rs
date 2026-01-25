//! 緩存管理模組
//!
//! 管理音樂庫數據的本地緩存，包括讀寫和序列化

pub mod cache_manager;

pub use cache_manager::{CacheManager, CacheError};
