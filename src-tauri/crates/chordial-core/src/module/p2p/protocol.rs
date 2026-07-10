//! P2P 协议 — 帧定义、序列化、长度前缀读写。
//!
//! # 帧格式
//!
//! 每帧使用 4 字节大端长度前缀 + JSON 载荷：
//!
//! ```text
//! [u32 BE length][JSON payload bytes]
//! ```
//!
//! 大文件（音频、封面）通过 [`Response::Data`] 字段携带，载荷以 base64 编码
//! 嵌入 JSON（v1 简化方案；后续可换为混合二进制帧以省去 33% 膨胀）。

use base64::{engine::general_purpose::STANDARD as B64, Engine};
use serde::{Deserialize, Serialize};
use std::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// 协议版本。
pub const PROTOCOL_VERSION: u32 = 1;

/// 默认监听端口。
pub const DEFAULT_PORT: u16 = 58008;

/// UDP 信标端口（广播模式）。
pub const BROADCAST_PORT: u16 = 58009;

/// 共享权限。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Permission {
    /// 仅可查询 / 流式播放
    ReadOnly,
    /// 可查询 + 可向本库写入新歌曲
    Editable,
}

impl Permission {
    pub fn as_str(self) -> &'static str {
        match self {
            Permission::ReadOnly => "readonly",
            Permission::Editable => "editable",
        }
    }
}

// ── base64 字段辅助 ─────────────────────────────────────

mod base64_bytes {
    use super::{B64, Engine};
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(v: &Vec<u8>, s: S) -> Result<S::Ok, S::Error> {
        let encoded = B64.encode(v);
        s.serialize_str(&encoded)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        let s: String = Deserialize::deserialize(d)?;
        B64.decode(s).map_err(serde::de::Error::custom)
    }
}

// ── 帧定义 ──────────────────────────────────────────────

/// 协议帧 — 握手 + 查询 + 响应。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Frame {
    /// 客户端 → 服务器：开始握手
    Hello {
        version: u32,
        match_code: String,
        client_name: String,
    },
    /// 服务器 → 客户端：直接接受或拒绝（无需用户确认时）
    HelloAck {
        accepted: bool,
        server_name: String,
        offered_permission: Permission,
        session_id: String,
        #[serde(default)]
        reason: Option<String>,
    },
    /// 服务器 → 客户端：等待用户确认
    MatchPending {
        request_id: String,
        server_name: String,
    },
    /// 服务器 → 客户端：用户确认结果
    MatchResult {
        request_id: String,
        accepted: bool,
        #[serde(default)]
        permission: Option<Permission>,
        #[serde(default)]
        session_id: Option<String>,
        #[serde(default)]
        reason: Option<String>,
    },
    /// 客户端 → 服务器：查询请求
    Query {
        id: u64,
        op: Op,
    },
    /// 服务器 → 客户端：查询响应
    Response {
        id: u64,
        ok: bool,
        #[serde(default)]
        data: serde_json::Value,
        #[serde(default)]
        error: Option<String>,
    },
}

/// 查询操作。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Op {
    SearchSongs { query: String },
    GetSong { id: String },
    GetArtist { id: String },
    GetAlbum { id: String },
    GetLyric { song_id: String },
    SongFileGet { entity_id: String },
    AlbumPictureGet { entity_id: String },
    LyricTextGet { song_id: String },
    /// 可编辑权限下，对端可推送新歌曲到本库
    AddSong { song: serde_json::Value },
    /// 拉取对端全部歌曲（用于editable 模式批量同步；只读亦可调用）
    GetAllSongs,
}

/// 响应中的二进制数据封装。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPayload {
    #[serde(with = "base64_bytes")]
    pub bytes: Vec<u8>,
}

// ── 长度前缀读写 ────────────────────────────────────────

/// 最大帧大小（32 MiB）— 防止恶意对端发送超长长度前缀耗尽内存。
const MAX_FRAME_SIZE: u32 = 32 * 1024 * 1024;

/// 写一帧。
pub async fn write_frame<W: AsyncWriteExt + Unpin>(
    writer: &mut W,
    frame: &Frame,
) -> io::Result<()> {
    let json = serde_json::to_vec(frame)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let len = json.len() as u32;
    if len > MAX_FRAME_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "frame too large",
        ));
    }
    writer.write_u32(len).await?;
    writer.write_all(&json).await?;
    writer.flush().await?;
    Ok(())
}

/// 读一帧。
pub async fn read_frame<R: AsyncReadExt + Unpin>(reader: &mut R) -> io::Result<Frame> {
    let len = reader.read_u32().await?;
    if len == 0 || len > MAX_FRAME_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid frame length: {len}"),
        ));
    }
    let mut buf = vec![0u8; len as usize];
    reader.read_exact(&mut buf).await?;
    serde_json::from_slice(&buf)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// 生成 6 位数字匹配码。
pub fn generate_match_code() -> String {
    // 简单 LCG，避免引入 rand 依赖；种子用系统时间
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0x1234_5678);
    let state = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    let n = (state >> 33) % 1_000_000;
    format!("{n:06}")
}
