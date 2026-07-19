//! 音乐库模块 — 定义音乐实体模型，并提供 CRUD、搜索和关系追溯功能。
//!
//! # 模块架构
//!
//! ```text
//! models.rs            ← Song, Artist, Album, Lyric 结构体定义
//! songs.rs             ← 歌曲 CRUD + 搜索
//! artists.rs           ← 艺术家 CRUD + 搜索
//! albums.rs            ← 专辑 CRUD + 搜索
//! lyrics.rs            ← 歌词 CRUD + 搜索
//! relations.rs         ← 跨实体关系追溯（song→artist, artist→songs 等）
//! library.rs           ← MusicLibrary 统一入口（持有 PersistentStore，委托各子模块）
//! ```
//!
//! # 使用示例
//!
//! ```ignore
//! use crate::module::music_library::library::MusicLibrary;
//! use crate::module::music_library::models::Song;
//!
//! let lib = MusicLibrary::new(path);
//! lib.add_song(&song)?;
//! let results = lib.search_songs("关键词");
//! lib.save()?;
//! ```

pub mod albums;
pub mod artists;
pub mod library;
pub mod lyrics;
pub mod models;
pub mod relations;
pub mod search;
pub mod songs;
