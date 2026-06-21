//! Storage 域路由 — 手动落盘持久化存储（含 Blob）。
//!
//! | 方法 | 路径 | 对应命令 |
//! |------|------|---------|
//! | GET | `/storage/:key` | `storage_get` |
//! | PUT | `/storage/:key` | `storage_set` |
//! | DELETE | `/storage/:key` | `storage_remove` |
//! | HEAD | `/storage/:key` | `storage_has` |
//! | GET | `/storage` | `storage_keys` |
//! | DELETE | `/storage` | `storage_clear` |
//! | POST | `/storage/save` | `storage_save` |
//! | PUT | `/storage/blob/:key` | `storage_set_blob` (body: bytes) |
//! | GET | `/storage/blob/:key` | `storage_get_blob` |
//! | DELETE | `/storage/blob/:key` | `storage_remove_blob` |
//! | HEAD | `/storage/blob/:key` | `storage_has_blob` |
//! | GET | `/storage/blob` | `storage_blob_keys` |
//! | DELETE | `/storage/blob` | `storage_clear_blobs` |

use crate::state::AppState;
use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::Value;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/storage", get(keys).delete(clear))
        .route("/storage/save", post(save))
        .route("/storage/:key", get(get_one).put(set).delete(remove).head(has))
        // Blob 存储
        .route("/storage/blob", get(blob_keys).delete(clear_blobs))
        .route(
            "/storage/blob/:key",
            get(blob_get).put(blob_set).delete(blob_remove).head(blob_has),
        )
}

/// `GET /storage/:key` → `storage_get`
async fn get_one(State(state): State<AppState>, Path(key): Path<String>) -> impl IntoResponse {
    match state.ctx.store.get_raw(&key) {
        Some(v) => Ok(Json(v)),
        None => Err((StatusCode::NOT_FOUND, format!("键 '{}' 不存在", key))),
    }
}

/// `PUT /storage/:key` → `storage_set`
async fn set(
    State(state): State<AppState>,
    Path(key): Path<String>,
    Json(value): Json<Value>,
) -> StatusCode {
    state.ctx.store.set_raw(&key, value);
    StatusCode::NO_CONTENT
}

/// `DELETE /storage/:key` → `storage_remove`
async fn remove(State(state): State<AppState>, Path(key): Path<String>) -> Json<bool> {
    Json(state.ctx.store.remove(&key))
}

/// `HEAD /storage/:key` → `storage_has`
async fn has(State(state): State<AppState>, Path(key): Path<String>) -> StatusCode {
    if state.ctx.store.has(&key) {
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

/// `GET /storage` → `storage_keys`
async fn keys(State(state): State<AppState>) -> Json<Vec<String>> {
    Json(state.ctx.store.keys())
}

/// `DELETE /storage` → `storage_clear`
async fn clear(State(state): State<AppState>) -> StatusCode {
    state.ctx.store.clear();
    StatusCode::NO_CONTENT
}

/// `POST /storage/save` → `storage_save`
async fn save(State(state): State<AppState>) -> Result<StatusCode, String> {
    state.ctx.store.save()?;
    Ok(StatusCode::NO_CONTENT)
}

// ── Blob ────────────────────────────────────────────

/// `PUT /storage/blob/:key` → `storage_set_blob`（raw bytes body）
async fn blob_set(
    State(state): State<AppState>,
    Path(key): Path<String>,
    body: Bytes,
) -> Result<StatusCode, String> {
    state.ctx.store.set_blob(&key, &body)?;
    Ok(StatusCode::NO_CONTENT)
}

/// `GET /storage/blob/:key` → `storage_get_blob`
async fn blob_get(State(state): State<AppState>, Path(key): Path<String>) -> impl IntoResponse {
    match state.ctx.store.get_blob(&key) {
        Some(data) => Ok::<_, (StatusCode, String)>(data),
        None => Err((StatusCode::NOT_FOUND, format!("Blob 存储项 '{}' 不存在", key))),
    }
}

/// `DELETE /storage/blob/:key` → `storage_remove_blob`
async fn blob_remove(State(state): State<AppState>, Path(key): Path<String>) -> Json<bool> {
    Json(state.ctx.store.remove_blob(&key))
}

/// `HEAD /storage/blob/:key` → `storage_has_blob`
async fn blob_has(State(state): State<AppState>, Path(key): Path<String>) -> StatusCode {
    if state.ctx.store.has_blob(&key) {
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

/// `GET /storage/blob` → `storage_blob_keys`
async fn blob_keys(State(state): State<AppState>) -> Json<Vec<String>> {
    Json(state.ctx.store.blob_keys())
}

/// `DELETE /storage/blob` → `storage_clear_blobs`
async fn clear_blobs(State(state): State<AppState>) -> StatusCode {
    state.ctx.store.clear_blobs();
    StatusCode::NO_CONTENT
}
