//! 音乐来源模块 — 定义来源类型、来源接口规范，并提供来源注册中心。
//!
//! # 模块架构
//!
//! ```text
//! SourceType / EntityType / SourceId   ← 来源标记类型
//! MusicSource (trait)                  ← 来源实现必须遵循的接口
//! SourceManager                        ← 元信息持久化 + 内存挂载
//! SourceRegistrar                      ← 注册/注销/查找 + MusicLibrary 联动清理
//! resource                             ← 资源获取调度（song_file / album_picture / lyric_text）
//! ```
//!
//! # 使用示例
//!
//! ```ignore
//! use crate::module::music_source::manager::{SourceEntry, SourceManager};
//! use crate::module::music_source::registrar::{SourceCleanup, SourceRegistrar};
//! use crate::module::music_source::traits::MusicSource;
//!
//! let manager = Arc::new(SourceManager::new(path));
//! let registrar = SourceRegistrar::new(manager.clone(), music_library_arc);
//!
//! // 启动时加载持久化的元信息，重新注入实现
//! registrar.load_all(|entry| {
//!     // 根据 entry.name / entry.source_type 重建实现
//!     Some(Arc::new(MyLocalSource::new()))
//! })?;
//!
//! // 获取资源
//! let audio = resource::get_song_file(&registrar, &source_id)?;
//! ```

pub mod manager;
pub mod registrar;
pub mod resource;
pub mod traits;
pub mod types;
