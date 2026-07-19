//! 本地音乐来源 — [`LocalMusicSource`] 是 [`MusicSource`] 的具体实现。
//!
//! 负责管理本地文件系统中的音乐文件，通过 symphonia 读取元数据，
//! 并通过 notify 监听文件夹变化实现增量同步到 [`MusicLibrary`]。
//!
//! # 设计要点
//!
//! - **entity_id 使用文件绝对路径**：`SourceId.entity_id` 即为音频文件的规范化路径。
//!   这使得资源获取（`song_file_get`）成为简单的文件读取操作。
//! - **must-source**：本地来源在初始化时自动注册，不允许注销。
//! - **初始文件夹**：首次启动时自动添加系统音乐目录（`dirs::audio_dir()`）。
//! - **跨平台路径**：通过 [`crate::module::platform::PlatformPath`] 适配桌面（`PathBuf`）
//!   和 Android（`String` / content URI）。

use super::folder::FolderManager;
use super::scanner::{self, AudioMeta};
use crate::module::music_library::library::MusicLibrary;
use crate::module::music_library::models::{Album, Artist, Lyric, Song};
use crate::module::perf;
use crate::module::music_source::traits::MusicSource;
use crate::module::music_source::types::{EntityType, SourceId, SourceType};
use crate::module::platform::{self, PlatformPath};
use crate::module::storage::persistent::PersistentStore;
use parking_lot::Mutex;
use parking_lot::RwLock;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use uuid::Uuid;

/// 封面缓存容量上限（条目数）。
/// 单条目平均 50-500KB，256 条 ≈ 12-128MB（最坏情况）。
const COVER_CACHE_CAP: usize = 256;

/// 本地音乐来源的名称常量。
pub const LOCAL_SOURCE_NAME: &str = "local";

/// 本地音乐来源。
///
/// 实现 [`MusicSource`]，将所有查询和资源操作映射到本地文件系统。
pub struct LocalMusicSource {
    /// 来源名称（固定为 "local"）
    name: String,
    /// 文件夹管理器（持久化 + 运行时）— 供命令层访问
    pub folder_manager: Arc<FolderManager>,
    /// 音乐库引用 — 用于增量同步，供命令层访问
    pub library: Arc<MusicLibrary>,
    /// 文件索引：规范路径 → 库内 Song ID，供命令层访问
    pub file_index: RwLock<HashMap<PlatformPath, String>>,
    /// 反向索引：库内 Song ID → 规范路径
    pub id_to_path: RwLock<HashMap<String, PlatformPath>>,
    /// 文件 mtime 缓存：规范路径字符串 → (modified_secs, file_size, song_id)
    /// 启动时对比文件系统 mtime，跳过未变化文件的重扫描
    pub file_mtimes: RwLock<HashMap<String, (u64, u64, String)>>,
    /// 独立的 mtime 持久化存储（与 library 分离，避免每次保存都序列化全部歌曲数据）
    mtime_store: PersistentStore,
    /// 封面图内存缓存：entity_id（路径）→ 图片字节
    /// 避免每次 chordial://image 请求都触发 extract_cover_art（5-50ms/次）
    cover_cache: Mutex<HashMap<String, Arc<Vec<u8>>>>,
}

impl LocalMusicSource {
    /// 创建本地音乐来源。
    ///
    /// # 参数
    /// - `folder_manager`: 文件夹管理器（持久化 + 运行时文件夹集合）
    /// - `library`: 音乐库引用
    pub fn new(
        folder_manager: Arc<FolderManager>,
        library: Arc<MusicLibrary>,
        mtime_store: PersistentStore,
    ) -> Self {
        Self {
            name: LOCAL_SOURCE_NAME.to_string(),
            folder_manager,
            library,
            file_index: RwLock::new(HashMap::new()),
            id_to_path: RwLock::new(HashMap::new()),
            file_mtimes: RwLock::new(HashMap::new()),
            mtime_store,
            cover_cache: Mutex::new(HashMap::new()),
        }
    }

    // ── 内部辅助方法 ─────────────────────────────────

    /// 扫描单个音频文件并添加到音乐库（或合并到已有条目）。
    ///
    /// 同时尝试读取同目录的 `.lrc` / `.txt` 歌词文件，若命中则构造 `Lyric` 实体
    /// 写入 PersistentStore，并让 `Song.lyric_id` 指向该 `Lyric.id`。
    /// 若无歌词文件，`lyric_id` 置为 `None`，避免库中残留孤儿引用。
    ///
    /// 返回 `true` 表示成功处理（新增或合并），`false` 表示跳过（非音频文件）。
    pub fn index_file(&self, path: &PlatformPath) -> Result<bool, String> {
        let _scope = perf::scope("source.index_file");
        let canonical = platform::canonicalize(path)
            .unwrap_or_else(|_| path.clone());

        // 跳过非音频文件
        if !scanner::is_supported_audio(&canonical) {
            return Ok(false);
        }

        // 若已索引过，跳过
        if self.file_index.read().contains_key(&canonical) {
            return Ok(true);
        }

        // 探测元数据
        let meta = scanner::probe_file(&canonical)?;

        // 读取同目录歌词文件（.lrc 优先，.txt 兜底）
        let lyric_text = scanner::read_lyric_file(&canonical);

        // 构建 Song（lyric_id 占位 UUID 仅在确实有歌词时保留）
        let mut song = self.build_song(&canonical, &meta);
        if lyric_text.is_none() {
            song.lyric_id = None;
        }

        // 添加到音乐库（自动去重合并），获取实际存储的 ID
        let stored_id = self.library.add_song(&song)?;

        // 写入 Lyric 实体到 PersistentStore
        // —— 这一步是关键：前端 library_get_lyric_of_song(songId) 直接查 lyrics 表
        if let Some(text) = lyric_text {
            let lyric = Lyric {
                id: song.lyric_id.clone().unwrap_or_else(|| Uuid::new_v4().to_string()),
                song_id: stored_id.clone(),
                text,
                source_id: song.source_ids[0].clone(),
            };
            // add_lyric 在 id 冲突时返回 Err，reindex 时旧 Lyric 已被
            // remove_specific_song_source_ids 清理，此处不会冲突
            if let Err(e) = self.library.add_lyric(&lyric) {
                eprintln!(
                    "[local_source] 写入歌词失败 '{}': {}",
                    platform::path_to_string(&canonical),
                    e
                );
            }
        }

        // 更新索引（使用库中实际存储的 ID）
        self.file_index
            .write()
            .insert(canonical.clone(), stored_id.clone());
        self.id_to_path
            .write()
            .insert(stored_id, canonical);

        Ok(true)
    }

    /// 从索引中移除文件引用，并从音乐库中移除对应的 SourceId。
    ///
    /// 若歌曲失去全部来源引用，则歌曲被自动删除；
    /// 级联空实体由 [`MusicLibrary::cleanup_empty_entities`] 处理。
    pub fn unindex_file(&self, path: &PlatformPath) -> Result<bool, String> {
        let canonical = platform::canonicalize(path)
            .unwrap_or_else(|_| path.clone());

        let song_id = {
            let index = self.file_index.read();
            index.get(&canonical).cloned()
        };

        let Some(song_id) = song_id else {
            return Ok(false); // 不在索引中
        };

        // 从音乐库中精准移除该文件的 SourceId
        let entity_id = platform::path_to_string(&canonical);
        let mut entity_ids = HashSet::new();
        entity_ids.insert(entity_id);
        self.library
            .remove_specific_song_source_ids(LOCAL_SOURCE_NAME, &entity_ids)?;

        // 更新本地索引
        self.file_index.write().remove(&canonical);
        self.id_to_path.write().remove(&song_id);

        Ok(true)
    }

    /// 重新索引文件（适用于文件修改事件）。
    pub fn reindex_file(&self, path: &PlatformPath) -> Result<bool, String> {
        let _scope = perf::scope("source.reindex_file");
        // 先卸载旧索引
        self.unindex_file(path)?;
        // 再重新索引
        self.index_file(path)
    }

    /// 批量索引音频文件 — 一次性加载库 + 并行探测 + 单次批量合并写回。
    ///
    /// 相比循环调用 [`index_file`](Self::index_file)，避免了每首歌曲都
    /// 反序列化整个库（songs/artists/albums）+ 重建索引 + 全量写回
    /// 带来的 O(N²) 开销。典型场景：1000 个文件
    /// - 旧路径（逐个 `index_file`）：~96ms × 1000 ≈ 96s
    /// - 新路径（本方法）：~150ms 总计（单次加载 + 并行探测 + 单次写回）
    ///
    /// # 流程
    /// 1. 规范化路径并过滤：跳过非音频文件、已在 file_index 中的文件
    /// 2. 并行 `probe_file` + `read_lyric_file`（CPU/IO 并行）
    /// 3. 构建 Song 列表
    /// 4. 一次性调用 [`MusicLibrary::add_songs_batch`] 合并入库
    /// 5. 串行写入 Lyric 实体、更新 file_index / id_to_path / mtime
    ///
    /// # 返回
    /// `(indexed, errors)` — 成功索引条目数 + 失败文件描述列表
    pub fn batch_index_files(&self, paths: &[PlatformPath]) -> Result<(usize, Vec<String>), String> {
        let _scope = perf::scope("source.batch_index_files");

        // 1. 规范化 + 过滤：跳过非音频文件、已索引文件
        // 预估待探测数量以减少扩容；上限为 paths.len()
        let mut needs_probe: Vec<PlatformPath> = Vec::with_capacity(paths.len());
        {
            let file_index = self.file_index.read();
            for path in paths {
                let canonical = platform::canonicalize(path)
                    .unwrap_or_else(|_| path.clone());
                if !scanner::is_supported_audio(&canonical) {
                    continue;
                }
                if file_index.contains_key(&canonical) {
                    continue;
                }
                needs_probe.push(canonical);
            }
        }

        if needs_probe.is_empty() {
            return Ok((0, Vec::new()));
        }

        // 2. 并行 probe + read_lyric_file
        // 线程数：取 CPU 核心数与文件数的较小值；至少 1
        let probe_count = needs_probe.len();
        let num_threads = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
            .min(probe_count)
            .max(1);
        let chunk_size = (probe_count + num_threads - 1) / num_threads;

        // 每个探测结果携带 (path, Result<(meta, lyric_text), error_msg>)
        let mut probe_results: Vec<(PlatformPath, Result<(AudioMeta, Option<String>), String>)> =
            Vec::with_capacity(probe_count);

        std::thread::scope(|s| {
            let mut handles = Vec::with_capacity(num_threads);
            for chunk in needs_probe.chunks(chunk_size) {
                // chunk.to_vec() 避免跨线程借用 needs_probe
                let chunk: Vec<PlatformPath> = chunk.to_vec();
                handles.push(s.spawn(move || {
                    let mut chunk_results = Vec::with_capacity(chunk.len());
                    for path in &chunk {
                        // probe + read_lyric 在同一线程内串行，
                        // 避免再次起线程的开销；线程间仍是并行的
                        let result = scanner::probe_file(path).and_then(|meta| {
                            // read_lyric_file 失败不影响 song 入库，返回 None 即可
                            let lyric = scanner::read_lyric_file(path);
                            Ok((meta, lyric))
                        });
                        chunk_results.push((path.clone(), result));
                    }
                    chunk_results
                }));
            }
            for handle in handles {
                match handle.join() {
                    Ok(chunk_results) => probe_results.extend(chunk_results),
                    Err(_) => eprintln!("[local_source] batch_index_files 探测线程异常退出"),
                }
            }
        });

        // 3. 构建 Song 列表，收集错误
        // 容量上限 = 探测成功的条目数
        let success_count = probe_results
            .iter()
            .filter(|r| r.1.is_ok())
            .count();
        let mut songs_and_lyrics: Vec<(PlatformPath, Song, Option<String>)> =
            Vec::with_capacity(success_count);
        let mut errors: Vec<String> = Vec::new();

        for (path, result) in probe_results {
            match result {
                Ok((meta, lyric_text)) => {
                    let mut song = self.build_song(&path, &meta);
                    if lyric_text.is_none() {
                        song.lyric_id = None;
                    }
                    songs_and_lyrics.push((path, song, lyric_text));
                }
                Err(e) => {
                    errors.push(format!("{}: {}", platform::path_to_string(&path), e));
                }
            }
        }

        if songs_and_lyrics.is_empty() {
            return Ok((0, errors));
        }

        // 4. 批量合并入库（单次加载 + 单次写回，O(N+K) 总复杂度）
        let songs: Vec<Song> = songs_and_lyrics
            .iter()
            .map(|(_, s, _)| s.clone())
            .collect();
        let stored_ids = self.library.add_songs_batch(&songs)?;

        // 5. 串行写入 Lyric + 更新索引/mtime
        // 这部分都是 O(1) 操作或单次 fs 调用，不在热路径
        for (i, (path, song, lyric_text)) in songs_and_lyrics.iter().enumerate() {
            let stored_id = &stored_ids[i];

            // 写入 Lyric 实体（若有歌词）
            if let Some(text) = lyric_text {
                let lyric = Lyric {
                    id: song.lyric_id.clone().unwrap_or_else(|| Uuid::new_v4().to_string()),
                    song_id: stored_id.clone(),
                    text: text.clone(),
                    source_id: song.source_ids[0].clone(),
                };
                if let Err(e) = self.library.add_lyric(&lyric) {
                    eprintln!(
                        "[local_source] 批量写入歌词失败 '{}': {}",
                        platform::path_to_string(path),
                        e
                    );
                }
            }

            // 更新索引（使用库中实际存储的 ID）
            self.file_index
                .write()
                .insert(path.clone(), stored_id.clone());
            self.id_to_path
                .write()
                .insert(stored_id.clone(), path.clone());
            self.update_file_mtime(path, stored_id);
        }

        Ok((songs_and_lyrics.len(), errors))
    }

    /// 批量取消索引音频文件 — 单次库调用替代 N 次 `unindex_file`。
    ///
    /// 旧路径循环 `unindex_file` 每次都会触发 `remove_specific_song_source_ids`
    /// → `cleanup_empty_entities` → `save`，N 个文件 = N 次全量序列化写盘。
    /// 本方法仅调用一次库清理，再批量更新本地索引。
    ///
    /// # 返回
    /// 成功从索引中移除的条目数
    pub fn batch_unindex_files(&self, paths: &[PlatformPath]) -> Result<usize, String> {
        let _scope = perf::scope("source.batch_unindex_files");

        // 1. 规范化路径 + 收集对应的 entity_id 和 song_id
        let mut entity_ids: HashSet<String> = HashSet::with_capacity(paths.len());
        let mut to_remove_from_index: Vec<(PlatformPath, String)> = Vec::with_capacity(paths.len());

        for path in paths {
            let canonical = platform::canonicalize(path)
                .unwrap_or_else(|_| path.clone());
            let entity_id = platform::path_to_string(&canonical);
            entity_ids.insert(entity_id);
            // 同时记录 song_id 用于清理 id_to_path
            if let Some(song_id) = self.file_index.read().get(&canonical).cloned() {
                to_remove_from_index.push((canonical, song_id));
            }
        }

        if entity_ids.is_empty() {
            return Ok(0);
        }

        // 2. 一次性库清理（内部已用 get_entries_filtered 避免全量反序列化）
        self.library.remove_specific_song_source_ids(
            LOCAL_SOURCE_NAME,
            &entity_ids,
        )?;

        // 3. 批量更新本地索引
        let removed = to_remove_from_index.len();
        {
            let mut file_index = self.file_index.write();
            let mut id_to_path = self.id_to_path.write();
            for (canonical, song_id) in &to_remove_from_index {
                file_index.remove(canonical);
                id_to_path.remove(song_id);
            }
        }

        Ok(removed)
    }

    /// 从 AudioMeta 构建 Song 模型。
    ///
    /// 关键逻辑：
    /// - 将 `meta.artist` 按 `/`、`&`、`、`、`，`、` feat. `、` ft. `、` featuring ` 等
    ///   分隔符拆分为多个独立 artist，每个生成独立 UUID。
    /// - 写入 `song.year = meta.year`，供后续 album 聚合使用。
    pub fn build_song(&self, file_path: &PlatformPath, meta: &AudioMeta) -> Song {
        let entity_id = platform::path_to_string(file_path);
        let song_id = Uuid::new_v4().to_string();
        let artist_names = split_artist_names(meta.artist.as_deref());
        let artist_ids: Vec<String> = (0..artist_names.len())
            .map(|_| Uuid::new_v4().to_string())
            .collect();
        let album_title = meta.album.clone();
        let album_id = album_title.as_ref().map(|_| Uuid::new_v4().to_string());
        let lyric_id = Some(Uuid::new_v4().to_string());

        let source_id = SourceId {
            source_name: LOCAL_SOURCE_NAME.to_string(),
            source_type: SourceType::Local,
            entity_type: EntityType::Song,
            entity_id: entity_id.clone(),
        };

        Song {
            id: song_id,
            title: meta.title.clone().unwrap_or_else(|| "未知歌曲".to_string()),
            artist_names,
            album_title,
            duration: meta.duration_secs,
            artist_ids,
            album_id,
            lyric_id,
            source_ids: vec![source_id],
            year: meta.year,
        }
    }

    /// 按文件路径查找对应的库内 Song ID。
    pub fn find_song_id_by_path(&self, path: &PlatformPath) -> Option<String> {
        let canonical = platform::canonicalize(path)
            .unwrap_or_else(|_| path.clone());
        self.file_index.read().get(&canonical).cloned()
    }

    /// 按文件路径查找对应的 SourceId。
    pub fn find_source_id_by_path(&self, path: &PlatformPath) -> Option<SourceId> {
        let canonical = platform::canonicalize(path)
            .unwrap_or_else(|_| path.clone());
        let entity_id = platform::path_to_string(&canonical);
        Some(SourceId {
            source_name: LOCAL_SOURCE_NAME.to_string(),
            source_type: SourceType::Local,
            entity_type: EntityType::Song,
            entity_id,
        })
    }

    /// 从持久化的 MusicLibrary 重建内存索引并加载 mtime 缓存（启动时调用）。
    ///
    /// 遍历库中所有带 `"local"` SourceId 的歌曲，以 `entity_id`（即文件路径）
    /// 重建 `file_index` 和 `id_to_path`。同时从持久存储加载文件 mtime 缓存，
    /// 用于启动时跳过未变化文件的重扫描。文件已不存在的条目自动从库中和 mtime 缓存中移除。
    ///
    /// # 返回
    /// `(restored, removed)` — 成功恢复的条目数和因文件丢失而移除的条目数。
    pub fn restore_index_from_library(&self) -> (usize, usize) {
        use std::time::Instant;

        let _scope = perf::scope("source.restore_index");

        let _t0 = Instant::now();
        let all_songs = self.library.get_all_songs();
        let t0 = _t0.elapsed();
        eprintln!(
            "[local_source] ⏱ 4a. 从库加载 {} 首歌曲: {:?}",
            all_songs.len(),
            t0
        );

        // 从独立 mtime 存储加载
        let _t1 = Instant::now();
        let mut mtimes: HashMap<String, (u64, u64, String)> = self
            .mtime_store
            .get("file_mtimes")
            .unwrap_or_default();
        let t1 = _t1.elapsed();
        eprintln!(
            "[local_source] ⏱ 4b. 加载 mtime 缓存 ({} 条): {:?}",
            mtimes.len(),
            t1
        );

        // 收集需要 canonicalize 的路径（预分配上界避免多次扩容）
        let mut to_canonicalize: Vec<(String, String)> = Vec::with_capacity(all_songs.len());
        for (song_id, song) in &all_songs {
            let local_sid = song.source_ids.iter().find(|sid| {
                sid.source_name == LOCAL_SOURCE_NAME && sid.entity_type == EntityType::Song
            });
            if let Some(sid) = local_sid {
                to_canonicalize.push((song_id.clone(), sid.entity_id.clone()));
            }
        }

        let _t2 = Instant::now();
        let mut restored = 0usize;
        let mut removed_paths: HashSet<String> = HashSet::new();
        let canonicalize_count = to_canonicalize.len();

        // 并行 canonicalize
        let num_threads = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
            .min(canonicalize_count.max(1));
        let chunk_size = (canonicalize_count + num_threads - 1) / num_threads;

        let mut results: Vec<(String, String, bool)> = Vec::with_capacity(canonicalize_count);

        std::thread::scope(|s| {
            let mut handles = Vec::with_capacity(num_threads);
            for chunk in to_canonicalize.chunks(chunk_size) {
                let chunk: Vec<(String, String)> = chunk.to_vec();
                handles.push(s.spawn(move || {
                    let mut chunk_results = Vec::with_capacity(chunk.len());
                    for (song_id, path_str) in &chunk {
                        let path = PlatformPath::from(path_str.as_str());
                        let ok = platform::canonicalize(&path).is_ok();
                        chunk_results.push((song_id.clone(), path_str.clone(), ok));
                    }
                    chunk_results
                }));
            }
            for handle in handles {
                if let Ok(chunk_results) = handle.join() {
                    results.extend(chunk_results);
                }
            }
        });

        // 串行写入索引
        for (song_id, path_str, ok) in &results {
            if *ok {
                let canonical = PlatformPath::from(path_str.as_str());
                self.file_index
                    .write()
                    .insert(canonical.clone(), song_id.clone());
                self.id_to_path
                    .write()
                    .insert(song_id.clone(), canonical);
                restored += 1;
            } else {
                removed_paths.insert(path_str.clone());
            }
        }

        let t2 = _t2.elapsed();
        eprintln!(
            "[local_source] ⏱ 4c. 并行 canonicalize {} 首 ({} 线程, {} 个丢失): {:?}",
            canonicalize_count,
            num_threads,
            removed_paths.len(),
            t2
        );

        let _t3 = Instant::now();
        // 清理不存在的文件对应的 mtime 条目
        mtimes.retain(|path_str, _| {
            let p = PlatformPath::from(path_str.as_str());
            platform::exists(&p)
        });
        *self.file_mtimes.write() = mtimes;
        let t3 = _t3.elapsed();
        eprintln!(
            "[local_source] ⏱ 4d. 清理 mtime 缓存: {:?}",
            t3
        );

        if !removed_paths.is_empty() {
            let _t4 = Instant::now();
            if let Err(e) = self
                .library
                .remove_specific_song_source_ids(LOCAL_SOURCE_NAME, &removed_paths)
            {
                eprintln!(
                    "[local_source] 清理已删除文件的 SourceId 失败: {}",
                    e
                );
            }
            let t4 = _t4.elapsed();
            let removed = removed_paths.len();
            eprintln!(
                "[local_source] ⏱ 4e. 批量移除 {} 个丢失文件: {:?}",
                removed,
                t4
            );
            eprintln!(
                "[local_source] 索引恢复: {} 首歌曲, {} 个已删除文件已清理",
                restored, removed
            );
            return (restored, removed);
        }

        eprintln!("[local_source] 索引恢复: {} 首歌曲已从库中恢复", restored);
        (restored, 0)
    }

    /// 检查文件 mtime+size 是否与缓存一致，返回缓存的 song_id（若未变化）。
    ///
    /// `path` 应为已规范化的路径。
    pub fn check_file_unchanged(&self, path: &PlatformPath) -> Option<String> {
        let path_str = platform::path_to_string(path);
        let mtimes = self.file_mtimes.read();
        let (cached_mtime, cached_size, song_id) = mtimes.get(&path_str)?;

        let current_mtime = platform::file_modified_secs(
            &PlatformPath::from(path_str.as_str()),
        )
        .ok()?;
        let current_size = platform::file_size(
            &PlatformPath::from(path_str.as_str()),
        )
        .ok()?;

        if current_mtime == *cached_mtime && current_size == *cached_size {
            Some(song_id.clone())
        } else {
            None
        }
    }

    /// 更新文件的 mtime 缓存条目。
    ///
    /// `path` 应为已规范化的路径。
    pub fn update_file_mtime(&self, path: &PlatformPath, song_id: &str) {
        let path_str = platform::path_to_string(path);
        let platform_path = PlatformPath::from(path_str.as_str());
        if let (Ok(mtime), Ok(size)) = (
            platform::file_modified_secs(&platform_path),
            platform::file_size(&platform_path),
        ) {
            self.file_mtimes.write().insert(
                path_str,
                (mtime, size, song_id.to_string()),
            );
        }
    }

    /// 将 mtime 缓存持久化到独立存储（与 library 分离，避免每次保存都序列化全部歌曲）。
    pub fn save_mtime_cache(&self) -> Result<(), String> {
        let mtimes = self.file_mtimes.read().clone();
        self.mtime_store.set("file_mtimes", &mtimes)?;
        self.mtime_store.save()
    }

    /// 从磁盘提取专辑封面字节（无缓存）。
    ///
    /// 提取顺序：
    /// 1. 音频文件嵌入封面（FLAC / ID3v2）— symphonia 解析
    /// 2. 同目录同名图片（.jpg/.png/.webp/.bmp）
    /// 3. 目录下常见封面名（cover/folder/albumart/front）
    fn extract_album_picture(&self, path: &PlatformPath) -> Result<Vec<u8>, String> {
        // 1. 若为音频文件，尝试提取嵌入封面（FLAC / ID3v2）
        if platform::is_file(path) && super::scanner::is_supported_audio(path) {
            if let Ok(cover_data) = super::scanner::extract_cover_art(path) {
                return Ok(cover_data);
            }
        }

        // 2. 确定搜索目录
        let search_dir = if platform::is_dir(path) {
            path.clone()
        } else if let Some(parent) = platform::path_parent(path) {
            parent
        } else {
            return Err(format!("无法确定搜索目录: {}", platform::path_to_string(path)));
        };

        // 3. 同目录下与音频文件同名的图片文件
        if !platform::is_dir(path) {
            for ext in &["jpg", "jpeg", "png", "webp", "bmp"] {
                let sibling = platform::path_with_extension(path, ext);
                if platform::exists(&sibling) {
                    return platform::read_bytes(&sibling)
                        .map_err(|e| format!("读取封面文件失败 '{}': {}", platform::path_to_string(&sibling), e));
                }
            }
        }

        // 4. 目录下常见封面文件名
        let candidates = [
            platform::path_join(&search_dir, "cover.jpg"),
            platform::path_join(&search_dir, "cover.png"),
            platform::path_join(&search_dir, "folder.jpg"),
            platform::path_join(&search_dir, "folder.png"),
            platform::path_join(&search_dir, "albumart.jpg"),
            platform::path_join(&search_dir, "albumart.png"),
            platform::path_join(&search_dir, "front.jpg"),
            platform::path_join(&search_dir, "front.png"),
        ];
        for candidate in &candidates {
            if platform::exists(candidate) {
                return platform::read_bytes(candidate)
                    .map_err(|e| format!("读取封面文件失败 '{}': {}", platform::path_to_string(candidate), e));
            }
        }

        Err(format!("未找到封面图片: {}", platform::path_to_string(path)))
    }
}

impl MusicSource for LocalMusicSource {
    fn name(&self) -> &str {
        &self.name
    }

    fn source_type(&self) -> SourceType {
        SourceType::Local
    }

    fn search_songs(&self, query: &str) -> Result<Vec<Song>, String> {
        let _scope = perf::scope("source.search_songs");
        // 优化：委托给 library 内存搜索，避免每次查询重新探测文件（O(n) 磁盘 I/O）
        // 旧实现：`self.file_index.read().keys().cloned()` + 循环 `scanner::probe_file()`
        // 每次查询触发 N 次磁盘读取，对 1000 首歌 = 1000 次 I/O。
        let results: Vec<Song> = self
            .library
            .search_songs(query)
            .into_iter()
            .filter(|song| {
                song.source_ids
                    .iter()
                    .any(|sid| sid.source_name == LOCAL_SOURCE_NAME)
            })
            .collect();
        Ok(results)
    }

    fn get_song(&self, id: &str) -> Result<Option<Song>, String> {
        // id 可能是库内 UUID 或文件路径
        let path = if let Some(p) = self.id_to_path.read().get(id) {
            p.clone()
        } else {
            let p = PlatformPath::from(id);
            if platform::exists(&p) {
                p
            } else {
                return Ok(None);
            }
        };

        match scanner::probe_file(&path) {
            Ok(meta) => Ok(Some(self.build_song(&path, &meta))),
            Err(_) => Ok(None),
        }
    }

    fn get_artist(&self, _id: &str) -> Result<Option<Artist>, String> {
        // 本地来源的艺人信息从歌曲标签派生，直接从音乐库查询即可
        Ok(None)
    }

    fn get_album(&self, _id: &str) -> Result<Option<Album>, String> {
        // 同上
        Ok(None)
    }

    fn get_lyric(&self, _song_id: &str) -> Result<Option<Lyric>, String> {
        Ok(None)
    }

    fn song_file_get(&self, entity_id: &str) -> Result<Vec<u8>, String> {
        let path = PlatformPath::from(entity_id);
        platform::read_bytes(&path)
            .map_err(|e| format!("读取音频文件失败 '{}': {}", entity_id, e))
    }

    fn song_file_path(&self, entity_id: &str) -> Option<String> {
        let path = PlatformPath::from(entity_id);
        if platform::is_file(&path) {
            Some(entity_id.to_string())
        } else {
            // 尝试通过 id_to_path 查找
            self.id_to_path
                .read()
                .get(entity_id)
                .map(|p| platform::path_to_string(p))
        }
    }

    fn album_picture_get(&self, entity_id: &str) -> Result<Vec<u8>, String> {
        // 1. 命中内存缓存直接返回（典型命中：浏览/播放同一专辑时多次请求封面）
        //    extract_cover_art 平均 5-50ms（symphonia 全文件解析），缓存命中 ~0.01ms
        //    外层 resource::get_album_picture 已有 perf::scope 计时，命中时显示 ~0ms
        {
            let cache = self.cover_cache.lock();
            if let Some(hit) = cache.get(entity_id) {
                return Ok(hit.as_ref().clone());
            }
        }

        let path = PlatformPath::from(entity_id);
        let data = self.extract_album_picture(&path)?;

        // 2. 写入缓存（容量上限简单淘汰策略：超出 cap 时清空一半）
        let mut cache = self.cover_cache.lock();
        if cache.len() >= COVER_CACHE_CAP {
            // 简单 FIFO 半量淘汰（避免单条 LRU 链维护开销）
            let drop_count = COVER_CACHE_CAP / 2;
            let keys: Vec<String> = cache.keys().take(drop_count).cloned().collect();
            for k in keys {
                cache.remove(&k);
            }
        }
        cache.insert(entity_id.to_string(), Arc::new(data.clone()));

        Ok(data)
    }

    fn lyric_text_get(&self, song_id: &str) -> Result<String, String> {
        // 通过 song_id（或直接当作路径）定位音频文件，再读同目录 .lrc / .txt
        // 复用 scanner::read_lyric_file，与扫描时入库的逻辑保持一致
        let audio_path = if let Some(p) = self.id_to_path.read().get(song_id) {
            p.clone()
        } else {
            PlatformPath::from(song_id)
        };

        scanner::read_lyric_file(&audio_path)
            .ok_or_else(|| format!("未找到歌词文件: {}", platform::path_to_string(&audio_path)))
    }
}

// ── 模块级辅助 ───────────────────────────────────────────────────────────────

/// 将单个 artist 标签字符串按常见分隔符拆分为多个独立 artist 名称。
///
/// 支持的分隔符（不区分大小写）：
/// - 半角：`/`、`&`、``,`、`;`
/// - 全角：`、`、`，`、`；`
/// - 关键字：`feat.`、`ft.`、`featuring`、`with`、`vs.`、`versus`
///
/// 例：`"周杰伦 & 方文山"` → `["周杰伦", "方文山"]`
/// 例：`"A feat. B"` → `["A", "B"]`
/// 例：`"AC/DC"` → `["AC/DC"]`（不含空格的 `/` 视为乐队名一部分，不拆分）
///
/// 空字符串返回 `["未知艺术家"]`，保留与旧实现兼容的默认值。
pub fn split_artist_names(raw: Option<&str>) -> Vec<String> {
    let raw = match raw {
        Some(s) => s.trim(),
        None => return vec!["未知艺术家".to_string()],
    };
    if raw.is_empty() {
        return vec!["未知艺术家".to_string()];
    }

    // 策略：先按 ASCII 分隔符 (`/`、`&`、`,`、`;`) 和全角分隔符拆分，
    // 但 `/` 仅在两侧有空格时才视为分隔符（避免误拆 "AC/DC"）。
    // 关键字分隔符（feat./ft./featuring/with/vs./versus）使用正则式分割。
    let mut parts: Vec<String> = Vec::new();

    // 第一轮：按 ASCII & 全角分隔符拆分
    for segment in raw.split(|c: char| matches!(c, '&' | ',' | ';' | '、' | '，' | '；')) {
        // `/` 仅在两侧均有空格时才拆分
        for sub in split_on_slash_with_spaces(segment) {
            let trimmed = sub.trim();
            if !trimmed.is_empty() {
                parts.push(trimmed.to_string());
            }
        }
    }

    // 第二轮：按关键字分隔符拆分（feat./ft./featuring/with/vs./versus）
    let mut final_parts: Vec<String> = Vec::with_capacity(parts.len());
    for part in parts {
        for sub in split_on_keywords(&part) {
            let trimmed = sub.trim();
            if !trimmed.is_empty() {
                final_parts.push(trimmed.to_string());
            }
        }
    }

    if final_parts.is_empty() {
        vec!["未知艺术家".to_string()]
    } else {
        final_parts
    }
}

/// 拆分形如 `"A / B"` 的字符串（要求 `/` 两侧至少有一个空格）。
///
/// `"AC/DC"` → `["AC/DC"]`（不拆）
/// `"A / B"` → `["A", "B"]`
/// `"A/B"` → `["A/B"]`（不拆）
fn split_on_slash_with_spaces(s: &str) -> Vec<&str> {
    let mut result: Vec<&str> = Vec::new();
    let bytes = s.as_bytes();
    let mut last = 0usize;
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'/' && i > 0 && i + 1 < bytes.len() {
            let left_space = bytes[i - 1] == b' ';
            let right_space = bytes[i + 1] == b' ';
            if left_space && right_space {
                result.push(&s[last..i]);
                last = i + 1;
            }
        }
        i += 1;
    }
    result.push(&s[last..]);
    result
}

/// 按关键字分隔符拆分（不区分大小写）。
///
/// 支持的关键字：`feat.`、`ft.`、`featuring`、`with`、`vs.`、`versus`
/// 关键字前后必须是空格或字符串边界。
fn split_on_keywords(s: &str) -> Vec<String> {
    let lower = s.to_lowercase();
    let keywords = ["feat.", "ft.", "featuring", "with", "vs.", "versus"];

    let mut indices: Vec<usize> = Vec::new();
    for kw in &keywords {
        let mut start = 0usize;
        while let Some(pos) = lower[start..].find(kw) {
            let abs = start + pos;
            // 关键字前必须是字符串开头或空格
            let before_ok = abs == 0 || s.as_bytes()[abs - 1] == b' ';
            // 关键字后必须是字符串结尾或空格
            let after_pos = abs + kw.len();
            let after_ok = after_pos >= s.len()
                || s.as_bytes()[after_pos] == b' '
                || s.as_bytes()[after_pos] == b'(';
            if before_ok && after_ok {
                indices.push(abs);
            }
            start = abs + kw.len();
            if start >= s.len() {
                break;
            }
        }
    }

    if indices.is_empty() {
        return vec![s.to_string()];
    }

    indices.sort();
    let mut result: Vec<String> = Vec::with_capacity(indices.len() + 1);
    let mut last = 0usize;
    for idx in indices {
        result.push(s[last..idx].trim().to_string());
        // 跳过关键字本身和后续空格
        let mut kw_end = idx;
        while kw_end < s.len() && s.as_bytes()[kw_end] != b' ' {
            kw_end += 1;
        }
        last = kw_end;
    }
    if last < s.len() {
        result.push(s[last..].trim().to_string());
    }
    result.retain(|s| !s.is_empty());
    if result.is_empty() {
        vec![s.to_string()]
    } else {
        result
    }
}

#[cfg(test)]
mod tests {
    use super::split_artist_names;

    #[test]
    fn test_split_single_artist() {
        assert_eq!(split_artist_names(Some("周杰伦")), vec!["周杰伦"]);
    }

    #[test]
    fn test_split_ampersand() {
        assert_eq!(
            split_artist_names(Some("周杰伦 & 方文山")),
            vec!["周杰伦", "方文山"]
        );
    }

    #[test]
    fn test_split_slash_with_spaces() {
        assert_eq!(
            split_artist_names(Some("A / B / C")),
            vec!["A", "B", "C"]
        );
    }

    #[test]
    fn test_no_split_ac_dc() {
        // AC/DC 不应被拆分（`/` 两侧无空格）
        assert_eq!(split_artist_names(Some("AC/DC")), vec!["AC/DC"]);
    }

    #[test]
    fn test_split_feat() {
        assert_eq!(
            split_artist_names(Some("A feat. B")),
            vec!["A", "B"]
        );
    }

    #[test]
    fn test_split_ft() {
        assert_eq!(
            split_artist_names(Some("A ft. B")),
            vec!["A", "B"]
        );
    }

    #[test]
    fn test_split_chinese_comma() {
        assert_eq!(
            split_artist_names(Some("周杰伦、方文山")),
            vec!["周杰伦", "方文山"]
        );
    }

    #[test]
    fn test_split_none() {
        assert_eq!(split_artist_names(None), vec!["未知艺术家"]);
    }

    #[test]
    fn test_split_empty() {
        assert_eq!(split_artist_names(Some("")), vec!["未知艺术家"]);
    }

    #[test]
    fn test_split_mixed() {
        assert_eq!(
            split_artist_names(Some("A & B feat. C、D")),
            vec!["A", "B", "C", "D"]
        );
    }
}
