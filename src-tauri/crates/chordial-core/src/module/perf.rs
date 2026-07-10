//! # 性能计时探针（仅 dev 启用）
//!
//! 提供统一的耗时测量工具，编译期受 `cfg(debug_assertions)` 门控——
//! release 构建中所有计时逻辑会被编译器完全消除，零运行时开销。
//!
//! 也可通过环境变量 `CHORDIAL_PERF=1` 在 release 中强制开启，
//! `CHORDIAL_PERF=0` 在 dev 中强制关闭。
//!
//! # 用法
//!
//! ```ignore
//! use chordial_core::module::perf;
//!
//! // 1. 函数级计时
//! perf::time("library.add_song", || {
//!     // 同步代码
//! });
//!
//! // 2. 异步函数计时
//! let result = perf::time_async("library.search", async { /* ... */ }).await;
//!
//! // 3. RAII scope 计时（最常用）
//! let _scope = perf::scope("library.get_home_stats");
//! // ... 任意代码 ...
//! // _scope drop 时自动输出耗时
//!
//! // 4. 手动 start/end
//! let token = perf::start("scan.probe");
//! // ...
//! perf::end(&token, None);
//! ```

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use std::time::Instant;

// ── 启用判断 ──────────────────────────────────────────────────────────────

/// 全局开关。dev 默认开，release 默认关，可被 env 覆盖。
static ENABLED: AtomicBool = AtomicBool::new(cfg!(debug_assertions));
static ENV_CHECKED: OnceLock<()> = OnceLock::new();

/// 首次调用时读取环境变量 CHORDIAL_PERF 覆盖默认值
fn ensure_env_applied() {
    ENV_CHECKED.get_or_init(|| {
        if let Ok(v) = std::env::var("CHORDIAL_PERF") {
            match v.as_str() {
                "1" | "true" => ENABLED.store(true, Ordering::Relaxed),
                "0" | "false" => ENABLED.store(false, Ordering::Relaxed),
                _ => {}
            }
        }
    });
}

/// 检查计时是否启用
pub fn enabled() -> bool {
    ensure_env_applied();
    ENABLED.load(Ordering::Relaxed)
}

/// 运行时切换开关
pub fn set_enabled(v: bool) {
    ENABLED.store(v, Ordering::Relaxed);
}

// ── 格式化 ────────────────────────────────────────────────────────────────

fn fmt_duration(ms: u128) -> String {
    if ms < 1 {
        format!("{}μs", ms * 1000)
    } else if ms < 1000 {
        format!("{:.1}ms", ms as f64)
    } else {
        format!("{:.2}s", ms as f64 / 1000.0)
    }
}

// ── 核心 API ──────────────────────────────────────────────────────────────

/// 手动启动计时，返回 token。配合 [`end`] 使用。
///
/// 优化：perf 关闭时跳过 `label.to_string()` 分配，返回空 token。
/// `enabled()` 仅一次 AtomicBool load，开销可忽略。
pub fn start(label: &str) -> PerfToken {
    if !enabled() {
        return PerfToken::disabled();
    }
    PerfToken {
        label: label.to_string(),
        start: Instant::now(),
        enabled: true,
    }
}

/// 结束计时并输出。传入 [`start`] 返回的 token 引用。
pub fn end(token: &PerfToken, meta: Option<&str>) {
    if !token.enabled {
        return;
    }
    let dur = token.start.elapsed().as_millis();
    let meta_str = meta.map(|m| format!(" {}", m)).unwrap_or_default();
    eprintln!(
        "⏱ [Perf] {}  {}{}",
        token.label,
        fmt_duration(dur),
        meta_str
    );
}

/// RAII scope 计时。返回 guard，drop 时自动输出。
///
/// 优化：perf 关闭时跳过 `label.to_string()` 分配。
pub fn scope(label: &str) -> PerfScope {
    if !enabled() {
        return PerfScope::disabled();
    }
    PerfScope {
        label: label.to_string(),
        start: Instant::now(),
        enabled: true,
    }
}

/// 测量同步闭包耗时并返回结果
pub fn time<T, F: FnOnce() -> T>(label: &str, f: F) -> T {
    if !enabled() {
        return f();
    }
    let start = Instant::now();
    let result = f();
    let dur = start.elapsed().as_millis();
    eprintln!("⏱ [Perf] {}  {}", label, fmt_duration(dur));
    result
}

/// 测量异步 future 耗时并返回结果
pub async fn time_async<F>(label: &str, f: F) -> F::Output
where
    F: std::future::Future,
{
    if !enabled() {
        return f.await;
    }
    let start = Instant::now();
    let result = f.await;
    let dur = start.elapsed().as_millis();
    eprintln!("⏱ [Perf] {}  {}", label, fmt_duration(dur));
    result
}

// ── 计时 Token / Scope ────────────────────────────────────────────────────

/// 手动计时的 token
pub struct PerfToken {
    label: String,
    start: Instant,
    enabled: bool,
}

impl PerfToken {
    /// 构造一个已禁用的 token（perf 关闭时使用，避免 label 分配）
    fn disabled() -> Self {
        Self {
            label: String::new(),
            start: Instant::now(),
            enabled: false,
        }
    }
}

/// RAII scope guard
pub struct PerfScope {
    label: String,
    start: Instant,
    enabled: bool,
}

impl PerfScope {
    /// 构造一个已禁用的 scope（perf 关闭时使用，避免 label 分配）
    fn disabled() -> Self {
        Self {
            label: String::new(),
            start: Instant::now(),
            enabled: false,
        }
    }
}

impl Drop for PerfScope {
    fn drop(&mut self) {
        if !self.enabled {
            return;
        }
        let dur = self.start.elapsed().as_millis();
        eprintln!("⏱ [Perf] {}  {}", self.label, fmt_duration(dur));
    }
}

// ── 宏：用于在调用点声明 scope ───────────────────────────────────────────

/// 便捷宏：`perf_scope!("library.add_song")` 等价于
/// `let _perf = chordial_core::module::perf::scope("library.add_song");`
#[macro_export]
macro_rules! perf_scope {
    ($label:expr) => {
        let _perf_scope = $crate::module::perf::scope($label);
    };
}
