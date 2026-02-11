//! Tauri 命令模块
//!
//! 包含所有前端可调用的后端命令，按功能分组：
//! - source: 音乐源管理
//! - library: 音乐库查询
//! - playback: 音频播放控制
//! - analysis: BPM 分析和节拍检测
//! - window: 窗口控制

pub mod source;
pub mod library;
pub mod playback;
pub mod analysis;
pub mod window;

// 重新导出所有命令，方便在 lib.rs 中统一注册
pub use source::*;
pub use library::*;
pub use playback::*;
pub use analysis::*;
pub use window::*;
