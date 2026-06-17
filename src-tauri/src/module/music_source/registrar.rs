//! 来源注册器 — 管理来源实现的注册、注销和查找。
//!
//! [`SourceRegistrar`] 持有：
//! - 一个 [`SourceManager`] 用于元信息持久化
//! - 一个 `HashMap<String, Arc<dyn MusicSource>>` 用于存放来源实现
//! - 一个 [`SourceCleanup`] 回调，在注销时联动清理 [`MusicLibrary`](crate::module::music_library::library::MusicLibrary) 中的孤儿数据
//!
//! # 注销时的级联清理
//!
//! 当调用 [`unregister`](SourceRegistrar::unregister) 注销某个来源时，注册器会
//! 通过 [`SourceCleanup`] 通知音乐库移除所有引用该来源的实体。
//! 若某实体的 `source_ids` 被全部清空，该实体本身也会被删除。

use super::manager::{SourceEntry, SourceManager};
use super::traits::MusicSource;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// 来源注销时的清理回调接口。
///
/// 由 [`MusicLibrary`](crate::module::music_library::library::MusicLibrary) 实现，
/// 在来源注销时清理所有引用该来源的实体（歌曲、艺人、专辑、歌词）。
pub trait SourceCleanup: Send + Sync {
    /// 从所有实体中移除指定来源的 `SourceId`，若某实体因此失去全部来源引用则删除该实体。
    fn remove_source_from_all_entities(&self, source_name: &str) -> Result<(), String>;
}

/// 来源注册器。
///
/// 负责来源的 **注册/注销/查询** 生命周期，并在注销时联动音乐库做清理。
pub struct SourceRegistrar {
    /// 元信息管理器（持久化 + 内存缓存）
    manager: Arc<SourceManager>,
    /// 已注册的来源实现（name → trait object）
    sources: RwLock<HashMap<String, Arc<dyn MusicSource>>>,
    /// 清理回调（指向 MusicLibrary）
    cleanup: Arc<dyn SourceCleanup>,
}

impl SourceRegistrar {
    /// 创建注册器。
    ///
    /// # 参数
    ///
    /// - `manager`：来源元信息管理器
    /// - `cleanup`：清理回调，通常传入 [`MusicLibrary`](crate::module::music_library::library::MusicLibrary) 的 `Arc`
    pub fn new(manager: Arc<SourceManager>, cleanup: Arc<dyn SourceCleanup>) -> Self {
        Self {
            manager,
            sources: RwLock::new(HashMap::new()),
            cleanup,
        }
    }

    // ── 元信息代理 ────────────────────────────────────

    /// 返回已持久化的来源条目列表。
    pub fn get_entries(&self) -> Vec<SourceEntry> {
        self.manager.get_entries()
    }

    /// 按名称查找来源条目。
    pub fn find_entry(&self, name: &str) -> Option<SourceEntry> {
        self.manager.find_entry(name)
    }

    // ── 注册 / 注销 ───────────────────────────────────

    /// 注册一个来源实现。
    ///
    /// 1. 若元信息尚未持久化，则通过 [`SourceManager`] 持久化
    /// 2. 始终将实现写入内存映射表（覆盖已有实现，支持重启后重新注册）
    ///
    /// 幂等：重复注册同名来源不会报错，只会更新实现。
    pub fn register(&self, source: Arc<dyn MusicSource>) -> Result<(), String> {
        let name = source.name().to_string();
        let source_type = source.source_type();

        // 仅在元信息不存在时才持久化（重启时条目已从磁盘恢复）
        if !self.manager.has(&name) {
            self.manager.add_entry(&name, source_type)?;
        }

        // 始终写入内存（覆盖已有实现）
        self.sources.write().insert(name, source);
        Ok(())
    }

    /// 注销一个来源。
    ///
    /// 执行以下操作：
    /// 1. 从内存映射表中移除来源实现
    /// 2. 从 [`SourceManager`] 中移除元信息
    /// 3. 通过 [`SourceCleanup`] 回调，通知音乐库移除所有引用该来源的实体
    ///
    /// 返回被移除的实现，若来源不存在则返回 `None`。
    pub fn unregister(&self, name: &str) -> Option<Arc<dyn MusicSource>> {
        // 1. 从内存中移除
        let removed = self.sources.write().remove(name);

        // 2. 从持久化中移除
        self.manager.remove_entry(name);

        // 3. 通知音乐库清理关联数据
        if removed.is_some() {
            let _ = self.cleanup.remove_source_from_all_entities(name);
        }

        removed
    }

    /// 批量加载持久化条目并注册。
    ///
    /// 对每个已持久化的 [`SourceEntry`] 调用 `factory` 重建来源实现，
    /// 然后注册到注册器中。已注册的同名来源会被跳过。
    ///
    /// # 参数
    ///
    /// - `factory`：接收 `&SourceEntry`，返回 `Option<Arc<dyn MusicSource>>`。
    ///   返回 `None` 表示该条目暂时无法重建（例如需要用户输入 API key），跳过即可。
    pub fn load_all<F>(&self, factory: F) -> Result<usize, String>
    where
        F: Fn(&SourceEntry) -> Option<Arc<dyn MusicSource>>,
    {
        let entries = self.manager.get_entries();
        let mut loaded = 0;
        for entry in &entries {
            if self.sources.read().contains_key(&entry.name) {
                continue; // 已注册，跳过
            }
            if let Some(source) = factory(entry) {
                let name = source.name().to_string();
                self.sources.write().insert(name, source);
                loaded += 1;
            }
        }
        Ok(loaded)
    }

    // ── 查询 ──────────────────────────────────────────

    /// 按名称获取来源实现。
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

    /// 返回元信息管理器的引用。
    pub fn manager(&self) -> &Arc<SourceManager> {
        &self.manager
    }
}
