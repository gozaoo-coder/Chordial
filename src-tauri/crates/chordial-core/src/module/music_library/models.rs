use crate::module::music_source::types::SourceId;
use serde::{Deserialize, Serialize};

/// 歌曲。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Song {
    /// 库内统一 ID（UUID）
    pub id: String,
    /// 歌曲标题
    pub title: String,
    /// 艺人名称（无序），与 `artist_ids` 一一对应
    pub artist_names: Vec<String>,
    /// 专辑名称，与 `album_id` 对应
    pub album_title: Option<String>,
    /// 时长（秒）
    pub duration: Option<u64>,
    /// 关联的艺术家 ID 列表
    pub artist_ids: Vec<String>,
    /// 关联的专辑 ID
    pub album_id: Option<String>,
    /// 关联的歌词 ID
    pub lyric_id: Option<String>,
    /// 来源引用 — 该歌曲在哪些来源中存在
    pub source_ids: Vec<SourceId>,
}

/// 艺术家。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artist {
    /// 库内统一 ID（UUID）
    pub id: String,
    /// 艺术家名称
    pub name: String,
    /// 简介
    pub bio: Option<String>,
    /// 来源引用
    pub source_ids: Vec<SourceId>,
}

/// 专辑。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Album {
    /// 库内统一 ID（UUID）
    pub id: String,
    /// 专辑标题
    pub title: String,
    /// 所属艺术家 ID
    pub artist_id: String,
    /// 封面图片 URL
    pub cover_url: Option<String>,
    /// 包含的歌曲 ID 列表
    pub song_ids: Vec<String>,
    /// 来源引用
    pub source_ids: Vec<SourceId>,
}

/// 歌词。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lyric {
    /// 库内统一 ID（UUID）
    pub id: String,
    /// 关联的歌曲 ID
    pub song_id: String,
    /// 歌词文本
    pub text: String,
    /// 来源引用（歌词通常只有一个来源）
    pub source_id: SourceId,
}
