//! 核心业务模块 — server 层。
//!
//! 这些模块**零 Tauri 依赖**，可被以下消费者复用：
//! - `chordial-tauri`：以库调用形式（Tauri 命令）向 front 提供数据
//! - `chordial-server`：以 web 服务器形式（axum HTTP）向外部提供数据
//!
//! # 模块一览
//!
//! | 模块 | 职责 |
//! |------|------|
//! | [`platform`] | 跨平台文件访问（桌面 / Android） |
//! | [`storage`] | 通用持久化后端抽象 |
//! | [`config`] | 自动防抖落盘配置存储 |
//! | [`cache`] | 内存 TTL 缓存（含可选 Blob 磁盘存储） |
//! | [`music_source`] | 来源接口 + 注册器 + 资源调度 |
//! | [`music_localSource`] | 本地文件系统来源实现 |
//! | [`music_library`] | 音乐库（Song/Artist/Album/Lyric CRUD + 关系） |
//! | [`p2p`] | P2P 资源共享（实例间对等交换曲库） |

pub mod cache;
pub mod config;
#[allow(non_snake_case)]
pub mod music_localSource;
pub mod music_library;
pub mod music_source;
pub mod p2p;
pub mod platform;
pub mod storage;
