use super::backend::StorageBackend;
use parking_lot::RwLock;
use serde_json::Value;
use std::collections::HashMap;

/// 内存存储后端。
///
/// 数据保存在进程内存中，通过 `RwLock` 保证并发安全。
/// 进程退出后数据丢失——适合缓存、临时状态等场景。
///
/// # 并发
///
/// 内部使用 `parking_lot::RwLock`，允许多读单写。
pub struct MemoryBackend {
    data: RwLock<HashMap<String, Value>>,
}

impl MemoryBackend {
    /// 创建空的内存后端。
    pub fn new() -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for MemoryBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl StorageBackend for MemoryBackend {
    fn read(&self) -> Result<HashMap<String, Value>, String> {
        Ok(self.data.read().clone())
    }

    fn write(&self, data: &HashMap<String, Value>) -> Result<(), String> {
        *self.data.write() = data.clone();
        Ok(())
    }
}
