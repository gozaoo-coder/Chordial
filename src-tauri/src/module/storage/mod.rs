//! 存储模块 — 为后端和前端提供统一的数据持久化与缓存能力。
//!
//! # 模块架构
//!
//! ```text
//! StorageBackend (trait)     ← 抽象读写接口
//!   ├── FileBackend          ← JSON 文件持久化
//!   └── MemoryBackend        ← 内存 HashMap
//!
//! PersistentStore            ← 上层封装（文件后端 + 内存缓存）
//! CacheStore                 ← 上层封装（内存后端 + TTL 过期）
//! ```
//!
//! # 使用示例
//!
//! ```ignore
//! use crate::module::storage::{persistent, cache, Ttl};
//!
//! // 持久化存储（磁盘文件，重启保留）
//! let store = persistent::PersistentStore::new(config_path);
//! store.set("theme", &"dark")?;
//! store.save()?;
//!
//! // 缓存存储（内存，支持过期）
//! let cache = cache::CacheStore::new();
//! cache.set("now_playing", &track_id, &Ttl::DurationSecs(300))?;
//! ```

pub mod backend;
pub mod cache;
pub mod entry;
pub mod file;
pub mod memory;
pub mod persistent;
