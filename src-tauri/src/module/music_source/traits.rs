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

    /// 按来源内部歌曲 ID 获取歌词。
    fn get_lyric(&self, song_id: &str) -> Result<Option<Lyric>, String>;
}
