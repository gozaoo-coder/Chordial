//! Chordial web server — HTTP/REST API over chordial-core.
//!
//! 启动 axum HTTP 服务器，对外暴露完整的音乐系统 API。
//! 监听地址可通过环境变量 `CHORDIAL_BIND` 覆盖，默认 `127.0.0.1:7878`。

use chordial_core::AppContext;
use chordial_server::routes;
use chordial_server::state::AppState;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let bind_addr = std::env::var("CHORDIAL_BIND")
        .unwrap_or_else(|_| "127.0.0.1:7878".to_string());

    let ctx = Arc::new(
        AppContext::new_default_dir()
            .expect("初始化 Chordial server 层上下文失败"),
    );

    let state = AppState { ctx };

    let app = routes::build(state);

    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .expect("绑定端口失败");

    println!("[chordial-server] 监听 {}", bind_addr);

    axum::serve(listener, app)
        .await
        .expect("服务器运行错误");
}
