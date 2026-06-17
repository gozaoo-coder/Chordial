//! Tauri 命令层 — 将 Config / Storage / Cache 三层存储封装为前端可调用的 API。
//!
//! # 命令一览
//!
//! ## 配置（config_*）— 自动防抖落盘，适合音量、主题等设置
//!
//! | 命令 | 功能 |
//! |------|------|
//! | `config_get` | 读取配置项 |
//! | `config_set` | 写入配置项（自动防抖落盘） |
//! | `config_remove` | 删除配置项 |
//! | `config_has` | 检查配置项是否存在 |
//! | `config_keys` | 获取所有配置键 |
//! | `config_clear` | 清空所有配置 |
//! | `config_flush` | 立即落盘（跳过防抖） |
//! | `config_reload` | 从磁盘重新加载 |
//!
//! ## 持久化存储（storage_*）— 手动落盘，适合播放列表、乐库索引
//!
//! | 命令 | 功能 |
//! |------|------|
//! | `storage_get` | 读取键值 |
//! | `storage_set` | 写入键值（不落盘） |
//! | `storage_remove` | 删除键 |
//! | `storage_has` | 检查键是否存在 |
//! | `storage_keys` | 获取所有键 |
//! | `storage_clear` | 清空所有数据 |
//! | `storage_save` | 手动持久化到磁盘 |
//!
//! ## 缓存（cache_*）— 纯内存，支持 TTL 过期
//!
//! | 命令 | 功能 |
//! |------|------|
//! | `cache_get` | 读取缓存值（自动检查过期） |
//! | `cache_set` | 写入缓存值并指定 TTL |
//! | `cache_remove` | 删除缓存项 |
//! | `cache_has` | 检查缓存项是否存在 |
//! | `cache_keys` | 获取所有未过期的键 |
//! | `cache_clear` | 清空所有缓存 |
//! | `cache_clear_expired` | 清理过期条目 |
//! | `cache_touch` | 续期缓存项 |
//!
//! # 数据文件
//!
//! | 存储层 | 文件路径 |
//! |--------|---------|
//! | Config | `{config_dir}/chordial/config.json` |
//! | Storage | `{config_dir}/chordial/storage.json` |
//! | Cache | 无（纯内存） |
//!
//! - Windows: `%APPDATA%/chordial/`
//! - macOS: `~/Library/Application Support/chordial/`
//! - Linux: `~/.config/chordial/`

use crate::module::cache::store::CacheStore;
use crate::module::config::store::ConfigStore;
use crate::module::music_library::library::MusicLibrary;
use crate::module::music_localSource;
use crate::module::music_localSource::source::LocalMusicSource;
use crate::module::music_source::manager::SourceManager;
use crate::module::music_source::registrar::{SourceCleanup, SourceRegistrar};
use crate::module::music_source::resource;
use crate::module::music_source::types::SourceId;
use crate::module::storage::entry::Ttl;
use crate::module::storage::persistent::PersistentStore;
use serde::Deserialize;
use serde_json::Value;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock, OnceLock};

// ══════════════════════════════════════════════════════════════════════════════
// 全局单例
// ══════════════════════════════════════════════════════════════════════════════

/// 获取 chordial 配置目录路径
pub(crate) fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("chordial")
}

/// 全局配置存储 — 修改后自动防抖落盘
static CONFIG: LazyLock<ConfigStore> = LazyLock::new(|| {
    let path = config_dir().join("config.json");
    ConfigStore::new(path)
});

/// 全局持久化存储 — 手动调用 save() 落盘
static STORE: LazyLock<PersistentStore> = LazyLock::new(|| {
    let path = config_dir().join("storage.json");
    PersistentStore::new(path)
});

/// 全局缓存存储 — 纯内存，支持 TTL
static CACHE: LazyLock<CacheStore> = LazyLock::new(CacheStore::new);

// ══════════════════════════════════════════════════════════════════════════════
// 音乐来源系统全局单例
// ══════════════════════════════════════════════════════════════════════════════

/// 全局音乐库
static MUSIC_LIBRARY: OnceLock<Arc<MusicLibrary>> = OnceLock::new();
/// 全局来源管理器
static SOURCE_MANAGER: OnceLock<Arc<SourceManager>> = OnceLock::new();
/// 全局来源注册器
static SOURCE_REGISTRAR: OnceLock<Arc<SourceRegistrar>> = OnceLock::new();
/// 全局本地音乐来源
static LOCAL_SOURCE: OnceLock<Arc<LocalMusicSource>> = OnceLock::new();

/// 初始化音乐来源系统（音乐库 + 来源管理器 + 注册器）+ Blob 缓存 + 本地来源。
///
/// 应在 Tauri `setup` 阶段调用。
pub fn init_music_system() {
    // ── 音乐库 + 来源系统 ──
    let lib_path = config_dir().join("music_library.json");
    let source_path = config_dir().join("source_registry.json");

    let library = Arc::new(MusicLibrary::new(lib_path));
    let manager = Arc::new(SourceManager::new(source_path));

    // Arc<MusicLibrary> → Arc<dyn SourceCleanup>
    let cleanup: Arc<dyn SourceCleanup> = library.clone();
    let registrar = Arc::new(SourceRegistrar::new(manager.clone(), cleanup));

    MUSIC_LIBRARY.set(library.clone()).ok();
    SOURCE_MANAGER.set(manager).ok();
    SOURCE_REGISTRAR.set(registrar.clone()).ok();

    // ── Blob 缓存磁盘目录 ──
    let cache_blob_dir = config_dir().join("cache_blobs");
    if let Err(e) = CACHE.enable_blob_storage(cache_blob_dir) {
        eprintln!("[chordial] 启用 Blob 缓存失败: {}", e);
    }

    // ── 本地音乐来源（must-source，自动初始化）──
    let local_folder_store_path = config_dir().join("local_source_folders.json");
    match crate::module::music_localSource::init_local_source(
        local_folder_store_path,
        library,
        &registrar,
    ) {
        Ok(local_source) => {
            let _ = LOCAL_SOURCE.set(local_source);
        }
        Err(e) => {
            eprintln!("[chordial] 初始化本地音乐来源失败: {}", e);
        }
    }
}

/// 获取全局来源注册器的引用（用于 Tauri 命令）。
fn source_registrar() -> &'static Arc<SourceRegistrar> {
    SOURCE_REGISTRAR
        .get()
        .expect("音乐来源系统未初始化，请先调用 init_music_system()")
}

/// 获取全局音乐库的引用（用于 Tauri 命令）。
fn music_library() -> &'static Arc<MusicLibrary> {
    MUSIC_LIBRARY
        .get()
        .expect("音乐来源系统未初始化，请先调用 init_music_system()")
}

// ══════════════════════════════════════════════════════════════════════════════
// TTL 参数辅助类型
// ══════════════════════════════════════════════════════════════════════════════

/// 前端传入的 TTL 参数。
///
/// 支持三种格式（`serde_json` 可自动识别）：
/// - `"forever"` → [`Ttl::Forever`]
/// - `"session"` → [`Ttl::Session`]
/// - `{"duration_secs": 600}` → [`Ttl::DurationSecs(600)`]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum TtlArg {
    Forever,
    Session,
    DurationSecs(u64),
}

impl From<TtlArg> for Ttl {
    fn from(arg: TtlArg) -> Self {
        match arg {
            TtlArg::Forever => Ttl::Forever,
            TtlArg::Session => Ttl::Session,
            TtlArg::DurationSecs(n) => Ttl::DurationSecs(n),
        }
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Config 命令 — 自动防抖落盘
// ══════════════════════════════════════════════════════════════════════════════

/// 读取配置项的值。
#[tauri::command]
pub fn config_get(key: String) -> Result<Value, String> {
    CONFIG
        .get_raw(&key)
        .ok_or_else(|| format!("配置项 '{}' 不存在", key))
}

/// 写入配置项，自动防抖落盘（500ms 内连续修改合并为一次写盘）。
#[tauri::command]
pub fn config_set(key: String, value: Value) -> Result<(), String> {
    CONFIG.set_raw(&key, value);
    Ok(())
}

/// 删除配置项，自动防抖落盘。
#[tauri::command]
pub fn config_remove(key: String) -> Result<bool, String> {
    Ok(CONFIG.remove(&key))
}

/// 检查配置项是否存在。
#[tauri::command]
pub fn config_has(key: String) -> Result<bool, String> {
    Ok(CONFIG.has(&key))
}

/// 获取所有配置项的键。
#[tauri::command]
pub fn config_keys() -> Result<Vec<String>, String> {
    Ok(CONFIG.keys())
}

/// 清空所有配置，自动防抖落盘。
#[tauri::command]
pub fn config_clear() -> Result<(), String> {
    CONFIG.clear();
    Ok(())
}

/// 立即同步落盘，跳过防抖定时器。
///
/// 适合在应用退出前调用，确保所有配置修改已持久化。
#[tauri::command]
pub fn config_flush() -> Result<(), String> {
    CONFIG.flush()
}

/// 从磁盘重新加载配置，**丢弃内存中所有未落盘的修改**。
#[tauri::command]
pub fn config_reload() -> Result<(), String> {
    CONFIG.reload();
    Ok(())
}

// ══════════════════════════════════════════════════════════════════════════════
// Storage 命令 — 手动落盘
// ══════════════════════════════════════════════════════════════════════════════

/// 读取指定 key 的值。
#[tauri::command]
pub fn storage_get(key: String) -> Result<Value, String> {
    STORE
        .get_raw(&key)
        .ok_or_else(|| format!("键 '{}' 不存在", key))
}

/// 写入键值对到内存缓存，不立即落盘。
///
/// 如需持久化，调用 [`storage_save`]。
#[tauri::command]
pub fn storage_set(key: String, value: Value) -> Result<(), String> {
    STORE.set_raw(&key, value);
    Ok(())
}

/// 删除指定 key。
#[tauri::command]
pub fn storage_remove(key: String) -> Result<bool, String> {
    Ok(STORE.remove(&key))
}

/// 检查 key 是否存在。
#[tauri::command]
pub fn storage_has(key: String) -> Result<bool, String> {
    Ok(STORE.has(&key))
}

/// 返回所有 key 的列表。
#[tauri::command]
pub fn storage_keys() -> Result<Vec<String>, String> {
    Ok(STORE.keys())
}

/// 清空所有数据（仅修改内存缓存，不立即落盘）。
#[tauri::command]
pub fn storage_clear() -> Result<(), String> {
    STORE.clear();
    Ok(())
}

/// 将内存中的全部数据持久化到磁盘。
///
/// 建议在应用退出前或重要数据修改后调用。
#[tauri::command]
pub fn storage_save() -> Result<(), String> {
    STORE.save()
}

// ══════════════════════════════════════════════════════════════════════════════
// Cache 命令 — 纯内存，TTL 过期
// ══════════════════════════════════════════════════════════════════════════════

/// 读取缓存值，自动检查过期。
#[tauri::command]
pub fn cache_get(key: String) -> Result<Value, String> {
    CACHE
        .get_raw(&key)
        .ok_or_else(|| format!("缓存项 '{}' 不存在或已过期", key))
}

/// 写入缓存值，指定 TTL 策略。
///
/// `ttl` 参数格式：
/// - `"forever"` — 永不过期
/// - `"session"` — 当前进程生命周期
/// - `{"duration_secs": 600}` — 600 秒后过期
///
/// 前端调用示例：
/// ```js
/// await invoke('cache_set', { key: 'recent', value: data, ttl: { duration_secs: 600 } });
/// await invoke('cache_set', { key: 'state', value: data, ttl: 'forever' });
/// ```
#[tauri::command]
pub fn cache_set(key: String, value: Value, ttl: TtlArg) -> Result<(), String> {
    CACHE.set_raw(&key, value, &ttl.into());
    Ok(())
}

/// 删除缓存项。
#[tauri::command]
pub fn cache_remove(key: String) -> Result<bool, String> {
    Ok(CACHE.remove(&key))
}

/// 检查缓存项是否存在且未过期。
#[tauri::command]
pub fn cache_has(key: String) -> Result<bool, String> {
    Ok(CACHE.has(&key))
}

/// 获取所有未过期的缓存键。
#[tauri::command]
pub fn cache_keys() -> Result<Vec<String>, String> {
    Ok(CACHE.keys())
}

/// 清空所有缓存（含未过期条目）。
#[tauri::command]
pub fn cache_clear() -> Result<(), String> {
    CACHE.clear();
    Ok(())
}

/// 清理所有已过期条目，返回清理数量。
#[tauri::command]
pub fn cache_clear_expired() -> Result<usize, String> {
    Ok(CACHE.clear_expired())
}

/// 续期缓存项：按给定 TTL 重置过期倒计时。
///
/// `ttl` 参数格式同 [`cache_set`]。
/// 返回 `true` 表示键存在且续期成功。
#[tauri::command]
pub fn cache_touch(key: String, ttl: TtlArg) -> Result<bool, String> {
    Ok(CACHE.touch(&key, &ttl.into()))
}

// ══════════════════════════════════════════════════════════════════════════════
// Blob Cache 命令 — 磁盘文件存储 + 内存 TTL 过期
// ══════════════════════════════════════════════════════════════════════════════

/// 启用 Blob 缓存磁盘存储。
///
/// 必须在使用 [`cache_set_blob`] / [`cache_get_blob`] 前调用。
/// `dir` 为缓存目录的绝对路径，目录不存在时自动创建。
#[tauri::command]
pub fn cache_enable_blob_storage(dir: String) -> Result<(), String> {
    CACHE.enable_blob_storage(PathBuf::from(dir))
}

/// 检查 Blob 缓存是否已启用。
#[tauri::command]
pub fn cache_blob_storage_enabled() -> Result<bool, String> {
    Ok(CACHE.blob_storage_enabled())
}

/// 写入二进制数据到 Blob 缓存。
///
/// 数据作为文件保存到磁盘，TTL 元数据保存在内存中。
/// `data` 应为 base64 编码的字符串，由前端编码。
/// `ttl` 参数格式同 [`cache_set`]。
#[tauri::command]
pub fn cache_set_blob(key: String, data: Vec<u8>, ttl: TtlArg) -> Result<(), String> {
    CACHE.set_blob(&key, &data, &ttl.into())
}

/// 读取 Blob 缓存的二进制数据，自动检查过期。
///
/// 返回 base64 编码的字节数据，或 key 不存在/已过期时返回 `Err`。
#[tauri::command]
pub fn cache_get_blob(key: String) -> Result<Vec<u8>, String> {
    CACHE
        .get_blob(&key)
        .ok_or_else(|| format!("Blob 缓存项 '{}' 不存在或已过期", key))
}

/// 删除 Blob 缓存项（含磁盘文件）。
#[tauri::command]
pub fn cache_remove_blob(key: String) -> Result<bool, String> {
    Ok(CACHE.remove_blob(&key))
}

/// 检查 Blob 缓存项是否存在且未过期。
#[tauri::command]
pub fn cache_has_blob(key: String) -> Result<bool, String> {
    Ok(CACHE.has_blob(&key))
}

/// 获取所有未过期的 Blob 缓存 key。
#[tauri::command]
pub fn cache_blob_keys() -> Result<Vec<String>, String> {
    Ok(CACHE.blob_keys())
}

/// 清空所有 Blob 缓存（含磁盘文件）。
#[tauri::command]
pub fn cache_clear_blobs() -> Result<(), String> {
    CACHE.clear_blobs();
    Ok(())
}

/// 清理所有已过期的 Blob 缓存条目，返回清理数量。
#[tauri::command]
pub fn cache_clear_expired_blobs() -> Result<usize, String> {
    Ok(CACHE.clear_expired_blobs())
}

// ══════════════════════════════════════════════════════════════════════════════
// Blob Storage 命令 — 持久化二进制文件存储
// ══════════════════════════════════════════════════════════════════════════════

/// 写入二进制数据到持久化 Blob 存储（立即落盘）。
///
/// 文件保存在 `storage.json` 同级的 `blobs/` 目录下。
#[tauri::command]
pub fn storage_set_blob(key: String, data: Vec<u8>) -> Result<(), String> {
    STORE.set_blob(&key, &data)
}

/// 读取持久化 Blob 存储的二进制数据。
#[tauri::command]
pub fn storage_get_blob(key: String) -> Result<Vec<u8>, String> {
    STORE
        .get_blob(&key)
        .ok_or_else(|| format!("Blob 存储项 '{}' 不存在", key))
}

/// 删除持久化 Blob 存储项。
#[tauri::command]
pub fn storage_remove_blob(key: String) -> Result<bool, String> {
    Ok(STORE.remove_blob(&key))
}

/// 检查持久化 Blob 存储项是否存在。
#[tauri::command]
pub fn storage_has_blob(key: String) -> Result<bool, String> {
    Ok(STORE.has_blob(&key))
}

/// 获取所有持久化 Blob 存储的 key 列表。
#[tauri::command]
pub fn storage_blob_keys() -> Result<Vec<String>, String> {
    Ok(STORE.blob_keys())
}

/// 清空所有持久化 Blob 存储项。
#[tauri::command]
pub fn storage_clear_blobs() -> Result<(), String> {
    STORE.clear_blobs();
    Ok(())
}

// ══════════════════════════════════════════════════════════════════════════════
// Music Source 资源获取命令 — 大文件通过 raw payload 返回前端
// ══════════════════════════════════════════════════════════════════════════════

/// 获取歌曲的音频文件数据。
///
/// `source_id_json` 为 [`SourceId`] 的 JSON 序列化字符串。
/// 返回音频文件的原始字节数据。
///
/// # 链路
/// front → tauri → resource::get_song_file(registrar, source_id)
///   → trait MusicSource::song_file_get → Vec<u8> → 前端
#[tauri::command]
pub fn get_song_file(source_id_json: String) -> Result<Vec<u8>, String> {
    let source_id: SourceId = serde_json::from_str(&source_id_json)
        .map_err(|e| format!("解析 SourceId 失败: {}", e))?;
    resource::get_song_file(source_registrar(), &source_id)
}

/// 获取专辑的封面图片数据。
///
/// `source_id_json` 为 [`SourceId`] 的 JSON 序列化字符串。
/// 返回图片文件的原始字节数据（JPEG / PNG）。
///
/// # 链路
/// front → tauri → resource::get_album_picture(registrar, source_id)
///   → trait MusicSource::album_picture_get → Vec<u8> → 前端
#[tauri::command]
pub fn get_album_picture(source_id_json: String) -> Result<Vec<u8>, String> {
    let source_id: SourceId = serde_json::from_str(&source_id_json)
        .map_err(|e| format!("解析 SourceId 失败: {}", e))?;
    resource::get_album_picture(source_registrar(), &source_id)
}

/// 获取歌曲的歌词文本。
///
/// `source_id_json` 为 [`SourceId`] 的 JSON 序列化字符串。
/// 返回歌词原始文本（LRC 或纯文本格式）。
///
/// # 链路
/// front → tauri → resource::get_lyric_text(registrar, source_id)
///   → trait MusicSource::lyric_text_get → String → 前端
#[tauri::command]
pub fn get_lyric_text(source_id_json: String) -> Result<String, String> {
    let source_id: SourceId = serde_json::from_str(&source_id_json)
        .map_err(|e| format!("解析 SourceId 失败: {}", e))?;
    resource::get_lyric_text(source_registrar(), &source_id)
}

// ══════════════════════════════════════════════════════════════════════════════
// Local Source 文件夹管理命令
// ══════════════════════════════════════════════════════════════════════════════

/// 获取本地来源的索引统计信息。
#[tauri::command]
pub fn local_stats() -> Result<serde_json::Value, String> {
    let source = LOCAL_SOURCE
        .get()
        .ok_or("本地来源未初始化")?;

    let folder_count = source.folder_manager.count();
    let indexed_count = source.file_index.read().len();

    Ok(serde_json::json!({
        "folder_count": folder_count,
        "indexed_files": indexed_count,
    }))
}

/// 添加本地音乐文件夹。
///
/// 操作：
/// 1. 持久化文件夹路径
/// 2. 扫描文件夹中所有音频文件并导入音乐库
/// 3. 启动对该文件夹的文件系统监听
#[tauri::command]
pub fn local_add_folder(path: String) -> Result<serde_json::Value, String> {
    let source = LOCAL_SOURCE
        .get()
        .ok_or("本地来源未初始化")?;

    let folder_path = PathBuf::from(&path);
    source.folder_manager.add_folder(&folder_path)?;

    // 扫描并索引文件夹中的音频文件
    let files = music_localSource::folder::collect_audio_files(&folder_path);
    let mut indexed = 0u64;
    let mut errors = Vec::new();

    for file in &files {
        match source.index_file(file) {
            Ok(true) => indexed += 1,
            Ok(false) => {} // 跳过非音频文件
            Err(e) => errors.push(format!("{}: {}", file.display(), e)),
        }
    }

    // 持久化音乐库
    source.library.save()?;

    Ok(serde_json::json!({
        "added": true,
        "path": path,
        "files_found": files.len(),
        "indexed": indexed,
        "errors": errors,
    }))
}

/// 移除本地音乐文件夹。
///
/// 操作：
/// 1. 遍历文件夹中所有已知音频文件，从音乐库中移除对应的 SourceId
/// 2. 级联清理空实体
/// 3. 从文件夹管理器中移除
/// 4. 持久化
#[tauri::command]
pub fn local_remove_folder(path: String) -> Result<serde_json::Value, String> {
    let source = LOCAL_SOURCE
        .get()
        .ok_or("本地来源未初始化")?;

    let folder_path = PathBuf::from(&path);

    // 1. 清理该文件夹下所有文件的 SourceId
    use std::collections::HashSet;
    let files = music_localSource::folder::collect_audio_files(&folder_path);
    let entity_ids: HashSet<String> = files
        .iter()
        .filter_map(|f| {
            f.canonicalize()
                .ok()
                .map(|c| c.to_string_lossy().to_string())
        })
        .collect();

    if !entity_ids.is_empty() {
        source.library.remove_specific_song_source_ids(
            music_localSource::source::LOCAL_SOURCE_NAME,
            &entity_ids,
        )?;
    }

    // 清理本地索引
    for file in &files {
        let _ = source.unindex_file(file);
    }

    // 2. 从文件夹管理器移除
    let removed = source.folder_manager.remove_folder(&folder_path);

    // 3. 持久化
    source.library.save()?;

    Ok(serde_json::json!({
        "removed": removed,
        "path": path,
        "cleaned_files": entity_ids.len(),
    }))
}

/// 获取本地来源的监听文件夹列表。
#[tauri::command]
pub fn local_get_folders() -> Result<Vec<String>, String> {
    let source = LOCAL_SOURCE
        .get()
        .ok_or("本地来源未初始化")?;

    Ok(source
        .folder_manager
        .get_folders()
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect())
}

/// 手动触发全量重新扫描所有监听文件夹（调试用）。
#[tauri::command]
pub fn local_rescan() -> Result<serde_json::Value, String> {
    let source = LOCAL_SOURCE
        .get()
        .ok_or("本地来源未初始化")?;

    let folders = source.folder_manager.get_folders();
    let mut total = 0u64;

    for folder in &folders {
        let files = music_localSource::folder::collect_audio_files(folder);
        for file in &files {
            match source.index_file(file) {
                Ok(true) => total += 1,
                Ok(false) => {}
                Err(e) => {
                    eprintln!("[local_rescan] {}: {}", file.display(), e);
                }
            }
        }
    }

    source.library.save()?;

    Ok(serde_json::json!({
        "indexed": total,
        "folders_scanned": folders.len(),
    }))
}

// ══════════════════════════════════════════════════════════════════════════════
// MusicLibrary 命令 — 音乐库 CRUD / 搜索 / 关系查询
// ══════════════════════════════════════════════════════════════════════════════

// ── 持久化 ──────────────────────────────────────────

/// 立即将音乐库所有未落盘数据写入磁盘。
#[tauri::command]
pub fn library_save() -> Result<(), String> {
    music_library().save()
}

/// 清理所有 `source_ids` 为空的实体。
#[tauri::command]
pub fn library_cleanup_empty_entities() -> Result<(), String> {
    music_library().cleanup_empty_entities()?;
    music_library().save()
}

// ── Song ────────────────────────────────────────────

#[tauri::command]
pub fn library_song_count() -> Result<usize, String> {
    Ok(music_library().song_count())
}

#[tauri::command]
pub fn library_get_song(id: String) -> Result<serde_json::Value, String> {
    let song = music_library()
        .get_song(&id)
        .ok_or_else(|| format!("歌曲 '{}' 不存在", id))?;
    serde_json::to_value(&song).map_err(|e| format!("序列化失败: {}", e))
}

#[tauri::command]
pub fn library_get_all_songs() -> Result<serde_json::Value, String> {
    let songs = music_library().get_all_songs();
    serde_json::to_value(&songs).map_err(|e| format!("序列化失败: {}", e))
}

#[tauri::command]
pub fn library_search_songs(query: String) -> Result<serde_json::Value, String> {
    let songs = music_library().search_songs(&query);
    serde_json::to_value(&songs).map_err(|e| format!("序列化失败: {}", e))
}

// ── Artist ──────────────────────────────────────────

#[tauri::command]
pub fn library_artist_count() -> Result<usize, String> {
    Ok(music_library().artist_count())
}

#[tauri::command]
pub fn library_get_artist(id: String) -> Result<serde_json::Value, String> {
    let artist = music_library()
        .get_artist(&id)
        .ok_or_else(|| format!("艺术家 '{}' 不存在", id))?;
    serde_json::to_value(&artist).map_err(|e| format!("序列化失败: {}", e))
}

#[tauri::command]
pub fn library_get_all_artists() -> Result<serde_json::Value, String> {
    let artists = music_library().get_all_artists();
    serde_json::to_value(&artists).map_err(|e| format!("序列化失败: {}", e))
}

#[tauri::command]
pub fn library_search_artists(query: String) -> Result<serde_json::Value, String> {
    let artists = music_library().search_artists(&query);
    serde_json::to_value(&artists).map_err(|e| format!("序列化失败: {}", e))
}

// ── Album ───────────────────────────────────────────

#[tauri::command]
pub fn library_album_count() -> Result<usize, String> {
    Ok(music_library().album_count())
}

#[tauri::command]
pub fn library_get_album(id: String) -> Result<serde_json::Value, String> {
    let album = music_library()
        .get_album(&id)
        .ok_or_else(|| format!("专辑 '{}' 不存在", id))?;
    serde_json::to_value(&album).map_err(|e| format!("序列化失败: {}", e))
}

#[tauri::command]
pub fn library_get_all_albums() -> Result<serde_json::Value, String> {
    let albums = music_library().get_all_albums();
    serde_json::to_value(&albums).map_err(|e| format!("序列化失败: {}", e))
}

#[tauri::command]
pub fn library_search_albums(query: String) -> Result<serde_json::Value, String> {
    let albums = music_library().search_albums(&query);
    serde_json::to_value(&albums).map_err(|e| format!("序列化失败: {}", e))
}

// ── Lyric ───────────────────────────────────────────

#[tauri::command]
pub fn library_lyric_count() -> Result<usize, String> {
    Ok(music_library().lyric_count())
}

#[tauri::command]
pub fn library_get_lyric(id: String) -> Result<serde_json::Value, String> {
    let lyric = music_library()
        .get_lyric(&id)
        .ok_or_else(|| format!("歌词 '{}' 不存在", id))?;
    serde_json::to_value(&lyric).map_err(|e| format!("序列化失败: {}", e))
}

#[tauri::command]
pub fn library_get_all_lyrics() -> Result<serde_json::Value, String> {
    let lyrics = music_library().get_all_lyrics();
    serde_json::to_value(&lyrics).map_err(|e| format!("序列化失败: {}", e))
}

#[tauri::command]
pub fn library_search_lyrics(query: String) -> Result<serde_json::Value, String> {
    let lyrics = music_library().search_lyrics(&query);
    serde_json::to_value(&lyrics).map_err(|e| format!("序列化失败: {}", e))
}

// ── Relations ───────────────────────────────────────

#[tauri::command]
pub fn library_get_artists_of_song(song_id: String) -> Result<serde_json::Value, String> {
    let artists = music_library().get_artists_of_song(&song_id);
    serde_json::to_value(&artists).map_err(|e| format!("序列化失败: {}", e))
}

#[tauri::command]
pub fn library_get_album_of_song(song_id: String) -> Result<serde_json::Value, String> {
    let album = music_library().get_album_of_song(&song_id);
    serde_json::to_value(&album).map_err(|e| format!("序列化失败: {}", e))
}

#[tauri::command]
pub fn library_get_lyric_of_song(song_id: String) -> Result<serde_json::Value, String> {
    let lyric = music_library().get_lyric_of_song(&song_id);
    serde_json::to_value(&lyric).map_err(|e| format!("序列化失败: {}", e))
}

#[tauri::command]
pub fn library_get_songs_by_artist(artist_id: String) -> Result<serde_json::Value, String> {
    let songs = music_library().get_songs_by_artist(&artist_id);
    serde_json::to_value(&songs).map_err(|e| format!("序列化失败: {}", e))
}

#[tauri::command]
pub fn library_get_albums_by_artist(artist_id: String) -> Result<serde_json::Value, String> {
    let albums = music_library().get_albums_by_artist(&artist_id);
    serde_json::to_value(&albums).map_err(|e| format!("序列化失败: {}", e))
}

#[tauri::command]
pub fn library_get_songs_in_album(album_id: String) -> Result<serde_json::Value, String> {
    let songs = music_library().get_songs_in_album(&album_id);
    serde_json::to_value(&songs).map_err(|e| format!("序列化失败: {}", e))
}

#[tauri::command]
pub fn library_get_source_ids_of_song(song_id: String) -> Result<serde_json::Value, String> {
    let ids = music_library().get_source_ids_of_song(&song_id);
    serde_json::to_value(&ids).map_err(|e| format!("序列化失败: {}", e))
}
