use crate::module::storage::backend::StorageBackend;
use crate::module::storage::file::FileBackend;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// 应用配置存储，修改后自动防抖落盘。
///
/// # 工作原理
///
/// 1. 内存中维护一份缓存（`RwLock<HashMap>`），读写不触碰磁盘。
/// 2. 每次 `set()` / `remove()` / `clear()` 会触发一个 500ms 的防抖定时器。
/// 3. 500ms 内无新修改时，后台线程自动将全量数据写入磁盘。
/// 4. 调用 [`flush()`](Self::flush) 可立即同步落盘（跳过防抖）。
///
/// # 线程安全
///
/// 读写操作通过 `parking_lot::RwLock` 保护，可在多线程中共享。
/// 落盘在独立的后台线程中执行，不阻塞调用者。
///
/// # 示例
///
/// ```ignore
/// let config = ConfigStore::new(path);
/// config.set("theme", &"dark")?;
/// // 500ms 后自动写入 config.json
/// ```
pub struct ConfigStore {
    /// 防抖落盘后台线程所使用的后端副本
    backend: FileBackend,
    /// 与后台线程共享的内部状态
    inner: Arc<Inner>,
}

/// 在 ConfigStore 和后台线程间共享的内部状态
struct Inner {
    /// 内存缓存（通过 Arc 与后台线程共享）
    cache: Arc<RwLock<HashMap<String, Value>>>,
    /// 防抖触发器——每收到一个 () 就重置 500ms 倒计时
    debounce_tx: mpsc::Sender<()>,
}

impl ConfigStore {
    /// 防抖窗口时长（毫秒）
    const DEBOUNCE_MS: u64 = 500;

    /// 创建配置存储实例。
    ///
    /// 启动时自动从 `path` 加载已有数据，若文件不存在则初始化为空。
    /// 同时启动一个后台线程负责防抖落盘。
    pub fn new(path: PathBuf) -> Self {
        let backend = FileBackend::new(path);
        let cache_data = backend.read().unwrap_or_default();

        let (debounce_tx, rx) = mpsc::channel();
        let backend_for_thread = backend.clone();
        let cache = Arc::new(RwLock::new(cache_data));
        let cache_for_thread = Arc::clone(&cache);

        // 后台防抖落盘线程
        thread::Builder::new()
            .name("config-debounce".into())
            .spawn(move || {
                while let Ok(()) = rx.recv() {
                    // 防抖循环：连续触发时不断重置 500ms 窗口
                    while rx
                        .recv_timeout(Duration::from_millis(Self::DEBOUNCE_MS))
                        .is_ok()
                    {}
                    let snapshot = cache_for_thread.read().clone();
                    let _ = backend_for_thread.write(&snapshot);
                }
            })
            .ok();

        Self {
            backend,
            inner: Arc::new(Inner { cache, debounce_tx }),
        }
    }

    // ── 读取 ─────────────────────────────────────────

    /// 读取值并反序列化为目标类型 `T`。
    ///
    /// key 不存在或类型不匹配时返回 `None`。
    pub fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        self.inner
            .cache
            .read()
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// 读取原始 [`Value`]，不做反序列化。
    pub fn get_raw(&self, key: &str) -> Option<Value> {
        self.inner.cache.read().get(key).cloned()
    }

    // ── 写入 ─────────────────────────────────────────

    /// 写入键值对，触发防抖落盘。
    ///
    /// 修改立即在内存中生效；500ms 内无新修改后自动写入磁盘。
    pub fn set<T: Serialize>(&self, key: &str, value: &T) -> Result<(), String> {
        let json = serde_json::to_value(value).map_err(|e| format!("序列化失败: {}", e))?;
        self.set_raw(key, json);
        Ok(())
    }

    /// 写入原始 JSON 值，触发防抖落盘。
    pub fn set_raw(&self, key: &str, value: Value) {
        self.inner.cache.write().insert(key.to_string(), value);
        let _ = self.inner.debounce_tx.send(());
    }

    // ── 删除 / 检查 ──────────────────────────────────

    /// 删除指定 key，触发防抖落盘。
    ///
    /// 返回 `true` 表示 key 存在并被删除。
    pub fn remove(&self, key: &str) -> bool {
        let existed = self.inner.cache.write().remove(key).is_some();
        if existed {
            let _ = self.inner.debounce_tx.send(());
        }
        existed
    }

    /// 检查 key 是否存在。
    pub fn has(&self, key: &str) -> bool {
        self.inner.cache.read().contains_key(key)
    }

    /// 获取所有已存储 key 的列表。
    pub fn keys(&self) -> Vec<String> {
        self.inner.cache.read().keys().cloned().collect()
    }

    /// 清空所有数据，触发防抖落盘。
    pub fn clear(&self) {
        self.inner.cache.write().clear();
        let _ = self.inner.debounce_tx.send(());
    }

    // ── 持久化控制 ───────────────────────────────────

    /// 立即同步落盘，跳过防抖定时器。
    ///
    /// 适合在应用退出前调用，确保所有修改已持久化。
    ///
    /// 优化：在读锁内序列化为字符串，释放锁后写盘，
    /// 避免 `cache.read().clone()` 全量 HashMap clone。
    pub fn flush(&self) -> Result<(), String> {
        let content = {
            let guard = self.inner.cache.read();
            serde_json::to_string(&*guard).map_err(|e| format!("序列化失败: {}", e))?
        };
        self.backend.write_str(&content)
    }

    /// 从磁盘重新加载数据，**丢弃内存中所有未落盘的修改**。
    pub fn reload(&self) {
        if let Ok(data) = self.backend.read() {
            *self.inner.cache.write() = data;
        }
    }
}
