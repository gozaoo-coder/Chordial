//! `P2pSource` — 远端对端在本地的 [`MusicSource`] 实现。
//!
//! 每个已握手的对端会创建一个 `P2pSource` 实例并注册到 `SourceRegistrar`。
//! 之后所有针对该来源的查询（`search_songs` / `get_song` / `song_file_get` …）
//! 都会通过 TCP 转发给远端执行，远端返回结果后本侧反序列化并改写 `source_ids`，
//! 使其指向本 `P2pSource` 的来源名。

use crate::module::music_library::models::{Album, Artist, Lyric, Song};
use crate::module::music_source::traits::MusicSource;
use crate::module::music_source::types::{EntityType, SourceId, SourceType};
use crate::module::p2p::protocol::{Op, Frame, Permission};
use parking_lot::Mutex;
use std::sync::Arc;
use tokio::runtime::Handle;
use tokio::sync::{mpsc, oneshot};

/// 对端任务命令：发送一条 Query 帧，并等待对应的 Response。
pub struct PeerCommand {
    pub op: Op,
    pub reply: oneshot::Sender<Result<serde_json::Value, String>>,
}

/// 远端对端在本地的 MusicSource 实现。
pub struct P2pSource {
    /// 本来源名（`p2p-{session_short}`），作为 SourceId.source_name
    name: String,
    /// 对端展示名
    peer_name: String,
    /// 对端授予本侧的权限
    permission: Permission,
    /// 命令通道：向对端 IO 任务投递查询
    cmd_tx: mpsc::UnboundedSender<PeerCommand>,
    /// 专用 tokio 运行时 handle（来自 P2pManager）
    runtime: Handle,
    /// 关闭标志：对端断开后置 true，让 trait 方法快速失败
    closed: Mutex<bool>,
}

impl P2pSource {
    pub fn new(
        name: String,
        peer_name: String,
        permission: Permission,
        cmd_tx: mpsc::UnboundedSender<PeerCommand>,
        runtime: Handle,
    ) -> Arc<Self> {
        Arc::new(Self {
            name,
            peer_name,
            permission,
            cmd_tx,
            runtime,
            closed: Mutex::new(false),
        })
    }

    /// 标记为已断开。
    pub fn mark_closed(&self) {
        *self.closed.lock() = true;
    }

    /// 对端展示名。
    pub fn peer_name(&self) -> &str {
        &self.peer_name
    }

    /// 当前权限。
    pub fn permission(&self) -> Permission {
        self.permission
    }

    /// 发送一次 Query 并阻塞等待响应。
    fn query(&self, op: Op) -> Result<serde_json::Value, String> {
        if *self.closed.lock() {
            return Err("对端已断开".to_string());
        }
        let (tx, rx) = oneshot::channel();
        let cmd = PeerCommand { op, reply: tx };
        if self.cmd_tx.send(cmd).is_err() {
            *self.closed.lock() = true;
            return Err("对端连接已关闭".to_string());
        }
        match self.runtime.block_on(rx) {
            Ok(res) => res,
            Err(_) => Err("对端任务已退出".to_string()),
        }
    }

    /// 把远端返回的 Song 列表重写 source_ids，使其指向本 P2pSource。
    fn rewrite_songs(&self, mut songs: Vec<Song>) -> Vec<Song> {
        for s in songs.iter_mut() {
            s.source_ids = vec![SourceId::new(
                &self.name,
                SourceType::Web("p2p".into()),
                EntityType::Song,
                s.id.clone(),
            )];
        }
        songs
    }
}

impl MusicSource for P2pSource {
    fn name(&self) -> &str {
        &self.name
    }

    fn source_type(&self) -> SourceType {
        SourceType::Web("p2p".into())
    }

    fn search_songs(&self, query: &str) -> Result<Vec<Song>, String> {
        let v = self.query(Op::SearchSongs { query: query.to_string() })?;
        let songs: Vec<Song> = serde_json::from_value(v).map_err(|e| e.to_string())?;
        Ok(self.rewrite_songs(songs))
    }

    fn get_song(&self, id: &str) -> Result<Option<Song>, String> {
        let v = self.query(Op::GetSong { id: id.to_string() })?;
        if v.is_null() {
            return Ok(None);
        }
        let mut song: Song = serde_json::from_value(v).map_err(|e| e.to_string())?;
        song.source_ids = vec![SourceId::new(
            &self.name,
            SourceType::Web("p2p".into()),
            EntityType::Song,
            song.id.clone(),
        )];
        Ok(Some(song))
    }

    fn get_artist(&self, id: &str) -> Result<Option<Artist>, String> {
        let v = self.query(Op::GetArtist { id: id.to_string() })?;
        if v.is_null() {
            return Ok(None);
        }
        let mut a: Artist = serde_json::from_value(v).map_err(|e| e.to_string())?;
        a.source_ids = vec![SourceId::new(
            &self.name,
            SourceType::Web("p2p".into()),
            EntityType::Artist,
            a.id.clone(),
        )];
        Ok(Some(a))
    }

    fn get_album(&self, id: &str) -> Result<Option<Album>, String> {
        let v = self.query(Op::GetAlbum { id: id.to_string() })?;
        if v.is_null() {
            return Ok(None);
        }
        let mut a: Album = serde_json::from_value(v).map_err(|e| e.to_string())?;
        a.source_ids = vec![SourceId::new(
            &self.name,
            SourceType::Web("p2p".into()),
            EntityType::Album,
            a.id.clone(),
        )];
        Ok(Some(a))
    }

    fn get_lyric(&self, song_id: &str) -> Result<Option<Lyric>, String> {
        let v = self.query(Op::GetLyric { song_id: song_id.to_string() })?;
        if v.is_null() {
            return Ok(None);
        }
        let mut l: Lyric = serde_json::from_value(v).map_err(|e| e.to_string())?;
        l.source_id = SourceId::new(
            &self.name,
            SourceType::Web("p2p".into()),
            EntityType::Lyric,
            l.id.clone(),
        );
        Ok(Some(l))
    }

    fn song_file_get(&self, entity_id: &str) -> Result<Vec<u8>, String> {
        let v = self.query(Op::SongFileGet { entity_id: entity_id.to_string() })?;
        // 期望返回 { "bytes": "<base64>" }
        let bytes = v
            .get("bytes")
            .and_then(|b| b.as_str())
            .ok_or_else(|| "对端返回的音频载荷格式无效".to_string())?;
        use base64::{engine::general_purpose::STANDARD as B64, Engine};
        B64.decode(bytes).map_err(|e| format!("解码音频数据失败: {e}"))
    }

    fn album_picture_get(&self, entity_id: &str) -> Result<Vec<u8>, String> {
        let v = self.query(Op::AlbumPictureGet { entity_id: entity_id.to_string() })?;
        let bytes = v
            .get("bytes")
            .and_then(|b| b.as_str())
            .ok_or_else(|| "对端返回的封面载荷格式无效".to_string())?;
        use base64::{engine::general_purpose::STANDARD as B64, Engine};
        B64.decode(bytes).map_err(|e| format!("解码封面数据失败: {e}"))
    }

    fn lyric_text_get(&self, song_id: &str) -> Result<String, String> {
        let v = self.query(Op::LyricTextGet { song_id: song_id.to_string() })?;
        v.get("text")
            .and_then(|t| t.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| "对端返回的歌词格式无效".to_string())
    }
}

/// 对端 IO 任务在收到 `Frame::Response` 后通过此 helper 把结果送回调用方。
pub fn dispatch_response(
    pending: &mut std::collections::HashMap<u64, oneshot::Sender<Result<serde_json::Value, String>>>,
    frame: Frame,
) {
    if let Frame::Response { id, ok, data, error } = frame {
        if let Some(reply) = pending.remove(&id) {
            let result = if ok {
                Ok(data)
            } else {
                Err(error.unwrap_or_else(|| "未知错误".to_string()))
            };
            let _ = reply.send(result);
        }
    }
}
