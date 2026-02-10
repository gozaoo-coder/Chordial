use rusqlite::{Connection, Result as SqliteResult};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};

use super::beat_detection::AnalysisResult;

/// 可序列化的分析结果（用于数据库存储）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableAnalysisResult {
    pub bpm: f64,
    pub beat_positions: Vec<f64>,
    pub onset_curve: Vec<f32>,
    pub downbeat_position: Option<f64>,
}

impl From<AnalysisResult> for SerializableAnalysisResult {
    fn from(result: AnalysisResult) -> Self {
        Self {
            bpm: result.bpm,
            beat_positions: result.beat_positions,
            onset_curve: result.onset_curve,
            downbeat_position: result.downbeat_position,
        }
    }
}

impl From<SerializableAnalysisResult> for AnalysisResult {
    fn from(result: SerializableAnalysisResult) -> Self {
        Self {
            bpm: result.bpm,
            beat_positions: result.beat_positions,
            onset_curve: result.onset_curve,
            downbeat_position: result.downbeat_position,
        }
    }
}

/// 分析缓存管理器
pub struct AnalysisCache {
    conn: Arc<Mutex<Connection>>,
    db_path: PathBuf,
}

impl AnalysisCache {
    /// 创建或打开缓存数据库
    pub fn new<P: AsRef<Path>>(db_path: P) -> SqliteResult<Self> {
        let conn = Connection::open(&db_path)?;
        
        let cache = Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path: db_path.as_ref().to_path_buf(),
        };
        
        cache.init_tables()?;
        Ok(cache)
    }
    
    /// 在应用数据目录创建缓存
    pub fn in_app_data() -> anyhow::Result<Self> {
        let app_data_dir = dirs::data_dir()
            .ok_or_else(|| anyhow::anyhow!("无法获取应用数据目录"))?
            .join("chordial");
        
        std::fs::create_dir_all(&app_data_dir)?;
        
        let db_path = app_data_dir.join("analysis_cache.db");
        Ok(Self::new(db_path)?)
    }
    
    /// 初始化数据库表
    fn init_tables(&self) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS analysis_cache (
                file_hash TEXT PRIMARY KEY,
                file_path TEXT NOT NULL,
                bpm REAL NOT NULL,
                beat_positions TEXT NOT NULL,
                onset_curve BLOB NOT NULL,
                downbeat_position REAL,
                file_size INTEGER,
                modified_time INTEGER,
                analyzed_at INTEGER NOT NULL
            )",
            [],
        )?;
        
        // 创建索引加速查询
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_file_path ON analysis_cache(file_path)",
            [],
        )?;
        
        Ok(())
    }
    
    /// 计算文件哈希（用于检测文件是否变化）
    fn compute_file_hash(path: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        path.hash(&mut hasher);
        
        // 尝试获取文件元数据
        if let Ok(metadata) = std::fs::metadata(path) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
                    duration.as_secs().hash(&mut hasher);
                }
            }
            metadata.len().hash(&mut hasher);
        }
        
        format!("{:016x}", hasher.finish())
    }
    
    /// 获取文件元数据
    fn get_file_metadata(path: &str) -> Option<(u64, u64)> {
        let metadata = std::fs::metadata(path).ok()?;
        let size = metadata.len();
        let modified = metadata.modified().ok()?;
        let modified_secs = modified.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs();
        Some((size, modified_secs))
    }
    
    /// 保存分析结果到缓存
    pub fn save(&self, file_path: &str, result: &AnalysisResult) -> anyhow::Result<()> {
        let file_hash = Self::compute_file_hash(file_path);
        let (file_size, modified_time) = Self::get_file_metadata(file_path)
            .unzip();
        
        // 序列化数据
        let beat_positions_json = serde_json::to_string(&result.beat_positions)?;
        let onset_curve_bytes = bincode::serialize(&result.onset_curve)?;
        
        let conn = self.conn.lock().unwrap();
        
        conn.execute(
            "INSERT OR REPLACE INTO analysis_cache 
             (file_hash, file_path, bpm, beat_positions, onset_curve, downbeat_position, 
              file_size, modified_time, analyzed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            [
                &file_hash as &dyn rusqlite::ToSql,
                &file_path as &dyn rusqlite::ToSql,
                &result.bpm as &dyn rusqlite::ToSql,
                &beat_positions_json as &dyn rusqlite::ToSql,
                &onset_curve_bytes as &dyn rusqlite::ToSql,
                &result.downbeat_position as &dyn rusqlite::ToSql,
                &file_size.map(|v| v as i64).unwrap_or(-1) as &dyn rusqlite::ToSql,
                &modified_time.map(|v| v as i64).unwrap_or(-1) as &dyn rusqlite::ToSql,
                &chrono::Utc::now().timestamp() as &dyn rusqlite::ToSql,
            ],
        )?;
        
        Ok(())
    }
    
    /// 从缓存加载分析结果
    pub fn load(&self, file_path: &str) -> anyhow::Result<Option<AnalysisResult>> {
        let file_hash = Self::compute_file_hash(file_path);
        
        let conn = self.conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT bpm, beat_positions, onset_curve, downbeat_position 
             FROM analysis_cache WHERE file_hash = ?1"
        )?;
        
        let result = stmt.query_row([&file_hash], |row| {
            let bpm: f64 = row.get(0)?;
            let beat_positions_json: String = row.get(1)?;
            let onset_curve_bytes: Vec<u8> = row.get(2)?;
            let downbeat_position: Option<f64> = row.get(3)?;
            
            let beat_positions: Vec<f64> = serde_json::from_str(&beat_positions_json)
                .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
            let onset_curve: Vec<f32> = bincode::deserialize(&onset_curve_bytes)
                .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
            
            Ok(AnalysisResult {
                bpm,
                beat_positions,
                onset_curve,
                downbeat_position,
            })
        });
        
        match result {
            Ok(analysis) => Ok(Some(analysis)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
    
    /// 检查缓存是否有效（文件是否被修改）
    pub fn is_cache_valid(&self, file_path: &str) -> anyhow::Result<bool> {
        let file_hash = Self::compute_file_hash(file_path);
        let current_metadata = Self::get_file_metadata(file_path);
        
        let conn = self.conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT file_size, modified_time FROM analysis_cache WHERE file_hash = ?1"
        )?;
        
        let cached = stmt.query_row([&file_hash], |row| {
            let size: i64 = row.get(0)?;
            let modified: i64 = row.get(1)?;
            Ok((size, modified))
        });
        
        match cached {
            Ok((cached_size, cached_modified)) => {
                if let Some((current_size, current_modified)) = current_metadata {
                    Ok(cached_size == current_size as i64 && cached_modified == current_modified as i64)
                } else {
                    Ok(false)
                }
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(false),
            Err(e) => Err(e.into()),
        }
    }
    
    /// 获取或创建分析结果（如果缓存有效则直接返回，否则执行分析）
    pub fn get_or_analyze<F>(&self, file_path: &str, analyzer: F) -> anyhow::Result<AnalysisResult>
    where
        F: FnOnce() -> anyhow::Result<AnalysisResult>,
    {
        // 先尝试从缓存加载
        if let Some(result) = self.load(file_path)? {
            if self.is_cache_valid(file_path)? {
                log::info!("使用缓存的分析结果: {}", file_path);
                return Ok(result);
            }
        }
        
        // 执行分析
        log::info!("分析音频文件: {}", file_path);
        let result = analyzer()?;
        
        // 保存到缓存
        if let Err(e) = self.save(file_path, &result) {
            log::warn!("保存分析缓存失败: {}", e);
        }
        
        Ok(result)
    }
    
    /// 删除指定文件的缓存
    pub fn invalidate(&self, file_path: &str) -> anyhow::Result<()> {
        let file_hash = Self::compute_file_hash(file_path);
        let conn = self.conn.lock().unwrap();
        
        conn.execute(
            "DELETE FROM analysis_cache WHERE file_hash = ?1",
            [&file_hash],
        )?;
        
        Ok(())
    }
    
    /// 清空所有缓存
    pub fn clear_all(&self) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM analysis_cache", [])?;
        Ok(())
    }
    
    /// 获取缓存统计信息
    pub fn get_stats(&self) -> anyhow::Result<CacheStats> {
        let conn = self.conn.lock().unwrap();
        
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM analysis_cache",
            [],
            |row| row.get(0)
        )?;
        
        let total_size: Option<i64> = conn.query_row(
            "SELECT SUM(LENGTH(onset_curve)) FROM analysis_cache",
            [],
            |row| row.get(0)
        ).ok();
        
        Ok(CacheStats {
            entry_count: count as usize,
            total_data_size: total_size.unwrap_or(0) as usize,
        })
    }
}

/// 缓存统计信息
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub entry_count: usize,
    pub total_data_size: usize,
}

/// 批量分析引擎（带缓存和并行处理）
pub struct BatchAnalysisEngine {
    cache: AnalysisCache,
}

impl BatchAnalysisEngine {
    pub fn new(cache: AnalysisCache) -> Self {
        Self { cache }
    }
    
    /// 批量分析文件列表（使用Rayon并行处理）
    pub fn batch_analyze<F>(
        &self,
        paths: Vec<String>,
        analyzer: F,
        progress_callback: impl Fn(usize, usize, &str) + Send + Sync,
    ) -> Vec<(String, anyhow::Result<AnalysisResult>)>
    where
        F: Fn(&str) -> anyhow::Result<AnalysisResult> + Send + Sync,
    {
        use rayon::prelude::*;
        
        let total = paths.len();
        let processed = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        
        paths
            .into_par_iter()
            .map(|path| {
                let result = self.cache.get_or_analyze(&path, || analyzer(&path));
                
                let count = processed.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                progress_callback(count, total, &path);
                
                (path, result)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_cache_creation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_cache.db");
        
        let cache = AnalysisCache::new(&db_path).unwrap();
        let stats = cache.get_stats().unwrap();
        
        assert_eq!(stats.entry_count, 0);
    }
    
    #[test]
    fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_cache.db");
        let cache = AnalysisCache::new(&db_path).unwrap();
        
        // 创建一个临时文件用于测试
        let test_file = temp_dir.path().join("test.mp3");
        std::fs::write(&test_file, b"test audio data").unwrap();
        let test_path = test_file.to_str().unwrap();
        
        let result = AnalysisResult {
            bpm: 128.5,
            beat_positions: vec![0.0, 0.46875, 0.9375],
            onset_curve: vec![0.1, 0.5, 1.0, 0.5, 0.1],
            downbeat_position: Some(0.0),
        };
        
        // 保存
        cache.save(test_path, &result).unwrap();
        
        // 加载
        let loaded = cache.load(test_path).unwrap().unwrap();
        assert!((loaded.bpm - result.bpm).abs() < 0.001);
        assert_eq!(loaded.beat_positions.len(), result.beat_positions.len());
        assert_eq!(loaded.downbeat_position, result.downbeat_position);
    }
    
    #[test]
    fn test_cache_invalidation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_cache.db");
        let cache = AnalysisCache::new(&db_path).unwrap();
        
        let test_file = temp_dir.path().join("test.mp3");
        std::fs::write(&test_file, b"test audio data").unwrap();
        let test_path = test_file.to_str().unwrap();
        
        let result = AnalysisResult {
            bpm: 128.5,
            beat_positions: vec![0.0, 0.46875],
            onset_curve: vec![0.1, 0.5, 1.0],
            downbeat_position: Some(0.0),
        };
        
        cache.save(test_path, &result).unwrap();
        
        // 验证缓存有效
        assert!(cache.is_cache_valid(test_path).unwrap());
        
        // 删除缓存
        cache.invalidate(test_path).unwrap();
        
        // 验证缓存已删除
        assert!(cache.load(test_path).unwrap().is_none());
    }
}
