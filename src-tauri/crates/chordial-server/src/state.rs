//! axum 共享状态 — 持有对 core [`AppContext`] 的引用。

use chordial_core::AppContext;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub ctx: Arc<AppContext>,
}
