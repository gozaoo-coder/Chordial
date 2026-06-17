use crate::module::storage::entry::{StoredEntry, Ttl};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// 内存缓存存储，支持 TTL 自动过期。
///
/// 数据仅存在于进程生命周期内，应用重启后全部清空。
/// 支持三种存活策略：
///
/// | TTL | 行为 |
/// |-----|------|
/// | [`Ttl::Forever`] | 永不过期，仅显式删除 |
/// | [`Ttl::DurationSecs(n)`] | n 秒后自动过期 |
/// | [`Ttl::Session`] | 进程退出后失效 |
///
/// # 并发
///
/// 内部使用 `parking_lot::RwLock` 保护条目映射表。
///
/// # 示例
///
/// ```ignore
/// use crate::module::cache::CacheStore;
/// use crate::module::storage::Ttl;
///
/// let cache = CacheStore::new();
///
/// // 缓存最近播放列表，10 分钟后过期
/// cache.set("recent_tracks", &vec!["id1", "id2"], &Ttl::DurationSecs(600))?;
///
/// // 读取时自动检查过期
/// if let Some(tracks) = cache.get::<Vec<String>>("recent_tracks") {
///     println!("{:?}", tracks);
/// }
///
/// // 清理所有过期条目
/// let removed = cache.clear_expired();
/// ```
pub struct CacheStore {
    /// 所有条目的 TTL 元数据
    entries: RwLock<HashMap<String, StoredEntry>>,
}

impl CacheStore {
    /// 创建空的缓存存储。
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
        }
    }

    /// 获取值并反序列化为目标类型，自动检查过期。
    ///
    /// 若 key 不存在或已过期，返回 `None`。
    /// 过期条目会在首次访问时被自动移除。
    pub fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
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
}

impl Default for CacheStore {
    fn default() -> Self {
        Self::new()
    }
}
