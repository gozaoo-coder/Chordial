//! Config 域路由 — 自动防抖落盘配置存储。
//!
//! | 方法 | 路径 | 对应命令 |
//! |------|------|---------|
//! | GET | `/config/:key` | `config_get` |
//! | PUT | `/config/:key` | `config_set` |
//! | DELETE | `/config/:key` | `config_remove` |
//! | HEAD | `/config/:key` | `config_has` |
//! | GET | `/config` | `config_keys` |
//! | DELETE | `/config` | `config_clear` |
//! | POST | `/config/flush` | `config_flush` |
//! | POST | `/config/reload` | `config_reload` |

use crate::state::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::Value;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/config", get(keys).delete(clear))
        .route("/config/flush", post(flush))
        .route("/config/reload", post(reload))
        .route("/config/:key", get(get_one).put(set).delete(remove).head(has))
}

/// `GET /config/:key` → `config_get`
async fn get_one(State(state): State<AppState>, Path(key): Path<String>) -> impl IntoResponse {
    match state.ctx.config.get_raw(&key) {
        Some(v) => Ok(Json(v)),
        None => Err((StatusCode::NOT_FOUND, format!("配置项 '{}' 不存在", key))),
    }
}

/// `PUT /config/:key` → `config_set`（body 为配置值）
async fn set(
    State(state): State<AppState>,
    Path(key): Path<String>,
    Json(value): Json<Value>,
) -> Result<StatusCode, String> {
    state.ctx.config.set_raw(&key, value);
    Ok(StatusCode::NO_CONTENT)
}

/// `DELETE /config/:key` → `config_remove`
async fn remove(State(state): State<AppState>, Path(key): Path<String>) -> Json<bool> {
    Json(state.ctx.config.remove(&key))
}

/// `HEAD /config/:key` → `config_has`
async fn has(State(state): State<AppState>, Path(key): Path<String>) -> StatusCode {
    if state.ctx.config.has(&key) {
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

/// `GET /config` → `config_keys`
async fn keys(State(state): State<AppState>) -> Json<Vec<String>> {
    Json(state.ctx.config.keys())
}

/// `DELETE /config` → `config_clear`
async fn clear(State(state): State<AppState>) -> StatusCode {
    state.ctx.config.clear();
    StatusCode::NO_CONTENT
}

/// `POST /config/flush` → `config_flush`
async fn flush(State(state): State<AppState>) -> Result<StatusCode, String> {
    state.ctx.config.flush()?;
    Ok(StatusCode::NO_CONTENT)
}

/// `POST /config/reload` → `config_reload`
async fn reload(State(state): State<AppState>) -> StatusCode {
    state.ctx.config.reload();
    StatusCode::NO_CONTENT
}
