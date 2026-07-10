//! Tauri 命令层 — chordial-core 的薄封装（库调用形式）。
//!
//! 所有命令通过 Tauri 的 `State<'_, Arc<AppContext>>` 提取由 `setup` 注入的
//! server 层上下文，再委托给 core 方法。命令本身不含业务逻辑。
//!
//! 这正是「库调用形式」的 front 层：前端 `invoke` → 本层 → core 同步函数调用，
//! 全程进程内，无网络开销。

use chordial_core::module::music_localSource;
use chordial_core::module::music_source::resource;
use chordial_core::module::music_source::types::SourceId;
use chordial_core::module::platform::{self, PlatformPath};
use chordial_core::module::storage::entry::Ttl;
use chordial_core::AppContext;
use serde::Deserialize;
use serde_json::Value;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;

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

#[tauri::command]
pub fn config_get(ctx: State<'_, Arc<AppContext>>, key: String) -> Result<Value, String> {
    ctx.config
        .get_raw(&key)
        .ok_or_else(|| format!("配置项 '{}' 不存在", key))
}

#[tauri::command]
pub fn config_set(ctx: State<'_, Arc<AppContext>>, key: String, value: Value) -> Result<(), String> {
    ctx.config.set_raw(&key, value);
    Ok(())
}

#[tauri::command]
pub fn config_remove(ctx: State<'_, Arc<AppContext>>, key: String) -> Result<bool, String> {
    Ok(ctx.config.remove(&key))
}

#[tauri::command]
pub fn config_has(ctx: State<'_, Arc<AppContext>>, key: String) -> Result<bool, String> {
    Ok(ctx.config.has(&key))
}

#[tauri::command]
pub fn config_keys(ctx: State<'_, Arc<AppContext>>) -> Result<Vec<String>, String> {
    Ok(ctx.config.keys())
}

#[tauri::command]
pub fn config_clear(ctx: State<'_, Arc<AppContext>>) -> Result<(), String> {
    ctx.config.clear();
    Ok(())
}

#[tauri::command]
pub fn config_flush(ctx: State<'_, Arc<AppContext>>) -> Result<(), String> {
    ctx.config.flush()
}

#[tauri::command]
pub fn config_reload(ctx: State<'_, Arc<AppContext>>) -> Result<(), String> {
    ctx.config.reload();
    Ok(())
}

// ══════════════════════════════════════════════════════════════════════════════
// Storage 命令 — 手动落盘
// ══════════════════════════════════════════════════════════════════════════════

#[tauri::command]
pub fn storage_get(ctx: State<'_, Arc<AppContext>>, key: String) -> Result<Value, String> {
    ctx.store
        .get_raw(&key)
        .ok_or_else(|| format!("键 '{}' 不存在", key))
}

#[tauri::command]
pub fn storage_set(ctx: State<'_, Arc<AppContext>>, key: String, value: Value) -> Result<(), String> {
    ctx.store.set_raw(&key, value);
    Ok(())
}

#[tauri::command]
pub fn storage_remove(ctx: State<'_, Arc<AppContext>>, key: String) -> Result<bool, String> {
    Ok(ctx.store.remove(&key))
}

#[tauri::command]
pub fn storage_has(ctx: State<'_, Arc<AppContext>>, key: String) -> Result<bool, String> {
    Ok(ctx.store.has(&key))
}

#[tauri::command]
pub fn storage_keys(ctx: State<'_, Arc<AppContext>>) -> Result<Vec<String>, String> {
    Ok(ctx.store.keys())
}

#[tauri::command]
pub fn storage_clear(ctx: State<'_, Arc<AppContext>>) -> Result<(), String> {
    ctx.store.clear();
    Ok(())
}

#[tauri::command]
pub fn storage_save(ctx: State<'_, Arc<AppContext>>) -> Result<(), String> {
    ctx.store.save()
}

// ══════════════════════════════════════════════════════════════════════════════
// Cache 命令 — 纯内存，TTL 过期
// ══════════════════════════════════════════════════════════════════════════════

#[tauri::command]
pub fn cache_get(ctx: State<'_, Arc<AppContext>>, key: String) -> Result<Value, String> {
    ctx.cache
        .get_raw(&key)
        .ok_or_else(|| format!("缓存项 '{}' 不存在或已过期", key))
}

#[tauri::command]
pub fn cache_set(
    ctx: State<'_, Arc<AppContext>>,
    key: String,
    value: Value,
    ttl: TtlArg,
) -> Result<(), String> {
    ctx.cache.set_raw(&key, value, &ttl.into());
    Ok(())
}

#[tauri::command]
pub fn cache_remove(ctx: State<'_, Arc<AppContext>>, key: String) -> Result<bool, String> {
    Ok(ctx.cache.remove(&key))
}

#[tauri::command]
pub fn cache_has(ctx: State<'_, Arc<AppContext>>, key: String) -> Result<bool, String> {
    Ok(ctx.cache.has(&key))
}

#[tauri::command]
pub fn cache_keys(ctx: State<'_, Arc<AppContext>>) -> Result<Vec<String>, String> {
    Ok(ctx.cache.keys())
}

#[tauri::command]
pub fn cache_clear(ctx: State<'_, Arc<AppContext>>) -> Result<(), String> {
    ctx.cache.clear();
    Ok(())
}

#[tauri::command]
pub fn cache_clear_expired(ctx: State<'_, Arc<AppContext>>) -> Result<usize, String> {
    Ok(ctx.cache.clear_expired())
}

#[tauri::command]
pub fn cache_touch(ctx: State<'_, Arc<AppContext>>, key: String, ttl: TtlArg) -> Result<bool, String> {
    Ok(ctx.cache.touch(&key, &ttl.into()))
}

// ══════════════════════════════════════════════════════════════════════════════
// Blob Cache 命令 — 磁盘文件存储 + 内存 TTL 过期
// ══════════════════════════════════════════════════════════════════════════════

#[tauri::command]
pub fn cache_enable_blob_storage(ctx: State<'_, Arc<AppContext>>, dir: String) -> Result<(), String> {
    ctx.cache.enable_blob_storage(PathBuf::from(dir))
}

#[tauri::command]
pub fn cache_blob_storage_enabled(ctx: State<'_, Arc<AppContext>>) -> Result<bool, String> {
    Ok(ctx.cache.blob_storage_enabled())
}

#[tauri::command]
pub fn cache_set_blob(
    ctx: State<'_, Arc<AppContext>>,
    key: String,
    data: Vec<u8>,
    ttl: TtlArg,
) -> Result<(), String> {
    ctx.cache.set_blob(&key, &data, &ttl.into())
}

#[tauri::command]
pub fn cache_get_blob(ctx: State<'_, Arc<AppContext>>, key: String) -> Result<Vec<u8>, String> {
    ctx.cache
        .get_blob(&key)
        .ok_or_else(|| format!("Blob 缓存项 '{}' 不存在或已过期", key))
}

#[tauri::command]
pub fn cache_remove_blob(ctx: State<'_, Arc<AppContext>>, key: String) -> Result<bool, String> {
    Ok(ctx.cache.remove_blob(&key))
}

#[tauri::command]
pub fn cache_has_blob(ctx: State<'_, Arc<AppContext>>, key: String) -> Result<bool, String> {
    Ok(ctx.cache.has_blob(&key))
}

#[tauri::command]
pub fn cache_blob_keys(ctx: State<'_, Arc<AppContext>>) -> Result<Vec<String>, String> {
    Ok(ctx.cache.blob_keys())
}

#[tauri::command]
pub fn cache_clear_blobs(ctx: State<'_, Arc<AppContext>>) -> Result<(), String> {
    ctx.cache.clear_blobs();
    Ok(())
}

#[tauri::command]
pub fn cache_clear_expired_blobs(ctx: State<'_, Arc<AppContext>>) -> Result<usize, String> {
    Ok(ctx.cache.clear_expired_blobs())
}

// ══════════════════════════════════════════════════════════════════════════════
// Blob Storage 命令 — 持久化二进制文件存储
// ══════════════════════════════════════════════════════════════════════════════

#[tauri::command]
pub fn storage_set_blob(ctx: State<'_, Arc<AppContext>>, key: String, data: Vec<u8>) -> Result<(), String> {
    ctx.store.set_blob(&key, &data)
}

#[tauri::command]
pub fn storage_get_blob(ctx: State<'_, Arc<AppContext>>, key: String) -> Result<Vec<u8>, String> {
    ctx.store
        .get_blob(&key)
        .ok_or_else(|| format!("Blob 存储项 '{}' 不存在", key))
}

#[tauri::command]
pub fn storage_remove_blob(ctx: State<'_, Arc<AppContext>>, key: String) -> Result<bool, String> {
    Ok(ctx.store.remove_blob(&key))
}

#[tauri::command]
pub fn storage_has_blob(ctx: State<'_, Arc<AppContext>>, key: String) -> Result<bool, String> {
    Ok(ctx.store.has_blob(&key))
}

#[tauri::command]
pub fn storage_blob_keys(ctx: State<'_, Arc<AppContext>>) -> Result<Vec<String>, String> {
    Ok(ctx.store.blob_keys())
}

#[tauri::command]
pub fn storage_clear_blobs(ctx: State<'_, Arc<AppContext>>) -> Result<(), String> {
    ctx.store.clear_blobs();
    Ok(())
}

// ══════════════════════════════════════════════════════════════════════════════
// Music Source 资源获取命令 — 大文件通过 raw payload 返回前端
// ══════════════════════════════════════════════════════════════════════════════

#[tauri::command]
pub fn get_song_file(
    ctx: State<'_, Arc<AppContext>>,
    source_id_json: String,
) -> Result<Vec<u8>, String> {
    let source_id: SourceId = serde_json::from_str(&source_id_json)
        .map_err(|e| format!("解析 SourceId 失败: {}", e))?;
    resource::get_song_file(&ctx.registrar, &source_id)
}

#[tauri::command]
pub fn get_album_picture(
    ctx: State<'_, Arc<AppContext>>,
    source_id_json: String,
) -> Result<Vec<u8>, String> {
    let source_id: SourceId = serde_json::from_str(&source_id_json)
        .map_err(|e| format!("解析 SourceId 失败: {}", e))?;
    resource::get_album_picture(&ctx.registrar, &source_id)
}

#[tauri::command]
pub fn get_lyric_text(
    ctx: State<'_, Arc<AppContext>>,
    source_id_json: String,
) -> Result<String, String> {
    let source_id: SourceId = serde_json::from_str(&source_id_json)
        .map_err(|e| format!("解析 SourceId 失败: {}", e))?;
    resource::get_lyric_text(&ctx.registrar, &source_id)
}

// ══════════════════════════════════════════════════════════════════════════════
// Local Source 文件夹管理命令
// ══════════════════════════════════════════════════════════════════════════════

#[tauri::command]
pub fn local_stats(ctx: State<'_, Arc<AppContext>>) -> Result<serde_json::Value, String> {
    let source = &ctx.local_source;
    let folder_count = source.folder_manager.count();
    let indexed_count = source.file_index.read().len();

    Ok(serde_json::json!({
        "folder_count": folder_count,
        "indexed_files": indexed_count,
    }))
}

#[tauri::command]
pub fn local_add_folder(
    ctx: State<'_, Arc<AppContext>>,
    path: String,
) -> Result<serde_json::Value, String> {
    let source = &ctx.local_source;
    let folder_path = PlatformPath::from(path.as_str());
    source.folder_manager.add_folder(&folder_path)?;

    // 扫描并索引文件夹中的音频文件
    let files = music_localSource::folder::collect_audio_files(&folder_path);
    let mut indexed = 0u64;
    let mut errors = Vec::new();

    for file in &files {
        match source.index_file(file) {
            Ok(true) => indexed += 1,
            Ok(false) => {} // 跳过非音频文件
            Err(e) => errors.push(format!("{}: {}", platform::path_to_string(file), e)),
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

#[tauri::command]
pub fn local_remove_folder(
    ctx: State<'_, Arc<AppContext>>,
    path: String,
) -> Result<serde_json::Value, String> {
    let source = &ctx.local_source;
    let folder_path = PlatformPath::from(path.as_str());

    // 1. 清理该文件夹下所有文件的 SourceId
    use std::collections::HashSet;
    let files = music_localSource::folder::collect_audio_files(&folder_path);
    let entity_ids: HashSet<String> = files
        .iter()
        .map(|f| {
            platform::path_to_string(&platform::canonicalize(f).unwrap_or_else(|_| f.clone()))
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

#[tauri::command]
pub fn local_get_folders(ctx: State<'_, Arc<AppContext>>) -> Result<Vec<String>, String> {
    let source = &ctx.local_source;
    Ok(source
        .folder_manager
        .get_folders()
        .iter()
        .map(|p| platform::path_to_string(p))
        .collect())
}

#[tauri::command]
pub fn local_rescan(ctx: State<'_, Arc<AppContext>>) -> Result<serde_json::Value, String> {
    let source = &ctx.local_source;
    let folders = source.folder_manager.get_folders();
    let mut total = 0u64;

    for folder in &folders {
        let files = music_localSource::folder::collect_audio_files(folder);
        for file in &files {
            match source.index_file(file) {
                Ok(true) => total += 1,
                Ok(false) => {}
                Err(e) => {
                    eprintln!("[local_rescan] {}: {}", platform::path_to_string(file), e);
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

#[tauri::command]
pub fn library_save(ctx: State<'_, Arc<AppContext>>) -> Result<(), String> {
    ctx.library.save()
}

#[tauri::command]
pub fn library_cleanup_empty_entities(ctx: State<'_, Arc<AppContext>>) -> Result<(), String> {
    ctx.library.cleanup_empty_entities()?;
    ctx.library.save()
}

// ── Song ────────────────────────────────────────────

#[tauri::command]
pub fn library_song_count(ctx: State<'_, Arc<AppContext>>) -> Result<usize, String> {
    Ok(ctx.library.song_count())
}

#[tauri::command]
pub fn library_get_song(ctx: State<'_, Arc<AppContext>>, id: String) -> Result<serde_json::Value, String> {
    let song = ctx
        .library
        .get_song(&id)
        .ok_or_else(|| format!("歌曲 '{}' 不存在", id))?;
    serde_json::to_value(&song).map_err(|e| format!("序列化失败: {}", e))
}

#[tauri::command]
pub fn library_get_all_songs(ctx: State<'_, Arc<AppContext>>) -> Result<serde_json::Value, String> {
    let songs = ctx.library.get_all_songs();
    serde_json::to_value(&songs).map_err(|e| format!("序列化失败: {}", e))
}

#[tauri::command]
pub fn library_get_songs_page(
    ctx: State<'_, Arc<AppContext>>,
    offset: usize,
    limit: usize,
) -> Result<serde_json::Value, String> {
    let songs = ctx.library.get_songs_page(offset, limit);
    serde_json::to_value(&songs).map_err(|e| format!("序列化失败: {}", e))
}

#[tauri::command]
pub fn library_search_songs(ctx: State<'_, Arc<AppContext>>, query: String) -> Result<serde_json::Value, String> {
    let songs = ctx.library.search_songs(&query);
    serde_json::to_value(&songs).map_err(|e| format!("序列化失败: {}", e))
}

// ── Artist ──────────────────────────────────────────

#[tauri::command]
pub fn library_artist_count(ctx: State<'_, Arc<AppContext>>) -> Result<usize, String> {
    Ok(ctx.library.artist_count())
}

#[tauri::command]
pub fn library_get_artist(ctx: State<'_, Arc<AppContext>>, id: String) -> Result<serde_json::Value, String> {
    let artist = ctx
        .library
        .get_artist(&id)
        .ok_or_else(|| format!("艺术家 '{}' 不存在", id))?;
    serde_json::to_value(&artist).map_err(|e| format!("序列化失败: {}", e))
}

#[tauri::command]
pub fn library_get_all_artists(ctx: State<'_, Arc<AppContext>>) -> Result<serde_json::Value, String> {
    let artists = ctx.library.get_all_artists();
    serde_json::to_value(&artists).map_err(|e| format!("序列化失败: {}", e))
}

#[tauri::command]
pub fn library_get_artists_page(
    ctx: State<'_, Arc<AppContext>>,
    offset: usize,
    limit: usize,
) -> Result<serde_json::Value, String> {
    let artists = ctx.library.get_artists_page(offset, limit);
    serde_json::to_value(&artists).map_err(|e| format!("序列化失败: {}", e))
}

#[tauri::command]
pub fn library_search_artists(ctx: State<'_, Arc<AppContext>>, query: String) -> Result<serde_json::Value, String> {
    let artists = ctx.library.search_artists(&query);
    serde_json::to_value(&artists).map_err(|e| format!("序列化失败: {}", e))
}

// ── Album ───────────────────────────────────────────

#[tauri::command]
pub fn library_album_count(ctx: State<'_, Arc<AppContext>>) -> Result<usize, String> {
    Ok(ctx.library.album_count())
}

#[tauri::command]
pub fn library_get_album(ctx: State<'_, Arc<AppContext>>, id: String) -> Result<serde_json::Value, String> {
    let album = ctx
        .library
        .get_album(&id)
        .ok_or_else(|| format!("专辑 '{}' 不存在", id))?;
    serde_json::to_value(&album).map_err(|e| format!("序列化失败: {}", e))
}

#[tauri::command]
pub fn library_get_all_albums(ctx: State<'_, Arc<AppContext>>) -> Result<serde_json::Value, String> {
    let albums = ctx.library.get_all_albums();
    serde_json::to_value(&albums).map_err(|e| format!("序列化失败: {}", e))
}

#[tauri::command]
pub fn library_get_albums_page(
    ctx: State<'_, Arc<AppContext>>,
    offset: usize,
    limit: usize,
) -> Result<serde_json::Value, String> {
    let albums = ctx.library.get_albums_page(offset, limit);
    serde_json::to_value(&albums).map_err(|e| format!("序列化失败: {}", e))
}

#[tauri::command]
pub fn library_get_home_stats(ctx: State<'_, Arc<AppContext>>) -> Result<serde_json::Value, String> {
    Ok(ctx.library.get_home_stats())
}

#[tauri::command]
pub fn library_search_albums(ctx: State<'_, Arc<AppContext>>, query: String) -> Result<serde_json::Value, String> {
    let albums = ctx.library.search_albums(&query);
    serde_json::to_value(&albums).map_err(|e| format!("序列化失败: {}", e))
}

// ── Lyric ───────────────────────────────────────────

#[tauri::command]
pub fn library_lyric_count(ctx: State<'_, Arc<AppContext>>) -> Result<usize, String> {
    Ok(ctx.library.lyric_count())
}

#[tauri::command]
pub fn library_get_lyric(ctx: State<'_, Arc<AppContext>>, id: String) -> Result<serde_json::Value, String> {
    let lyric = ctx
        .library
        .get_lyric(&id)
        .ok_or_else(|| format!("歌词 '{}' 不存在", id))?;
    serde_json::to_value(&lyric).map_err(|e| format!("序列化失败: {}", e))
}

#[tauri::command]
pub fn library_get_all_lyrics(ctx: State<'_, Arc<AppContext>>) -> Result<serde_json::Value, String> {
    let lyrics = ctx.library.get_all_lyrics();
    serde_json::to_value(&lyrics).map_err(|e| format!("序列化失败: {}", e))
}

#[tauri::command]
pub fn library_search_lyrics(ctx: State<'_, Arc<AppContext>>, query: String) -> Result<serde_json::Value, String> {
    let lyrics = ctx.library.search_lyrics(&query);
    serde_json::to_value(&lyrics).map_err(|e| format!("序列化失败: {}", e))
}

// ── Relations ───────────────────────────────────────

#[tauri::command]
pub fn library_get_artists_of_song(
    ctx: State<'_, Arc<AppContext>>,
    song_id: String,
) -> Result<serde_json::Value, String> {
    let artists = ctx.library.get_artists_of_song(&song_id);
    serde_json::to_value(&artists).map_err(|e| format!("序列化失败: {}", e))
}

#[tauri::command]
pub fn library_get_album_of_song(
    ctx: State<'_, Arc<AppContext>>,
    song_id: String,
) -> Result<serde_json::Value, String> {
    let album = ctx.library.get_album_of_song(&song_id);
    serde_json::to_value(&album).map_err(|e| format!("序列化失败: {}", e))
}

#[tauri::command]
pub fn library_get_lyric_of_song(
    ctx: State<'_, Arc<AppContext>>,
    song_id: String,
) -> Result<serde_json::Value, String> {
    let lyric = ctx.library.get_lyric_of_song(&song_id);
    serde_json::to_value(&lyric).map_err(|e| format!("序列化失败: {}", e))
}

#[tauri::command]
pub fn library_get_songs_by_artist(
    ctx: State<'_, Arc<AppContext>>,
    artist_id: String,
) -> Result<serde_json::Value, String> {
    let songs = ctx.library.get_songs_by_artist(&artist_id);
    serde_json::to_value(&songs).map_err(|e| format!("序列化失败: {}", e))
}

#[tauri::command]
pub fn library_get_albums_by_artist(
    ctx: State<'_, Arc<AppContext>>,
    artist_id: String,
) -> Result<serde_json::Value, String> {
    let albums = ctx.library.get_albums_by_artist(&artist_id);
    serde_json::to_value(&albums).map_err(|e| format!("序列化失败: {}", e))
}

#[tauri::command]
pub fn library_get_songs_in_album(
    ctx: State<'_, Arc<AppContext>>,
    album_id: String,
) -> Result<serde_json::Value, String> {
    let songs = ctx.library.get_songs_in_album(&album_id);
    serde_json::to_value(&songs).map_err(|e| format!("序列化失败: {}", e))
}

#[tauri::command]
pub fn library_get_source_ids_of_song(
    ctx: State<'_, Arc<AppContext>>,
    song_id: String,
) -> Result<serde_json::Value, String> {
    let ids = ctx.library.get_source_ids_of_song(&song_id);
    serde_json::to_value(&ids).map_err(|e| format!("序列化失败: {}", e))
}
