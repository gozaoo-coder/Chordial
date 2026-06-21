//! # Chordial Core — server 层核心
//!
//! 纯 Rust 业务逻辑库，**零 Tauri 依赖**。封装整个音乐系统：跨平台文件访问、
//! 持久化存储、缓存、音乐来源接口、音乐库、媒体流式传输。
//!
//! # 数据契约（wire contract）
//!
//! 以下 serde 结构是 server ↔ front 的线格式契约，手动与前端 JS 类镜像
//! （`src/class/*.js`）。后续可接入 `ts-rs` / `specta` 自动生成。
//! - [`module::music_source::types`]：`SourceId` / `SourceType` / `EntityType`
//! - [`module::music_library::models`]：`Song` / `Artist` / `Album` / `Lyric`
//! - [`module::storage::entry`]：`Ttl`
//!
//! # 消费方式
//!
//! - **库调用形式**：`chordial-tauri` 将本 crate 链接进 Tauri 应用，通过
//!   [`AppContext`] 在 Tauri 命令中直接调用。
//! - **web 服务器形式**：`chordial-server` 将本 crate 链接进 axum 服务，
//!   通过 HTTP 端点对外提供数据。
//!
//! ```ignore
//! use chordial_core::AppContext;
//!
//! let ctx = AppContext::new_default_dir()?;
//! println!("歌曲数: {}", ctx.library.song_count());
//! ```

pub mod app;
pub mod media;
pub mod module;

pub use app::AppContext;

// ── 对外重导出常用类型 ──────────────────────────────────────
pub use module::music_source::types::{EntityType, SourceId, SourceType};
