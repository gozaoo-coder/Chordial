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
#[cfg(not(target_os = "android"))]
pub mod watcher;

use crate::module::music_library::library::MusicLibrary;
use crate::module::music_library::models::Song;
use crate::module::music_source::registrar::SourceRegistrar;
use crate::module::platform::PlatformPath;
use source::LocalMusicSource;
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
/// 7. 启动文件系统监听器（后台线程，桌面端）
///
/// # 参数
/// - `folder_store_path`: 文件夹管理器的持久化存储路径
/// - `library`: 音乐库共享引用
/// - `registrar`: 来源注册器共享引用
///
/// # 返回
/// 成功时返回 `Arc<LocalMusicSource>`，失败时返回错误信息。
pub fn init_local_source(
    folder_store_path: std::path::PathBuf,
    library: Arc<MusicLibrary>,
    registrar: &SourceRegistrar,
) -> Result<Arc<LocalMusicSource>, String> {
    use crate::module::storage::persistent::PersistentStore;
    use folder::FolderManager;
    use std::time::Instant;

    let t0 = Instant::now();

    // 1. 创建文件夹管理器
    let folder_store = PersistentStore::new(folder_store_path);
    let folder_manager = Arc::new(FolderManager::new(folder_store));
    let t1 = Instant::now();
    eprintln!("[local_source] ⏱ 1. 文件夹管理器创建: {:?}", t1 - t0);

    // 2. 首次启动：自动添加系统音乐目录
    if folder_manager.count() == 0 {
        if let Some(audio_dir) = dirs::audio_dir() {
            if audio_dir.exists() {
                let platform_path = PlatformPath::from(audio_dir.to_string_lossy().as_ref());
                let _ = folder_manager.add_folder(&platform_path);
            }
        }
    }
    let t2 = Instant::now();
    eprintln!("[local_source] ⏱ 2. 系统音乐目录检查: {:?}", t2 - t1);

    // 3. 创建本地来源（使用独立的 mtime 存储，与 library 分离）
    let mtime_store_path = library
        .store_path()
        .parent()
        .map(|p| p.join("local_source_file_mtimes.json"))
        .unwrap_or_else(|| std::path::PathBuf::from("local_source_file_mtimes.json"));
    let mtime_store = PersistentStore::new(mtime_store_path);
    // 迁移：清理 library 中遗留的旧 mtime 数据（现使用独立文件存储）
    library.remove_store_key("local_source_file_mtimes");
    let local_source = Arc::new(LocalMusicSource::new(
        folder_manager.clone(),
        library.clone(),
        mtime_store,
    ));
    let t3 = Instant::now();
    eprintln!("[local_source] ⏱ 3. LocalMusicSource 创建: {:?}", t3 - t2);

    // 4. 从持久化库恢复内存索引（跳过已索引文件的重扫描）
    let (restored, _cleaned) = local_source.restore_index_from_library();
    let t4 = Instant::now();
    eprintln!(
        "[local_source] 库中歌曲总数（恢复后）: {}, ⏱ 4. 索引恢复: {:?}",
        library.song_count(),
        t4 - t3
    );
    // (remove_specific_song_source_ids 已在 restore_index_from_library 内部调用 save)

    // 5. 增量扫描：mtime 缓存跳过未变化文件 + 并行探测新文件
    let folders = folder_manager.get_folders();
    let mut skipped = 0usize;
    let mut needs_probe: Vec<PlatformPath> = Vec::new();

    // 5a. 收集所有音频文件 → 并行 canonicalize → 分类
    let mut all_files: Vec<PlatformPath> = Vec::new();
    for folder in &folders {
        let _t_dir = Instant::now();
        let files = folder::collect_audio_files(folder);
        let t_dir = _t_dir.elapsed();
        eprintln!(
            "[local_source] ⏱ 5a-dir '{}': {} 个音频文件, {:?}",
            crate::module::platform::path_to_string(folder),
            files.len(),
            t_dir
        );
        all_files.extend(files);
    }

    let canonicalize_count = all_files.len();
    // 并行 canonicalize
    let canonicalized: Vec<(PlatformPath, Option<PlatformPath>)> = if canonicalize_count > 0 {
        let num_threads = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
            .min(canonicalize_count);
        let chunk_size = (canonicalize_count + num_threads - 1) / num_threads;
        let mut results = Vec::with_capacity(canonicalize_count);

        std::thread::scope(|s| {
            let mut handles = Vec::new();
            for chunk in all_files.chunks(chunk_size) {
                let chunk: Vec<PlatformPath> = chunk.to_vec();
                handles.push(s.spawn(move || {
                    chunk
                        .into_iter()
                        .map(|file| {
                            let canonical = crate::module::platform::canonicalize(&file)
                                .ok();
                            (file, canonical)
                        })
                        .collect::<Vec<_>>()
                }));
            }
            for handle in handles {
                if let Ok(chunk_results) = handle.join() {
                    results.extend(chunk_results);
                }
            }
        });
        results
    } else {
        Vec::new()
    };

    // 串行分类（利用已规范化的路径）
    for (original, canon_opt) in &canonicalized {
        let canonical = canon_opt.clone().unwrap_or_else(|| original.clone());

        // 已在 file_index 中 → 跳过
        if local_source.file_index.read().contains_key(&canonical) {
            skipped += 1;
            continue;
        }

        // mtime 缓存命中（文件未变化）→ 从缓存重建索引
        if let Some(song_id) = local_source.check_file_unchanged(&canonical) {
            local_source
                .file_index
                .write()
                .insert(canonical.clone(), song_id.clone());
            local_source
                .id_to_path
                .write()
                .insert(song_id, canonical);
            skipped += 1;
            continue;
        }

        // 需要探测
        needs_probe.push(canonical);
    }

    let t5a = Instant::now();
    eprintln!(
        "[local_source] ⏱ 5a. 目录遍历+并行canonicalize+分类: {:?} ({} 个文件, {} 个线程, 已跳过 {}, 待探测 {})",
        t5a - t4,
        canonicalize_count,
        1, // already computed above but simplified for log
        skipped,
        needs_probe.len()
    );

    // 5b. 并行探测新文件（仅读取元数据，不修改共享状态）
    let new_count = if needs_probe.is_empty() {
        0usize
    } else {
        let num_threads = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
            .min(needs_probe.len());
        let chunk_size = (needs_probe.len() + num_threads - 1) / num_threads;

        let mut results: Vec<(PlatformPath, Result<scanner::AudioMeta, String>)> = Vec::new();

        let _t_probe = Instant::now();
        std::thread::scope(|s| {
            let mut handles = Vec::new();

            for chunk in needs_probe.chunks(chunk_size) {
                let chunk: Vec<PlatformPath> = chunk.to_vec();
                handles.push(s.spawn(move || {
                    let mut chunk_results = Vec::new();
                    for path in &chunk {
                        let result = scanner::probe_file(path);
                        chunk_results.push((path.clone(), result));
                    }
                    chunk_results
                }));
            }

            for handle in handles {
                match handle.join() {
                    Ok(chunk_results) => results.extend(chunk_results),
                    Err(_) => {
                        eprintln!("[local_source] 探测线程异常退出");
                    }
                }
            }
        });
        let t_probe = _t_probe.elapsed();
        eprintln!(
            "[local_source] ⏱ 5b. 并行探测 {} 个文件 ({} 线程): {:?}",
            needs_probe.len(),
            num_threads,
            t_probe
        );

        // 5c. 批量添加到音乐库（一次性加载，内存合并，一次性写回）
        let mut new_count = 0usize;
        {
            let songs_and_paths: Vec<(&PlatformPath, Song)> = results
                .iter()
                .filter_map(|(path, meta_result)| {
                    meta_result.as_ref().ok().map(|meta| (path, local_source.build_song(path, meta)))
                })
                .collect();

            if !songs_and_paths.is_empty() {
                let songs: Vec<Song> = songs_and_paths
                    .iter()
                    .map(|(_, song)| song.clone())
                    .collect();
                match local_source.library.add_songs_batch(&songs) {
                    Ok(stored_ids) => {
                        for (i, (path, _)) in songs_and_paths.iter().enumerate() {
                            let stored_id = &stored_ids[i];
                            local_source
                                .file_index
                                .write()
                                .insert((*path).clone(), stored_id.clone());
                            local_source
                                .id_to_path
                                .write()
                                .insert(stored_id.clone(), (*path).clone());
                            local_source.update_file_mtime(path, stored_id);
                            new_count += 1;
                        }
                    }
                    Err(e) => {
                        eprintln!("[local_source] 批量添加歌曲失败: {}", e);
                    }
                }
            }
        }

        // 打印探测失败的文件
        for (path, meta_result) in &results {
            if let Err(e) = meta_result {
                eprintln!(
                    "[local_source] 探测文件失败 '{}': {}",
                    crate::module::platform::path_to_string(path),
                    e
                );
            }
        }

        new_count
    };
    let t5c = Instant::now();
    eprintln!(
        "[local_source] ⏱ 5c. 添加到音乐库: {:?}, 库中歌曲总数: {}",
        t5c - t5a,
        library.song_count()
    );

    // 保存：library 仅在数据实际变化时落盘（dirty 标记由 add_songs_batch 控制），
    // mtime 始终写入独立存储文件。
    {
        let _t_save = Instant::now();
        library.save_if_dirty()?;
        let _ = local_source.save_mtime_cache();
        let t_save = _t_save.elapsed();
        eprintln!(
            "[local_source] ⏱ 6. 保存: {:?}, 库中歌曲总数: {}",
            t_save,
            library.song_count()
        );
    }

    // 注册为 must-source
    let _t_reg = Instant::now();
    registrar
        .register(local_source.clone())
        .map_err(|e| format!("注册本地来源失败: {}", e))?;
    let t_reg = _t_reg.elapsed();
    eprintln!("[local_source] ⏱ 7. 注册来源: {:?}", t_reg);

    // 启动后台文件监听器（仅桌面端）
    #[cfg(not(target_os = "android"))]
    {
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
    }
    let t8 = Instant::now();
    eprintln!("[local_source] ⏱ 8. 启动文件监听器: {:?}", t8 - t_reg);

    eprintln!(
        "[local_source] ⏱ 总计: {:?} — {} 文件夹, {} 已恢复, {} 跳过, {} 新索引",
        t8 - t0,
        folder_manager.count(),
        restored,
        skipped,
        new_count
    );

    Ok(local_source)
}
