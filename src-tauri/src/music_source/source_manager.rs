//! 音樂源管理器
//!
//! 管理所有音樂源的列表，提供添加、刪除、查詢等功能

use super::{MusicSource, SourceConfig, SourceType};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 音樂庫結構，包含所有音樂源和掃描結果
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MusicLibrary {
    /// 所有音樂源
    pub sources: Vec<SourceConfig>,
    /// 所有歌曲的元數據
    pub tracks: Vec<TrackMetadata>,
}

/// 單個歌曲的元數據
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackMetadata {
    /// 歌曲唯一標識
    pub id: String,
    /// 所屬源ID
    pub source_id: String,
    /// 文件路徑
    pub path: PathBuf,
    /// 文件名
    pub file_name: String,
    /// 標題
    pub title: Option<String>,
    /// 藝術家
    pub artist: Option<String>,
    /// 藝術家ID
    pub artist_id: Option<String>,
    /// 專輯
    pub album: Option<String>,
    /// 專輯ID
    pub album_id: Option<String>,
    /// 專輯封面數據 (Base64 編碼)
    pub album_cover_data: Option<String>,
    /// 時長（秒）
    pub duration: Option<u64>,
    /// 格式
    pub format: String,
    /// 文件大小（字節）
    pub file_size: u64,
    /// 比特率 (kbps)
    pub bitrate: Option<u32>,
    /// 采樣率 (Hz)
    pub sample_rate: Option<u32>,
    /// 声道数
    pub channels: Option<u16>,
    /// 年份
    pub year: Option<u32>,
    /// 流派
    pub genre: Option<String>,
    /// 作曲
    pub composer: Option<String>,
    /// 備注
    pub comment: Option<String>,
    /// 歌詞（純文本）
    pub lyrics: Option<String>,
    /// 同步歌詞（JSON 格式的時間戳歌詞）
    pub synced_lyrics: Option<String>,
    /// 添加時間
    pub added_at: chrono::DateTime<chrono::Utc>,
}

/// 音樂源管理器
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SourceManager {
    /// 所有音樂源
    sources: Vec<SourceConfig>,
}

impl SourceManager {
    /// 創建新的源管理器
    pub fn new() -> Self {
        Self { sources: Vec::new() }
    }

    /// 添加本地文件夾源
    pub fn add_local_folder(&mut self, path: PathBuf, recursive: bool) -> Result<SourceConfig, String> {
        if !path.exists() {
            return Err("路徑不存在".to_string());
        }
        if !path.is_dir() {
            return Err("路徑不是文件夾".to_string());
        }

        let source = SourceConfig::new_local_folder(path, None, recursive);
        self.sources.push(source.clone());
        Ok(source)
    }

    /// 添加網盤源
    pub fn add_web_disk(&mut self, url: PathBuf, auth: Option<(String, String)>) -> Result<SourceConfig, String> {
        let auth = auth.map(|(username, password)| crate::music_source::WebDiskAuth { username, password });
        let source = SourceConfig::new_web_disk(url, None, auth);
        self.sources.push(source.clone());
        Ok(source)
    }

    /// 移除音樂源
    pub fn remove_source(&mut self, id: &str) -> Option<SourceConfig> {
        self.sources.iter().position(|s| s.id() == id)
            .map(|idx| self.sources.remove(idx))
    }

    /// 獲取音樂源
    pub fn get_source(&self, id: &str) -> Option<&SourceConfig> {
        self.sources.iter().find(|s| s.id() == id)
    }

    /// 獲取可變音樂源
    pub fn get_source_mut(&mut self, id: &str) -> Option<&mut SourceConfig> {
        self.sources.iter_mut().find(|s| s.id() == id)
    }

    /// 獲取所有已啟用的源
    pub fn get_enabled_sources(&self) -> Vec<&SourceConfig> {
        self.sources.iter().filter(|s| s.is_enabled()).collect()
    }

    /// 獲取所有源
    pub fn get_all_sources(&self) -> &[SourceConfig] {
        &self.sources
    }

    /// 獲取所有源（可變）
    pub fn get_all_sources_mut(&mut self) -> &mut Vec<SourceConfig> {
        &mut self.sources
    }

    /// 設置源啟用狀態
    pub fn set_source_enabled(&mut self, id: &str, enabled: bool) -> bool {
        if let Some(source) = self.get_source_mut(id) {
            source.set_enabled(enabled);
            true
        } else {
            false
        }
    }

    /// 獲取本地文件夾源數量
    pub fn local_folder_count(&self) -> usize {
        self.sources.iter().filter(|s| s.source_type == SourceType::LocalFolder).count()
    }

    /// 獲取網盤源數量
    pub fn web_disk_count(&self) -> usize {
        self.sources.iter().filter(|s| s.source_type == SourceType::WebDisk).count()
    }

    /// 獲取所有源的數量
    pub fn len(&self) -> usize {
        self.sources.len()
    }

    /// 檢查是否沒有源
    pub fn is_empty(&self) -> bool {
        self.sources.is_empty()
    }
}
