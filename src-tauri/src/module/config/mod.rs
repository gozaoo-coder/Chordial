//! 应用配置存储模块。
//!
//! 基于文件持久化，修改后自动防抖落盘（500ms），适合音量、主题等高频变更的配置项。
//!
//! # 与 [`PersistentStore`](super::storage::persistent::PersistentStore) 的区别
//!
//! | 特性 | ConfigStore | PersistentStore |
//! |------|------------|-----------------|
//! | 落盘方式 | 修改后自动防抖（500ms） | 手动调用 [`save()`] |
//! | 典型用途 | 音量、主题、窗口位置等设置 | 播放列表、乐库索引等大批量数据 |
//! | 数据文件 | `config.json` | `storage.json` |
//!
//! # 使用示例
//!
//! ```ignore
//! use std::path::PathBuf;
//! use crate::module::config::ConfigStore;
//!
//! let path = dirs::config_dir().unwrap().join("chordial").join("config.json");
//! let config = ConfigStore::new(path);
//!
//! config.set("theme", &"dark")?;      // 自动防抖落盘
//! config.set("volume", &0.75)?;       // 500ms 内连续修改合并为一次写盘
//! ```

pub mod store;
