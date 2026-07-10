use crate::module::perf;
use crate::module::storage::entry::{StoredEntry, Ttl};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::time::Instant;

/// 二进制 Blob 条目的 TTL 元数据（数据在磁盘，元数据在内存）。
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct BlobEntry {
    /// 创建时间
    created_at: Instant,
    /// 过期截止时间，`None` 表示永不过期
    expires_at: Option<Instant>,
    /// 磁盘上的文件名（不含路径）
    file_name: String,
}

impl BlobEntry {
    fn new(file_name: String, ttl: &Ttl) -> Self {
        let expires_at = match ttl {
            Ttl::Forever => None,
            Ttl::DurationSecs(secs) => Some(Instant::now() + std::time::Duration::from_secs(*secs)),
            Ttl::Session => None,
        };
        Self {
            created_at: Instant::now(),
            expires_at,
            file_name,
        }
    }

    fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(deadline) => Instant::now() >= deadline,
            None => false,
        }
    }
}

/// 内存缓存存储，支持 TTL 自动过期 + 可选的磁盘 Blob 存储。
///
/// 提供两层存储能力：
///
/// | 层级 | 方法 | 存储位置 | 适用场景 |
/// |------|------|---------|---------|
/// | JSON 值 | `set` / `get` / `set_raw` / `get_raw` | 仅内存 | 配置、搜索结果、小型元数据 |
/// | 二进制 Blob | `set_blob` / `get_blob` | 内存元数据 + 磁盘文件 | 音频缓存、图片缓存等大文件 |
///
/// # Blob 磁盘存储
///
/// 调用 [`enable_blob_storage`](Self::enable_blob_storage) 启用后，
/// 二进制数据以单独文件形式保存到指定目录，文件名基于 key 的哈希值。
/// TTL 策略与 JSON 值一致（[`Ttl::Forever`] / [`Ttl::DurationSecs`] / [`Ttl::Session`]），
/// 过期检查在读取时自动进行。
///
/// # 并发
///
/// 内部使用 `parking_lot::RwLock` 保护条目映射表。
pub struct CacheStore {
    /// JSON 值条目的 TTL 元数据（值在内存中）
    entries: RwLock<HashMap<String, StoredEntry>>,
    /// Blob 条目的 TTL 元数据（值在磁盘文件中）
    blob_entries: RwLock<HashMap<String, BlobEntry>>,
    /// Blob 磁盘存储目录（`None` 表示未启用）
    blob_dir: RwLock<Option<PathBuf>>,
}

impl CacheStore {
    /// 创建空的缓存存储（不启用 Blob 磁盘存储）。
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            blob_entries: RwLock::new(HashMap::new()),
            blob_dir: RwLock::new(None),
        }
    }

    /// 启用 Blob 磁盘存储。
    ///
    /// 设置后，[`set_blob`](Self::set_blob) / [`get_blob`](Self::get_blob)
    /// 会将二进制数据写入/读取该目录下的文件。
    ///
    /// 目录不存在时自动创建。若已启用，调用此方法将覆盖旧路径。
    pub fn enable_blob_storage(&self, dir: PathBuf) -> Result<(), String> {
        fs::create_dir_all(&dir)
            .map_err(|e| format!("创建缓存目录失败: {}", e))?;
        *self.blob_dir.write() = Some(dir);
        Ok(())
    }

    /// 是否已启用 Blob 磁盘存储。
    pub fn blob_storage_enabled(&self) -> bool {
        self.blob_dir.read().is_some()
    }

    /// 获取 Blob 磁盘存储目录。
    pub fn blob_dir(&self) -> Option<PathBuf> {
        self.blob_dir.read().clone()
    }

    // ── 辅助：key → 安全文件名 ─────────────────────

    /// 将缓存 key 转换为安全的文件名。
    ///
    /// 使用哈希避免文件名中包含非法字符或路径分隔符。
    fn key_to_filename(key: &str) -> String {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        key.hash(&mut hasher);
        format!("{:016x}.blob", hasher.finish())
    }

    /// 获取指定 key 对应的 Blob 文件完整路径。
    fn blob_path(&self, key: &str) -> Option<PathBuf> {
        let dir = self.blob_dir.read();
        let dir = dir.as_ref()?;
        Some(dir.join(Self::key_to_filename(key)))
    }

    // ── JSON 值操作 ──────────────────────────────────

    /// 获取值并反序列化为目标类型，自动检查过期。
    ///
    /// 若 key 不存在或已过期，返回 `None`。
    /// 过期条目会在首次访问时被自动移除。
    pub fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        let _scope = perf::scope("cache.get");
        let entries = self.entries.read();
        let entry = entries.get(key)?;
        if entry.is_expired() {
            drop(entries);
            self.remove(key);
            return None;
        }
        serde_json::from_value(entry.value.clone()).ok()
    }

    /// 获取原始 JSON 值，自动检查过期。
    pub fn get_raw(&self, key: &str) -> Option<Value> {
        let entries = self.entries.read();
        let entry = entries.get(key)?;
        if entry.is_expired() {
            drop(entries);
            self.remove(key);
            return None;
        }
        Some(entry.value.clone())
    }

    /// 存入值，指定 TTL 策略。
    ///
    /// 若 key 已存在，旧的 TTL 会被覆盖。
    pub fn set<T: Serialize>(&self, key: &str, value: &T, ttl: &Ttl) -> Result<(), String> {
        let _scope = perf::scope("cache.set");
        let json =
            serde_json::to_value(value).map_err(|e| format!("序列化失败: {}", e))?;
        let entry = StoredEntry::new(json, ttl);
        self.entries.write().insert(key.to_string(), entry);
        Ok(())
    }

    /// 存入原始 JSON 值，指定 TTL 策略。
    pub fn set_raw(&self, key: &str, value: Value, ttl: &Ttl) {
        let entry = StoredEntry::new(value, ttl);
        self.entries.write().insert(key.to_string(), entry);
    }

    /// 续期：按给定的 `ttl` 重置 key 的过期时间。
    ///
    /// 返回 `true` 表示 key 存在且续期成功，`false` 表示 key 不存在。
    pub fn touch(&self, key: &str, ttl: &Ttl) -> bool {
        let mut entries = self.entries.write();
        if let Some(entry) = entries.get_mut(key) {
            entry.touch(ttl);
            true
        } else {
            false
        }
    }

    /// 删除指定 key，返回 `true` 表示存在并被删除。
    pub fn remove(&self, key: &str) -> bool {
        self.entries.write().remove(key).is_some()
    }

    /// 检查 key 是否存在且未过期。
    pub fn has(&self, key: &str) -> bool {
        match self.entries.read().get(key) {
            Some(entry) if !entry.is_expired() => true,
            _ => false,
        }
    }

    /// 获取所有未过期 key 的列表。
    pub fn keys(&self) -> Vec<String> {
        self.entries
            .read()
            .iter()
            .filter(|(_, e)| !e.is_expired())
            .map(|(k, _)| k.clone())
            .collect()
    }

    /// 清空所有数据（含未过期条目）。
    pub fn clear(&self) {
        self.entries.write().clear();
    }

    /// 清理所有已过期条目，返回清理数量。
    pub fn clear_expired(&self) -> usize {
        let mut entries = self.entries.write();
        let before = entries.len();
        entries.retain(|_, e| !e.is_expired());
        before - entries.len()
    }

    // ── Blob 二进制操作（磁盘存储 + 内存 TTL）───────

    /// 存入二进制数据到磁盘，指定 TTL 策略。
    ///
    /// 需先调用 [`enable_blob_storage`](Self::enable_blob_storage) 设置存储目录，
    /// 否则返回 `Err`。
    ///
    /// 若 key 已存在，旧的 TTL 和文件会被覆盖。
    pub fn set_blob(&self, key: &str, data: &[u8], ttl: &Ttl) -> Result<(), String> {
        let _scope = perf::scope("cache.set_blob");
        let path = self
            .blob_path(key)
            .ok_or_else(|| "Blob 存储未启用，请先调用 enable_blob_storage".to_string())?;

        // 写入磁盘
        fs::write(&path, data).map_err(|e| format!("写入缓存文件失败: {}", e))?;

        // 记录 TTL 元数据
        let file_name = Self::key_to_filename(key);
        let entry = BlobEntry::new(file_name, ttl);
        self.blob_entries.write().insert(key.to_string(), entry);

        Ok(())
    }

    /// 从磁盘读取二进制数据，自动检查过期。
    ///
    /// 若 key 不存在、已过期或文件丢失，返回 `None`。
    /// 过期条目会在首次访问时自动清理元数据和磁盘文件。
    pub fn get_blob(&self, key: &str) -> Option<Vec<u8>> {
        let _scope = perf::scope("cache.get_blob");
        // 检查 TTL
        let expired = {
            let blob_entries = self.blob_entries.read();
            let entry = blob_entries.get(key)?;
            entry.is_expired()
        };

        if expired {
            self.remove_blob(key);
            return None;
        }

        let path = self.blob_path(key)?;

        // 读取文件
        match fs::read(&path) {
            Ok(data) => Some(data),
            Err(_) => {
                // 文件丢失，清理元数据
                self.blob_entries.write().remove(key);
                None
            }
        }
    }

    /// 删除指定的 Blob 条目（含元数据和磁盘文件）。
    ///
    /// 返回 `true` 表示条目存在并被删除。
    pub fn remove_blob(&self, key: &str) -> bool {
        let existed = self.blob_entries.write().remove(key).is_some();
        if existed {
            if let Some(path) = self.blob_path(key) {
                let _ = fs::remove_file(&path);
            }
        }
        existed
    }

    /// 检查 Blob key 是否存在且未过期。
    pub fn has_blob(&self, key: &str) -> bool {
        match self.blob_entries.read().get(key) {
            Some(entry) => {
                if entry.is_expired() {
                    return false;
                }
                // 确认文件存在
                self.blob_path(key)
                    .map(|p| p.exists())
                    .unwrap_or(false)
            }
            None => false,
        }
    }

    /// 获取所有未过期的 Blob key 列表。
    pub fn blob_keys(&self) -> Vec<String> {
        self.blob_entries
            .read()
            .iter()
            .filter(|(_, e)| !e.is_expired())
            .map(|(k, _)| k.clone())
            .collect()
    }

    /// 清空所有 Blob 数据（含元数据和磁盘文件）。
    pub fn clear_blobs(&self) {
        let entries: Vec<_> = self.blob_entries.read().keys().cloned().collect();
        for key in &entries {
            self.remove_blob(key);
        }
    }

    /// 清理所有已过期的 Blob 条目，返回清理数量。
    pub fn clear_expired_blobs(&self) -> usize {
        let expired_keys: Vec<String> = {
            self.blob_entries
                .read()
                .iter()
                .filter(|(_, e)| e.is_expired())
                .map(|(k, _)| k.clone())
                .collect()
        };
        let count = expired_keys.len();
        for key in &expired_keys {
            self.remove_blob(key);
        }
        count
    }
}

impl Default for CacheStore {
    fn default() -> Self {
        Self::new()
    }
}
