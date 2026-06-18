//! 资源获取 — 从来源获取大文件/二进制资源。
//!
//! 该模块是前端请求音乐资源的调度中心：
//!
//! ```text
//! front → Tauri command → resource::get_XXXX(registrar, source_id)
//!   → registrar.get(source_id.source_name) → Arc<dyn MusicSource>
//!     → trait 方法 song_file_get / album_picture_get / lyric_text_get
//!       → 返回 Vec<u8> / String → Tauri Channel / raw payload → 前端
//! ```
//!
//! 每个函数接收 [`SourceId`](super::types::SourceId) 作为参数，
//! 从中提取 `source_name` 以查找来源实现，提取 `entity_id` 传给 trait 方法。

use super::registrar::SourceRegistrar;
use super::types::SourceId;

/// 获取歌曲的音频文件。
///
/// # 链路
/// 1. 用 `source_id.source_name` 查找来源实现
/// 2. 调用来源的 [`song_file_get`](super::traits::MusicSource::song_file_get)
/// 3. 返回音频字节数据
pub fn get_song_file(
    registrar: &SourceRegistrar,
    source_id: &SourceId,
) -> Result<Vec<u8>, String> {
    let source = registrar
        .get(&source_id.source_name)
        .ok_or_else(|| format!("来源 '{}' 未注册", source_id.source_name))?;

    source.song_file_get(&source_id.entity_id)
}

/// 获取歌曲文件的本地路径（用于自定义协议流式传输）。
///
/// 仅本地来源支持；网络来源返回 `None`。
pub fn get_song_file_path(
    registrar: &SourceRegistrar,
    source_id: &SourceId,
) -> Option<String> {
    let source = registrar
        .get(&source_id.source_name)?;
    source.song_file_path(&source_id.entity_id)
}

/// 获取专辑的封面图片。
///
/// # 链路
/// 1. 用 `source_id.source_name` 查找来源实现
/// 2. 调用来源的 [`album_picture_get`](super::traits::MusicSource::album_picture_get)
/// 3. 返回图片字节数据
pub fn get_album_picture(
    registrar: &SourceRegistrar,
    source_id: &SourceId,
) -> Result<Vec<u8>, String> {
    let source = registrar
        .get(&source_id.source_name)
        .ok_or_else(|| format!("来源 '{}' 未注册", source_id.source_name))?;

    source.album_picture_get(&source_id.entity_id)
}

/// 获取歌曲的歌词文本。
///
/// # 链路
/// 1. 用 `source_id.source_name` 查找来源实现
/// 2. 调用来源的 [`lyric_text_get`](super::traits::MusicSource::lyric_text_get)
/// 3. 返回歌词文本
pub fn get_lyric_text(
    registrar: &SourceRegistrar,
    source_id: &SourceId,
) -> Result<String, String> {
    let source = registrar
        .get(&source_id.source_name)
        .ok_or_else(|| format!("来源 '{}' 未注册", source_id.source_name))?;

    source.lyric_text_get(&source_id.entity_id)
}
