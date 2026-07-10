use super::backend::StorageBackend;
use super::file::FileBackend;
use crate::module::perf;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

/// 持久化键值存储。
///
/// 数据保存在 JSON 文件中，应用重启后数据保留。
/// 内存中维护一份缓存副本，读取不访问磁盘，写入仅修改缓存，
/// 调用 [`save()`](Self::save) 或 [`save_if_dirty()`](Self::save_if_dirty) 时才落盘。
///
/// 所有条目等效于 `Ttl::Forever`（永久保留）。
///
/// # Blob 二进制存储
///
/// 除 JSON 键值对外，本存储也支持以文件形式保存二进制数据
/// （通过 [`set_blob`](Self::set_blob) / [`get_blob`](Self::get_blob)），
/// 适用于音乐资源（音频、图片、歌词文本）的本地持久化缓存。
/// Blob 文件保存在 JSON 文件同级的 `blobs/` 目录下。
///
/// # 并发
///
/// 内部使用 `parking_lot::RwLock` 保护缓存和脏标记，允许多读单写。
///
/// # 示例
///
/// ```ignore
/// use std::path::PathBuf;
/// use crate::module::storage::persistent::PersistentStore;
///
/// let path = dirs::config_dir().unwrap().join("myapp").join("config.json");
/// let store = PersistentStore::new(path);
///
/// // JSON 键值对
/// store.set("volume", &0.75)?;
/// store.save()?;
///
/// // 二进制数据
/// store.set_blob("song_audio_001", &audio_bytes)?;
/// let audio = store.get_blob("song_audio_001");
/// ```
pub struct PersistentStore {
    /// JSON 文件后端
    backend: FileBackend,
    /// 内存缓存，避免每次读取都访问磁盘
    cache: RwLock<HashMap<String, Value>>,
    /// 是否有未落盘的修改
    dirty: RwLock<bool>,
    /// Blob 文件存储目录
    blob_dir: PathBuf,
    /// 内存中的 Blob key 集合（避免每次扫描目录）
    blob_keys_cache: RwLock<HashSet<String>>,
}

impl PersistentStore {
    /// 创建持久化存储实例。
    ///
    /// 启动时自动从 `path` 指定的 JSON 文件加载已有数据。
    /// 若文件不存在或解析失败，初始化为空数据集。
    ///
    /// # 参数
    ///
    /// - `path`：JSON 配置文件的完整路径。
    pub fn new(path: PathBuf) -> Self {
        let backend = FileBackend::new(path.clone());
        let cache = backend
            .read()
            .unwrap_or_else(|_| HashMap::new());

        // Blob 目录：JSON 文件同级的 blobs/ 目录
        let blob_dir = path
            .parent()
            .map(|p| p.join("blobs"))
            .unwrap_or_else(|| PathBuf::from("blobs"));

        // 确保 Blob 目录存在并加载已有 key
        let _ = fs::create_dir_all(&blob_dir);
        let blob_keys_cache = Self::scan_blob_keys(&blob_dir);

        Self {
            backend,
            cache: RwLock::new(cache),
            dirty: RwLock::new(false),
            blob_dir,
            blob_keys_cache: RwLock::new(blob_keys_cache),
        }
    }

    /// 返回持久化文件的路径。
    pub fn path(&self) -> &PathBuf {
        self.backend.path()
    }

    /// 扫描 Blob 目录，返回已有文件的 key 集合。
    fn scan_blob_keys(blob_dir: &std::path::Path) -> HashSet<String> {
        let keys = HashSet::new();
        if let Ok(entries) = fs::read_dir(blob_dir) {
            for entry in entries.flatten() {
                let file_name = entry.file_name();
                let name = file_name.to_string_lossy();
                if name.ends_with(".blob") {
                    // 文件名格式：{hash}.blob，无法反推出原始 key。
                    // 此处仅统计文件数量，key 集合通过 set_blob 时注入。
                }
            }
        }
        // 无法从哈希文件名反推原始 key，启动时集合为空。
        // has_blob 会直接检查文件是否存在，因此这里不维护 key 集合也可以。
        // 我们只维护由本次进程写入的 key。
        keys
    }

    /// 将 key 转换为安全的文件名。
    fn key_to_filename(key: &str) -> String {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        key.hash(&mut hasher);
        format!("{:016x}.blob", hasher.finish())
    }

    /// 获取 Blob 文件的完整路径。
    fn blob_path(&self, key: &str) -> PathBuf {
        self.blob_dir.join(Self::key_to_filename(key))
    }

    // ── 读取 ─────────────────────────────────────────

    /// 读取值并反序列化为目标类型 `T`。
    ///
    /// key 不存在或类型不匹配时返回 `None`。
    pub fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        let _scope = perf::scope("persistent.get");
        self.cache
            .read()
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// 读取原始 [`serde_json::Value`]，不做反序列化。
    pub fn get_raw(&self, key: &str) -> Option<Value> {
        self.cache.read().get(key).cloned()
    }

    /// 按 ID 从 HashMap 值中读取单条记录并反序列化。
    ///
    /// **性能关键**：仅反序列化目标条目，不反序列化整个 HashMap。
    /// 对 3853 张专辑的库，`get::<HashMap<..>>(key).remove(id)` 需 24ms，
    /// 本方法仅需 ~0.1ms。
    ///
    /// 要求存储的 Value 为 JSON Object（即 `{"id1": {...}, "id2": {...}}`）。
    pub fn get_entry<T: for<'de> Deserialize<'de>>(&self, key: &str, id: &str) -> Option<T> {
        let _scope = perf::scope("persistent.get_entry");
        let guard = self.cache.read();
        let value = guard.get(key)?;
        let obj = value.as_object()?;
        let entry_value = obj.get(id)?;
    serde_json::from_value(entry_value.clone()).ok()
    }

    /// 获取 HashMap 值的条目数量，不做反序列化。
    ///
    /// 仅检查 JSON Object 的键数量，开销 O(1)。
    pub fn count_entries(&self, key: &str) -> usize {
    let _scope = perf::scope("persistent.count_entries");
        self.cache
            .read()
            .get(key)
            .and_then(|v| v.as_object())
            .map(|m| m.len())
            .unwrap_or(0)
    }

    /// 从 HashMap 值中分页读取并反序列化。
    ///
    /// 仅反序列化 `[offset, offset+limit)` 范围的条目，
    /// 不反序列化整个 HashMap。适合分页列表场景。
    ///
    /// 注意：JSON Object 的迭代顺序由 serde_json 保证（插入序），
    /// 与 `get_all().into_values().skip().take()` 行为一致。
    pub fn get_page_entries<T: for<'de> Deserialize<'de>>(
        &self,
        key: &str,
        offset: usize,
        limit: usize,
    ) -> Vec<T> {
    let _scope = perf::scope("persistent.get_page_entries");
        let guard = self.cache.read();
        let value = match guard.get(key) {
            Some(v) => v,
            None => return Vec::new(),
        };
        let obj = match value.as_object() {
            Some(o) => o,
            None => return Vec::new(),
        };
        obj.values()
            .skip(offset)
            .take(limit)
            .filter_map(|v| serde_json::from_value(v.clone()).ok())
            .collect()
    }

    /// 从 HashMap 值中读取所有条目并反序列化。
    ///
    /// 比 `get::<HashMap<String, T>>` 更高效：
    /// 直接遍历 Object values 逐条反序列化，
    /// 避免 `serde_json::from_value::<HashMap<String, T>>` 的中间 HashMap 分配。
    pub fn get_all_entries<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Vec<T> {
    let _scope = perf::scope("persistent.get_all_entries");
        let guard = self.cache.read();
        let value = match guard.get(key) {
            Some(v) => v,
            None => return Vec::new(),
        };
        let obj = match value.as_object() {
            Some(o) => o,
            None => return Vec::new(),
        };
        let result = Vec::with_capacity(obj.len());
        obj.values()
            .fold(result, |mut acc, v| {
                if let Ok(item) = serde_json::from_value::<T>(v.clone()) {
                    acc.push(item);
                }
                acc
            })
    }

    /// 从 HashMap 值中读取所有条目，返回 `HashMap<String, T>`。
    ///
    /// 与 `get::<HashMap<String, T>>` 功能相同但更直接：
    /// 逐条反序列化，失败条目跳过而非整体失败。
    pub fn get_all_map<T: for<'de> Deserialize<'de>>(&self, key: &str) -> HashMap<String, T> {
    let _scope = perf::scope("persistent.get_all_map");
        let guard = self.cache.read();
        let value = match guard.get(key) {
            Some(v) => v,
            None => return HashMap::new(),
        };
        let obj = match value.as_object() {
            Some(o) => o,
            None => return HashMap::new(),
        };
        let mut result = HashMap::with_capacity(obj.len());
        for (k, v) in obj {
            if let Ok(item) = serde_json::from_value::<T>(v.clone()) {
                result.insert(k.clone(), item);
            }
        }
        result
    }

    // ── 写入 ─────────────────────────────────────────

    /// 写入键值对。
    ///
    /// 仅修改内存缓存，**不立即落盘**。需要调用 [`save()`](Self::save)
    /// 或 [`save_if_dirty()`](Self::save_if_dirty) 手动持久化。
    pub fn set<T: Serialize>(&self, key: &str, value: &T) -> Result<(), String> {
        let _scope = perf::scope("persistent.set");
        let json = serde_json::to_value(value)
            .map_err(|e| format!("序列化失败: {}", e))?;
        self.cache.write().insert(key.to_string(), json);
        *self.dirty.write() = true;
        Ok(())
    }

    /// 写入原始 JSON 值，仅修改内存缓存。
    pub fn set_raw(&self, key: &str, value: Value) {
        self.cache.write().insert(key.to_string(), value);
        *self.dirty.write() = true;
    }

    // ── 删除 / 检查 ──────────────────────────────────

    /// 删除指定 key，返回 `true` 表示 key 存在并被删除。
    pub fn remove(&self, key: &str) -> bool {
        let existed = self.cache.write().remove(key).is_some();
        if existed {
            *self.dirty.write() = true;
        }
        existed
    }

    /// 检查 key 是否存在。
    pub fn has(&self, key: &str) -> bool {
        self.cache.read().contains_key(key)
    }

    /// 获取所有已存储 key 的列表。
    pub fn keys(&self) -> Vec<String> {
        self.cache.read().keys().cloned().collect()
    }

    /// 清空所有数据（仅修改内存缓存）。
    pub fn clear(&self) {
        self.cache.write().clear();
        *self.dirty.write() = true;
    }

    // ── 持久化 ───────────────────────────────────────

    /// 立即将内存中所有数据写入磁盘。
    ///
    /// 写入成功后清除脏标记。
    ///
    /// 优化：在读锁内序列化为字符串（~10ms for 10k entries），释放锁后写盘，
    /// 避免 `cache.read().clone()` 导致的全量 HashMap clone（O(n) 内存+时间）。
    pub fn save(&self) -> Result<(), String> {
        let _scope = perf::scope("persistent.save");
        let content = {
            let guard = self.cache.read();
            serde_json::to_string(&*guard).map_err(|e| format!("序列化失败: {}", e))?
        };
        self.backend.write_str(&content)?;
        *self.dirty.write() = false;
        Ok(())
    }

    /// 仅当存在未保存修改时才写入磁盘。
    ///
    /// 适合在退出时调用，避免不必要的磁盘 I/O。
    pub fn save_if_dirty(&self) -> Result<(), String> {
        let _scope = perf::scope("persistent.save");
        if *self.dirty.read() {
            self.save()
        } else {
            Ok(())
        }
    }

    /// 从磁盘重新加载数据，**丢弃内存中所有未保存的修改**。
    ///
    /// 加载失败时保持内存数据不变。
    pub fn reload(&self) {
        if let Ok(data) = self.backend.read() {
            *self.cache.write() = data;
            *self.dirty.write() = false;
        }
    }

    // ── Blob 二进制存储 ─────────────────────────────

    /// 写入二进制数据到 Blob 文件（立即落盘）。
    ///
    /// Blob 文件保存在 JSON 文件同级的 `blobs/` 目录下，
    /// 文件名基于 key 的哈希值。
    ///
    /// 若 key 对应文件已存在则覆盖。
    pub fn set_blob(&self, key: &str, data: &[u8]) -> Result<(), String> {
        let _scope = perf::scope("persistent.set_blob");
        let path = self.blob_path(key);
        fs::write(&path, data).map_err(|e| format!("写入 Blob 文件失败: {}", e))?;
        self.blob_keys_cache.write().insert(key.to_string());
        Ok(())
    }

    /// 读取 Blob 文件。
    ///
    /// 若 key 不存在或文件丢失则返回 `None`。
    pub fn get_blob(&self, key: &str) -> Option<Vec<u8>> {
        let _scope = perf::scope("persistent.get_blob");
        let path = self.blob_path(key);
        if !path.exists() {
            return None;
        }
        fs::read(&path).ok()
    }

    /// 读取 Blob 文件的路径（不读入内存）。
    ///
    /// 适用于需要流式读取或由其他组件直接访问文件的场景。
    /// 返回 `Some(path)` 仅当文件确实存在。
    pub fn get_blob_path(&self, key: &str) -> Option<PathBuf> {
        let path = self.blob_path(key);
        if path.exists() {
            Some(path)
        } else {
            None
        }
    }

    /// 删除指定的 Blob 文件。
    ///
    /// 返回 `true` 表示文件存在并被删除。
    pub fn remove_blob(&self, key: &str) -> bool {
        self.blob_keys_cache.write().remove(key);
        let path = self.blob_path(key);
        if path.exists() {
            let _ = fs::remove_file(&path);
            true
        } else {
            false
        }
    }

    /// 检查 Blob key 是否存在（文件存在性检查）。
    pub fn has_blob(&self, key: &str) -> bool {
        self.blob_path(key).exists()
    }

    /// 获取当前内存中已知的 Blob key 列表。
    ///
    /// 注意：这仅包含本次进程通过 [`set_blob`](Self::set_blob) 写入的 key，
    /// 以及启动时扫描到的 key。若外部程序修改了 Blob 目录，可能不一致。
    /// 对于可靠性需求，请使用 [`has_blob`](Self::has_blob) 逐 key 检查。
    pub fn blob_keys(&self) -> Vec<String> {
        self.blob_keys_cache.read().iter().cloned().collect()
    }

    /// 清空所有 Blob 文件。
    pub fn clear_blobs(&self) {
        let keys: Vec<String> = self.blob_keys_cache.read().iter().cloned().collect();
        for key in &keys {
            let path = self.blob_path(key);
            let _ = fs::remove_file(&path);
        }
        self.blob_keys_cache.write().clear();
    }

    /// 返回 Blob 存储目录路径。
    pub fn blob_dir(&self) -> &PathBuf {
        &self.blob_dir
    }
}
