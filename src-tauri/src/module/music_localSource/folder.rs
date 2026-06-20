//! 文件夹管理 — 持久化监听文件夹列表。
//!
//! [`FolderManager`] 管理用户自定义的音乐文件夹：
//! - 持久化文件夹路径列表到 `PersistentStore`
//! - 提供增删查接口
//! - 在添加新文件夹时自动扫描已有文件
//! - 在移除文件夹时级联清理音乐库
//!
//! ## 跨平台
//!
//! 通过 [`crate::module::platform::PlatformPath`] 适配不同平台：
//! - 桌面端：`PathBuf` → `std::fs`
//! - Android：`String`（content URI）→ JNI 桥接

use crate::module::platform::{self, PlatformPath};
use crate::module::storage::persistent::PersistentStore;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

/// 持久化的文件夹条目。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FolderEntry {
    /// 用户添加的原始路径
    pub path: String,
}

/// 文件夹管理器。
///
/// 负责音乐文件夹路径的持久化存储和运行时访问。
pub struct FolderManager {
    /// 持久化存储
    store: PersistentStore,
    /// 运行时文件夹集合（规范路径）
    folders: RwLock<Vec<PlatformPath>>,
}

impl FolderManager {
    const KEY: &str = "local_source_folders";

    /// 创建文件夹管理器，从持久化存储加载已有文件夹列表。
    pub fn new(store: PersistentStore) -> Self {
        let entries: Vec<FolderEntry> = store
            .get::<Vec<FolderEntry>>(Self::KEY)
            .unwrap_or_default();

        let folders: Vec<PlatformPath> = entries
            .iter()
            .map(|e| PlatformPath::from(e.path.as_str()))
            .filter(|p| platform::exists(p))
            .collect();

        Self {
            store,
            folders: RwLock::new(folders),
        }
    }

    // ── 查询 ──────────────────────────────────────────

    /// 获取所有监听文件夹的路径。
    pub fn get_folders(&self) -> Vec<PlatformPath> {
        self.folders.read().clone()
    }

    /// 检查文件夹是否已在监听列表中。
    pub fn has_folder(&self, path: &PlatformPath) -> bool {
        let canonical = platform::canonicalize(path)
            .unwrap_or_else(|_| path.clone());
        self.folders
            .read()
            .iter()
            .any(|f| {
                let f_canon = platform::canonicalize(f)
                    .unwrap_or_else(|_| f.clone());
                f_canon == canonical
            })
    }

    /// 获取文件夹数量。
    pub fn count(&self) -> usize {
        self.folders.read().len()
    }

    // ── 修改 ──────────────────────────────────────────

    /// 添加一个音乐文件夹。
    ///
    /// 若文件夹不存在则返回 `Err`。若已存在则跳过。
    pub fn add_folder(&self, path: &PlatformPath) -> Result<(), String> {
        if !platform::exists(path) {
            return Err(format!("文件夹不存在: {}", platform::path_to_string(path)));
        }
        if !platform::is_dir(path) {
            return Err(format!("路径不是文件夹: {}", platform::path_to_string(path)));
        }

        let canonical = platform::canonicalize(path).map_err(|e| {
            format!("无法解析路径 '{}': {}", platform::path_to_string(path), e)
        })?;

        let mut folders = self.folders.write();
        if folders.iter().any(|f| {
            let f_canon = platform::canonicalize(f)
                .unwrap_or_else(|_| f.clone());
            f_canon == canonical
        }) {
            return Ok(()); // 已存在，跳过
        }

        folders.push(canonical);
        drop(folders);

        self.save()
    }

    /// 移除一个音乐文件夹。
    ///
    /// 返回 `true` 表示文件夹存在并被移除。
    /// 注意：此方法仅从管理列表中移除，不负责级联清理音乐库数据。
    /// 调用者应在调用此方法之前或之后自行清理。
    pub fn remove_folder(&self, path: &PlatformPath) -> bool {
        let canonical = platform::canonicalize(path)
            .unwrap_or_else(|_| path.clone());
        let mut folders = self.folders.write();
        let len_before = folders.len();
        folders.retain(|f| {
            let f_canon = platform::canonicalize(f)
                .unwrap_or_else(|_| f.clone());
            f_canon != canonical
        });
        let removed = folders.len() < len_before;
        drop(folders);

        if removed {
            let _ = self.save();
        }
        removed
    }

    // ── 持久化 ───────────────────────────────────────

    /// 保存当前文件夹列表到磁盘。
    fn save(&self) -> Result<(), String> {
        let entries: Vec<FolderEntry> = self
            .folders
            .read()
            .iter()
            .map(|p| FolderEntry {
                path: platform::path_to_string(p),
            })
            .collect();
        self.store.set(Self::KEY, &entries)?;
        self.store.save()
    }

    /// 返回文件夹管理器的持久化存储引用。
    pub fn store(&self) -> &PersistentStore {
        &self.store
    }
}

/// 递归收集文件夹下所有受支持的音频文件。
///
/// 遍历 `root` 目录及其所有子目录，返回所有扩展名匹配的音频文件路径。
pub fn collect_audio_files(root: &PlatformPath) -> Vec<PlatformPath> {
    let mut files = Vec::new();
    collect_audio_files_recursive(root, &mut files);
    files
}

fn collect_audio_files_recursive(dir: &PlatformPath, files: &mut Vec<PlatformPath>) {
    if let Ok(entries) = platform::read_dir_entries(dir) {
        for entry in entries {
            if platform::is_dir(&entry) {
                collect_audio_files_recursive(&entry, files);
            } else if super::scanner::is_supported_audio(&entry) {
                files.push(entry);
            }
        }
    }
}
