use super::backend::StorageBackend;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// JSON 文件存储后端。
///
/// 将数据以美化 JSON 格式持久化到指定文件路径。
/// 读取时若文件不存在或为空，返回空的 `HashMap`。
///
/// # 线程安全
///
/// 不内置锁——上层封装（[`PersistentStore`](super::persistent::PersistentStore)）
/// 负责通过 `RwLock` 保护并发访问。
#[derive(Clone)]
pub struct FileBackend {
    /// JSON 文件的完整路径
    path: PathBuf,
}

impl FileBackend {
    /// 创建文件后端。
    ///
    /// # 参数
    ///
    /// - `path`：JSON 文件的完整路径。若文件不存在，首次 `write()` 时自动创建。
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// 确保文件所在目录存在，不存在则递归创建。
    fn ensure_dir(&self) -> Result<(), String> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("创建目录失败: {}", e))
        } else {
            Ok(())
        }
    }
}

impl StorageBackend for FileBackend {
    fn read(&self) -> Result<HashMap<String, Value>, String> {
        if !self.path.exists() {
            return Ok(HashMap::new());
        }
        let content = fs::read_to_string(&self.path)
            .map_err(|e| format!("读取文件失败: {}", e))?;
        if content.trim().is_empty() {
            return Ok(HashMap::new());
        }
        serde_json::from_str(&content)
            .map_err(|e| format!("解析 JSON 失败: {}", e))
    }

    fn write(&self, data: &HashMap<String, Value>) -> Result<(), String> {
        self.ensure_dir()?;
        let content = serde_json::to_string_pretty(data)
            .map_err(|e| format!("序列化失败: {}", e))?;
        fs::write(&self.path, &content)
            .map_err(|e| format!("写文件失败: {}", e))
    }
}
