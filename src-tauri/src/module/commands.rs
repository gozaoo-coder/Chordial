//! Tauri 命令层 — 将 Config / Storage / Cache 三层存储封装为前端可调用的 API。
//!
//! # 命令一览
//!
//! ## 配置（config_*）— 自动防抖落盘，适合音量、主题等设置
//!
//! | 命令 | 功能 |
//! |------|------|
//! | `config_get` | 读取配置项 |
//! | `config_set` | 写入配置项（自动防抖落盘） |
//! | `config_remove` | 删除配置项 |
//! | `config_has` | 检查配置项是否存在 |
//! | `config_keys` | 获取所有配置键 |
//! | `config_clear` | 清空所有配置 |
//! | `config_flush` | 立即落盘（跳过防抖） |
//! | `config_reload` | 从磁盘重新加载 |
//!
//! ## 持久化存储（storage_*）— 手动落盘，适合播放列表、乐库索引
//!
//! | 命令 | 功能 |
//! |------|------|
//! | `storage_get` | 读取键值 |
//! | `storage_set` | 写入键值（不落盘） |
//! | `storage_remove` | 删除键 |
//! | `storage_has` | 检查键是否存在 |
//! | `storage_keys` | 获取所有键 |
//! | `storage_clear` | 清空所有数据 |
//! | `storage_save` | 手动持久化到磁盘 |
//!
//! ## 缓存（cache_*）— 纯内存，支持 TTL 过期
//!
//! | 命令 | 功能 |
//! |------|------|
//! | `cache_get` | 读取缓存值（自动检查过期） |
//! | `cache_set` | 写入缓存值并指定 TTL |
//! | `cache_remove` | 删除缓存项 |
//! | `cache_has` | 检查缓存项是否存在 |
//! | `cache_keys` | 获取所有未过期的键 |
//! | `cache_clear` | 清空所有缓存 |
//! | `cache_clear_expired` | 清理过期条目 |
//! | `cache_touch` | 续期缓存项 |
//!
//! # 数据文件
//!
//! | 存储层 | 文件路径 |
//! |--------|---------|
//! | Config | `{config_dir}/chordial/config.json` |
//! | Storage | `{config_dir}/chordial/storage.json` |
//! | Cache | 无（纯内存） |
//!
//! - Windows: `%APPDATA%/chordial/`
//! - macOS: `~/Library/Application Support/chordial/`
//! - Linux: `~/.config/chordial/`

use crate::module::cache::store::CacheStore;
use crate::module::config::store::ConfigStore;
use crate::module::storage::entry::Ttl;
use crate::module::storage::persistent::PersistentStore;
use serde::Deserialize;
use serde_json::Value;
use std::path::PathBuf;
use std::sync::LazyLock;

// ══════════════════════════════════════════════════════════════════════════════
// 全局单例
// ══════════════════════════════════════════════════════════════════════════════

/// 获取 chordial 配置目录路径
fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("chordial")
}

/// 全局配置存储 — 修改后自动防抖落盘
static CONFIG: LazyLock<ConfigStore> = LazyLock::new(|| {
    let path = config_dir().join("config.json");
    ConfigStore::new(path)
});

/// 全局持久化存储 — 手动调用 save() 落盘
static STORE: LazyLock<PersistentStore> = LazyLock::new(|| {
    let path = config_dir().join("storage.json");
    PersistentStore::new(path)
});

/// 全局缓存存储 — 纯内存，支持 TTL
static CACHE: LazyLock<CacheStore> = LazyLock::new(CacheStore::new);

// ══════════════════════════════════════════════════════════════════════════════
// TTL 参数辅助类型
// ══════════════════════════════════════════════════════════════════════════════

/// 前端传入的 TTL 参数。
///
/// 支持三种格式（`serde_json` 可自动识别）：
/// - `"forever"` → [`Ttl::Forever`]
/// - `"session"` → [`Ttl::Session`]
/// - `{"duration_secs": 600}` → [`Ttl::DurationSecs(600)`]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum TtlArg {
    Forever,
    Session,
    DurationSecs(u64),
}

impl From<TtlArg> for Ttl {
    fn from(arg: TtlArg) -> Self {
        match arg {
            TtlArg::Forever => Ttl::Forever,
            TtlArg::Session => Ttl::Session,
            TtlArg::DurationSecs(n) => Ttl::DurationSecs(n),
        }
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Config 命令 — 自动防抖落盘
// ══════════════════════════════════════════════════════════════════════════════

/// 读取配置项的值。
#[tauri::command]
pub fn config_get(key: String) -> Result<Value, String> {
    CONFIG
        .get_raw(&key)
        .ok_or_else(|| format!("配置项 '{}' 不存在", key))
}

/// 写入配置项，自动防抖落盘（500ms 内连续修改合并为一次写盘）。
#[tauri::command]
pub fn config_set(key: String, value: Value) -> Result<(), String> {
    CONFIG.set_raw(&key, value);
    Ok(())
}

/// 删除配置项，自动防抖落盘。
#[tauri::command]
pub fn config_remove(key: String) -> Result<bool, String> {
    Ok(CONFIG.remove(&key))
}

/// 检查配置项是否存在。
#[tauri::command]
pub fn config_has(key: String) -> Result<bool, String> {
    Ok(CONFIG.has(&key))
}

/// 获取所有配置项的键。
#[tauri::command]
pub fn config_keys() -> Result<Vec<String>, String> {
    Ok(CONFIG.keys())
}

/// 清空所有配置，自动防抖落盘。
#[tauri::command]
pub fn config_clear() -> Result<(), String> {
    CONFIG.clear();
    Ok(())
}

/// 立即同步落盘，跳过防抖定时器。
///
/// 适合在应用退出前调用，确保所有配置修改已持久化。
#[tauri::command]
pub fn config_flush() -> Result<(), String> {
    CONFIG.flush()
}

/// 从磁盘重新加载配置，**丢弃内存中所有未落盘的修改**。
#[tauri::command]
pub fn config_reload() -> Result<(), String> {
    CONFIG.reload();
    Ok(())
}

// ══════════════════════════════════════════════════════════════════════════════
// Storage 命令 — 手动落盘
// ══════════════════════════════════════════════════════════════════════════════

/// 读取指定 key 的值。
#[tauri::command]
pub fn storage_get(key: String) -> Result<Value, String> {
    STORE
        .get_raw(&key)
        .ok_or_else(|| format!("键 '{}' 不存在", key))
}

/// 写入键值对到内存缓存，不立即落盘。
///
/// 如需持久化，调用 [`storage_save`]。
#[tauri::command]
pub fn storage_set(key: String, value: Value) -> Result<(), String> {
    STORE.set_raw(&key, value);
    Ok(())
}

/// 删除指定 key。
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
/// 建议在应用退出前或重要数据修改后调用。
#[tauri::command]
pub fn storage_save() -> Result<(), String> {
    STORE.save()
}

// ══════════════════════════════════════════════════════════════════════════════
// Cache 命令 — 纯内存，TTL 过期
// ══════════════════════════════════════════════════════════════════════════════

/// 读取缓存值，自动检查过期。
#[tauri::command]
pub fn cache_get(key: String) -> Result<Value, String> {
    CACHE
        .get_raw(&key)
        .ok_or_else(|| format!("缓存项 '{}' 不存在或已过期", key))
}

/// 写入缓存值，指定 TTL 策略。
///
/// `ttl` 参数格式：
/// - `"forever"` — 永不过期
/// - `"session"` — 当前进程生命周期
/// - `{"duration_secs": 600}` — 600 秒后过期
///
/// 前端调用示例：
/// ```js
/// await invoke('cache_set', { key: 'recent', value: data, ttl: { duration_secs: 600 } });
/// await invoke('cache_set', { key: 'state', value: data, ttl: 'forever' });
/// ```
#[tauri::command]
pub fn cache_set(key: String, value: Value, ttl: TtlArg) -> Result<(), String> {
    CACHE.set_raw(&key, value, &ttl.into());
    Ok(())
}

/// 删除缓存项。
#[tauri::command]
pub fn cache_remove(key: String) -> Result<bool, String> {
    Ok(CACHE.remove(&key))
}

/// 检查缓存项是否存在且未过期。
#[tauri::command]
pub fn cache_has(key: String) -> Result<bool, String> {
    Ok(CACHE.has(&key))
}

/// 获取所有未过期的缓存键。
#[tauri::command]
pub fn cache_keys() -> Result<Vec<String>, String> {
    Ok(CACHE.keys())
}

/// 清空所有缓存（含未过期条目）。
#[tauri::command]
pub fn cache_clear() -> Result<(), String> {
    CACHE.clear();
    Ok(())
}

/// 清理所有已过期条目，返回清理数量。
#[tauri::command]
pub fn cache_clear_expired() -> Result<usize, String> {
    Ok(CACHE.clear_expired())
}

/// 续期缓存项：按给定 TTL 重置过期倒计时。
///
/// `ttl` 参数格式同 [`cache_set`]。
/// 返回 `true` 表示键存在且续期成功。
#[tauri::command]
pub fn cache_touch(key: String, ttl: TtlArg) -> Result<bool, String> {
    Ok(CACHE.touch(&key, &ttl.into()))
}
