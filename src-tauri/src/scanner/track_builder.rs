//! Track 构建模块
//!
//! 提供从各种来源创建 TrackMetadata 的功能，统一处理本地文件和 WebDev API 数据的转换。
//!
//! # 主要功能
//! - 从本地音频文件读取元数据并创建 TrackMetadata
//! - 从 WebDev API 响应数据创建 TrackMetadata
//! - 自动查找和解析外部歌词文件
//!
//! # 性能优化
//! - 封面数据使用 Arc<String> 共享，避免大体积二进制数据克隆

use std::path::PathBuf;
use std::fs;
use std::sync::Arc;
use uuid::Uuid;
use crate::music_source::{SourceConfig, TrackMetadata};
use crate::audio_metadata::read_metadata;
use crate::lyric_enhancer::{find_lyric_file, enhance_metadata_with_lyrics};
use super::music_scanner::ScanError;

/// WebDev API 曲目数据结构
///
/// 用于反序列化 WebDev API 返回的曲目信息
#[derive(Debug, serde::Deserialize)]
pub struct WebDevTrack {
    /// 曲目唯一标识
    pub id: String,
    /// 曲目标题
    pub title: String,
    /// 艺术家名称
    pub artist: Option<String>,
    /// 专辑名称
    pub album: Option<String>,
    /// 时长（秒）
    pub duration: Option<u64>,
    /// 文件 URL
    pub file_url: String,
    /// 文件大小（字节）
    pub file_size: Option<u64>,
    /// 比特率（kbps）
    pub bitrate: Option<u32>,
    /// 采样率（Hz）
    pub sample_rate: Option<u32>,
    /// 声道数
    pub channels: Option<u16>,
    /// 年份
    pub year: Option<u32>,
    /// 流派
    pub genre: Option<String>,
    /// 封面图片 URL
    pub cover_url: Option<String>,
}

/// Track 构建器
///
/// 提供静态方法用于从不同来源构建 TrackMetadata，统一处理元数据提取和转换逻辑。
pub struct TrackBuilder;

impl TrackBuilder {
    /// 从本地文件创建 TrackMetadata
    pub fn build_from_file(
        file_path: &PathBuf,
        source: &SourceConfig,
    ) -> Result<Option<TrackMetadata>, ScanError> {
        // 检查文件是否存在
        if !file_path.exists() {
            return Err(ScanError::FileNotFound(
                file_path.to_string_lossy().to_string()
            ));
        }

        let metadata = match fs::metadata(file_path) {
            Ok(m) => m,
            Err(e) => return Err(ScanError::IoError(format!(
                "无法读取文件元数据: {}",
                e
            ))),
        };

        // 检查是否为文件
        if !metadata.is_file() {
            return Err(ScanError::InvalidFileFormat(
                file_path.to_string_lossy().to_string()
            ));
        }

        let file_name = file_path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "未知".to_string());

        let mut audio_metadata = match read_metadata(file_path) {
            Ok(meta) => meta,
            Err(e) => return Err(ScanError::from(e)),
        };

        // 尝试查找并解析外部歌词文件
        if let Some(lyric_content) = find_lyric_file(file_path) {
            enhance_metadata_with_lyrics(&mut audio_metadata, Some(lyric_content));
        }

        let track = TrackMetadata {
            id: Uuid::new_v4().to_string(),
            source_id: source.id.clone(),
            path: file_path.clone(),
            file_name: file_name.clone(),
            title: audio_metadata.title.or(Some(file_path.file_stem()
                .and_then(|s| Some(s.to_string_lossy().to_string()))
                .unwrap_or_else(|| "未知标题".to_string()))),
            artist: audio_metadata.artist,
            artist_id: None,
            artist_summaries: Vec::new(),
            album: audio_metadata.album,
            album_id: None,
            album_summary: None,
            album_cover_data: None,
            duration: audio_metadata.duration.map(|d| d.as_secs()),
            format: audio_metadata.format.to_string(),
            file_size: metadata.len(),
            bitrate: audio_metadata.bitrate.map(|b| b as u32),
            sample_rate: audio_metadata.sample_rate.map(|s| s as u32),
            channels: audio_metadata.channels.map(|c| c as u16),
            year: None,
            genre: audio_metadata.genre,
            composer: audio_metadata.composer,
            comment: None,
            lyrics: audio_metadata.lyrics,
            synced_lyrics: audio_metadata.synced_lyrics.map(|lines| {
                let lyric_data: Vec<serde_json::Value> = lines.iter()
                    .map(|line| serde_json::json!({
                        "timestamp": line.timestamp.as_millis() as u64,
                        "text": line.text
                    }))
                    .collect();
                serde_json::to_string(&lyric_data).unwrap_or_default()
            }),
            added_at: chrono::Utc::now(),
        };

        Ok(Some(track))
    }

    /// 从 WebDev API 数据创建 TrackMetadata
    pub fn build_from_webdev(
        track: &WebDevTrack,
        source: &SourceConfig,
    ) -> Result<TrackMetadata, ScanError> {
        let track_metadata = TrackMetadata {
            id: track.id.clone(),
            source_id: source.id.clone(),
            path: PathBuf::from(&track.file_url),
            file_name: format!("{}.mp3", track.title),
            title: Some(track.title.clone()),
            artist: track.artist.clone(),
            artist_id: None,
            artist_summaries: Vec::new(),
            album: track.album.clone(),
            album_id: None,
            album_summary: None,
            album_cover_data: None,
            duration: track.duration,
            format: "MP3".to_string(),
            file_size: track.file_size.unwrap_or(0),
            bitrate: track.bitrate,
            sample_rate: track.sample_rate,
            channels: track.channels,
            year: track.year,
            genre: track.genre.clone(),
            composer: None,
            comment: None,
            lyrics: None,
            synced_lyrics: None,
            added_at: chrono::Utc::now(),
        };

        Ok(track_metadata)
    }
}
