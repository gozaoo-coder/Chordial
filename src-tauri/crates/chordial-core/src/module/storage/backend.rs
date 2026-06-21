use serde_json::Value;
use std::collections::HashMap;

/// 存储后端抽象 trait。
///
/// 实现者只需关心如何将 `HashMap<String, Value>` 写入/读出存储介质。
/// 上层封装（[`super::persistent::PersistentStore`]、[`super::cache::CacheStore`]）
/// 在此基础上提供类型安全、TTL 过期等高级功能。
///
/// # 线程安全
///
/// 该 trait 要求 `Send + Sync`，实现者需自行保证内部并发安全。
///
/// # 实现指南
///
/// - `read()` 应返回当前后端中所有数据的完整快照。
/// - `write()` 应以全量数据替换后端中的现有内容。
pub trait StorageBackend: Send + Sync {
    /// 从后端读取全部数据。
    ///
    /// 如果后端中不存在任何数据（例如首次启动），返回空的 `HashMap`。
    fn read(&self) -> Result<HashMap<String, Value>, String>;

    /// 将全部数据写入后端。
    ///
    /// `data` 参数是内存中的完整数据集，应全量覆盖后端已有内容。
    fn write(&self, data: &HashMap<String, Value>) -> Result<(), String>;
}
