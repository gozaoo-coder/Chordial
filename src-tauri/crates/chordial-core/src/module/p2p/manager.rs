//! `P2pManager` — P2P 共享的顶层控制中心。
//!
//! 负责：
//! - 启停 TCP 监听 + UDP 广播
//! - 处理入站握手（等待本机用户确认）
//! - 主动发起出站握手
//! - 已握手对端的生命周期（创建 `P2pSource`、注册到 `SourceRegistrar`、断开清理）
//! - 维护当前匹配码 / 共享权限 / 监听地址
//!
//! 所有公共方法均为同步方法，内部通过自有的 tokio 运行时驱动异步任务。

use crate::module::music_library::library::MusicLibrary;
use crate::module::music_library::models::Song;
use crate::module::music_source::registrar::SourceRegistrar;
use crate::module::music_source::resource as src_resource;
use crate::module::music_source::traits::MusicSource;
use crate::module::music_source::types::{EntityType, SourceType};
use crate::module::p2p::broadcast::{Beacon, BroadcastDiscovery, DiscoveredPeer};
use crate::module::p2p::protocol::{
    generate_match_code, read_frame, write_frame, Frame, Op, Permission, DEFAULT_PORT,
    PROTOCOL_VERSION,
};
use crate::module::p2p::source::{PeerCommand, P2pSource};
use parking_lot::RwLock;
use serde::Serialize;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::runtime::{Builder, Runtime};
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

/// P2P 事件 — 通过外部注入的事件通道送出，由 Tauri 层转发为前端事件。
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum P2pEvent {
    /// 收到入站握手请求，等待本机用户确认
    MatchRequested {
        request_id: String,
        peer_addr: String,
        peer_name: String,
    },
    /// 握手完成、对端已连接
    PeerConnected {
        peer_id: String,
        peer_name: String,
        addr: String,
        permission: Permission,
    },
    /// 对端断开
    PeerDisconnected {
        peer_id: String,
        reason: String,
    },
    /// 出站握手结果
    MatchResult {
        request_id: String,
        accepted: bool,
        reason: Option<String>,
    },
}

/// 已连接的对端信息（用于 status() 输出）。
#[derive(Debug, Clone, Serialize)]
pub struct PeerInfo {
    pub id: String,
    pub name: String,
    pub addr: String,
    pub permission: Permission,
}

/// 管理器状态快照。
#[derive(Debug, Clone, Serialize)]
pub struct P2pStatus {
    pub listening: bool,
    pub listen_addr: String,
    pub match_code: String,
    pub permission: Permission,
    pub broadcast_enabled: bool,
    pub peers: Vec<PeerInfo>,
    pub discovered: Vec<DiscoveredPeer>,
}

struct PeerEntry {
    source: Arc<P2pSource>,
    name: String,
    addr: String,
    permission: Permission,
    /// 关闭信号：peer task 收到后退出
    shutdown_tx: oneshot::Sender<()>,
}

struct PendingRequest {
    reply: oneshot::Sender<bool>,
}

/// P2P 管理器。
pub struct P2pManager {
    library: Arc<MusicLibrary>,
    registrar: Arc<SourceRegistrar>,
    server_name: String,

    inner: RwLock<Inner>,
    discovery: BroadcastDiscovery,
    runtime: Runtime,
}

struct Inner {
    listening: bool,
    listen_addr: String,
    match_code: String,
    permission: Permission,
    peers: HashMap<String, PeerEntry>,
    pending_requests: HashMap<String, PendingRequest>,
    /// 接收方为外部事件订阅者；manager 内部 clone 后发出事件
    event_tx: Option<mpsc::UnboundedSender<P2pEvent>>,
    /// 关闭整个监听 accept 循环
    server_shutdown: Option<oneshot::Sender<()>>,
}

impl P2pManager {
    /// 创建管理器。不会立即启动监听。
    pub fn new(library: Arc<MusicLibrary>, registrar: Arc<SourceRegistrar>) -> Arc<Self> {
        let runtime = Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .thread_name("p2p-runtime")
            .build()
            .expect("构建 P2P tokio 运行时失败");

        let server_name = std::env::var("USERNAME")
            .or_else(|_| std::env::var("USER"))
            .unwrap_or_else(|_| "chordial".to_string());
        let server_name = format!("{}-{}", server_name, &Uuid::new_v4().to_string()[..4]);

        let inner = Inner {
            listening: false,
            listen_addr: String::new(),
            match_code: generate_match_code(),
            permission: Permission::ReadOnly,
            peers: HashMap::new(),
            pending_requests: HashMap::new(),
            event_tx: None,
            server_shutdown: None,
        };

        Arc::new(Self {
            library,
            registrar,
            server_name,
            inner: RwLock::new(inner),
            discovery: BroadcastDiscovery::new(),
            runtime,
        })
    }

    /// 注入事件通道 — Tauri 层用此通道消费事件并转发为前端事件。
    pub fn set_event_channel(self: &Arc<Self>, tx: mpsc::UnboundedSender<P2pEvent>) {
        self.inner.write().event_tx = Some(tx);
    }

    fn emit(&self, evt: P2pEvent) {
        if let Some(tx) = self.inner.read().event_tx.as_ref() {
            let _ = tx.send(evt);
        }
    }

    /// 启动共享服务。
    pub fn start_server(
        self: &Arc<Self>,
        broadcast: bool,
        permission: Permission,
    ) -> Result<(), String> {
        let mut g = self.inner.write();
        if g.listening {
            return Err("服务已在运行".into());
        }
        g.permission = permission;

        let listener = self
            .runtime
            .block_on(async move {
                TcpListener::bind(("0.0.0.0", DEFAULT_PORT)).await
            })
            .map_err(|e| format!("绑定端口 {DEFAULT_PORT} 失败: {e}"))?;

        let local_addr = listener
            .local_addr()
            .map(|a| a.to_string())
            .unwrap_or_else(|_| format!("0.0.0.0:{DEFAULT_PORT}"));

        g.listen_addr = local_addr.clone();
        g.listening = true;

        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        g.server_shutdown = Some(shutdown_tx);

        // 入站 accept 循环
        let this = self.clone();
        self.runtime.spawn(async move {
            let _ = Self::accept_loop(this, listener, shutdown_rx).await;
        });

        drop(g);

        // 广播
        if broadcast {
            let _ = self.start_broadcast();
        }

        Ok(())
    }

    /// 停止共享服务。
    pub fn stop_server(self: &Arc<Self>) {
        let mut g = self.inner.write();
        g.listening = false;
        g.listen_addr.clear();
        if let Some(tx) = g.server_shutdown.take() {
            let _ = tx.send(());
        }
        // 关闭所有对端
        let peers: Vec<(String, PeerEntry)> = g.peers.drain().collect();
        let pending: Vec<(String, PendingRequest)> = g.pending_requests.drain().collect();
        drop(g);

        for (id, entry) in peers {
            let _ = entry.shutdown_tx.send(());
            self.registrar.unregister(&entry.source.name());
            entry.source.mark_closed();
            self.emit(P2pEvent::PeerDisconnected {
                peer_id: id,
                reason: "服务停止".into(),
            });
        }
        for (_, req) in pending {
            let _ = req.reply.send(false);
        }

        self.stop_broadcast();
    }

    /// 主动发起握手。
    pub fn request_match(self: &Arc<Self>, addr: String, match_code: String) -> Result<(), String> {
        let this = self.clone();
        self.runtime.spawn(async move {
            let result = Self::outgoing_handshake(this.clone(), addr.clone(), match_code).await;
            if let Err(e) = result {
                eprintln!("[p2p] 出站握手失败 {addr}: {e}");
                // 出站握手没有 request_id（不需要本机用户确认），直接以失败事件回报
                this.emit(P2pEvent::MatchResult {
                    request_id: String::new(),
                    accepted: false,
                    reason: Some(e),
                });
            }
        });
        Ok(())
    }

    /// 响应入站握手请求。
    pub fn respond_match(&self, request_id: String, accepted: bool) -> Result<(), String> {
        let req = self.inner.write().pending_requests.remove(&request_id);
        match req {
            Some(r) => {
                let _ = r.reply.send(accepted);
                if !accepted {
                    self.emit(P2pEvent::MatchResult {
                        request_id,
                        accepted: false,
                        reason: Some("本机用户拒绝".into()),
                    });
                }
                Ok(())
            }
            None => Err(format!("未找到匹配请求 {request_id}")),
        }
    }

    /// 断开指定对端。
    pub fn disconnect_peer(&self, peer_id: String) {
        let entry = self.inner.write().peers.remove(&peer_id);
        if let Some(e) = entry {
            let _ = e.shutdown_tx.send(());
            self.registrar.unregister(&e.source.name());
            e.source.mark_closed();
            self.emit(P2pEvent::PeerDisconnected {
                peer_id,
                reason: "本机用户断开".into(),
            });
        }
    }

    /// 设置本机共享权限（影响后续握手；已连接对端不变）。
    pub fn set_permission(&self, permission: Permission) {
        self.inner.write().permission = permission;
    }

    /// 重新生成匹配码。
    pub fn regenerate_match_code(&self) -> String {
        let code = generate_match_code();
        self.inner.write().match_code = code.clone();
        // 同步广播信标
        if self.discovery.is_enabled() {
            let port = self
                .inner
                .read()
                .listen_addr
                .rsplit(':')
                .next()
                .and_then(|s| s.parse::<u16>().ok())
                .unwrap_or(DEFAULT_PORT);
            self.discovery.update_beacon(Beacon {
                name: self.server_name.clone(),
                port,
                match_code: code.clone(),
            });
        }
        code
    }

    /// 切换广播发现。
    pub fn set_broadcast(self: &Arc<Self>, enabled: bool) -> Result<(), String> {
        if enabled {
            self.start_broadcast()
        } else {
            self.stop_broadcast();
            Ok(())
        }
    }

    fn start_broadcast(self: &Arc<Self>) -> Result<(), String> {
        let port = self
            .inner
            .read()
            .listen_addr
            .rsplit(':')
            .next()
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(DEFAULT_PORT);
        let code = self.inner.read().match_code.clone();
        let beacon = Beacon {
            name: self.server_name.clone(),
            port,
            match_code: code,
        };
        self.runtime
            .block_on(self.discovery.enable(beacon))
            .map_err(|e| format!("启用广播失败: {e}"))
    }

    fn stop_broadcast(&self) {
        self.discovery.disable();
    }

    /// 当前状态快照。
    pub fn status(&self) -> P2pStatus {
        let g = self.inner.read();
        P2pStatus {
            listening: g.listening,
            listen_addr: g.listen_addr.clone(),
            match_code: g.match_code.clone(),
            permission: g.permission,
            broadcast_enabled: self.discovery.is_enabled(),
            peers: g
                .peers
                .iter()
                .map(|(id, e)| PeerInfo {
                    id: id.clone(),
                    name: e.name.clone(),
                    addr: e.addr.clone(),
                    permission: e.permission,
                })
                .collect(),
            discovered: self.discovery.list_discovered(),
        }
    }

    // ── 内部：入站 accept 循环 ──────────────────────────

    async fn accept_loop(
        self: Arc<Self>,
        listener: TcpListener,
        mut shutdown: oneshot::Receiver<()>,
    ) {
        loop {
            tokio::select! {
                _ = &mut shutdown => {
                    eprintln!("[p2p] accept 循环收到关闭信号");
                    return;
                }
                accept = listener.accept() => {
                    let (stream, peer_addr) = match accept {
                        Ok(v) => v,
                        Err(e) => {
                            eprintln!("[p2p] accept 失败: {e}");
                            continue;
                        }
                    };
                    let this = self.clone();
                    tokio::spawn(async move {
                        if let Err(e) = this.incoming_handshake(stream, peer_addr).await {
                            eprintln!("[p2p] 入站握手失败 {}: {e}", peer_addr);
                        }
                    });
                }
            }
        }
    }

    async fn incoming_handshake(
        self: Arc<Self>,
        mut stream: TcpStream,
        peer_addr: SocketAddr,
    ) -> Result<(), String> {
        // 1. 读 Hello
        let hello = match read_frame(&mut stream).await.map_err(|e| e.to_string())? {
            Frame::Hello { version, match_code, client_name } => {
                if version != PROTOCOL_VERSION {
                    let perm = self.inner.read().permission;
                    let _ = write_frame(
                        &mut stream,
                        &Frame::HelloAck {
                            accepted: false,
                            server_name: self.server_name.clone(),
                            offered_permission: perm,
                            session_id: String::new(),
                            reason: Some(format!("协议版本不匹配: 本机 {PROTOCOL_VERSION}, 对端 {version}")),
                        },
                    )
                    .await;
                    return Err(format!("协议版本不匹配: {version}"));
                }
                (match_code, client_name)
            }
            _ => return Err("期望 Hello 帧".into()),
        };

        // 2. 校验匹配码
        let (code_ok, our_code) = {
            let g = self.inner.read();
            (hello.0 == g.match_code, g.match_code.clone())
        };
        if !code_ok {
            let perm = self.inner.read().permission;
            let _ = write_frame(
                &mut stream,
                &Frame::HelloAck {
                    accepted: false,
                    server_name: self.server_name.clone(),
                    offered_permission: perm,
                    session_id: String::new(),
                    reason: Some("匹配码错误".into()),
                },
            )
            .await;
            return Err(format!("匹配码错误 (对端 {})", hello.0));
        }
        let _ = our_code; // 仅用于明确语义

        // 3. 请求本机用户确认
        let request_id = Uuid::new_v4().to_string();
        let (tx, rx) = oneshot::channel::<bool>();
        self.inner.write().pending_requests.insert(
            request_id.clone(),
            PendingRequest {
                reply: tx,
            },
        );
        self.emit(P2pEvent::MatchRequested {
            request_id: request_id.clone(),
            peer_addr: peer_addr.to_string(),
            peer_name: hello.1.clone(),
        });

        // 通知对端：等待用户确认
        let _ = write_frame(
            &mut stream,
            &Frame::MatchPending {
                request_id: request_id.clone(),
                server_name: self.server_name.clone(),
            },
        )
        .await;

        // 4. 等待本机用户响应（超时 60 秒）
        let accepted = match tokio::time::timeout(Duration::from_secs(60), rx).await {
            Ok(Ok(b)) => b,
            _ => {
                self.inner.write().pending_requests.remove(&request_id);
                let _ = write_frame(
                    &mut stream,
                    &Frame::MatchResult {
                        request_id: request_id.clone(),
                        accepted: false,
                        permission: None,
                        session_id: None,
                        reason: Some("超时或取消".into()),
                    },
                )
                .await;
                return Err("等待用户确认超时".into());
            }
        };

        if !accepted {
            let _ = write_frame(
                &mut stream,
                &Frame::MatchResult {
                    request_id: request_id.clone(),
                    accepted: false,
                    permission: None,
                    session_id: None,
                    reason: Some("本机用户拒绝".into()),
                },
            )
            .await;
            self.emit(P2pEvent::MatchResult {
                request_id,
                accepted: false,
                reason: Some("本机用户拒绝".into()),
            });
            return Ok(());
        }

        // 5. 接受 → 发送 MatchResult，开始对端 IO
        let permission = self.inner.read().permission;
        let session_id = Uuid::new_v4().to_string();
        let _ = write_frame(
            &mut stream,
            &Frame::MatchResult {
                request_id: request_id.clone(),
                accepted: true,
                permission: Some(permission),
                session_id: Some(session_id.clone()),
                reason: None,
            },
        )
        .await;

        self.emit(P2pEvent::MatchResult {
            request_id,
            accepted: true,
            reason: None,
        });

        // 6. 启动对端 IO 任务 + 注册 P2pSource
        self.spawn_peer_task(
            stream,
            session_id,
            hello.1,
            peer_addr.to_string(),
            permission,
            /* incoming = */ true,
        )
        .await;

        Ok(())
    }

    // ── 内部：出站握手 ──────────────────────────────────

    async fn outgoing_handshake(
        self: Arc<Self>,
        addr: String,
        match_code: String,
    ) -> Result<(), String> {
        let mut stream = TcpStream::connect(&addr)
            .await
            .map_err(|e| format!("连接 {addr} 失败: {e}"))?;

        // 发 Hello
        write_frame(
            &mut stream,
            &Frame::Hello {
                version: PROTOCOL_VERSION,
                match_code,
                client_name: self.server_name.clone(),
            },
        )
        .await
        .map_err(|e| format!("发送 Hello 失败: {e}"))?;

        // 读回复：可能是 HelloAck（对端自动接受/拒绝）或 MatchPending + MatchResult
        let mut pending_request_id: Option<String> = None;
        let mut peer_name = String::from("unknown");
        loop {
            let frame = read_frame(&mut stream).await.map_err(|e| format!("读握手响应失败: {e}"))?;
            match frame {
                Frame::HelloAck { accepted, server_name, reason, .. } => {
                    peer_name = server_name;
                    if !accepted {
                        return Err(reason.unwrap_or_else(|| "对端拒绝握手".into()));
                    }
                    // 对端自动接受（无需用户确认）— 但本协议规范下对端总是要求用户确认，
                    // 所以这条分支理论上只在对端实现快捷路径时到达。直接结束握手。
                    break;
                }
                Frame::MatchPending { request_id, server_name } => {
                    pending_request_id = Some(request_id);
                    peer_name = server_name;
                    // 继续等 MatchResult
                }
                Frame::MatchResult { accepted, permission, session_id, reason, .. } => {
                    if !accepted {
                        return Err(reason.unwrap_or_else(|| "对端用户拒绝".into()));
                    }
                    let session_id = session_id.unwrap_or_else(|| Uuid::new_v4().to_string());
                    let permission = permission.unwrap_or(Permission::ReadOnly);
                    let _ = pending_request_id; // 仅用于去重
                    self.spawn_peer_task(
                        stream,
                        session_id,
                        peer_name,
                        addr,
                        permission,
                        /* incoming = */ false,
                    )
                    .await;
                    return Ok(());
                }
                other => {
                    return Err(format!("握手期间收到非预期帧: {:?}", other.tag()));
                }
            }
        }

        // 走到此处说明对端用 HelloAck 直接接受（无 MatchResult）
        let session_id = Uuid::new_v4().to_string();
        let permission = self.inner.read().permission; // 对端未告知，用本机默认推断
        self.spawn_peer_task(
            stream,
            session_id,
            peer_name,
            addr,
            permission,
            /* incoming = */ false,
        )
        .await;
        Ok(())
    }

    // ── 内部：对端 IO 任务 ─────────────────────────────

    async fn spawn_peer_task(
        self: Arc<Self>,
        stream: TcpStream,
        session_id: String,
        peer_name: String,
        peer_addr: String,
        permission: Permission,
        _incoming: bool,
    ) {
        let source_name = format!("p2p-{}", &session_id[..8.min(session_id.len())]);
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel::<PeerCommand>();
        let runtime = self.runtime.handle().clone();
        let source = P2pSource::new(
            source_name.clone(),
            peer_name.clone(),
            permission,
            cmd_tx,
            runtime,
        );

        // 注册到 SourceRegistrar
        if let Err(e) = self.registrar.register(source.clone()) {
            eprintln!("[p2p] 注册 P2pSource 失败: {e}");
            return;
        }

        let peer_id = session_id.clone();
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        self.inner.write().peers.insert(
            peer_id.clone(),
            PeerEntry {
                source: source.clone(),
                name: peer_name.clone(),
                addr: peer_addr.clone(),
                permission,
                shutdown_tx,
            },
        );

        self.emit(P2pEvent::PeerConnected {
            peer_id: peer_id.clone(),
            peer_name: peer_name.clone(),
            addr: peer_addr.clone(),
            permission,
        });

        let library = self.library.clone();
        let registrar = self.registrar.clone();
        let offered_permission = self.inner.read().permission;
        let pid_for_task = peer_id.clone();
        let this_for_task = self.clone();

        self.runtime.spawn(async move {
            Self::peer_io_loop(
                this_for_task,
                pid_for_task,
                source,
                stream,
                cmd_rx,
                shutdown_rx,
                library,
                registrar,
                offered_permission,
            )
            .await;
        });
    }

    async fn peer_io_loop(
        self: Arc<Self>,
        peer_id: String,
        source: Arc<P2pSource>,
        stream: TcpStream,
        mut cmd_rx: mpsc::UnboundedReceiver<PeerCommand>,
        mut shutdown: oneshot::Receiver<()>,
        library: Arc<MusicLibrary>,
        registrar: Arc<SourceRegistrar>,
        offered_permission: Permission,
    ) {
        let (mut reader, mut writer) = stream.into_split();
        let mut pending: HashMap<u64, oneshot::Sender<Result<serde_json::Value, String>>> =
            HashMap::new();
        let mut next_query_id: u64 = 1;

        loop {
            tokio::select! {
                _ = &mut shutdown => {
                    eprintln!("[p2p:{}] 收到关闭信号", peer_id);
                    break;
                }
                // 本地 P2pSource 发来一条查询
                cmd = cmd_rx.recv() => {
                    let Some(cmd) = cmd else { break; };
                    let id = next_query_id;
                    next_query_id += 1;
                    pending.insert(id, cmd.reply);
                    if write_frame(&mut writer, &Frame::Query { id, op: cmd.op }).await.is_err() {
                        if let Some(tx) = pending.remove(&id) {
                            let _ = tx.send(Err("发送查询失败".into()));
                        }
                        break;
                    }
                }
                // 对端发来一帧
                frame_result = read_frame(&mut reader) => {
                    let frame = match frame_result {
                        Ok(f) => f,
                        Err(e) => {
                            eprintln!("[p2p:{}] 读帧失败: {e}", peer_id);
                            break;
                        }
                    };
                    match frame {
                        Frame::Response { id, ok, data, error } => {
                            let res = if ok { Ok(data) } else { Err(error.unwrap_or_else(|| "未知错误".into())) };
                            if let Some(tx) = pending.remove(&id) {
                                let _ = tx.send(res);
                            }
                        }
                        Frame::Query { id, op } => {
                            let resp = Self::handle_query(&library, &registrar, offered_permission, op).await;
                            let (ok, data, error) = match resp {
                                Ok(v) => (true, v, None),
                                Err(e) => (false, serde_json::Value::Null, Some(e)),
                            };
                            if write_frame(&mut writer, &Frame::Response { id, ok, data, error }).await.is_err() {
                                break;
                            }
                        }
                        _ => {
                            eprintln!("[p2p:{}] 握手后收到非查询帧，忽略", peer_id);
                        }
                    }
                }
            }
        }

        // 清理
        source.mark_closed();
        self.registrar.unregister(&source.name());
        self.inner.write().peers.remove(&peer_id);
        self.emit(P2pEvent::PeerDisconnected {
            peer_id,
            reason: "连接断开".into(),
        });
    }

    /// 处理对端发来的查询 — 操作本地库 / 来源系统。
    async fn handle_query(
        library: &MusicLibrary,
        registrar: &SourceRegistrar,
        offered_permission: Permission,
        op: Op,
    ) -> Result<serde_json::Value, String> {
        use base64::{engine::general_purpose::STANDARD as B64, Engine};
        match op {
            Op::SearchSongs { query } => {
                let songs = library.search_songs(&query);
                serde_json::to_value(songs).map_err(|e| e.to_string())
            }
            Op::GetSong { id } => {
                let s = library.get_song(&id);
                serde_json::to_value(s).map_err(|e| e.to_string())
            }
            Op::GetArtist { id } => {
                let a = library.get_artist(&id);
                serde_json::to_value(a).map_err(|e| e.to_string())
            }
            Op::GetAlbum { id } => {
                let a = library.get_album(&id);
                serde_json::to_value(a).map_err(|e| e.to_string())
            }
            Op::GetLyric { song_id } => {
                let l = library.get_lyric_of_song(&song_id);
                serde_json::to_value(l).map_err(|e| e.to_string())
            }
            Op::SongFileGet { entity_id } => {
                // entity_id 是本库内 song UUID；找一个 Local 来源的 source_id 取音频
                let song = library.get_song(&entity_id)
                    .ok_or_else(|| format!("歌曲 {entity_id} 不存在"))?;
                let local_sid = song.source_ids.iter().find(|s| s.source_type == SourceType::Local)
                    .ok_or_else(|| "该歌曲无可流式播放的本地来源".to_string())?;
                let bytes = src_resource::get_song_file(registrar, local_sid)?;
                Ok(serde_json::json!({ "bytes": B64.encode(&bytes) }))
            }
            Op::AlbumPictureGet { entity_id } => {
                let album = library.get_album(&entity_id)
                    .ok_or_else(|| format!("专辑 {entity_id} 不存在"))?;
                let local_sid = album.source_ids.iter().find(|s| s.source_type == SourceType::Local)
                    .ok_or_else(|| "该专辑无本地来源".to_string())?;
                let bytes = src_resource::get_album_picture(registrar, local_sid)?;
                Ok(serde_json::json!({ "bytes": B64.encode(&bytes) }))
            }
            Op::LyricTextGet { song_id } => {
                let sids = library.get_source_ids_of_song(&song_id);
                let local_sid = sids.iter().find(|s| s.source_type == SourceType::Local && s.entity_type == EntityType::Song)
                    .ok_or_else(|| "该歌曲无本地来源".to_string())?;
                let text = src_resource::get_lyric_text(registrar, local_sid)?;
                Ok(serde_json::json!({ "text": text }))
            }
            Op::AddSong { song } => {
                if offered_permission != Permission::Editable {
                    return Err("本机未开放可编辑权限".into());
                }
                let song: Song = serde_json::from_value(song).map_err(|e| e.to_string())?;
                let stored_id = library.add_song(&song)?;
                library.save_if_dirty()?;
                Ok(serde_json::json!({ "id": stored_id }))
            }
            Op::GetAllSongs => {
                let songs = library.get_all_songs();
                let v: Vec<Song> = songs.into_values().collect();
                serde_json::to_value(v).map_err(|e| e.to_string())
            }
        }
    }
}

// Frame tag 调试用
impl Frame {
    fn tag(&self) -> &'static str {
        match self {
            Frame::Hello { .. } => "hello",
            Frame::HelloAck { .. } => "hello_ack",
            Frame::MatchPending { .. } => "match_pending",
            Frame::MatchResult { .. } => "match_result",
            Frame::Query { .. } => "query",
            Frame::Response { .. } => "response",
        }
    }
}
