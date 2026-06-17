use super::backend::StorageBackend;
use super::file::FileBackend;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;

/// 持久化键值存储。
///
/// 数据保存在 JSON 文件中，应用重启后数据保留。
/// 内存中维护一份缓存副本，读取不访问磁盘，写入仅修改缓存，
/// 调用 [`save()`](Self::save) 或 [`save_if_dirty()`](Self::save_if_dirty) 时才落盘。
///
/// 所有条目等效于 `Ttl::Forever`（永久保留）。
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
/// // 写入配置
/// store.set("volume", &0.75)?;
/// store.set("theme", &"dark")?;
///
/// // 持久化到磁盘
/// store.save()?;
///
/// // 读取配置
/// let volume: Option<f64> = store.get("volume");
/// ```
pub struct PersistentStore {
    /// 文件后端
    backend: FileBackend,
    /// 内存缓存，避免每次读取都访问磁盘
    cache: RwLock<HashMap<String, Value>>,
    /// 是否有未落盘的修改
    dirty: RwLock<bool>,
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
        let backend = FileBackend::new(path);
        let cache = backend
            .read()
            .unwrap_or_else(|_| HashMap::new());
        Self {
            backend,
            cache: RwLock::new(cache),
            dirty: RwLock::new(false),
        }
    }

    // ── 读取 ─────────────────────────────────────────

    /// 读取值并反序列化为目标类型 `T`。
    ///
    /// key 不存在或类型不匹配时返回 `None`。
    pub fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        self.cache
            .read()
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// 读取原始 [`serde_json::Value`]，不做反序列化。
    pub fn get_raw(&self, key: &str) -> Option<Value> {
        self.cache.read().get(key).cloned()
    }

    // ── 写入 ─────────────────────────────────────────

    /// 写入键值对。
    ///
    /// 仅修改内存缓存，**不立即落盘**。需要调用 [`save()`](Self::save)
    /// 或 [`save_if_dirty()`](Self::save_if_dirty) 手动持久化。
    pub fn set<T: Serialize>(&self, key: &str, value: &T) -> Result<(), String> {
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
    pub fn save(&self) -> Result<(), String> {
        let data = self.cache.read().clone();
        self.backend.write(&data)?;
        *self.dirty.write() = false;
        Ok(())
    }

    /// 仅当存在未保存修改时才写入磁盘。
    ///
    /// 适合在退出时调用，避免不必要的磁盘 I/O。
    pub fn save_if_dirty(&self) -> Result<(), String> {
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
}
