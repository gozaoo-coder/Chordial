//! RPC 路由 — 通用命令代理，对应 Tauri invoke。
//!
//! `POST /rpc` 接收 `{ "name": "command_name", "args": {...} }`，
//! 分发到对应的 core 方法并返回结果。
//!
//! 覆盖全部 62 个命令（与 commands.rs 一一对应）。

use crate::state::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{Json, Router};
use chordial_core::module::music_localSource;
use chordial_core::module::music_source::resource;
use chordial_core::module::music_source::types::SourceId;
use chordial_core::module::platform::{self, PlatformPath};
use chordial_core::module::storage::entry::Ttl;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashSet;

#[derive(Debug, Deserialize)]
struct RpcRequest {
    name: String,
    #[serde(default)]
    args: Value,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/rpc", post(handle_rpc))
}

async fn handle_rpc(
    State(state): State<AppState>,
    Json(req): Json<RpcRequest>,
) -> impl IntoResponse {
    match dispatch(&state, &req.name, &req.args) {
        Ok(value) => Ok(Json(value)),
        Err(msg) => Err((StatusCode::BAD_REQUEST, msg)),
    }
}

fn dispatch(state: &AppState, name: &str, args: &Value) -> Result<Value, String> {
    match name {
        // Config
        "config_get" => {
            let key = args["key"].as_str().ok_or("缺少 key")?;
            state.ctx.config.get_raw(key).ok_or_else(|| format!("配置项 '{}' 不存在", key))
        }
        "config_set" => {
            let key = args["key"].as_str().ok_or("缺少 key")?;
            let value = args.get("value").ok_or("缺少 value")?;
            state.ctx.config.set_raw(key, value.clone());
            Ok(Value::Null)
        }
        "config_remove" => {
            let key = args["key"].as_str().ok_or("缺少 key")?;
            Ok(json!(state.ctx.config.remove(key)))
        }
        "config_has" => {
            let key = args["key"].as_str().ok_or("缺少 key")?;
            Ok(json!(state.ctx.config.has(key)))
        }
        "config_keys" => Ok(json!(state.ctx.config.keys())),
        "config_clear" => { state.ctx.config.clear(); Ok(Value::Null) }
        "config_flush" => { state.ctx.config.flush()?; Ok(Value::Null) }
        "config_reload" => { state.ctx.config.reload(); Ok(Value::Null) }

        // Storage
        "storage_get" => {
            let key = args["key"].as_str().ok_or("缺少 key")?;
            state.ctx.store.get_raw(key).ok_or_else(|| format!("键 '{}' 不存在", key))
        }
        "storage_set" => {
            let key = args["key"].as_str().ok_or("缺少 key")?;
            let value = args.get("value").ok_or("缺少 value")?;
            state.ctx.store.set_raw(key, value.clone());
            Ok(Value::Null)
        }
        "storage_remove" => {
            let key = args["key"].as_str().ok_or("缺少 key")?;
            Ok(json!(state.ctx.store.remove(key)))
        }
        "storage_has" => {
            let key = args["key"].as_str().ok_or("缺少 key")?;
            Ok(json!(state.ctx.store.has(key)))
        }
        "storage_keys" => Ok(json!(state.ctx.store.keys())),
        "storage_clear" => { state.ctx.store.clear(); Ok(Value::Null) }
        "storage_save" => { state.ctx.store.save()?; Ok(Value::Null) }

        // Cache
        "cache_get" => {
            let key = args["key"].as_str().ok_or("缺少 key")?;
            state.ctx.cache.get_raw(key).ok_or_else(|| format!("缓存项 '{}' 不存在或已过期", key))
        }
        "cache_set" => {
            let key = args["key"].as_str().ok_or("缺少 key")?;
            let value = args.get("value").ok_or("缺少 value")?;
            let ttl = parse_ttl(args)?;
            state.ctx.cache.set_raw(key, value.clone(), &ttl);
            Ok(Value::Null)
        }
        "cache_remove" => {
            let key = args["key"].as_str().ok_or("缺少 key")?;
            Ok(json!(state.ctx.cache.remove(key)))
        }
        "cache_has" => {
            let key = args["key"].as_str().ok_or("缺少 key")?;
            Ok(json!(state.ctx.cache.has(key)))
        }
        "cache_keys" => Ok(json!(state.ctx.cache.keys())),
        "cache_clear" => { state.ctx.cache.clear(); Ok(Value::Null) }
        "cache_clear_expired" => Ok(json!(state.ctx.cache.clear_expired())),
        "cache_touch" => {
            let key = args["key"].as_str().ok_or("缺少 key")?;
            let ttl = parse_ttl(args)?;
            Ok(json!(state.ctx.cache.touch(key, &ttl)))
        }

        // Blob Cache
        "cache_enable_blob_storage" => {
            let dir = args["dir"].as_str().ok_or("缺少 dir")?;
            state.ctx.cache.enable_blob_storage(std::path::PathBuf::from(dir))?;
            Ok(Value::Null)
        }
        "cache_blob_storage_enabled" => Ok(json!(state.ctx.cache.blob_storage_enabled())),
        "cache_set_blob" => {
            let key = args["key"].as_str().ok_or("缺少 key")?;
            let data_b64 = args["data"].as_str().ok_or("缺少 data")?;
            let ttl = parse_ttl(args)?;
            use base64::Engine;
            let data = base64::engine::general_purpose::STANDARD
                .decode(data_b64)
                .map_err(|e| format!("base64 解码失败: {}", e))?;
            state.ctx.cache.set_blob(key, &data, &ttl)?;
            Ok(Value::Null)
        }
        "cache_get_blob" => {
            let key = args["key"].as_str().ok_or("缺少 key")?;
            let data = state.ctx.cache.get_blob(key)
                .ok_or_else(|| format!("Blob 缓存项 '{}' 不存在或已过期", key))?;
            use base64::Engine;
            Ok(json!(base64::engine::general_purpose::STANDARD.encode(&data)))
        }
        "cache_remove_blob" => {
            let key = args["key"].as_str().ok_or("缺少 key")?;
            Ok(json!(state.ctx.cache.remove_blob(key)))
        }
        "cache_has_blob" => {
            let key = args["key"].as_str().ok_or("缺少 key")?;
            Ok(json!(state.ctx.cache.has_blob(key)))
        }
        "cache_blob_keys" => Ok(json!(state.ctx.cache.blob_keys())),
        "cache_clear_blobs" => { state.ctx.cache.clear_blobs(); Ok(Value::Null) }
        "cache_clear_expired_blobs" => Ok(json!(state.ctx.cache.clear_expired_blobs())),

        // Blob Storage
        "storage_set_blob" => {
            let key = args["key"].as_str().ok_or("缺少 key")?;
            let data_b64 = args["data"].as_str().ok_or("缺少 data")?;
            use base64::Engine;
            let data = base64::engine::general_purpose::STANDARD
                .decode(data_b64)
                .map_err(|e| format!("base64 解码失败: {}", e))?;
            state.ctx.store.set_blob(key, &data)?;
            Ok(Value::Null)
        }
        "storage_get_blob" => {
            let key = args["key"].as_str().ok_or("缺少 key")?;
            let data = state.ctx.store.get_blob(key)
                .ok_or_else(|| format!("Blob 存储项 '{}' 不存在", key))?;
            use base64::Engine;
            Ok(json!(base64::engine::general_purpose::STANDARD.encode(&data)))
        }
        "storage_remove_blob" => {
            let key = args["key"].as_str().ok_or("缺少 key")?;
            Ok(json!(state.ctx.store.remove_blob(key)))
        }
        "storage_has_blob" => {
            let key = args["key"].as_str().ok_or("缺少 key")?;
            Ok(json!(state.ctx.store.has_blob(key)))
        }
        "storage_blob_keys" => Ok(json!(state.ctx.store.blob_keys())),
        "storage_clear_blobs" => { state.ctx.store.clear_blobs(); Ok(Value::Null) }

        // Music Source Resources
        "get_song_file" => {
            let sid: SourceId = serde_json::from_value(args.clone())
                .map_err(|e| format!("解析 SourceId: {}", e))?;
            let data = resource::get_song_file(&state.ctx.registrar, &sid)?;
            use base64::Engine;
            Ok(json!(base64::engine::general_purpose::STANDARD.encode(&data)))
        }
        "get_album_picture" => {
            let sid: SourceId = serde_json::from_value(args.clone())
                .map_err(|e| format!("解析 SourceId: {}", e))?;
            let data = resource::get_album_picture(&state.ctx.registrar, &sid)?;
            use base64::Engine;
            Ok(json!(base64::engine::general_purpose::STANDARD.encode(&data)))
        }
        "get_lyric_text" => {
            let sid: SourceId = serde_json::from_value(args.clone())
                .map_err(|e| format!("解析 SourceId: {}", e))?;
            Ok(json!(resource::get_lyric_text(&state.ctx.registrar, &sid)?))
        }

        // Local Source
        "local_stats" => {
            let source = &state.ctx.local_source;
            Ok(json!({
                "folder_count": source.folder_manager.count(),
                "indexed_files": source.file_index.read().len(),
            }))
        }
        "local_add_folder" => {
            let path = args["path"].as_str().ok_or("缺少 path")?;
            let source = &state.ctx.local_source;
            let folder_path = PlatformPath::from(path);
            source.folder_manager.add_folder(&folder_path)?;
            let files = music_localSource::folder::collect_audio_files(&folder_path);
            let mut indexed = 0u64;
            let mut errors = Vec::new();
            for file in &files {
                match source.index_file(file) {
                    Ok(true) => indexed += 1,
                    Ok(false) => {}
                    Err(e) => errors.push(format!("{}: {}", platform::path_to_string(file), e)),
                }
            }
            source.library.save()?;
            Ok(json!({ "added": true, "path": path, "files_found": files.len(), "indexed": indexed, "errors": errors }))
        }
        "local_remove_folder" => {
            let path = args["path"].as_str().ok_or("缺少 path")?;
            let source = &state.ctx.local_source;
            let folder_path = PlatformPath::from(path);
            let files = music_localSource::folder::collect_audio_files(&folder_path);
            let entity_ids: HashSet<String> = files.iter()
                .map(|f| platform::path_to_string(&platform::canonicalize(f).unwrap_or_else(|_| f.clone())))
                .collect();
            if !entity_ids.is_empty() {
                source.library.remove_specific_song_source_ids(
                    music_localSource::source::LOCAL_SOURCE_NAME, &entity_ids,
                )?;
            }
            for file in &files { let _ = source.unindex_file(file); }
            let removed = source.folder_manager.remove_folder(&folder_path);
            source.library.save()?;
            Ok(json!({ "removed": removed, "path": path, "cleaned_files": entity_ids.len() }))
        }
        "local_get_folders" => Ok(json!(state.ctx.local_source.folder_manager.get_folders()
            .iter().map(|p| platform::path_to_string(p)).collect::<Vec<_>>())),
        "local_rescan" => {
            let source = &state.ctx.local_source;
            let folders = source.folder_manager.get_folders();
            let mut total = 0u64;
            for folder in &folders {
                for file in &music_localSource::folder::collect_audio_files(folder) {
                    match source.index_file(file) {
                        Ok(true) => total += 1,
                        _ => {}
                    }
                }
            }
            source.library.save()?;
            Ok(json!({ "indexed": total, "folders_scanned": folders.len() }))
        }

        // Library persistence
        "library_save" => { state.ctx.library.save()?; Ok(Value::Null) }
        "library_cleanup_empty_entities" => { state.ctx.library.cleanup_empty_entities()?; state.ctx.library.save()?; Ok(Value::Null) }

        // Library Song
        "library_song_count" => Ok(json!(state.ctx.library.song_count())),
        "library_get_song" => {
            let id = args["id"].as_str().ok_or("缺少 id")?;
            let song = state.ctx.library.get_song(id).ok_or_else(|| format!("歌曲 '{}' 不存在", id))?;
            serde_json::to_value(&song).map_err(|e| format!("序列化失败: {}", e))
        }
        "library_get_all_songs" => {
            let songs: Vec<_> = state.ctx.library.get_all_songs().into_values().collect();
            serde_json::to_value(&songs).map_err(|e| format!("序列化失败: {}", e))
        }
        "library_search_songs" => {
            let q = args["q"].as_str().ok_or("缺少 q")?;
            serde_json::to_value(&state.ctx.library.search_songs(q)).map_err(|e| format!("序列化失败: {}", e))
        }

        // Library Artist
        "library_artist_count" => Ok(json!(state.ctx.library.artist_count())),
        "library_get_artist" => {
            let id = args["id"].as_str().ok_or("缺少 id")?;
            let artist = state.ctx.library.get_artist(id).ok_or_else(|| format!("艺术家 '{}' 不存在", id))?;
            serde_json::to_value(&artist).map_err(|e| format!("序列化失败: {}", e))
        }
        "library_get_all_artists" => {
            let artists: Vec<_> = state.ctx.library.get_all_artists().into_values().collect();
            serde_json::to_value(&artists).map_err(|e| format!("序列化失败: {}", e))
        }
        "library_search_artists" => {
            let q = args["q"].as_str().ok_or("缺少 q")?;
            serde_json::to_value(&state.ctx.library.search_artists(q)).map_err(|e| format!("序列化失败: {}", e))
        }

        // Library Album
        "library_album_count" => Ok(json!(state.ctx.library.album_count())),
        "library_get_album" => {
            let id = args["id"].as_str().ok_or("缺少 id")?;
            let album = state.ctx.library.get_album(id).ok_or_else(|| format!("专辑 '{}' 不存在", id))?;
            serde_json::to_value(&album).map_err(|e| format!("序列化失败: {}", e))
        }
        "library_get_all_albums" => {
            let albums: Vec<_> = state.ctx.library.get_all_albums().into_values().collect();
            serde_json::to_value(&albums).map_err(|e| format!("序列化失败: {}", e))
        }
        "library_search_albums" => {
            let q = args["q"].as_str().ok_or("缺少 q")?;
            serde_json::to_value(&state.ctx.library.search_albums(q)).map_err(|e| format!("序列化失败: {}", e))
        }

        // Library Lyric
        "library_lyric_count" => Ok(json!(state.ctx.library.lyric_count())),
        "library_get_lyric" => {
            let id = args["id"].as_str().ok_or("缺少 id")?;
            let lyric = state.ctx.library.get_lyric(id).ok_or_else(|| format!("歌词 '{}' 不存在", id))?;
            serde_json::to_value(&lyric).map_err(|e| format!("序列化失败: {}", e))
        }
        "library_get_all_lyrics" => {
            let lyrics: Vec<_> = state.ctx.library.get_all_lyrics().into_values().collect();
            serde_json::to_value(&lyrics).map_err(|e| format!("序列化失败: {}", e))
        }
        "library_search_lyrics" => {
            let q = args["q"].as_str().ok_or("缺少 q")?;
            serde_json::to_value(&state.ctx.library.search_lyrics(q)).map_err(|e| format!("序列化失败: {}", e))
        }

        // Library Relations
        "library_get_artists_of_song" => {
            let id = args["song_id"].as_str().ok_or("缺少 song_id")?;
            serde_json::to_value(&state.ctx.library.get_artists_of_song(id)).map_err(|e| format!("序列化失败: {}", e))
        }
        "library_get_album_of_song" => {
            let id = args["song_id"].as_str().ok_or("缺少 song_id")?;
            let album = state.ctx.library.get_album_of_song(id).ok_or_else(|| format!("歌曲 '{}' 没有关联专辑", id))?;
            serde_json::to_value(&album).map_err(|e| format!("序列化失败: {}", e))
        }
        "library_get_lyric_of_song" => {
            let id = args["song_id"].as_str().ok_or("缺少 song_id")?;
            let lyric = state.ctx.library.get_lyric_of_song(id).ok_or_else(|| format!("歌曲 '{}' 没有关联歌词", id))?;
            serde_json::to_value(&lyric).map_err(|e| format!("序列化失败: {}", e))
        }
        "library_get_songs_by_artist" => {
            let id = args["artist_id"].as_str().ok_or("缺少 artist_id")?;
            serde_json::to_value(&state.ctx.library.get_songs_by_artist(id)).map_err(|e| format!("序列化失败: {}", e))
        }
        "library_get_albums_by_artist" => {
            let id = args["artist_id"].as_str().ok_or("缺少 artist_id")?;
            serde_json::to_value(&state.ctx.library.get_albums_by_artist(id)).map_err(|e| format!("序列化失败: {}", e))
        }
        "library_get_songs_in_album" => {
            let id = args["album_id"].as_str().ok_or("缺少 album_id")?;
            serde_json::to_value(&state.ctx.library.get_songs_in_album(id)).map_err(|e| format!("序列化失败: {}", e))
        }
        "library_get_source_ids_of_song" => {
            let id = args["song_id"].as_str().ok_or("缺少 song_id")?;
            serde_json::to_value(&state.ctx.library.get_source_ids_of_song(id)).map_err(|e| format!("序列化失败: {}", e))
        }

        _ => Err(format!("未知命令: {}", name)),
    }
}

fn parse_ttl(args: &Value) -> Result<Ttl, String> {
    match args.get("ttl") {
        Some(Value::String(s)) => match s.as_str() {
            "forever" => Ok(Ttl::Forever),
            "session" => Ok(Ttl::Session),
            other => Err(format!("无效的 TTL: {}", other)),
        },
        Some(Value::Object(obj)) => {
            if let Some(secs) = obj.get("duration_secs").and_then(|v| v.as_u64()) {
                Ok(Ttl::DurationSecs(secs))
            } else {
                Err("TTL 对象缺少 duration_secs".into())
            }
        }
        None => Ok(Ttl::Session), // 默认
        _ => Err("无效的 TTL 格式".into()),
    }
}
