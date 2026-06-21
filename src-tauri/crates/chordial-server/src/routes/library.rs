//! 音乐库路由 — Song / Artist / Album / Lyric CRUD + 搜索 + 关系查询。
//!
//! # CRUD
//! | 方法 | 路径 | 对应命令 |
//! |------|------|---------|
//! | GET | `/library/songs` | `library_get_all_songs` |
//! | GET | `/library/songs/:id` | `library_get_song` |
//! | GET | `/library/songs/search?q=` | `library_search_songs` |
//! | GET | `/library/songs/count` | `library_song_count` |
//! | GET | `/library/artists` | `library_get_all_artists` |
//! | GET | `/library/artists/:id` | `library_get_artist` |
//! | GET | `/library/artists/search?q=` | `library_search_artists` |
//! | GET | `/library/artists/count` | `library_artist_count` |
//! | GET | `/library/albums` | `library_get_all_albums` |
//! | GET | `/library/albums/:id` | `library_get_album` |
//! | GET | `/library/albums/search?q=` | `library_search_albums` |
//! | GET | `/library/albums/count` | `library_album_count` |
//! | GET | `/library/lyrics` | `library_get_all_lyrics` |
//! | GET | `/library/lyrics/:id` | `library_get_lyric` |
//! | GET | `/library/lyrics/search?q=` | `library_search_lyrics` |
//! | GET | `/library/lyrics/count` | `library_lyric_count` |
//! | POST | `/library/save` | `library_save` |
//! | POST | `/library/cleanup` | `library_cleanup_empty_entities` |
//!
//! # Relations
//! | 方法 | 路径 | 对应命令 |
//! |------|------|---------|
//! | GET | `/library/songs/:id/artists` | `library_get_artists_of_song` |
//! | GET | `/library/songs/:id/album` | `library_get_album_of_song` |
//! | GET | `/library/songs/:id/lyric` | `library_get_lyric_of_song` |
//! | GET | `/library/songs/:id/source-ids` | `library_get_source_ids_of_song` |
//! | GET | `/library/artists/:id/songs` | `library_get_songs_by_artist` |
//! | GET | `/library/artists/:id/albums` | `library_get_albums_by_artist` |
//! | GET | `/library/albums/:id/songs` | `library_get_songs_in_album` |

use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;

pub fn router() -> Router<AppState> {
    Router::new()
        // 持久化
        .route("/library/save", post(library_save))
        .route("/library/cleanup", post(library_cleanup))
        // Song
        .route("/library/songs", get(get_all_songs))
        .route("/library/songs/count", get(song_count))
        .route("/library/songs/search", get(search_songs))
        .route(
            "/library/songs/:id",
            get(get_song),
        )
        .route("/library/songs/:id/artists", get(artists_of_song))
        .route("/library/songs/:id/album", get(album_of_song))
        .route("/library/songs/:id/lyric", get(lyric_of_song))
        .route("/library/songs/:id/source-ids", get(source_ids_of_song))
        // Artist
        .route("/library/artists", get(get_all_artists))
        .route("/library/artists/count", get(artist_count))
        .route("/library/artists/search", get(search_artists))
        .route(
            "/library/artists/:id",
            get(get_artist),
        )
        .route("/library/artists/:id/songs", get(songs_by_artist))
        .route("/library/artists/:id/albums", get(albums_by_artist))
        // Album
        .route("/library/albums", get(get_all_albums))
        .route("/library/albums/count", get(album_count))
        .route("/library/albums/search", get(search_albums))
        .route(
            "/library/albums/:id",
            get(get_album),
        )
        .route("/library/albums/:id/songs", get(songs_in_album))
        // Lyric
        .route("/library/lyrics", get(get_all_lyrics))
        .route("/library/lyrics/count", get(lyric_count))
        .route("/library/lyrics/search", get(search_lyrics))
        .route(
            "/library/lyrics/:id",
            get(get_lyric),
        )
}

// ── 持久化 ──────────────────────────────────────────

async fn library_save(State(state): State<AppState>) -> Result<StatusCode, String> {
    state.ctx.library.save()?;
    Ok(StatusCode::NO_CONTENT)
}

async fn library_cleanup(State(state): State<AppState>) -> Result<StatusCode, String> {
    state.ctx.library.cleanup_empty_entities()?;
    state.ctx.library.save()?;
    Ok(StatusCode::NO_CONTENT)
}

// ── Song ────────────────────────────────────────────

async fn song_count(State(state): State<AppState>) -> Json<usize> {
    Json(state.ctx.library.song_count())
}

async fn get_song(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    match state.ctx.library.get_song(&id) {
        Some(song) => Ok(Json(serde_json::to_value(&song).unwrap())),
        None => Err((StatusCode::NOT_FOUND, format!("歌曲 '{}' 不存在", id))),
    }
}

async fn get_all_songs(State(state): State<AppState>) -> Json<serde_json::Value> {
    let songs: Vec<_> = state.ctx.library.get_all_songs().into_values().collect();
    Json(serde_json::to_value(&songs).unwrap())
}

#[derive(Debug, Deserialize)]
struct SearchQuery {
    q: String,
}

async fn search_songs(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Json<serde_json::Value> {
    let songs = state.ctx.library.search_songs(&query.q);
    Json(serde_json::to_value(&songs).unwrap())
}

// ── Artist ──────────────────────────────────────────

async fn artist_count(State(state): State<AppState>) -> Json<usize> {
    Json(state.ctx.library.artist_count())
}

async fn get_artist(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    match state.ctx.library.get_artist(&id) {
        Some(artist) => Ok(Json(serde_json::to_value(&artist).unwrap())),
        None => Err((StatusCode::NOT_FOUND, format!("艺术家 '{}' 不存在", id))),
    }
}

async fn get_all_artists(State(state): State<AppState>) -> Json<serde_json::Value> {
    let artists: Vec<_> = state.ctx.library.get_all_artists().into_values().collect();
    Json(serde_json::to_value(&artists).unwrap())
}

async fn search_artists(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Json<serde_json::Value> {
    let artists = state.ctx.library.search_artists(&query.q);
    Json(serde_json::to_value(&artists).unwrap())
}

// ── Album ───────────────────────────────────────────

async fn album_count(State(state): State<AppState>) -> Json<usize> {
    Json(state.ctx.library.album_count())
}

async fn get_album(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    match state.ctx.library.get_album(&id) {
        Some(album) => Ok(Json(serde_json::to_value(&album).unwrap())),
        None => Err((StatusCode::NOT_FOUND, format!("专辑 '{}' 不存在", id))),
    }
}

async fn get_all_albums(State(state): State<AppState>) -> Json<serde_json::Value> {
    let albums: Vec<_> = state.ctx.library.get_all_albums().into_values().collect();
    Json(serde_json::to_value(&albums).unwrap())
}

async fn search_albums(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Json<serde_json::Value> {
    let albums = state.ctx.library.search_albums(&query.q);
    Json(serde_json::to_value(&albums).unwrap())
}

// ── Lyric ───────────────────────────────────────────

async fn lyric_count(State(state): State<AppState>) -> Json<usize> {
    Json(state.ctx.library.lyric_count())
}

async fn get_lyric(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    match state.ctx.library.get_lyric(&id) {
        Some(lyric) => Ok(Json(serde_json::to_value(&lyric).unwrap())),
        None => Err((StatusCode::NOT_FOUND, format!("歌词 '{}' 不存在", id))),
    }
}

async fn get_all_lyrics(State(state): State<AppState>) -> Json<serde_json::Value> {
    let lyrics: Vec<_> = state.ctx.library.get_all_lyrics().into_values().collect();
    Json(serde_json::to_value(&lyrics).unwrap())
}

async fn search_lyrics(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Json<serde_json::Value> {
    let lyrics = state.ctx.library.search_lyrics(&query.q);
    Json(serde_json::to_value(&lyrics).unwrap())
}

// ── Relations ───────────────────────────────────────

async fn artists_of_song(
    State(state): State<AppState>,
    Path(song_id): Path<String>,
) -> Json<serde_json::Value> {
    let artists = state.ctx.library.get_artists_of_song(&song_id);
    Json(serde_json::to_value(&artists).unwrap())
}

async fn album_of_song(
    State(state): State<AppState>,
    Path(song_id): Path<String>,
) -> impl IntoResponse {
    match state.ctx.library.get_album_of_song(&song_id) {
        Some(album) => Ok(Json(serde_json::to_value(&album).unwrap())),
        None => Err((StatusCode::NOT_FOUND, format!("歌曲 '{}' 没有关联专辑", song_id))),
    }
}

async fn lyric_of_song(
    State(state): State<AppState>,
    Path(song_id): Path<String>,
) -> impl IntoResponse {
    match state.ctx.library.get_lyric_of_song(&song_id) {
        Some(lyric) => Ok(Json(serde_json::to_value(&lyric).unwrap())),
        None => Err((StatusCode::NOT_FOUND, format!("歌曲 '{}' 没有关联歌词", song_id))),
    }
}

async fn source_ids_of_song(
    State(state): State<AppState>,
    Path(song_id): Path<String>,
) -> Json<serde_json::Value> {
    let ids = state.ctx.library.get_source_ids_of_song(&song_id);
    Json(serde_json::to_value(&ids).unwrap())
}

async fn songs_by_artist(
    State(state): State<AppState>,
    Path(artist_id): Path<String>,
) -> Json<serde_json::Value> {
    let songs = state.ctx.library.get_songs_by_artist(&artist_id);
    Json(serde_json::to_value(&songs).unwrap())
}

async fn albums_by_artist(
    State(state): State<AppState>,
    Path(artist_id): Path<String>,
) -> Json<serde_json::Value> {
    let albums = state.ctx.library.get_albums_by_artist(&artist_id);
    Json(serde_json::to_value(&albums).unwrap())
}

async fn songs_in_album(
    State(state): State<AppState>,
    Path(album_id): Path<String>,
) -> Json<serde_json::Value> {
    let songs = state.ctx.library.get_songs_in_album(&album_id);
    Json(serde_json::to_value(&songs).unwrap())
}
