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
use crate::module::music_source::traits::MusicSource;
use crate::module::music_source::types::{EntityType, SourceId, SourceType};
use crate::module::platform::{self, PlatformPath};
use crate::module::storage::persistent::PersistentStore;
use parking_lot::RwLock;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use uuid::Uuid;

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
        }
    }

    // ── 内部辅助方法 ─────────────────────────────────

    /// 扫描单个音频文件并添加到音乐库（或合并到已有条目）。
    ///
    /// 返回 `true` 表示成功处理（新增或合并），`false` 表示跳过（非音频文件）。
    pub fn index_file(&self, path: &PlatformPath) -> Result<bool, String> {
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

        // 构建 Song
        let song = self.build_song(&canonical, &meta);

        // 添加到音乐库（自动去重合并），获取实际存储的 ID
        let stored_id = self.library.add_song(&song)?;

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
        // 先卸载旧索引
        self.unindex_file(path)?;
        // 再重新索引
        self.index_file(path)
    }

    /// 从 AudioMeta 构建 Song 模型。
    pub fn build_song(&self, file_path: &PlatformPath, meta: &AudioMeta) -> Song {
        let entity_id = platform::path_to_string(file_path);
        let song_id = Uuid::new_v4().to_string();
        let artist_name = meta.artist.clone().unwrap_or_else(|| "未知艺术家".to_string());
        let artist_id = Uuid::new_v4().to_string();
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
            artist_names: vec![artist_name],
            album_title,
            duration: meta.duration_secs,
            artist_ids: vec![artist_id],
            album_id,
            lyric_id,
            source_ids: vec![source_id],
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

        // 收集需要 canonicalize 的路径
        let mut to_canonicalize: Vec<(String, String)> = Vec::new();
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
            let mut handles = Vec::new();
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
}

impl MusicSource for LocalMusicSource {
    fn name(&self) -> &str {
        &self.name
    }

    fn source_type(&self) -> SourceType {
        SourceType::Local
    }

    fn search_songs(&self, query: &str) -> Result<Vec<Song>, String> {
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        let paths: Vec<PlatformPath> = self.file_index.read().keys().cloned().collect();
        for path in paths {
            if let Ok(meta) = scanner::probe_file(&path) {
                let title = meta.title.as_deref().unwrap_or("");
                let artist = meta.artist.as_deref().unwrap_or("");
                if title.to_lowercase().contains(&query_lower)
                    || artist.to_lowercase().contains(&query_lower)
                {
                    let song = self.build_song(&path, &meta);
                    results.push(song);
                }
            }
        }
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
        let path = PlatformPath::from(entity_id);

        // 1. 若为音频文件，尝试提取嵌入封面（FLAC / ID3v2）
        if platform::is_file(&path) && super::scanner::is_supported_audio(&path) {
            if let Ok(cover_data) = super::scanner::extract_cover_art(&path) {
                return Ok(cover_data);
            }
        }

        // 2. 确定搜索目录
        let search_dir = if platform::is_dir(&path) {
            path.clone()
        } else if let Some(parent) = platform::path_parent(&path) {
            parent
        } else {
            return Err(format!("无法确定搜索目录: {}", platform::path_to_string(&path)));
        };

        // 3. 同目录下与音频文件同名的图片文件
        if !platform::is_dir(&path) {
            for ext in &["jpg", "jpeg", "png", "webp", "bmp"] {
                let sibling = platform::path_with_extension(&path, ext);
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

        Err(format!("未找到封面图片: {}", platform::path_to_string(&path)))
    }

    fn lyric_text_get(&self, song_id: &str) -> Result<String, String> {
        // 尝试查找与音频文件同名的 .lrc 文件
        let audio_path = if let Some(p) = self.id_to_path.read().get(song_id) {
            p.clone()
        } else {
            PlatformPath::from(song_id)
        };

        let lrc_path = platform::path_with_extension(&audio_path, "lrc");
        if platform::exists(&lrc_path) {
            let bytes = platform::read_bytes(&lrc_path)
                .map_err(|e| format!("读取歌词文件失败 '{}': {}", platform::path_to_string(&lrc_path), e))?;
            return String::from_utf8(bytes)
                .map_err(|e| format!("歌词文件编码无效: {}", e));
        }

        // 也尝试 .txt 扩展名
        let txt_path = platform::path_with_extension(&audio_path, "txt");
        if platform::exists(&txt_path) {
            let bytes = platform::read_bytes(&txt_path)
                .map_err(|e| format!("读取歌词文件失败 '{}': {}", platform::path_to_string(&txt_path), e))?;
            return String::from_utf8(bytes)
                .map_err(|e| format!("歌词文件编码无效: {}", e));
        }

        Err(format!("未找到歌词文件: {}", platform::path_to_string(&audio_path)))
    }
}
