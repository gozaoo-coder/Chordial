use super::traits::MusicSource;
use super::types::SourceType;
use crate::module::storage::persistent::PersistentStore;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

/// 已注册来源的持久化条目（仅保存元信息，不保存实现）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SourceEntry {
    name: String,
    source_type: SourceType,
}

/// 来源注册中心。
///
/// 管理所有 [`MusicSource`] 实现的注册与注销，并通过 [`PersistentStore`]
/// 持久化已注册来源的元信息（名称 + 类型），在每次启动后自动加载。
///
/// 注意：`Arc<dyn MusicSource>` 不可序列化，因此只持久化元信息。
/// 实际实现器需要在启动时由调用方重新注入。
pub struct SourceRegistry {
    /// 已注册的来源实现
    sources: RwLock<HashMap<String, Arc<dyn MusicSource>>>,
    /// 持久化存储（仅存元信息）
    store: PersistentStore,
}

impl SourceRegistry {
    const KEY: &str = "source_registry_entries";

    /// 创建注册中心，从 `path` 加载已持久化的来源元信息。
    ///
    /// 注意：此方法只恢复元信息，实际的实现器需要通过 [`register`](Self::register) 重新注入。
    pub fn new(path: PathBuf) -> Self {
        Self {
            sources: RwLock::new(HashMap::new()),
            store: PersistentStore::new(path),
        }
    }

    // ── 持久化的元信息 ────────────────────────────────

    /// 读取已持久化的来源条目列表（名称 + 类型），用于启动时重建注册。
    pub fn load_entries(&self) -> Vec<SourceEntry> {
        self.store.get::<Vec<SourceEntry>>(Self::KEY).unwrap_or_default()
    }

    /// 将当前注册的来源元信息写入持久化存储。
    fn save_entries(&self) -> Result<(), String> {
        let entries: Vec<SourceEntry> = self
            .sources
            .read()
            .values()
            .map(|s| SourceEntry {
                name: s.name().to_string(),
                source_type: s.source_type(),
            })
            .collect();
        self.store.set(Self::KEY, &entries)?;
        self.store.save()
    }

    // ── 注册 / 注销 ───────────────────────────────────

    /// 注册一个来源实现。
    ///
    /// 若同名来源已存在则返回 `Err`。
    pub fn register(&self, source: Arc<dyn MusicSource>) -> Result<(), String> {
        let name = source.name().to_string();
        let mut sources = self.sources.write();
        if sources.contains_key(&name) {
            return Err(format!("来源 '{}' 已注册", name));
        }
        sources.insert(name, source);
        drop(sources);
        self.save_entries()
    }

    /// 注销一个来源。
    ///
    /// 返回被移除的实现，若来源不存在则返回 `None`。
    pub fn unregister(&self, name: &str) -> Option<Arc<dyn MusicSource>> {
        let result = self.sources.write().remove(name);
        if result.is_some() {
            let _ = self.save_entries();
        }
        result
    }

    // ── 查询 ──────────────────────────────────────────

    /// 按名称获取来源。
    pub fn get(&self, name: &str) -> Option<Arc<dyn MusicSource>> {
        self.sources.read().get(name).cloned()
    }

    /// 列出所有已注册来源的名称。
    pub fn list_names(&self) -> Vec<String> {
        self.sources.read().keys().cloned().collect()
    }

    /// 检查来源是否已注册。
    pub fn has(&self, name: &str) -> bool {
        self.sources.read().contains_key(name)
    }

    /// 获取已注册来源的数量。
    pub fn count(&self) -> usize {
        self.sources.read().len()
    }

    // ── 持久化 ───────────────────────────────────────

    /// 手动落盘。
    pub fn save(&self) -> Result<(), String> {
        self.save_entries()
    }
}
