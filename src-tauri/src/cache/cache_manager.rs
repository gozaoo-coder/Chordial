//! 緩存管理器
//!
//! 負責音樂庫數據的本地緩存，支持讀寫和序列化

use super::super::music_source::{MusicLibrary, SourceManager, SourceConfig, TrackMetadata};
use serde::{Deserialize, Serialize};
use std::path::{PathBuf, Path};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CacheError {
    #[error("無法獲取緩存目錄: {0}")]
    CacheDirError(String),
    #[error("創建緩存目錄失敗: {0}")]
    CreateDirError(String),
    #[error("讀取緩存文件失敗: {0}")]
    ReadError(String),
    #[error("寫入緩存文件失敗: {0}")]
    WriteError(String),
    #[error("解析緩存文件失敗: {0}")]
    ParseError(String),
    #[error("緩存文件不存在")]
    NotFound,
}

/// 緩存管理器
#[derive(Debug, Clone)]
pub struct CacheManager {
    cache_dir: PathBuf,
}

impl CacheManager {
    /// 創建新的緩存管理器
    pub fn new() -> Result<Self, CacheError> {
        let cache_dir = get_cache_dir()?;
        Ok(Self { cache_dir })
    }

    /// 使用自定義目錄創建緩存管理器
    pub fn with_directory(cache_dir: PathBuf) -> Self {
        Self { cache_dir }
    }

    /// 獲取緩存目錄
    pub fn cache_dir(&self) -> &PathBuf {
        &self.cache_dir
    }

    /// 獲取音樂庫緩存路徑
    fn library_cache_path(&self) -> PathBuf {
        self.cache_dir.join("library.json")
    }

    /// 獲取源配置緩存路徑
    fn sources_cache_path(&self) -> PathBuf {
        self.cache_dir.join("sources.json")
    }

    /// 獲取指定源的緩存路徑
    fn source_cache_path(&self, source_id: &str) -> PathBuf {
        self.cache_dir.join(format!("source_{}.json", source_id))
    }

    /// 保存音樂庫緩存
    pub fn save_library(&self, library: &MusicLibrary) -> Result<(), CacheError> {
        let path = self.library_cache_path();
        let json = serde_json::to_string_pretty(library)
            .map_err(|e| CacheError::WriteError(e.to_string()))?;
        write_to_file(&path, &json)
    }

    /// 加載音樂庫緩存
    pub fn load_library(&self) -> Result<MusicLibrary, CacheError> {
        let path = self.library_cache_path();
        let json = read_from_file(&path)?;
        serde_json::from_str(&json)
            .map_err(|e| CacheError::ParseError(e.to_string()))
    }

    /// 保存源配置緩存
    pub fn save_sources(&self, sources: &SourceManager) -> Result<(), CacheError> {
        let path = self.sources_cache_path();
        let sources_list = sources.get_all_sources().to_vec();
        let json = serde_json::to_string_pretty(&sources_list)
            .map_err(|e| CacheError::WriteError(e.to_string()))?;
        write_to_file(&path, &json)
    }

    /// 加載源配置緩存
    pub fn load_sources(&self) -> Result<Vec<SourceConfig>, CacheError> {
        let path = self.sources_cache_path();
        let json = read_from_file(&path)?;
        serde_json::from_str(&json)
            .map_err(|e| CacheError::ParseError(e.to_string()))
    }

    /// 保存單個源的掃描結果
    pub fn save_source_cache(&self, source_id: &str, tracks: &[TrackMetadata]) -> Result<(), CacheError> {
        let path = self.source_cache_path(source_id);
        let json = serde_json::to_string_pretty(tracks)
            .map_err(|e| CacheError::WriteError(e.to_string()))?;
        write_to_file(&path, &json)
    }

    /// 加載單個源的掃描結果
    pub fn load_source_cache(&self, source_id: &str) -> Result<Vec<TrackMetadata>, CacheError> {
        let path = self.source_cache_path(source_id);
        let json = read_from_file(&path)?;
        serde_json::from_str(&json)
            .map_err(|e| CacheError::ParseError(e.to_string()))
    }

    /// 檢查源緩存是否存在
    pub fn source_cache_exists(&self, source_id: &str) -> bool {
        self.source_cache_path(source_id).exists()
    }

    /// 刪除源的緩存
    pub fn delete_source_cache(&self, source_id: &str) -> Result<(), CacheError> {
        let path = self.source_cache_path(source_id);
        if path.exists() {
            fs::remove_file(&path)
                .map_err(|e| CacheError::WriteError(e.to_string()))?;
        }
        Ok(())
    }

    /// 清除所有緩存
    pub fn clear_all_cache(&self) -> Result<(), CacheError> {
        let entries = fs::read_dir(&self.cache_dir)
            .map_err(|e| CacheError::ReadError(e.to_string()))?;
        
        for entry in entries {
            let entry = entry.map_err(|e| CacheError::ReadError(e.to_string()))?;
            if entry.path().is_file() {
                fs::remove_file(entry.path())
                    .map_err(|e| CacheError::WriteError(e.to_string()))?;
            }
        }
        Ok(())
    }

    /// 獲取緩存大小（字節）
    pub fn cache_size(&self) -> Result<u64, CacheError> {
        let mut total: u64 = 0;
        let entries = fs::read_dir(&self.cache_dir)
            .map_err(|e| CacheError::ReadError(e.to_string()))?;
        
        for entry in entries {
            let entry = entry.map_err(|e| CacheError::ReadError(e.to_string()))?;
            if let Ok(metadata) = entry.metadata() {
                total += metadata.len();
            }
        }
        Ok(total)
    }
}

/// 獲取緩存目錄
pub fn get_cache_dir() -> Result<PathBuf, CacheError> {
    let path = dirs::cache_dir()
        .ok_or_else(|| CacheError::CacheDirError("無法獲取系統緩存目錄".to_string()))?;
    
    let mut path = path;
    path.push("chordial.app");
    
    if !path.exists() {
        fs::create_dir_all(&path)
            .map_err(|e| CacheError::CreateDirError(e.to_string()))?;
    }
    
    Ok(path)
}

/// 寫入文件
fn write_to_file(path: &Path, content: &str) -> Result<(), CacheError> {
    let mut file = File::create(path)
        .map_err(|e| CacheError::WriteError(e.to_string()))?;
    
    file.write_all(content.as_bytes())
        .map_err(|e| CacheError::WriteError(e.to_string()))?;
    
    Ok(())
}

/// 讀取文件
fn read_from_file(path: &Path) -> Result<String, CacheError> {
    let mut file = File::open(path)
        .map_err(|e| {
            if e.kind() == io::ErrorKind::NotFound {
                CacheError::NotFound
            } else {
                CacheError::ReadError(e.to_string())
            }
        })?;
    
    let mut content = String::new();
    file.read_to_string(&mut content)
        .map_err(|e| CacheError::ReadError(e.to_string()))?;
    
    Ok(content)
}
