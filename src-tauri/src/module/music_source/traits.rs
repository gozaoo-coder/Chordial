use super::types::SourceType;
use crate::module::music_library::models::{Album, Artist, Lyric, Song};

/// 音乐来源插件必须实现的接口。
///
/// 每个来源实现负责与具体的来源后端通信（如本地文件系统、Web API 等），
/// 并将结果转换为统一的库模型。
///
/// 所有查询方法返回 `Result<..., String>`，其中 `Err` 表示通信或解析失败，
/// `Ok(None)` 表示来源中不存在该实体。
pub trait MusicSource: Send + Sync {
    /// 来源的唯一名称，如 `"my_local"`, `"netease"`。
    fn name(&self) -> &str;

    /// 来源类型。
    fn source_type(&self) -> SourceType;

    /// 按关键词搜索歌曲。
    fn search_songs(&self, query: &str) -> Result<Vec<Song>, String>;

    /// 按来源内部 ID 获取单首歌曲。
    fn get_song(&self, id: &str) -> Result<Option<Song>, String>;

    /// 按来源内部 ID 获取艺术家。
    fn get_artist(&self, id: &str) -> Result<Option<Artist>, String>;

    /// 按来源内部 ID 获取专辑。
    fn get_album(&self, id: &str) -> Result<Option<Album>, String>;

    /// 按来源内部歌曲 ID 获取歌词（元数据，含 `text` 字段）。
    fn get_lyric(&self, song_id: &str) -> Result<Option<Lyric>, String>;

    // ── 资源获取（大文件 / 二进制）────────────────────

    /// 获取歌曲的音频文件数据。
    ///
    /// `entity_id` 为来源内部的歌曲 ID。返回完整的音频文件字节。
    /// 用于前端播放或离线缓存。
    fn song_file_get(&self, entity_id: &str) -> Result<Vec<u8>, String>;

    /// 获取歌曲文件的本地路径（如果可用）。
    ///
    /// 用于自定义协议流式传输，避免将整个文件加载到内存。
    /// 默认返回 `None`；本地来源应覆盖此方法返回实际路径。
    fn song_file_path(&self, _entity_id: &str) -> Option<String> {
        None
    }

    /// 获取专辑的封面图片数据。
    ///
    /// `entity_id` 为来源内部的专辑 ID。返回图片字节（JPEG/PNG 等）。
    fn album_picture_get(&self, entity_id: &str) -> Result<Vec<u8>, String>;

    /// 获取歌曲的歌词文本。
    ///
    /// `song_id` 为来源内部的歌曲 ID。返回原始歌词文本（LRC 或纯文本）。
    /// 与 [`get_lyric`](Self::get_lyric) 不同，此方法直接从来源拉取原始文本，
    /// 而不是返回库内已存储的 [`Lyric`] 结构体。
    fn lyric_text_get(&self, song_id: &str) -> Result<String, String>;
}
