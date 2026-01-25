//! 音樂掃描器
//!
//! 使用多線程並行掃描音樂文件，支持進度回調和取消操作

use super::super::music_source::{SourceConfig, SourceType, TrackMetadata, MusicSource};
use super::super::audio_metadata::{read_metadata, AudioFormat};
use super::super::lyric_enhancer::{find_lyric_file, enhance_metadata_with_lyrics};
use super::super::cache::CacheManager;
use rayon::prelude::*;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use uuid::Uuid;

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
            cache_manager: CacheManager::new().ok(),
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

        match source.source_type() {
            SourceType::LocalFolder => {
                result.tracks = self.scan_local_folder(source, options);
            }
            SourceType::WebDisk => {
                result.tracks = self.scan_web_disk(source, options);
            }
        }

        result.duration = start_time.elapsed();
        result
    }

    /// 批量掃描多個源（並行）
    pub fn scan_sources(&self, sources: &[SourceConfig], options: &ScanOptions) -> Vec<ScanResult> {
        let parallel_tasks = options.parallel_tasks.clamp(1, 8);
        
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
    fn scan_local_folder(&self, source: &SourceConfig, options: &ScanOptions) -> Vec<TrackMetadata> {
        let extensions: std::collections::HashSet<String> = 
            source.options().extensions.iter().map(|s| s.to_lowercase()).collect();
        
        let exclude_patterns: Vec<String> = source.options().exclude_patterns.clone();
        
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
                    Err(_) => {
                        error_count.fetch_add(1, Ordering::SeqCst);
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

        tracks
    }

    /// 掃描網盤源
    fn scan_web_disk(&self, source: &SourceConfig, _options: &ScanOptions) -> Vec<TrackMetadata> {
        let url = source.path().to_string_lossy();
        
        if !url.starts_with("webdev://") {
            return Vec::new();
        }

        let path_str = &url[9..];
        
        Vec::new()
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
                
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    let should_exclude = exclude_patterns.iter().any(|pattern| {
                        file_name.contains(pattern)
                    });
                    if should_exclude {
                        continue;
                    }
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
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            extensions.contains(&ext.to_lowercase())
        } else {
            false
        }
    }

    /// 處理單個文件
    fn process_file(&self, file_path: &PathBuf, source: &SourceConfig) -> Result<Option<TrackMetadata>, ()> {
        let metadata = match std::fs::metadata(file_path) {
            Ok(m) => m,
            Err(_) => return Err(()),
        };

        let file_name = file_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("未知")
            .to_string();

        let mut audio_metadata = match read_metadata(file_path) {
            Ok(meta) => meta,
            Err(_) => return Err(()),
        };

        // 尝试查找并解析外部歌词文件
        if let Some(lyric_content) = find_lyric_file(file_path) {
            enhance_metadata_with_lyrics(&mut audio_metadata, Some(lyric_content));
        }

        let track = TrackMetadata {
            id: Uuid::new_v4().to_string(),
            source_id: source.id().to_string(),
            path: file_path.clone(),
            file_name,
            title: audio_metadata.title.or(Some(file_path.file_stem()
                .and_then(|s| Some(s.to_string_lossy().to_string()))
                .unwrap_or_else(|| "未知標題".to_string()))),
            artist: audio_metadata.artist,
            artist_id: None,
            album: audio_metadata.album,
            album_id: None,
            // 提取专辑封面（第一张封面图片）
            album_cover_data: audio_metadata.pictures.iter()
                .find(|p| p.is_cover())
                .or_else(|| audio_metadata.pictures.first())
                .map(|p| {
                    // 将图片数据编码为 Base64
                    let base64_data = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &p.data);
                    // 根据 MIME 类型添加前缀
                    let mime_prefix = match p.mime_type.as_str() {
                        "image/jpeg" => "data:image/jpeg;base64,",
                        "image/png" => "data:image/png;base64,",
                        "image/gif" => "data:image/gif;base64,",
                        _ => "data:image/jpeg;base64,", // 默认使用 JPEG
                    };
                    format!("{}{}", mime_prefix, base64_data)
                }),
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
            // 提取歌词
            lyrics: audio_metadata.lyrics,
            synced_lyrics: audio_metadata.synced_lyrics.map(|lines| {
                // 将 LyricLine 序列化为 JSON 字符串
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

use std::fs;
