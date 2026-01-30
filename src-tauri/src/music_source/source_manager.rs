//! 音樂源管理器
//!
//! 管理所有音樂源的列表，提供添加、刪除、查詢等功能

use super::{MusicSource, SourceConfig, SourceType};
use super::{Artist, Album, ArtistSummary, AlbumSummary};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 音樂庫結構，包含所有音樂源和掃描結果
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MusicLibrary {
    /// 所有音樂源
    pub sources: Vec<SourceConfig>,
    /// 所有歌曲的元數據
    pub tracks: Vec<TrackMetadata>,
    /// 所有歌手信息
    pub artists: Vec<Artist>,
    /// 所有专辑信息
    pub albums: Vec<Album>,
}

impl MusicLibrary {
    /// 创建新的音乐库
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
            tracks: Vec::new(),
            artists: Vec::new(),
            albums: Vec::new(),
        }
    }

    /// 添加或更新歌手
    pub fn upsert_artist(&mut self, artist: Artist) {
        if let Some(existing) = self.artists.iter_mut().find(|a| a.id == artist.id) {
            *existing = artist;
        } else {
            self.artists.push(artist);
        }
    }

    /// 添加或更新专辑
    pub fn upsert_album(&mut self, album: Album) {
        if let Some(existing) = self.albums.iter_mut().find(|a| a.id == album.id) {
            *existing = album;
        } else {
            self.albums.push(album);
        }
    }

    /// 根据ID获取歌手
    pub fn get_artist(&self, id: &str) -> Option<&Artist> {
        self.artists.iter().find(|a| a.id == id)
    }

    /// 根据ID获取专辑
    pub fn get_album(&self, id: &str) -> Option<&Album> {
        self.albums.iter().find(|a| a.id == id)
    }

    /// 根据名称获取歌手（模糊匹配）
    pub fn find_artist_by_name(&self, name: &str) -> Option<&Artist> {
        self.artists.iter().find(|a| a.name == name)
    }

    /// 根据名称获取专辑（模糊匹配）
    pub fn find_album_by_title(&self, title: &str, artist_name: &str) -> Option<&Album> {
        self.albums.iter().find(|a| a.title == title && a.artist_name == artist_name)
    }
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
    /// 藝術家（原始字符串，可能包含多個歌手）
    pub artist: Option<String>,
    /// 藝術家ID
    pub artist_id: Option<String>,
    /// 歌手摘要信息（用於快速展示）
    pub artist_summary: Option<ArtistSummary>,
    /// 專輯
    pub album: Option<String>,
    /// 專輯ID
    pub album_id: Option<String>,
    /// 专辑摘要信息（用於快速展示）
    pub album_summary: Option<AlbumSummary>,
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

impl TrackMetadata {
    /// 获取解析后的歌手列表
    /// 支持 "/" 和 "&" 分隔符
    pub fn parsed_artists(&self) -> Vec<String> {
        match &self.artist {
            Some(artist_str) => super::ArtistParser::parse(artist_str),
            None => Vec::new(),
        }
    }

    /// 获取主歌手名稱（第一个歌手）
    pub fn primary_artist(&self) -> Option<String> {
        self.parsed_artists().into_iter().next()
    }

    /// 更新歌手摘要
    pub fn set_artist_summary(&mut self, summary: ArtistSummary) {
        self.artist_summary = Some(summary);
    }

    /// 更新专辑摘要
    pub fn set_album_summary(&mut self, summary: AlbumSummary) {
        self.album_summary = Some(summary);
    }
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
