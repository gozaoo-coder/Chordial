//! P2P 资源共享模块 — Chordial 实例之间的对等音乐库共享。
//!
//! # 设计
//!
//! 两台 Chordial 实例通过 TCP + 自定义 JSON 帧协议互相握手；
//! 握手成功后双方各自创建一个 [`P2pSource`] 实例（实现 [`MusicSource`]），
//! 注册到本地 [`SourceRegistrar`]，于是对方的曲库就变成可查询、可播放的来源。
//!
//! # 权限
//!
//! 由本机用户决定向对方暴露的权限（[`Permission`]）：
//! - [`Permission::ReadOnly`]（默认）：对方只能查询、流式播放本库
//! - [`Permission::Editable`]：对方还能向本库推送新歌曲
//!
//! # 发现方式
//!
//! - **不广播**：仅监听 TCP，对方必须知道本机 `IP:端口` + 当前的 6 位匹配码
//! - **广播**：额外向局域网发送 UDP 信标，前端可展示已发现的设备并点击发起握手
//!
//! # 模块布局
//!
//! | 文件 | 职责 |
//! |------|------|
//! | [`protocol`] | 帧定义 / serde / 长度前缀读写 |
//! | [`source`] | `P2pSource` — `MusicSource` 实现 |
//! | [`manager`] | `P2pManager` — 服务器 / 客户端 / 握手 / 对端生命周期 |
//! | [`broadcast`] | UDP 信标发送 + 接收 |

pub mod broadcast;
pub mod manager;
pub mod protocol;
pub mod source;

pub use manager::{P2pEvent, P2pManager, P2pStatus, PeerInfo, TrustedDevice};
pub use protocol::Permission;
pub use source::P2pSource;
