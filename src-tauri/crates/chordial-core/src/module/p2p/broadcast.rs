//! UDP 信标广播 — 局域网内 Chordial 实例互相发现。
//!
//! 当用户开启广播模式时：
//! - **发送方**：每 5 秒向 `255.255.255.255:58009` 发送一个 [`Beacon`] JSON
//! - **接收方**：监听 `0.0.0.0:58009`，缓存最近 30 秒内收到的对端信标
//!
//! 前端通过 [`BroadcastDiscovery::list_discovered`] 拿到对端列表后，
//! 可点击某条目触发 `request_match(addr, match_code)` 主动握手。

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio::sync::Notify;

use super::protocol::BROADCAST_PORT;

/// 单条信标。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Beacon {
    /// 服务端显示名（取本机 hostname 或用户自定义）
    pub name: String,
    /// TCP 监听端口
    pub port: u16,
    /// 当前匹配码
    pub match_code: String,
    /// 持久实例 ID（用于可信设备识别）
    #[serde(default)]
    pub instance_id: Option<String>,
}

/// 已发现的设备。
#[derive(Debug, Clone, Serialize)]
pub struct DiscoveredPeer {
    pub name: String,
    pub addr: String,
    pub port: u16,
    pub match_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
    pub last_seen_secs: u64,
}

const BEACON_INTERVAL: Duration = Duration::from_secs(5);
const EXPIRY: Duration = Duration::from_secs(30);
const MAX_PACKET: usize = 1024;

/// 广播发现管理器。
pub struct BroadcastDiscovery {
    inner: Arc<Inner>,
    send_notify: Arc<Notify>,
}

struct Inner {
    enabled: RwLock<bool>,
    beacon: RwLock<Option<Beacon>>,
    discovered: RwLock<HashMap<SocketAddrV4, (Beacon, Instant)>>,
}

impl BroadcastDiscovery {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Inner {
                enabled: RwLock::new(false),
                beacon: RwLock::new(None),
                discovered: RwLock::new(HashMap::new()),
            }),
            send_notify: Arc::new(Notify::new()),
        }
    }

    /// 启用广播：设置本机信标并启动发送 / 接收任务。
    pub async fn enable(&self, beacon: Beacon) -> io::Result<()> {
        *self.inner.beacon.write() = Some(beacon);
        *self.inner.enabled.write() = true;
        self.send_notify.notify_waiters();

        let inner = self.inner.clone();
        let send_notify = self.send_notify.clone();
        // 发送任务
        tokio::spawn(async move {
            let sock = match UdpSocket::bind("0.0.0.0:0").await {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("[p2p:broadcast] 绑定发送 socket 失败: {e}");
                    return;
                }
            };
            let _ = sock.set_broadcast(true);
            let bcast_addr = SocketAddrV4::new(Ipv4Addr::BROADCAST, BROADCAST_PORT);

            loop {
                let enabled = *inner.enabled.read();
                if !enabled {
                    send_notify.notified().await;
                    continue;
                }
                let beacon_opt = inner.beacon.read().clone();
                if let Some(beacon) = beacon_opt {
                    if let Ok(json) = serde_json::to_vec(&beacon) {
                        if json.len() <= MAX_PACKET {
                            let _ = sock.send_to(&json, bcast_addr).await;
                        }
                    }
                }
                tokio::time::sleep(BEACON_INTERVAL).await;
            }
        });

        // 接收任务
        let inner = self.inner.clone();
        tokio::spawn(async move {
            let sock = match UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, BROADCAST_PORT)).await {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("[p2p:broadcast] 绑定接收 socket 失败: {e}（可能已有实例在运行）");
                    return;
                }
            };
            let mut buf = [0u8; MAX_PACKET];
            loop {
                match sock.recv_from(&mut buf).await {
                    Ok((n, SocketAddr::V4(from))) => {
                        if let Ok(beacon) = serde_json::from_slice::<Beacon>(&buf[..n]) {
                            // 跳过自己（端口相同时可能是回环）
                            let self_port = inner.beacon.read().as_ref().map(|b| b.port);
                            if self_port == Some(beacon.port) && from.ip().is_loopback() {
                                continue;
                            }
                            inner
                                .discovered
                                .write()
                                .insert(from, (beacon, Instant::now()));
                        }
                    }
                    _ => {}
                }
                // 顺便清理过期条目
                let now = Instant::now();
                inner.discovered.write().retain(|_, (_, seen)| now.duration_since(*seen) < EXPIRY);
            }
        });

        Ok(())
    }

    /// 关闭广播。
    pub fn disable(&self) {
        *self.inner.enabled.write() = false;
        *self.inner.beacon.write() = None;
        self.inner.discovered.write().clear();
    }

    /// 更新本机信标（匹配码或名称变更时）。
    pub fn update_beacon(&self, beacon: Beacon) {
        *self.inner.beacon.write() = Some(beacon);
    }

    /// 当前是否启用。
    pub fn is_enabled(&self) -> bool {
        *self.inner.enabled.read()
    }

    /// 列出已发现的对端。
    pub fn list_discovered(&self) -> Vec<DiscoveredPeer> {
        let now = Instant::now();
        self.inner
            .discovered
            .read()
            .iter()
            .filter_map(|(ip, (beacon, seen))| {
                if now.duration_since(*seen) > EXPIRY {
                    return None;
                }
                Some(DiscoveredPeer {
                    name: beacon.name.clone(),
                    addr: ip.to_string(),
                    port: beacon.port,
                    match_code: beacon.match_code.clone(),
                    instance_id: beacon.instance_id.clone(),
                    last_seen_secs: now.duration_since(*seen).as_secs(),
                })
            })
            .collect()
    }
}
