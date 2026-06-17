//! 音乐来源模块 — 定义来源类型、来源接口规范，并提供来源注册中心。
//!
//! # 模块架构
//!
//! ```text
//! SourceType / EntityType / SourceId   ← 来源标记类型
//! MusicSource (trait)                  ← 来源实现必须遵循的接口
//! SourceRegistry                       ← 注册中心（注册 + 持久化元信息）
//! ```
//!
//! # 使用示例
//!
//! ```ignore
//! use crate::module::music_source::registry::SourceRegistry;
//! use crate::module::music_source::traits::MusicSource;
//!
//! let registry = SourceRegistry::new(path);
//! registry.register(Box::new(my_local_source))?;
//!
//! // 启动时加载持久化的元信息，重新注入实现
//! for entry in registry.load_entries() {
//!     // 根据 entry.name / entry.source_type 重建实现并 register
//! }
//! ```

pub mod registry;
pub mod traits;
pub mod types;
