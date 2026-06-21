//! `chordial://` 自定义 URI scheme 协议适配器（Tauri 端）。
//!
//! 这是 transport 适配层：从 Tauri 的 URI scheme 回调取到请求，委托给
//! [`chordial_core::media::handle`]（与 server 端共用同一逻辑），再将返回的
//! `http::Response` 交给 Tauri 的 [`UriSchemeResponder`]。
//!
//! 由于 Tauri 的协议回调签名不提供 `State` 访问，这里用一个运行时桥接
//! [`REGISTRAR`]（在 `setup` 中由 `lib.rs` 写入）暂存对 core 的引用。

use chordial_core::module::music_source::registrar::SourceRegistrar;
use chordial_core::media;
use http::{Request, Response};
use std::sync::{Arc, OnceLock};
use tauri::UriSchemeResponder;

/// 运行时桥接：setup 阶段写入，协议回调读取。
///
/// 持有对来源注册器的静态引用（替代原 commands::source_registrar() 全局单例）。
static REGISTRAR: OnceLock<Arc<SourceRegistrar>> = OnceLock::new();

/// 由 `lib.rs` 的 setup 调用，注入来源注册器。
pub fn init(registrar: Arc<SourceRegistrar>) {
    let _ = REGISTRAR.set(registrar);
}

/// 将 core 的 `http::Response<Vec<u8>>` 适配为 Tauri 期望的响应。
fn to_tauri_response(response: Response<Vec<u8>>) -> tauri::http::Response<Vec<u8>> {
    let (parts, body) = response.into_parts();
    tauri::http::Response::from_parts(parts, body)
}

/// 处理 chordial 协议请求。
///
/// 被 Tauri 的 `register_asynchronous_uri_scheme_protocol` 调用，
/// 在独立线程中执行，可以安全地进行阻塞 I/O。
pub fn handle_protocol(request: Request<Vec<u8>>, responder: UriSchemeResponder) {
    let path = request.uri().path().to_string();

    let response = match REGISTRAR.get() {
        Some(registrar) => media::handle(registrar, &path, &request),
        None => media::error_response(
            http::StatusCode::SERVICE_UNAVAILABLE,
            "音乐来源系统未初始化",
        ),
    };

    responder.respond(to_tauri_response(response));
}
