//! Tauri 命令层 — 将持久化存储封装为前端可调用的 API。
//!
//! 本模块向 Tauri 前端暴露以下命令：
//!
//! | 命令 | 功能 |
//! |------|------|
//! | `storage_get` | 读取键值 |
//! | `storage_set` | 写入键值（不立即落盘） |
//! | `storage_remove` | 删除键 |
//! | `storage_has` | 检查键是否存在 |
//! | `storage_keys` | 获取所有键 |
//! | `storage_clear` | 清空所有数据 |
//! | `storage_save` | 持久化到磁盘 |
//!
//! 命令前缀 `storage_` 避免与其他模块的命令冲突。
//!
//! # 持久化路径
//!
//! 配置文件存储在 `{系统配置目录}/chordial/storage.json`。
//! - Windows: `%APPDATA%/chordial/storage.json`
//! - macOS: `~/Library/Application Support/chordial/storage.json`
//! - Linux: `~/.config/chordial/storage.json`
//!
//! # 注册方式
//!
//! 在 `lib.rs` 中通过 `generate_handler![]` 注册：
//! ```ignore
//! .invoke_handler(tauri::generate_handler![
//!     crate::module::commands::storage_get,
//!     crate::module::commands::storage_set,
//!     ...
//! ])
//! ```

use super::storage::persistent::PersistentStore;
use serde_json::Value;
use std::path::PathBuf;
use std::sync::LazyLock;

/// 全局持久化存储单例。
///
/// 首次访问时从磁盘自动加载已有数据。
/// 所有 Tauri 命令共享此实例，通过内部 `RwLock` 保证线程安全。
static STORE: LazyLock<PersistentStore> = LazyLock::new(|| {
    let path = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("chordial")
        .join("storage.json");
    PersistentStore::new(path)
});

/// 读取指定 key 的值，返回 JSON。
///
/// 前端调用示例：
/// ```js
/// const volume = await invoke('storage_get', { key: 'volume' });
/// ```
#[tauri::command]
pub fn storage_get(key: String) -> Result<Value, String> {
    STORE
        .get_raw(&key)
        .ok_or_else(|| format!("键 '{}' 不存在", key))
}

/// 写入键值对到内存缓存，不立即落盘。
///
/// 如需持久化，调用 [`storage_save`]。
///
/// 前端调用示例：
/// ```js
/// await invoke('storage_set', { key: 'volume', value: 0.75 });
/// ```
#[tauri::command]
pub fn storage_set(key: String, value: Value) -> Result<(), String> {
    STORE.set_raw(&key, value);
    Ok(())
}

/// 删除指定 key，返回 `true` 表示 key 存在并被删除。
#[tauri::command]
pub fn storage_remove(key: String) -> Result<bool, String> {
    Ok(STORE.remove(&key))
}

/// 检查 key 是否存在。
#[tauri::command]
pub fn storage_has(key: String) -> Result<bool, String> {
    Ok(STORE.has(&key))
}

/// 返回所有 key 的列表。
#[tauri::command]
pub fn storage_keys() -> Result<Vec<String>, String> {
    Ok(STORE.keys())
}

/// 清空所有数据（仅修改内存缓存，不立即落盘）。
#[tauri::command]
pub fn storage_clear() -> Result<(), String> {
    STORE.clear();
    Ok(())
}

/// 将内存中的全部数据持久化到磁盘。
///
/// 建议在应用退出前或重要配置修改后调用。
#[tauri::command]
pub fn storage_save() -> Result<(), String> {
    STORE.save()
}
