//! 来源管理器 — 负责来源元信息的持久化存储和内存挂载。
//!
//! [`SourceManager`] 只关心"哪些来源存在、它们的名称和类型是什么"，
//! 不负责来源实现的注册/注销/查找——那些交给 [`super::registrar::SourceRegistrar`]。
//!
//! # 持久化格式
//!
//! 使用 [`PersistentStore`] 将 `Vec<SourceEntry>` 保存在键 `"source_registry_entries"` 下。

use super::types::SourceType;
use crate::module::storage::persistent::PersistentStore;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 已注册来源的持久化条目（仅保存元信息，不保存实现）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceEntry {
    pub name: String,
    pub source_type: SourceType,
}

/// 来源管理器。
///
/// 负责来源元信息的 **持久化存储** 和 **内存挂载**：
/// - 启动时从磁盘加载已注册来源的元信息
/// - 注册/注销时同步更新内存和磁盘
///
/// 注意：`Arc<dyn MusicSource>` 不可序列化，因此只持久化元信息。
/// 实际的来源实现由 [`SourceRegistrar`](super::registrar::SourceRegistrar) 管理。
pub struct SourceManager {
    /// 持久化存储
    store: PersistentStore,
    /// 内存中的来源条目缓存
    entries: RwLock<Vec<SourceEntry>>,
}

impl SourceManager {
    const KEY: &str = "source_registry_entries";

    /// 创建管理器，从 `path` 加载已持久化的来源元信息。
    pub fn new(path: PathBuf) -> Self {
        let store = PersistentStore::new(path.clone());
        let entries = store
            .get::<Vec<SourceEntry>>(Self::KEY)
            .unwrap_or_default();
        Self {
            store,
            entries: RwLock::new(entries),
        }
    }

    // ── 元信息查询 ────────────────────────────────────

    /// 获取所有已持久化的来源条目。
    pub fn get_entries(&self) -> Vec<SourceEntry> {
        self.entries.read().clone()
    }

    /// 按名称查找来源条目。
    pub fn find_entry(&self, name: &str) -> Option<SourceEntry> {
        self.entries
            .read()
            .iter()
            .find(|e| e.name == name)
            .cloned()
    }

    /// 检查指定名称的来源是否已存在。
    pub fn has(&self, name: &str) -> bool {
        self.entries.read().iter().any(|e| e.name == name)
    }

    /// 获取已注册来源的数量。
    pub fn count(&self) -> usize {
        self.entries.read().len()
    }

    /// 获取所有来源名称列表。
    pub fn list_names(&self) -> Vec<String> {
        self.entries.read().iter().map(|e| e.name.clone()).collect()
    }

    // ── 元信息写入 ────────────────────────────────────

    /// 添加一个来源条目到内存，并持久化到磁盘。
    ///
    /// 若同名来源已存在则返回 `Err`。
    pub fn add_entry(&self, name: &str, source_type: SourceType) -> Result<(), String> {
        let mut entries = self.entries.write();
        if entries.iter().any(|e| e.name == name) {
            return Err(format!("来源 '{}' 已存在", name));
        }
        entries.push(SourceEntry {
            name: name.to_string(),
            source_type,
        });
        drop(entries);
        self.save()
    }

    /// 从内存中移除来源条目，并持久化到磁盘。
    ///
    /// 返回 `true` 表示条目存在并被移除。
    pub fn remove_entry(&self, name: &str) -> bool {
        let mut entries = self.entries.write();
        let len_before = entries.len();
        entries.retain(|e| e.name != name);
        let removed = entries.len() < len_before;
        drop(entries);
        if removed {
            let _ = self.save();
        }
        removed
    }

    // ── 持久化 ───────────────────────────────────────

    /// 将当前内存中的条目写入磁盘。
    pub fn save(&self) -> Result<(), String> {
        let entries = self.entries.read().clone();
        self.store.set(Self::KEY, &entries)?;
        self.store.save()
    }

    /// 手动落盘（委托给 `save`）。
    pub fn flush(&self) -> Result<(), String> {
        self.save()
    }

    /// 从磁盘重新加载条目，覆盖内存中的缓存。
    pub fn reload(&self) {
        self.store.reload();
        if let Some(entries) = self.store.get::<Vec<SourceEntry>>(Self::KEY) {
            *self.entries.write() = entries;
        }
    }

    /// 返回持久化存储的引用，供上层需要直接访问 PersistentStore 时使用。
    pub fn store(&self) -> &PersistentStore {
        &self.store
    }
}
