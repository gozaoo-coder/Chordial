//! 缓存管理器
//!
//! 管理音乐库数据的本地缓存，包括读写和序列化

use crate::music_source::{MusicLibrary, SourceConfig, SourceManager, TrackMetadata};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// 缓存错误类型
#[derive(Debug, Clone, PartialEq)]
pub enum CacheError {
    /// IO 错误
    IoError(String),
    /// 序列化错误
    SerializationError(String),
    /// 反序列化错误
    DeserializationError(String),
    /// 缓存目录不存在
    DirectoryNotFound(String),
    /// 缓存文件不存在
    FileNotFound(String),
    /// 无效的缓存数据
    InvalidData(String),
}

impl std::fmt::Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheError::IoError(msg) => write!(f, "IO 错误: {}", msg),
            CacheError::SerializationError(msg) => write!(f, "序列化错误: {}", msg),
            CacheError::DeserializationError(msg) => write!(f, "反序列化错误: {}", msg),
            CacheError::DirectoryNotFound(path) => write!(f, "缓存目录不存在: {}", path),
            CacheError::FileNotFound(path) => write!(f, "缓存文件不存在: {}", path),
            CacheError::InvalidData(msg) => write!(f, "无效的缓存数据: {}", msg),
        }
    }
}

impl std::error::Error for CacheError {}

impl From<io::Error> for CacheError {
    fn from(err: io::Error) -> Self {
        CacheError::IoError(err.to_string())
    }
}

impl From<serde_json::Error> for CacheError {
    fn from(err: serde_json::Error) -> Self {
        if err.is_io() {
            CacheError::IoError(err.to_string())
        } else {
            CacheError::SerializationError(err.to_string())
        }
    }
}

/// 缓存管理器
///
/// 负责管理音乐库数据的本地缓存，包括：
/// - 音乐库主数据 (library.json)
/// - 源配置列表 (sources.json)
/// - 各源的曲目缓存 (sources/{source_id}.json)
#[derive(Debug, Clone)]
pub struct CacheManager {
    /// 缓存目录路径
    cache_dir: PathBuf,
}

impl CacheManager {
    /// 使用默认缓存目录创建缓存管理器
    ///
    /// 默认缓存目录为系统本地数据目录下的 `chordial/cache`
    pub fn new() -> Self {
        let cache_dir = dirs::data_local_dir()
            .map(|dir| dir.join("chordial").join("cache"))
            .unwrap_or_else(|| PathBuf::from("./cache"));

        Self::with_directory(cache_dir)
    }

    /// 使用指定目录创建缓存管理器
    ///
    /// 如果目录不存在，会自动创建
    pub fn with_directory(path: PathBuf) -> Self {
        // 确保缓存目录存在
        if !path.exists() {
            let _ = fs::create_dir_all(&path);
        }

        // 确保 sources 子目录存在
        let sources_dir = path.join("sources");
        if !sources_dir.exists() {
            let _ = fs::create_dir_all(&sources_dir);
        }

        Self { cache_dir: path }
    }

    /// 获取缓存目录路径
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// 获取 sources 子目录路径
    fn sources_dir(&self) -> PathBuf {
        self.cache_dir.join("sources")
    }

    /// 获取 library.json 文件路径
    fn library_path(&self) -> PathBuf {
        self.cache_dir.join("library.json")
    }

    /// 获取 sources.json 文件路径
    fn sources_path(&self) -> PathBuf {
        self.cache_dir.join("sources.json")
    }

    /// 获取指定源的缓存文件路径
    fn source_cache_path(&self, source_id: &str) -> PathBuf {
        self.sources_dir().join(format!("{}.json", source_id))
    }

    /// 保存音乐库到缓存
    ///
    /// 将 MusicLibrary 序列化为 JSON 并保存到 library.json
    pub fn save_library(&self, library: &MusicLibrary) -> Result<(), CacheError> {
        let path = self.library_path();
        let json = serde_json::to_string_pretty(library)
            .map_err(|e| CacheError::SerializationError(e.to_string()))?;
        fs::write(&path, json)?;
        Ok(())
    }

    /// 从缓存加载音乐库
    ///
    /// 从 library.json 反序列化 MusicLibrary
    pub fn load_library(&self) -> Result<MusicLibrary, CacheError> {
        let path = self.library_path();
        if !path.exists() {
            return Err(CacheError::FileNotFound(path.to_string_lossy().to_string()));
        }

        let content = fs::read_to_string(&path)?;
        let library: MusicLibrary = serde_json::from_str(&content)
            .map_err(|e| CacheError::DeserializationError(e.to_string()))?;
        Ok(library)
    }

    /// 保存源配置列表到缓存
    ///
    /// 将 SourceManager 中的所有源配置序列化为 JSON 并保存到 sources.json
    pub fn save_sources(&self, source_manager: &SourceManager) -> Result<(), CacheError> {
        let path = self.sources_path();
        let sources = source_manager.get_all_sources();
        let json = serde_json::to_string_pretty(sources)
            .map_err(|e| CacheError::SerializationError(e.to_string()))?;
        fs::write(&path, json)?;
        Ok(())
    }

    /// 从缓存加载源配置列表
    ///
    /// 从 sources.json 反序列化 SourceConfig 列表
    pub fn load_sources(&self) -> Result<Vec<SourceConfig>, CacheError> {
        let path = self.sources_path();
        if !path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&path)?;
        let sources: Vec<SourceConfig> = serde_json::from_str(&content)
            .map_err(|e| CacheError::DeserializationError(e.to_string()))?;
        Ok(sources)
    }

    /// 保存指定源的曲目缓存
    ///
    /// 将曲目列表序列化为 JSON 并保存到 sources/{source_id}.json
    pub fn save_source_cache(&self, source_id: &str, tracks: &[TrackMetadata]) -> Result<(), CacheError> {
        let path = self.source_cache_path(source_id);
        let json = serde_json::to_string_pretty(tracks)
            .map_err(|e| CacheError::SerializationError(e.to_string()))?;
        fs::write(&path, json)?;
        Ok(())
    }

    /// 从缓存加载指定源的曲目
    ///
    /// 从 sources/{source_id}.json 反序列化 TrackMetadata 列表
    pub fn load_source_cache(&self, source_id: &str) -> Result<Vec<TrackMetadata>, CacheError> {
        let path = self.source_cache_path(source_id);
        if !path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&path)?;
        let tracks: Vec<TrackMetadata> = serde_json::from_str(&content)
            .map_err(|e| CacheError::DeserializationError(e.to_string()))?;
        Ok(tracks)
    }

    /// 删除指定源的缓存文件
    ///
    /// 删除 sources/{source_id}.json 文件
    pub fn delete_source_cache(&self, source_id: &str) -> Result<(), CacheError> {
        let path = self.source_cache_path(source_id);
        if path.exists() {
            fs::remove_file(&path)?;
        }
        Ok(())
    }

    /// 清除所有缓存
    ///
    /// 删除缓存目录中的所有文件和子目录
    pub fn clear_all_cache(&self) -> Result<(), CacheError> {
        // 删除 library.json
        let library_path = self.library_path();
        if library_path.exists() {
            fs::remove_file(&library_path)?;
        }

        // 删除 sources.json
        let sources_path = self.sources_path();
        if sources_path.exists() {
            fs::remove_file(&sources_path)?;
        }

        // 删除 sources 目录中的所有文件
        let sources_dir = self.sources_dir();
        if sources_dir.exists() {
            for entry in fs::read_dir(&sources_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    fs::remove_file(&path)?;
                }
            }
        }

        Ok(())
    }

    /// 计算缓存总大小（字节）
    ///
    /// 遍历缓存目录中的所有文件，计算总大小
    pub fn cache_size(&self) -> Result<u64, CacheError> {
        let mut total_size: u64 = 0;

        // 计算 library.json 大小
        let library_path = self.library_path();
        if library_path.exists() {
            if let Ok(metadata) = fs::metadata(&library_path) {
                total_size += metadata.len();
            }
        }

        // 计算 sources.json 大小
        let sources_path = self.sources_path();
        if sources_path.exists() {
            if let Ok(metadata) = fs::metadata(&sources_path) {
                total_size += metadata.len();
            }
        }

        // 计算 sources 目录中所有文件的大小
        let sources_dir = self.sources_dir();
        if sources_dir.exists() {
            for entry in fs::read_dir(&sources_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    if let Ok(metadata) = fs::metadata(&path) {
                        total_size += metadata.len();
                    }
                }
            }
        }

        Ok(total_size)
    }
}

impl Default for CacheManager {
    fn default() -> Self {
        Self::with_directory(PathBuf::from("./cache"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::music_source::{Album, Artist, SourceType};
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn create_test_cache_manager() -> (CacheManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let cache_manager = CacheManager::with_directory(temp_dir.path().to_path_buf());
        (cache_manager, temp_dir)
    }

    fn create_test_library() -> MusicLibrary {
        let mut library = MusicLibrary::new();

        // 添加测试歌手
        let artist = Artist::new("artist_1".to_string(), "测试歌手".to_string());
        library.artists.push(artist);

        // 添加测试专辑
        let album = Album::new(
            "album_1".to_string(),
            "测试专辑".to_string(),
            "artist_1".to_string(),
            "测试歌手".to_string(),
        );
        library.albums.push(album);

        library
    }

    #[test]
    fn test_cache_manager_creation() {
        let (cache_manager, _temp_dir) = create_test_cache_manager();
        assert!(cache_manager.cache_dir().exists());
        assert!(cache_manager.sources_dir().exists());
    }

    #[test]
    fn test_save_and_load_library() {
        let (cache_manager, _temp_dir) = create_test_cache_manager();
        let library = create_test_library();

        // 保存音乐库
        cache_manager.save_library(&library).unwrap();
        assert!(cache_manager.library_path().exists());

        // 加载音乐库
        let loaded = cache_manager.load_library().unwrap();
        assert_eq!(loaded.artists.len(), library.artists.len());
        assert_eq!(loaded.albums.len(), library.albums.len());
    }

    #[test]
    fn test_save_and_load_sources() {
        let (cache_manager, temp_dir) = create_test_cache_manager();
        let mut source_manager = SourceManager::new();

        // 创建真实存在的临时目录
        let test_path = temp_dir.path().join("music");
        std::fs::create_dir(&test_path).unwrap();

        // 添加测试源
        source_manager.add_local_folder(test_path, true).unwrap();

        // 保存源配置
        cache_manager.save_sources(&source_manager).unwrap();
        assert!(cache_manager.sources_path().exists());

        // 加载源配置
        let loaded = cache_manager.load_sources().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].source_type, SourceType::LocalFolder);
    }

    #[test]
    fn test_save_and_load_source_cache() {
        let (cache_manager, _temp_dir) = create_test_cache_manager();
        let source_id = "test_source";
        let tracks: Vec<TrackMetadata> = vec![];

        // 保存源缓存
        cache_manager.save_source_cache(source_id, &tracks).unwrap();
        assert!(cache_manager.source_cache_path(source_id).exists());

        // 加载源缓存
        let loaded = cache_manager.load_source_cache(source_id).unwrap();
        assert_eq!(loaded.len(), 0);
    }

    #[test]
    fn test_delete_source_cache() {
        let (cache_manager, _temp_dir) = create_test_cache_manager();
        let source_id = "test_source";
        let tracks: Vec<TrackMetadata> = vec![];

        // 保存源缓存
        cache_manager.save_source_cache(source_id, &tracks).unwrap();
        assert!(cache_manager.source_cache_path(source_id).exists());

        // 删除源缓存
        cache_manager.delete_source_cache(source_id).unwrap();
        assert!(!cache_manager.source_cache_path(source_id).exists());
    }

    #[test]
    fn test_clear_all_cache() {
        let (cache_manager, temp_dir) = create_test_cache_manager();
        let library = create_test_library();
        let mut source_manager = SourceManager::new();

        // 创建真实存在的临时目录
        let test_path = temp_dir.path().join("music");
        std::fs::create_dir(&test_path).unwrap();
        source_manager.add_local_folder(test_path, true).unwrap();

        // 保存数据
        cache_manager.save_library(&library).unwrap();
        cache_manager.save_sources(&source_manager).unwrap();
        cache_manager.save_source_cache("test", &[]).unwrap();

        // 清除所有缓存
        cache_manager.clear_all_cache().unwrap();

        // 验证文件已删除
        assert!(!cache_manager.library_path().exists());
        assert!(!cache_manager.sources_path().exists());
    }

    #[test]
    fn test_cache_size() {
        let (cache_manager, _temp_dir) = create_test_cache_manager();
        let library = create_test_library();

        // 初始大小应为 0
        let initial_size = cache_manager.cache_size().unwrap();
        assert_eq!(initial_size, 0);

        // 保存数据
        cache_manager.save_library(&library).unwrap();

        // 检查大小是否增加
        let size_after_save = cache_manager.cache_size().unwrap();
        assert!(size_after_save > 0);
    }

    #[test]
    fn test_load_nonexistent_library() {
        let (cache_manager, _temp_dir) = create_test_cache_manager();
        let result = cache_manager.load_library();
        assert!(matches!(result, Err(CacheError::FileNotFound(_))));
    }

    #[test]
    fn test_load_nonexistent_source_cache() {
        let (cache_manager, _temp_dir) = create_test_cache_manager();
        let result = cache_manager.load_source_cache("nonexistent");
        assert!(result.is_ok()); // 应该返回空列表而不是错误
        assert!(result.unwrap().is_empty());
    }
}
