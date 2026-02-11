//! 音樂掃描器
//!
//! 使用多線程並行掃描音樂文件，支持進度回調和取消操作

use super::super::music_source::{SourceConfig, SourceType, TrackMetadata, MusicSource};
use super::super::music_source::{Artist, Album, ArtistParser, AlbumIdGenerator};
use super::super::audio_metadata::MetadataError;
use super::super::cache::CacheManager;
use super::track_builder::{TrackBuilder, WebDevTrack};
use rayon::prelude::*;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::collections::HashMap;
use std::time::Duration;
use thiserror::Error;
use std::fs;
use uuid::Uuid;

/// 掃描錯誤類型
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ScanError {
    /// 文件不存在
    #[error("文件不存在: {0}")]
    FileNotFound(String),
    /// 元數據讀取錯誤
    #[error("元數據讀取錯誤: {0}")]
    MetadataReadError(String),
    /// 無效的文件格式
    #[error("無效的文件格式: {0}")]
    InvalidFileFormat(String),
    /// IO 錯誤
    #[error("IO 錯誤: {0}")]
    IoError(String),
    /// WebDAV 錯誤
    #[error("WebDAV 錯誤: {0}")]
    WebDavError(String),
    /// WebDev API 錯誤
    #[error("WebDev API 錯誤: {0}")]
    WebDevError(String),
    /// 不支持的文件格式
    #[error("不支持的文件格式: {0}")]
    UnsupportedFormat(String),
    /// 掃描被取消
    #[error("掃描被取消")]
    ScanCancelled,
}

/// WebDev API 響應數據結構
#[derive(Debug, serde::Deserialize)]
struct WebDevResponse {
    tracks: Vec<WebDevTrack>,
}

impl From<MetadataError> for ScanError {
    fn from(e: MetadataError) -> Self {
        match e {
            MetadataError::IoError(msg) => ScanError::IoError(msg),
            MetadataError::InvalidFormat(msg) => ScanError::InvalidFileFormat(msg),
            MetadataError::UnsupportedFormat(msg) => ScanError::UnsupportedFormat(msg),
            MetadataError::ParseError(msg) => ScanError::MetadataReadError(msg),
            MetadataError::FileTooLarge => ScanError::InvalidFileFormat("文件過大".to_string()),
            MetadataError::Unknown => ScanError::MetadataReadError("未知錯誤".to_string()),
        }
    }
}

impl From<std::io::Error> for ScanError {
    fn from(e: std::io::Error) -> Self {
        ScanError::IoError(e.to_string())
    }
}

/// 掃描進度
#[derive(Debug, Clone)]
pub struct ScanProgress {
    /// 當前處理的源ID
    pub source_id: String,
    /// 當前處理的源名稱
    pub source_name: String,
    /// 已掃描的文件數
    pub scanned_count: usize,
    /// 總文件數
    pub total_count: usize,
    /// 找到的歌曲數
    pub found_count: usize,
    /// 錯誤數
    pub error_count: usize,
    /// 是否完成
    pub is_complete: bool,
    /// 錯誤消息
    pub error_message: Option<String>,
}

impl Default for ScanProgress {
    fn default() -> Self {
        Self {
            source_id: String::new(),
            source_name: String::new(),
            scanned_count: 0,
            total_count: 0,
            found_count: 0,
            error_count: 0,
            is_complete: false,
            error_message: None,
        }
    }
}

/// 掃描選項
#[derive(Clone)]
pub struct ScanOptions {
    /// 是否重新掃描所有文件
    pub force_rescan: bool,
    /// 最大並行任務數
    pub parallel_tasks: usize,
    /// 進度回調函數
    pub progress_callback: Option<Arc<dyn Fn(ScanProgress) + Send + Sync>>,
}

/// 掃描結果
#[derive(Debug, Clone)]
pub struct ScanResult {
    /// 源配置
    pub source: SourceConfig,
    /// 掃描到的歌曲列表
    pub tracks: Vec<TrackMetadata>,
    /// 掃描到的歌手列表
    pub artists: Vec<Artist>,
    /// 掃描到的专辑列表
    pub albums: Vec<Album>,
    /// 掃描耗時
    pub duration: Duration,
    /// 錯誤消息
    pub error: Option<String>,
}

impl Default for ScanResult {
    fn default() -> Self {
        Self {
            source: SourceConfig::new_local_folder(PathBuf::new(), None, true),
            tracks: Vec::new(),
            artists: Vec::new(),
            albums: Vec::new(),
            duration: Duration::ZERO,
            error: None,
        }
    }
}

/// 音樂掃描器
#[derive(Debug, Clone)]
pub struct MusicScanner {
    cache_manager: Option<CacheManager>,
}

impl MusicScanner {
    /// 創建新的音樂掃描器
    pub fn new() -> Self {
        Self {
            cache_manager: Some(CacheManager::new()),
        }
    }

    /// 創建帶緩存管理器的音樂掃描器
    pub fn with_cache_manager(cache_manager: CacheManager) -> Self {
        Self {
            cache_manager: Some(cache_manager),
        }
    }

    /// 掃描單個源
    pub fn scan_source(&self, source: &SourceConfig, options: &ScanOptions) -> ScanResult {
        let start_time = std::time::Instant::now();
        let mut result = ScanResult {
            source: source.clone(),
            ..Default::default()
        };

        // 先扫描获取所有曲目
        let tracks_result = match source.source_type() {
            SourceType::LocalFolder => {
                self.scan_local_folder(source, options)
            }
            SourceType::WebDisk => {
                self.scan_web_disk(source, options)
            }
            SourceType::WebDev => {
                self.scan_web_dev(source, options)
            }
        };

        match tracks_result {
            Ok(tracks) => {
                // 构建 Artist 和 Album 数据
                let (artists, albums, enriched_tracks) = self.build_artist_album_data(&tracks);

                result.tracks = enriched_tracks;
                result.artists = artists;
                result.albums = albums;
            }
            Err(e) => {
                // 記錄錯誤信息
                result.error = Some(e.to_string());
                log::error!("掃描源失敗 {}: {}", source.id(), e);
            }
        }

        result.duration = start_time.elapsed();
        result
    }

    /// 构建歌手和专辑数据
    /// 从曲目列表中提取并构建 Artist 和 Album 信息
    /// 
    /// # 性能优化
    /// - 使用并行迭代器处理 tracks（多核加速）
    /// - 预分配 HashMap 容量（减少重新哈希）
    /// - 使用 DashMap 实现无锁并发（减少锁竞争）
    pub fn build_artist_album_data(&self, tracks: &[TrackMetadata]) -> (Vec<Artist>, Vec<Album>, Vec<TrackMetadata>) {
        use dashmap::DashMap;
        use rayon::prelude::*;
        
        // 预分配容量：根据经验，artist 数量约为 track 数量的 1/2，album 约为 1/4
        let artists_map: DashMap<String, Artist> = DashMap::with_capacity(tracks.len() / 2);
        let albums_map: DashMap<String, Album> = DashMap::with_capacity(tracks.len() / 4);
        
        // 并行处理 tracks，构建 artist 和 album 信息
        let enriched_tracks: Vec<TrackMetadata> = tracks
            .par_iter()
            .map(|track| {
                let mut track = track.clone();
                let artist_names = track.parsed_artists();
                let primary_artist_name = artist_names.first()
                    .cloned()
                    .unwrap_or_else(|| "未知歌手".to_string());

                // 为每个歌手创建/更新信息
                let mut artist_summaries = Vec::with_capacity(artist_names.len());
                let mut combined_artist_id = String::with_capacity(artist_names.len() * 32);

                for artist_name in &artist_names {
                    let artist_id = ArtistParser::generate_id(artist_name);
                    
                    // 使用 DashMap 实现无锁并发插入
                    artists_map.entry(artist_id.clone()).or_insert_with(|| {
                        Artist::new(artist_id.clone(), artist_name.clone())
                    });
                    
                    // 获取歌手引用并添加曲目
                    if let Some(mut artist) = artists_map.get_mut(&artist_id) {
                        artist.add_track(track.id.clone());
                        artist_summaries.push(artist.to_summary());
                    }

                    // 构建组合ID
                    if !combined_artist_id.is_empty() {
                        combined_artist_id.push('_');
                    }
                    combined_artist_id.push_str(&artist_id);
                }

                // 确定最终歌手ID
                let final_artist_id = if artist_names.len() == 1 {
                    ArtistParser::generate_id(&primary_artist_name)
                } else {
                    ArtistParser::generate_combined_id(&artist_names)
                };

                // 处理专辑信息
                if let Some(album_title) = &track.album {
                    let album_id = AlbumIdGenerator::generate_id(album_title, &primary_artist_name);

                    // 使用 DashMap 并发创建或更新专辑
                    albums_map.entry(album_id.clone()).or_insert_with(|| {
                        Album::new(
                            album_id.clone(),
                            album_title.clone(),
                            final_artist_id.clone(),
                            primary_artist_name.clone(),
                        )
                    });

                    // 更新专辑信息
                    if let Some(mut album) = albums_map.get_mut(&album_id) {
                        album.add_track(track.id.clone());
                        
                        // 更新专辑年份
                        if let Some(year) = track.year {
                            if album.year.is_none() {
                                album.year = Some(year);
                            }
                        }
                        
                        // 更新专辑封面
                        if album.cover_data.is_none() && track.album_cover_data.is_some() {
                            album.cover_data = track.album_cover_data.clone();
                        }
                        
                        // 更新专辑总时长
                        if let Some(duration) = track.duration {
                            album.total_duration += duration;
                        }
                        
                        // 创建专辑摘要
                        let album_summary = album.to_summary();
                        track.set_album_summary(album_summary);
                    }

                    // 添加专辑到所有歌手
                    for artist_name in &artist_names {
                        let artist_id = ArtistParser::generate_id(artist_name);
                        if let Some(mut artist) = artists_map.get_mut(&artist_id) {
                            artist.add_album(album_id.clone());
                        }
                    }

                    track.album_id = Some(album_id);
                }

                // 设置歌手摘要数组
                track.set_artist_summaries(artist_summaries);
                track.artist_id = Some(final_artist_id);

                track
            })
            .collect();

        // 将 DashMap 转换为 Vec（并行收集）
        let artists: Vec<Artist> = artists_map.into_iter().map(|(_, v)| v).collect();
        let albums: Vec<Album> = albums_map.into_iter().map(|(_, v)| v).collect();

        (artists, albums, enriched_tracks)
    }

    /// 批量掃描多個源（並行）
    pub fn scan_sources(&self, sources: &[SourceConfig], options: &ScanOptions) -> Vec<ScanResult> {
        let _parallel_tasks = options.parallel_tasks.clamp(1, 8);
        
        sources.par_iter()
            .filter(|s| s.is_enabled())
            .map(|source| {
                let opts = ScanOptions {
                    progress_callback: options.progress_callback.clone(),
                    ..options.clone()
                };
                self.scan_source(source, &opts)
            })
            .collect()
    }

    /// 掃描本地文件夾
    fn scan_local_folder(&self, source: &SourceConfig, options: &ScanOptions) -> Result<Vec<TrackMetadata>, ScanError> {
        let extensions: std::collections::HashSet<String> = 
            source.options().extensions.iter().map(|s| s.to_lowercase()).collect();
        
        let exclude_patterns: Vec<String> = source.options().exclude_patterns.clone();
        
        // 檢查源路徑是否存在
        if !source.path().exists() {
            return Err(ScanError::FileNotFound(
                source.path().to_string_lossy().to_string()
            ));
        }

        let progress = Arc::new(Mutex::new(ScanProgress {
            source_id: source.id().to_string(),
            source_name: source.name().to_string(),
            ..Default::default()
        }));

        let scanned_count = Arc::new(AtomicUsize::new(0));
        let found_count = Arc::new(AtomicUsize::new(0));
        let error_count = Arc::new(AtomicUsize::new(0));

        let files: Vec<PathBuf> = if source.options().recursive {
            self.collect_files_recursive(&source.path(), &extensions, &exclude_patterns)
        } else {
            self.collect_files_non_recursive(&source.path(), &extensions)
        };

        let total_count = files.len();
        {
            let mut progress = progress.lock().unwrap();
            progress.total_count = total_count;
        }

        let tracks: Vec<TrackMetadata> = files.into_par_iter()
            .filter_map(|file_path| {
                let scanned = scanned_count.fetch_add(1, Ordering::SeqCst);
                
                if let Some(callback) = &options.progress_callback {
                    let mut progress = progress.lock().unwrap();
                    progress.scanned_count = scanned + 1;
                    callback(progress.clone());
                }

                match self.process_file(&file_path, source) {
                    Ok(Some(track)) => {
                        found_count.fetch_add(1, Ordering::SeqCst);
                        Some(track)
                    }
                    Ok(None) => None,
                    Err(e) => {
                        error_count.fetch_add(1, Ordering::SeqCst);
                        // 記錄錯誤但不中斷掃描
                        log::warn!("處理文件失敗 {}: {}", file_path.display(), e);
                        None
                    }
                }
            })
            .collect();

        {
            let mut progress = progress.lock().unwrap();
            progress.scanned_count = total_count;
            progress.found_count = found_count.load(Ordering::SeqCst);
            progress.error_count = error_count.load(Ordering::SeqCst);
            progress.is_complete = true;
            if let Some(callback) = &options.progress_callback {
                callback(progress.clone());
            }
        }

        Ok(tracks)
    }

    /// 掃描網盤源
    fn scan_web_disk(&self, source: &SourceConfig, options: &ScanOptions) -> Result<Vec<TrackMetadata>, ScanError> {
        let url = source.path().to_string_lossy();
        
        if !url.starts_with("webdav://") {
            return Err(ScanError::InvalidFileFormat(
                format!("不支持的網盤 URL 格式: {}", url)
            ));
        }

        // 解析 webdav://user:pass@host/path 格式
        let url_str = &url[9..]; // 去掉 webdav:// 前缀
        let (auth_part, path_part) = if let Some(at_pos) = url_str.rfind('@') {
            (&url_str[..at_pos], &url_str[at_pos + 1..])
        } else {
            ("", url_str)
        };

        let username: String;
        let password: String;
        if auth_part.contains(':') {
            let parts: Vec<&str> = auth_part.splitn(2, ':').collect();
            username = parts[0].to_string();
            password = parts.get(1).unwrap_or(&"").to_string();
        } else {
            username = String::new();
            password = String::new();
        }

        // 构建基础 URL (https://host)
        let base_url = if path_part.starts_with("http://") || path_part.starts_with("https://") {
            path_part.to_string()
        } else {
            format!("https://{}", path_part)
        };

        // 创建 WebDAV 客户端
        let client = super::webdav::WebDavClient::new(&base_url)
            .map_err(|e| ScanError::WebDavError(e.to_string()))?;

        let client = if !username.is_empty() {
            client.with_auth(&username, &password)
        } else {
            client
        };

        // 测试连接
        client.test_connection()
            .map_err(|e| ScanError::WebDavError(format!("连接失败: {}", e)))?;

        // 获取所有音频文件
        let dav_items = client.list_all_files("")
            .map_err(|e| ScanError::WebDavError(e.to_string()))?;

        let total_count = dav_items.len();
        let progress = Arc::new(Mutex::new(ScanProgress {
            source_id: source.id().to_string(),
            source_name: source.name().to_string(),
            total_count,
            ..Default::default()
        }));

        let scanned_count = Arc::new(AtomicUsize::new(0));
        let found_count = Arc::new(AtomicUsize::new(0));
        let error_count = Arc::new(AtomicUsize::new(0));

        // 处理每个文件
        let tracks: Vec<TrackMetadata> = dav_items.into_iter()
            .filter_map(|item| {
                let scanned = scanned_count.fetch_add(1, Ordering::SeqCst);
                
                if let Some(callback) = &options.progress_callback {
                    let mut progress = progress.lock().unwrap();
                    progress.scanned_count = scanned + 1;
                    callback(progress.clone());
                }

                match self.process_webdav_file(&client, &item, source) {
                    Ok(Some(track)) => {
                        found_count.fetch_add(1, Ordering::SeqCst);
                        Some(track)
                    }
                    Ok(None) => None,
                    Err(e) => {
                        error_count.fetch_add(1, Ordering::SeqCst);
                        log::warn!("处理 WebDAV 文件失败 {}: {}", item.path, e);
                        None
                    }
                }
            })
            .collect();

        {
            let mut progress = progress.lock().unwrap();
            progress.scanned_count = total_count;
            progress.found_count = found_count.load(Ordering::SeqCst);
            progress.error_count = error_count.load(Ordering::SeqCst);
            progress.is_complete = true;
            if let Some(callback) = &options.progress_callback {
                callback(progress.clone());
            }
        }

        Ok(tracks)
    }

    /// 处理 WebDAV 文件
    fn process_webdav_file(
        &self,
        client: &super::webdav::WebDavClient,
        item: &super::webdav::DavItem,
        source: &SourceConfig
    ) -> Result<Option<TrackMetadata>, ScanError> {
        use std::io::Cursor;

        // 只处理音频文件
        if !item.is_audio_file() {
            return Ok(None);
        }

        let file_name = item.file_name();

        // 下载文件内容用于读取元数据（只下载前 512KB 用于读取标签）
        let file_data = client.download_file(&item.path)
            .map_err(|e| ScanError::WebDavError(e.to_string()))?;

        // 使用 Cursor 模拟文件读取
        let cursor = Cursor::new(&file_data);
        
        // 根据扩展名选择对应的元数据读取器
        let audio_metadata = if item.path.to_lowercase().ends_with(".flac") {
            super::super::audio_metadata::readers::flac::FlacReader::read_from(cursor)
                .map_err(|e| ScanError::MetadataReadError(e.to_string()))?
        } else if item.path.to_lowercase().ends_with(".mp3") {
            super::super::audio_metadata::readers::mp3::Mp3Reader::read_from(cursor)
                .map_err(|e| ScanError::MetadataReadError(e.to_string()))?
        } else if item.path.to_lowercase().ends_with(".m4a") || item.path.to_lowercase().ends_with(".mp4") {
            super::super::audio_metadata::readers::m4a::M4aReader::read_from(cursor)
                .map_err(|e| ScanError::MetadataReadError(e.to_string()))?
        } else if item.path.to_lowercase().ends_with(".ogg") {
            super::super::audio_metadata::readers::ogg::OggReader::read_from(cursor)
                .map_err(|e| ScanError::MetadataReadError(e.to_string()))?
        } else if item.path.to_lowercase().ends_with(".wav") {
            super::super::audio_metadata::readers::wav::WavReader::read_from(cursor)
                .map_err(|e| ScanError::MetadataReadError(e.to_string()))?
        } else {
            return Err(ScanError::UnsupportedFormat(format!("不支持的格式: {}", item.path)));
        };

        // 构建 TrackMetadata
        let track = TrackMetadata {
            id: Uuid::new_v4().to_string(),
            source_id: source.id().to_string(),
            path: PathBuf::from(&item.path),
            file_name: file_name.clone(),
            title: audio_metadata.title.or(Some(file_name)),
            artist: audio_metadata.artist,
            artist_id: None,
            artist_summaries: Vec::new(),
            album: audio_metadata.album,
            album_id: None,
            album_summary: None,
            album_cover_data: None,
            duration: audio_metadata.duration.map(|d| d.as_secs()),
            format: audio_metadata.format.to_string(),
            file_size: item.size,
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

    /// 掃描 WebDev 源
    fn scan_web_dev(&self, source: &SourceConfig, options: &ScanOptions) -> Result<Vec<TrackMetadata>, ScanError> {
        let auth = source.options().webdev_auth.as_ref()
            .ok_or_else(|| ScanError::WebDevError("WebDev 認證信息缺失".to_string()))?;

        let api_base_url = &auth.api_base_url;

        // 創建 HTTP 客戶端
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| ScanError::WebDevError(format!("創建 HTTP 客戶端失敗: {}", e)))?;

        // 構建請求
        let mut request = client.get(format!("{}/api/music/list", api_base_url));

        // 添加認證頭
        if let Some(api_key) = &auth.api_key {
            request = request.header("X-API-Key", api_key);
        }
        if let Some(auth_token) = &auth.auth_token {
            request = request.header("Authorization", format!("Bearer {}", auth_token));
        }

        // 發送請求
        let response = request.send()
            .map_err(|e| ScanError::WebDevError(format!("API 請求失敗: {}", e)))?;

        if !response.status().is_success() {
            return Err(ScanError::WebDevError(
                format!("API 返回錯誤狀態: {}", response.status())
            ));
        }

        let api_response: WebDevResponse = response.json()
            .map_err(|e| ScanError::WebDevError(format!("解析 API 響應失敗: {}", e)))?;

        let total_count = api_response.tracks.len();
        let progress = Arc::new(Mutex::new(ScanProgress {
            source_id: source.id().to_string(),
            source_name: source.name().to_string(),
            total_count,
            ..Default::default()
        }));

        let scanned_count = Arc::new(AtomicUsize::new(0));
        let found_count = Arc::new(AtomicUsize::new(0));
        let error_count = Arc::new(AtomicUsize::new(0));

        // 處理每個曲目
        let tracks: Vec<TrackMetadata> = api_response.tracks.into_iter()
            .filter_map(|track| {
                let scanned = scanned_count.fetch_add(1, Ordering::SeqCst);

                if let Some(callback) = &options.progress_callback {
                    let mut progress = progress.lock().unwrap();
                    progress.scanned_count = scanned + 1;
                    callback(progress.clone());
                }

                match self.create_track_from_webdev(&track, source) {
                    Ok(track_metadata) => {
                        found_count.fetch_add(1, Ordering::SeqCst);
                        Some(track_metadata)
                    }
                    Err(e) => {
                        error_count.fetch_add(1, Ordering::SeqCst);
                        log::warn!("處理 WebDev 曲目失敗 {}: {}", track.id, e);
                        None
                    }
                }
            })
            .collect();

        {
            let mut progress = progress.lock().unwrap();
            progress.scanned_count = total_count;
            progress.found_count = found_count.load(Ordering::SeqCst);
            progress.error_count = error_count.load(Ordering::SeqCst);
            progress.is_complete = true;
            if let Some(callback) = &options.progress_callback {
                callback(progress.clone());
            }
        }

        Ok(tracks)
    }

    /// 從 WebDev API 數據創建 TrackMetadata
    fn create_track_from_webdev(
        &self,
        track: &WebDevTrack,
        source: &SourceConfig
    ) -> Result<TrackMetadata, ScanError> {
        TrackBuilder::build_from_webdev(track, source)
    }

    /// 遞歸收集文件
    fn collect_files_recursive(
        &self,
        dir: &PathBuf,
        extensions: &std::collections::HashSet<String>,
        exclude_patterns: &[String],
    ) -> Vec<PathBuf> {
        let mut files = Vec::new();
        
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                
                let file_name = path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                let should_exclude = exclude_patterns.iter().any(|pattern| {
                    file_name.contains(pattern)
                });
                if should_exclude {
                    continue;
                }

                if path.is_dir() {
                    files.extend(self.collect_files_recursive(&path, extensions, exclude_patterns));
                } else if self.is_audio_file(&path, extensions) {
                    files.push(path);
                }
            }
        }
        
        files
    }

    /// 非遞歸收集文件
    fn collect_files_non_recursive(&self, dir: &PathBuf, extensions: &std::collections::HashSet<String>) -> Vec<PathBuf> {
        let mut files = Vec::new();
        
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && self.is_audio_file(&path, extensions) {
                    files.push(path);
                }
            }
        }
        
        files
    }

    /// 檢查是否為音頻文件
    fn is_audio_file(&self, path: &PathBuf, extensions: &std::collections::HashSet<String>) -> bool {
        let ext = path.extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        extensions.contains(&ext)
    }

    /// 處理單個文件
    fn process_file(&self, file_path: &PathBuf, source: &SourceConfig) -> Result<Option<TrackMetadata>, ScanError> {
        TrackBuilder::build_from_file(file_path, source)
    }

    /// 從緩存加載源的掃描結果
    pub fn load_from_cache(&self, source_id: &str) -> Option<Vec<TrackMetadata>> {
        if let Some(cache_manager) = &self.cache_manager {
            cache_manager.load_source_cache(source_id).ok()
        } else {
            None
        }
    }

    /// 保存掃描結果到緩存
    pub fn save_to_cache(&self, source_id: &str, tracks: &[TrackMetadata]) {
        if let Some(cache_manager) = &self.cache_manager {
            let _ = cache_manager.save_source_cache(source_id, tracks);
        }
    }
}

impl Default for MusicScanner {
    fn default() -> Self {
        Self::new()
    }
}
