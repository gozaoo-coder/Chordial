//! 音乐来源路由 — 本地文件夹管理 + 资源获取。
//!
//! | 方法 | 路径 | 对应命令 |
//! |------|------|---------|
//! | GET | `/sources/local/folders` | `local_get_folders` |
//! | POST | `/sources/local/folders` | `local_add_folder` |
//! | DELETE | `/sources/local/folders` | `local_remove_folder` (body: {path}) |
//! | GET | `/sources/local/stats` | `local_stats` |
//! | POST | `/sources/local/rescan` | `local_rescan` |
//! | POST | `/resource/song-file` | `get_song_file` (body: SourceId JSON) |
//! | POST | `/resource/album-picture` | `get_album_picture` (body: SourceId JSON) |
//! | POST | `/resource/lyric` | `get_lyric_text` (body: SourceId JSON) |

use crate::state::AppState;
use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use chordial_core::module::music_localSource;
use chordial_core::module::music_source::resource;
use chordial_core::module::music_source::types::SourceId;
use chordial_core::module::platform::{self, PlatformPath};
use serde::Deserialize;
use std::collections::HashSet;

pub fn router() -> Router<AppState> {
    Router::new()
        // 本地来源
        .route("/sources/local/folders", get(local_get_folders).post(local_add_folder).delete(local_remove_folder))
        .route("/sources/local/stats", get(local_stats))
        .route("/sources/local/rescan", post(local_rescan))
        // 资源获取
        .route("/resource/song-file", post(get_song_file))
        .route("/resource/album-picture", post(get_album_picture))
        .route("/resource/lyric", post(get_lyric_text))
}

// ── Local Source ────────────────────────────────────

async fn local_stats(State(state): State<AppState>) -> Json<serde_json::Value> {
    let source = &state.ctx.local_source;
    let folder_count = source.folder_manager.count();
    let indexed_count = source.file_index.read().len();
    Json(serde_json::json!({
        "folder_count": folder_count,
        "indexed_files": indexed_count,
    }))
}

#[derive(Debug, Deserialize)]
struct FolderPathBody {
    path: String,
}

async fn local_get_folders(State(state): State<AppState>) -> Json<Vec<String>> {
    let source = &state.ctx.local_source;
    Json(
        source
            .folder_manager
            .get_folders()
            .iter()
            .map(|p| platform::path_to_string(p))
            .collect(),
    )
}

async fn local_add_folder(
    State(state): State<AppState>,
    Json(body): Json<FolderPathBody>,
) -> Result<Json<serde_json::Value>, String> {
    let source = &state.ctx.local_source;
    let folder_path = PlatformPath::from(body.path.as_str());
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

    Ok(Json(serde_json::json!({
        "added": true,
        "path": body.path,
        "files_found": files.len(),
        "indexed": indexed,
        "errors": errors,
    })))
}

async fn local_remove_folder(
    State(state): State<AppState>,
    Json(body): Json<FolderPathBody>,
) -> Result<Json<serde_json::Value>, String> {
    let source = &state.ctx.local_source;
    let folder_path = PlatformPath::from(body.path.as_str());

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

    for file in &files {
        let _ = source.unindex_file(file);
    }

    let removed = source.folder_manager.remove_folder(&folder_path);
    source.library.save()?;

    Ok(Json(serde_json::json!({
        "removed": removed,
        "path": body.path,
        "cleaned_files": entity_ids.len(),
    })))
}

async fn local_rescan(State(state): State<AppState>) -> Result<Json<serde_json::Value>, String> {
    let source = &state.ctx.local_source;
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

    Ok(Json(serde_json::json!({
        "indexed": total,
        "folders_scanned": folders.len(),
    })))
}

// ── Resource ────────────────────────────────────────

async fn get_song_file(
    State(state): State<AppState>,
    Json(source_id): Json<SourceId>,
) -> Result<Vec<u8>, String> {
    resource::get_song_file(&state.ctx.registrar, &source_id)
}

async fn get_album_picture(
    State(state): State<AppState>,
    Json(source_id): Json<SourceId>,
) -> Result<Vec<u8>, String> {
    resource::get_album_picture(&state.ctx.registrar, &source_id)
}

async fn get_lyric_text(
    State(state): State<AppState>,
    Json(source_id): Json<SourceId>,
) -> Result<String, String> {
    resource::get_lyric_text(&state.ctx.registrar, &source_id)
}
