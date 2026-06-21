use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// 数据存活时长。
///
/// 控制 [`CacheStore`](super::cache::CacheStore) 中条目的生命周期。
/// [`PersistentStore`](super::persistent::PersistentStore) 中的所有条目等效于 `Ttl::Forever`。
///
/// # 示例
///
/// ```ignore
/// use crate::module::storage::Ttl;
///
/// let ten_minutes = Ttl::DurationSecs(600);
/// let current_session = Ttl::Session;
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Ttl {
    /// 永久保留，仅在显式删除时移除。
    Forever,

    /// 自创建起经过指定秒数后自动过期。
    DurationSecs(u64),

    /// 当前进程会话结束后过期。
    /// 数据仅存在于内存中，应用重启即失效。
    Session,
}

/// 带时间戳元数据的存储条目。
///
/// 仅在内存中使用，不由后端持久化（`created_at` 和 `expires_at` 不会序列化到磁盘）。
///
/// # 字段
///
/// - `value`：实际存储的 JSON 值（会被持久化）。
/// - `created_at`：条目创建时间（内存中，不持久化）。
/// - `expires_at`：过期截止时间，`None` 表示永不过期（内存中，不持久化）。
#[derive(Debug, Clone, Serialize)]
pub struct StoredEntry {
    /// 实际存储的 JSON 值
    pub value: serde_json::Value,

    /// 创建时间（仅内存中有效，不持久化）
    #[serde(skip)]
    pub created_at: Instant,

    /// 过期截止时间，`None` 表示永不过期（仅内存中有效，不持久化）
    #[serde(skip)]
    pub expires_at: Option<Instant>,
}

impl StoredEntry {
    /// 创建新条目，根据 `ttl` 计算过期时间。
    ///
    /// - `Forever` → `expires_at: None`
    /// - `DurationSecs(n)` → `expires_at: now + n秒`
    /// - `Session` → `expires_at: None`（过期由进程退出控制）
    pub fn new(value: serde_json::Value, ttl: &Ttl) -> Self {
        let expires_at = match ttl {
            Ttl::Forever => None,
            Ttl::DurationSecs(secs) => {
                Some(Instant::now() + Duration::from_secs(*secs))
            }
            Ttl::Session => None,
        };
        Self {
            value,
            created_at: Instant::now(),
            expires_at,
        }
    }

    /// 检查条目是否已过期。
    ///
    /// 返回 `true` 表示该条目应当被清理。
    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(deadline) => Instant::now() >= deadline,
            None => false,
        }
    }

    /// 续期：按给定的 `ttl` 重置过期时间。
    ///
    /// 适用场景：短期内频繁访问的缓存条目，每次访问时续期以延长有效期。
    pub fn touch(&mut self, ttl: &Ttl) {
        self.expires_at = match ttl {
            Ttl::Forever => None,
            Ttl::DurationSecs(secs) => {
                Some(Instant::now() + Duration::from_secs(*secs))
            }
            Ttl::Session => None,
        };
        self.created_at = Instant::now();
    }
}
