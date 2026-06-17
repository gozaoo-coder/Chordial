//! 内存缓存存储模块，支持 TTL 自动过期。
//!
//! 数据仅存在于进程生命周期内，应用重启后全部清空。
//!
//! # 使用示例
//!
//! ```ignore
//! use crate::module::cache::CacheStore;
//! use crate::module::storage::Ttl;
//!
//! let cache = CacheStore::new();
//! cache.set("recent", &data, &Ttl::DurationSecs(600))?;
//! ```

pub mod store;
