//! 应用上下文 — server 层的统一入口。
//!
//! `AppContext` 持有整个音乐系统的所有组件实例（配置 / 存储 / 缓存 / 音乐库 /
//! 来源管理器 / 注册器 / 本地来源），替代原 `commands.rs` 中的 `LazyLock` /
//! `OnceLock` 全局单例。
//!
//! # 设计目标
//!
//! - **进程无关**：不假设单进程，可同时被 `chordial-tauri`（库调用）和
//!   `chordial-server`（HTTP）实例化。
//! - **依赖注入**：所有消费者（Tauri 命令、axum 路由）通过持有的 `Arc<AppContext>`
//!   访问系统组件，而非全局静态。
//!
//! # 使用示例
//!
//! ```ignore
//! use chordial_core::AppContext;
//!
//! let ctx = AppContext::new_default_dir()?;
//! let count = ctx.library.song_count();
//! ```

use crate::module::cache::store::CacheStore;
use crate::module::config::store::ConfigStore;
use crate::module::music_library::library::MusicLibrary;
use crate::module::music_localSource;
use crate::module::music_localSource::source::LocalMusicSource;
use crate::module::music_source::manager::SourceManager;
use crate::module::music_source::registrar::{SourceCleanup, SourceRegistrar};
use crate::module::p2p::P2pManager;
use crate::module::perf;
use crate::module::storage::persistent::PersistentStore;
use std::path::PathBuf;
use std::sync::Arc;

/// 整个音乐系统的运行时上下文。
///
/// 各字段均为 `Arc`，便于在不同消费者间共享（Tauri State、axum State、后台任务）。
pub struct AppContext {
    /// 自动防抖落盘的配置存储（`config.json`）。
    pub config: Arc<ConfigStore>,
    /// 手动落盘的通用持久化存储（`storage.json`）。
    pub store: Arc<PersistentStore>,
    /// 内存 TTL 缓存（含可选 Blob 磁盘存储）。
    pub cache: Arc<CacheStore>,
    /// 音乐库（Song / Artist / Album / Lyric）。
    pub library: Arc<MusicLibrary>,
    /// 来源元信息管理器（持久化来源注册表）。
    pub manager: Arc<SourceManager>,
    /// 来源注册器（持有运行中的来源实现 + 级联清理回调）。
    pub registrar: Arc<SourceRegistrar>,
    /// 本地音乐来源实现。
    pub local_source: Arc<LocalMusicSource>,
    /// P2P 资源共享管理器。
    pub p2p: Arc<P2pManager>,
}

impl AppContext {
    /// 构建 AppContext，所有数据文件放在 `data_dir` 下。
    ///
    /// 文件布局：
    /// - `data_dir/config.json`
    /// - `data_dir/storage.json`
    /// - `data_dir/music_library.json`
    /// - `data_dir/source_registry.json`
    /// - `data_dir/local_source_folders.json`
    /// - `data_dir/cache_blobs/`（Blob 缓存目录）
    pub fn new(data_dir: PathBuf) -> Result<Self, String> {
        let _scope = perf::scope("app.new");
        // ── 配置 / 存储 / 缓存 ──
        let config = Arc::new(ConfigStore::new(data_dir.join("config.json")));
        let store = Arc::new(PersistentStore::new(data_dir.join("storage.json")));
        let cache = Arc::new(CacheStore::new());

        // ── 音乐库 + 来源系统 ──
        let library = Arc::new(MusicLibrary::new(data_dir.join("music_library.json")));
        let manager = Arc::new(SourceManager::new(data_dir.join("source_registry.json")));

        // Arc<MusicLibrary> → Arc<dyn SourceCleanup>（级联清理回调）
        let cleanup: Arc<dyn SourceCleanup> = library.clone();
        let registrar = Arc::new(SourceRegistrar::new(manager.clone(), cleanup));

        // ── Blob 缓存磁盘目录 ──
        if let Err(e) = cache.enable_blob_storage(data_dir.join("cache_blobs")) {
            eprintln!("[chordial] 启用 Blob 缓存失败: {}", e);
        }

        // ── 本地音乐来源（must-source，自动初始化）──
        let local_folder_store_path = data_dir.join("local_source_folders.json");
        let local_source = music_localSource::init_local_source(
            local_folder_store_path,
            library.clone(),
            &registrar,
        )
        .map_err(|e| {
            eprintln!("[chordial] 初始化本地音乐来源失败: {}", e);
            format!("初始化本地音乐来源失败: {}", e)
        })?;

        // ── P2P 资源共享管理器 ──
        let p2p = P2pManager::new(library.clone(), registrar.clone(), config.clone());

        Ok(Self {
            config,
            store,
            cache,
            library,
            manager,
            registrar,
            local_source,
            p2p,
        })
    }

    /// 使用系统默认配置目录（`dirs::config_dir()/chordial`）构建 AppContext。
    pub fn new_default_dir() -> Result<Self, String> {
        let data_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("chordial");
        Self::new(data_dir)
    }
}
