//! 路由模块聚合。

pub mod cache;
pub mod config;
pub mod library;
pub mod media;
pub mod rpc;
pub mod sources;
pub mod storage;

use crate::state::AppState;
use axum::Router;
use tower_http::cors::{Any, CorsLayer};

/// 组装所有域路由 + CORS。
pub fn build(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .merge(config::router())
        .merge(storage::router())
        .merge(cache::router())
        .merge(library::router())
        .merge(sources::router())
        .merge(media::router())
        .merge(rpc::router())
        .layer(cors)
        .with_state(state)
}
