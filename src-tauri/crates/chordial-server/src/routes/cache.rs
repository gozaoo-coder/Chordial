//! Cache 域路由 — 内存 TTL 缓存（含 Blob 磁盘存储）。
//!
//! | 方法 | 路径 | 对应命令 |
//! |------|------|---------|
//! | GET | `/cache/:key` | `cache_get` |
//! | PUT | `/cache/:key` | `cache_set` (body 含 ttl) |
//! | DELETE | `/cache/:key` | `cache_remove` |
//! | HEAD | `/cache/:key` | `cache_has` |
//! | GET | `/cache` | `cache_keys` |
//! | DELETE | `/cache` | `cache_clear` |
//! | POST | `/cache/clear-expired` | `cache_clear_expired` |
//! | POST | `/cache/touch` | `cache_touch` |
//! | POST | `/cache/blob/enable` | `cache_enable_blob_storage` |
//! | GET | `/cache/blob/enabled` | `cache_blob_storage_enabled` |
//! | PUT | `/cache/blob/:key` | `cache_set_blob` (body: bytes + ttl header) |
//! | GET | `/cache/blob/:key` | `cache_get_blob` |
//! | DELETE | `/cache/blob/:key` | `cache_remove_blob` |
//! | HEAD | `/cache/blob/:key` | `cache_has_blob` |
//! | GET | `/cache/blob` | `cache_blob_keys` |
//! | DELETE | `/cache/blob` | `cache_clear_blobs` |
//! | POST | `/cache/blob/clear-expired` | `cache_clear_expired_blobs` |

use crate::state::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use chordial_core::module::storage::entry::Ttl;
use serde::Deserialize;
use serde_json::Value;

/// 带 TTL 的缓存设置请求体。
#[derive(Debug, Deserialize)]
struct CacheSetBody {
    value: Value,
    #[serde(default)]
    ttl: TtlArg,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum TtlArg {
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

impl Default for TtlArg {
    fn default() -> Self {
        TtlArg::Session
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        // 纯内存缓存
        .route("/cache", get(keys).delete(clear))
        .route("/cache/clear-expired", post(clear_expired))
        .route("/cache/touch", post(touch))
        .route("/cache/:key", get(get_one).put(set).delete(remove).head(has))
        // Blob 缓存
        .route("/cache/blob/enable", post(enable_blob))
        .route("/cache/blob/enabled", get(blob_enabled))
        .route("/cache/blob", get(blob_keys).delete(clear_blobs))
        .route("/cache/blob/clear-expired", post(clear_expired_blobs))
        .route(
            "/cache/blob/:key",
            get(blob_get).put(blob_set).delete(blob_remove).head(blob_has),
        )
}

// ── 纯内存 ──────────────────────────────────────────

async fn get_one(State(state): State<AppState>, Path(key): Path<String>) -> impl IntoResponse {
    match state.ctx.cache.get_raw(&key) {
        Some(v) => Ok(Json(v)),
        None => Err((StatusCode::NOT_FOUND, format!("缓存项 '{}' 不存在或已过期", key))),
    }
}

async fn set(
    State(state): State<AppState>,
    Path(key): Path<String>,
    Json(body): Json<CacheSetBody>,
) -> StatusCode {
    state.ctx.cache.set_raw(&key, body.value, &body.ttl.into());
    StatusCode::NO_CONTENT
}

async fn remove(State(state): State<AppState>, Path(key): Path<String>) -> Json<bool> {
    Json(state.ctx.cache.remove(&key))
}

async fn has(State(state): State<AppState>, Path(key): Path<String>) -> StatusCode {
    if state.ctx.cache.has(&key) {
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

async fn keys(State(state): State<AppState>) -> Json<Vec<String>> {
    Json(state.ctx.cache.keys())
}

async fn clear(State(state): State<AppState>) -> StatusCode {
    state.ctx.cache.clear();
    StatusCode::NO_CONTENT
}

async fn clear_expired(State(state): State<AppState>) -> Json<usize> {
    Json(state.ctx.cache.clear_expired())
}

/// `POST /cache/touch` — 续期
/// body: `{ "key": "...", "ttl": "session" }`
#[derive(Debug, Deserialize)]
struct TouchBody {
    key: String,
    ttl: TtlArg,
}

async fn touch(
    State(state): State<AppState>,
    Json(body): Json<TouchBody>,
) -> Json<bool> {
    Json(state.ctx.cache.touch(&body.key, &body.ttl.into()))
}

// ── Blob ────────────────────────────────────────────

/// `POST /cache/blob/enable` — 启用 Blob 磁盘存储
/// body: `{ "dir": "/path/to/blobs" }`
#[derive(Debug, Deserialize)]
struct EnableBlobBody {
    dir: String,
}

async fn enable_blob(
    State(state): State<AppState>,
    Json(body): Json<EnableBlobBody>,
) -> Result<StatusCode, String> {
    state.ctx.cache.enable_blob_storage(std::path::PathBuf::from(body.dir))?;
    Ok(StatusCode::NO_CONTENT)
}

async fn blob_enabled(State(state): State<AppState>) -> Json<bool> {
    Json(state.ctx.cache.blob_storage_enabled())
}

/// Blob 设置请求体。
#[derive(Debug, Deserialize)]
struct BlobSetBody {
    data: String, // base64 encoded
    #[serde(default)]
    ttl: TtlArg,
}

async fn blob_set(
    State(state): State<AppState>,
    Path(key): Path<String>,
    Json(body): Json<BlobSetBody>,
) -> Result<StatusCode, String> {
    use base64::Engine;
    let data = base64::engine::general_purpose::STANDARD
        .decode(&body.data)
        .map_err(|e| format!("base64 解码失败: {}", e))?;
    state.ctx.cache.set_blob(&key, &data, &body.ttl.into())?;
    Ok(StatusCode::NO_CONTENT)
}

async fn blob_get(State(state): State<AppState>, Path(key): Path<String>) -> impl IntoResponse {
    match state.ctx.cache.get_blob(&key) {
        Some(data) => Ok::<_, (StatusCode, String)>(data),
        None => Err((StatusCode::NOT_FOUND, format!("Blob 缓存项 '{}' 不存在或已过期", key))),
    }
}

async fn blob_remove(State(state): State<AppState>, Path(key): Path<String>) -> Json<bool> {
    Json(state.ctx.cache.remove_blob(&key))
}

async fn blob_has(State(state): State<AppState>, Path(key): Path<String>) -> StatusCode {
    if state.ctx.cache.has_blob(&key) {
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

async fn blob_keys(State(state): State<AppState>) -> Json<Vec<String>> {
    Json(state.ctx.cache.blob_keys())
}

async fn clear_blobs(State(state): State<AppState>) -> StatusCode {
    state.ctx.cache.clear_blobs();
    StatusCode::NO_CONTENT
}

async fn clear_expired_blobs(State(state): State<AppState>) -> Json<usize> {
    Json(state.ctx.cache.clear_expired_blobs())
}
