//! 本地音乐来源模块 — 管理本地文件系统中的音乐。
//!
//! # 模块架构
//!
//! ```text
//! LocalMusicSource (source.rs)        ← MusicSource 实现
//!   ├── Scanner (scanner.rs)          ← symphonia 音频文件元数据提取
//!   ├── FolderManager (folder.rs)     ← 文件夹持久化 + 增删管理
//!   └── Watcher (watcher.rs)          ← notify 文件系统监听 + 增量同步
//! ```
//!
//! # 工作流程
//!
//! 1. **初始化**：`init_local_source()` 创建 `LocalMusicSource`，自动添加系统音乐目录，
//!    扫描已有文件并导入 `MusicLibrary`，注册为 must-source。
//! 2. **运行时**：用户通过 Tauri 命令 `local_add_folder` / `local_remove_folder` 管理文件夹；
//!    watcher 在后台监听文件变化，增量同步到音乐库。
//! 3. **资源获取**：前端通过 `get_song_file` / `get_album_picture` / `get_lyric_text`
//!    请求资源时，`LocalMusicSource` 直接从文件系统读取并返回。

pub mod folder;
pub mod scanner;
pub mod source;
pub mod watcher;

use crate::module::music_library::library::MusicLibrary;
use crate::module::music_source::registrar::SourceRegistrar;
use source::LocalMusicSource;
use std::path::PathBuf;
use std::sync::Arc;

/// 初始化本地音乐来源。
///
/// 执行以下操作：
/// 1. 创建 `FolderManager`（从持久化存储加载已有文件夹）
/// 2. 若文件夹列表为空（首次启动），自动添加系统音乐目录
/// 3. 创建 `LocalMusicSource`
/// 4. 从 MusicLibrary 恢复内存索引（跳过已索引文件的重扫描）
/// 5. 增量扫描文件夹，仅对不在索引中的新文件执行 symphonia 探测
/// 6. 将本地来源注册到注册器（must-source，不允许注销）
/// 7. 启动文件系统监听器（后台线程）
///
/// # 参数
/// - `folder_store_path`: 文件夹管理器的持久化存储路径
/// - `library`: 音乐库共享引用
/// - `registrar`: 来源注册器共享引用
///
/// # 返回
/// 成功时返回 `Arc<LocalMusicSource>`，失败时返回错误信息。
pub fn init_local_source(
    folder_store_path: PathBuf,
    library: Arc<MusicLibrary>,
    registrar: &SourceRegistrar,
) -> Result<Arc<LocalMusicSource>, String> {
    use crate::module::storage::persistent::PersistentStore;
    use folder::FolderManager;

    // 1. 创建文件夹管理器
    let folder_store = PersistentStore::new(folder_store_path);
    let folder_manager = Arc::new(FolderManager::new(folder_store));

    // 2. 首次启动：自动添加系统音乐目录
    if folder_manager.count() == 0 {
        if let Some(audio_dir) = dirs::audio_dir() {
            if audio_dir.exists() {
                let _ = folder_manager.add_folder(&audio_dir);
            }
        }
    }

    // 3. 创建本地来源
    let local_source = Arc::new(LocalMusicSource::new(
        folder_manager.clone(),
        library.clone(),
    ));

    // 4. 从持久化库恢复内存索引（跳过已索引文件的重扫描）
    let (restored, cleaned) = local_source.restore_index_from_library();
    if cleaned > 0 {
        // 有文件已被删除，落盘清理结果
        let _ = library.save();
    }

    // 5. 增量扫描：仅探测不在索引中的新文件
    let folders = folder_manager.get_folders();
    let mut indexed = 0usize;
    for folder in &folders {
        let files = folder::collect_audio_files(folder);
        for file in &files {
            // 已在索引中的文件会被 index_file 跳过（O(1) HashMap 查询）
            match local_source.index_file(file) {
                Ok(true) => indexed += 1,
                Ok(false) => {} // 非音频文件或已索引
                Err(e) => {
                    eprintln!("[local_source] 索引文件失败 '{}': {}", file.display(), e);
                }
            }
        }
    }

    // 仅当有新文件时才落盘
    if indexed > 0 {
        library.save()?;
    }

    // 6. 注册为 must-source
    registrar
        .register(local_source.clone())
        .map_err(|e| format!("注册本地来源失败: {}", e))?;

    // 7. 启动后台文件监听器
    let watcher_source = local_source.clone();
    let watcher_folders = folders;
    std::thread::Builder::new()
        .name("local-source-watcher".into())
        .spawn(move || {
            if let Err(e) = watcher::start_watcher(watcher_source, watcher_folders) {
                eprintln!("[local_source] 文件监听器退出: {}", e);
            }
        })
        .map_err(|e| format!("启动文件监听线程失败: {}", e))?;

    eprintln!(
        "[local_source] 本地来源初始化完成: {} 个文件夹, {} 首已恢复, {} 首新索引",
        folder_manager.count(),
        restored,
        indexed
    );

    Ok(local_source)
}
