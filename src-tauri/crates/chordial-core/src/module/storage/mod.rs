//! 存储核心模块 — 提供持久化后端抽象及通用持久化存储。
//!
//! # 模块架构
//!
//! ```text
//! StorageBackend (trait)     ← 抽象读写接口
//!   ├── FileBackend          ← JSON 文件持久化
//!   └── MemoryBackend        ← 内存 HashMap
//!
//! PersistentStore            ← 通用持久化存储（文件后端 + 内存缓存，手动落盘）
//! ```
//!
//! # 三层职责划分
//!
//! | 模块 | 落盘方式 | 文件 | 典型用途 |
//! |------|---------|------|---------|
//! | [`config`](super::config) | 自动防抖（500ms） | `config.json` | 音量、主题等设置 |
//! | [`storage`]（本模块） | 手动调用 `save()` | `storage.json` | 播放列表、乐库索引等 |
//! | [`cache`](super::cache) | 不落盘，支持 TTL | 无 | 临时缓存、最近播放等 |
//!
//! # 使用示例
//!
//! ```ignore
//! use crate::module::storage::persistent::PersistentStore;
//!
//! let store = PersistentStore::new(path);
//! store.set("playlist", &my_playlist)?;
//! store.save()?;  // 显式持久化
//! ```

pub mod backend;
pub mod entry;
pub mod file;
pub mod memory;
pub mod persistent;
