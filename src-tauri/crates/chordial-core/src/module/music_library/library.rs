use super::{albums, artists, lyrics, models::*, relations, songs};
use crate::module::music_source::registrar::SourceCleanup;
use crate::module::music_source::types::{EntityType, SourceId};
use crate::module::perf;
use crate::module::storage::persistent::PersistentStore;
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;

/// 一次性写入整个实体表（替代循环内逐条 update 的低效模式）。
///
/// 单次 `store.set` 比循环内 N 次 `songs::update`（每次反序列化整表）快 N 倍。
fn store_set_all<T: Serialize>(store: &PersistentStore, key: &str, value: &T) -> Result<(), String> {
    store.set(key, value)
}

/// 音乐库 — 所有音乐实体的统一管理入口。
///
/// 内部持有 [`PersistentStore`]，启动时自动加载已有数据，
/// 并通过各子模块提供 CRUD、搜索和关系追溯功能。
///
/// # 子模块职责
///
/// | 模块 | 职责 |
/// |------|------|
/// | [`songs`] | 歌曲 CRUD + 搜索 |
/// | [`artists`] | 艺术家 CRUD + 搜索 |
/// | [`albums`] | 专辑 CRUD + 搜索 |
/// | [`lyrics`] | 歌词 CRUD + 搜索 |
/// | [`relations`] | 跨实体关系追溯 |
pub struct MusicLibrary {
    store: PersistentStore,
}

impl MusicLibrary {
    /// 创建音乐库实例，从 `path` 指定的 JSON 文件加载已有数据。
    pub fn new(path: PathBuf) -> Self {
        Self {
            store: PersistentStore::new(path),
        }
    }

    // ── 持久化 ───────────────────────────────────────

    /// 立即将所有未落盘的修改写入磁盘。
    pub fn save(&self) -> Result<(), String> {
        self.store.save()
    }

    /// 仅当存在未保存修改时才写入磁盘。
    pub fn save_if_dirty(&self) -> Result<(), String> {
        self.store.save_if_dirty()
    }

    /// 从磁盘重新加载，丢弃所有未保存的修改。
    pub fn reload(&self) {
        self.store.reload()
    }

    /// 返回持久化存储的文件路径。
    pub fn store_path(&self) -> &std::path::PathBuf {
        self.store.path()
    }

    /// 从内存缓存中删除指定 key（标记脏，下次 save 时落盘）。
    pub fn remove_store_key(&self, key: &str) {
        self.store.remove(key);
    }

    // ── Song ─────────────────────────────────────────

    pub fn song_count(&self) -> usize {
        songs::count(&self.store)
    }

    pub fn get_song(&self, id: &str) -> Option<Song> {
        songs::get(&self.store, id)
    }

    /// 批量按 ID 获取歌曲（O(1) 每条，避免 N 次 IPC 调用）。
    pub fn get_songs_by_ids(&self, ids: &[String]) -> Vec<Song> {
        ids.iter()
            .filter_map(|id| songs::get(&self.store, id))
            .collect()
    }

    pub fn get_all_songs(&self) -> HashMap<String, Song> {
        self.store.get_all_map::<Song>(songs::KEY)
    }

    /// 分页获取歌曲。
    ///
    /// 当前存储为全量 HashMap，分页无法避免全量反序列化，
    /// 但可显著减少通过 IPC 传往前端的 JSON 载荷。
    pub fn get_songs_page(&self, offset: usize, limit: usize) -> Vec<Song> {
        songs::get_page(&self.store, offset, limit)
    }

    /// 添加一首歌曲（智能去重合并 + 自动初始化艺人/专辑）。
    ///
    /// 一次性加载 songs/artists/albums 到内存中处理，避免对每项实体反复反序列化。
    /// 仅当数据实际变化时才写回存储。
    /// 返回库中实际存储的歌曲 ID（合并模式返回已有歌曲 ID，新建模式返回新 ID）。
    pub fn add_song(&self, song: &Song) -> Result<String, String> {
        let _scope = perf::scope("library.add_song");
        let mut all_songs = songs::get_all(&self.store);
        let mut all_artists = artists::get_all(&self.store);
        let mut all_albums = albums::get_all(&self.store);

        let mut song_index = build_song_index(&all_songs);
        let mut artist_name_index = build_artist_name_index(&all_artists);
        let mut album_index = build_album_index(&all_albums);

        let (stored_id, sc, ac, alc) = merge_or_init_song_in_memory(
            song,
            &mut all_songs,
            &mut all_artists,
            &mut all_albums,
            &mut song_index,
            &mut artist_name_index,
            &mut album_index,
        );

        if sc {
            self.store.set("songs", &all_songs)?;
        }
        if ac {
            self.store.set("artists", &all_artists)?;
        }
        if alc {
            self.store.set("albums", &all_albums)?;
        }

        Ok(stored_id)
    }

    pub fn update_song(&self, song: &Song) -> Result<(), String> {
        songs::update(&self.store, song)
    }

    pub fn remove_song(&self, id: &str) -> Result<bool, String> {
        songs::remove(&self.store, id)
    }

    pub fn search_songs(&self, query: &str) -> Vec<Song> {
        let _scope = perf::scope("library.search");
        let artists_map = artists::get_all(&self.store);
        songs::search(&self.store, query, &artists_map)
    }

    // ── Artist ───────────────────────────────────────

    pub fn artist_count(&self) -> usize {
        artists::count(&self.store)
    }

    pub fn get_artist(&self, id: &str) -> Option<Artist> {
        artists::get(&self.store, id)
    }

    /// 批量按 ID 获取艺术家（O(1) 每条，避免 `get_all_artists` 全量反序列化）。
    pub fn get_artists_by_ids(&self, ids: &[String]) -> Vec<Artist> {
        ids.iter()
            .filter_map(|id| artists::get(&self.store, id))
            .collect()
    }

    pub fn get_all_artists(&self) -> HashMap<String, Artist> {
        self.store.get_all_map::<Artist>(artists::KEY)
    }

    /// 分页获取艺术家。
    pub fn get_artists_page(&self, offset: usize, limit: usize) -> Vec<Artist> {
        artists::get_page(&self.store, offset, limit)
    }

    pub fn add_artist(&self, artist: &Artist) -> Result<(), String> {
        artists::add(&self.store, artist)
    }

    pub fn update_artist(&self, artist: &Artist) -> Result<(), String> {
        artists::update(&self.store, artist)
    }

    pub fn remove_artist(&self, id: &str) -> Result<bool, String> {
        artists::remove(&self.store, id)
    }

    pub fn search_artists(&self, query: &str) -> Vec<Artist> {
        artists::search(&self.store, query)
    }

    // ── Album ────────────────────────────────────────

    pub fn album_count(&self) -> usize {
        albums::count(&self.store)
    }

    pub fn get_album(&self, id: &str) -> Option<Album> {
        albums::get(&self.store, id)
    }

    /// 批量按 ID 获取专辑（O(1) 每条，避免 `get_all_albums` 全量反序列化）。
    pub fn get_albums_by_ids(&self, ids: &[String]) -> Vec<Album> {
        ids.iter()
            .filter_map(|id| albums::get(&self.store, id))
            .collect()
    }

    pub fn get_all_albums(&self) -> HashMap<String, Album> {
        // 注：仍需 HashMap 返回（多处调用方 .into_values() 或按 key 查）
        // 用 get_all_map 替代 get::<HashMap> 避免整体反序列化失败 + 预分配容量
        self.store.get_all_map::<Album>(albums::KEY)
    }

    /// 分页获取专辑。
    pub fn get_albums_page(&self, offset: usize, limit: usize) -> Vec<Album> {
        albums::get_page(&self.store, offset, limit)
    }

    /// 获取首页所需的数据：计数 + 少量示例条目。
    ///
    /// 优化：计数用 `count_entries`（O(1) JSON 键数），样本用 `get_page_entries`
    /// 仅反序列化需要的 10/6/8 条，而非全部数千条。
    /// 旧实现 3x `get_all` = ~56ms，新实现 ~2ms。
    pub fn get_home_stats(&self) -> serde_json::Value {
        let _scope = perf::scope("library.get_home_stats");
        let tracks = self.store.count_entries(songs::KEY);
        let artists = self.store.count_entries(artists::KEY);
        let albums = self.store.count_entries(albums::KEY);

        let recent_songs: Vec<Song> = self.store.get_page_entries(songs::KEY, 0, 10);
        let recent_artists: Vec<Artist> = self.store.get_page_entries(artists::KEY, 0, 6);
        let recent_albums: Vec<Album> = self.store.get_page_entries(albums::KEY, 0, 8);

        serde_json::json!({
            "stats": {
                "tracks": tracks,
                "artists": artists,
                "albums": albums,
            },
            "recentTracks": recent_songs,
            "featuredArtists": recent_artists,
            "recentAlbums": recent_albums,
        })
    }

    pub fn add_album(&self, album: &Album) -> Result<(), String> {
        albums::add(&self.store, album)
    }

    pub fn update_album(&self, album: &Album) -> Result<(), String> {
        albums::update(&self.store, album)
    }

    pub fn remove_album(&self, id: &str) -> Result<bool, String> {
        albums::remove(&self.store, id)
    }

    pub fn search_albums(&self, query: &str) -> Vec<Album> {
        let artists_map = artists::get_all(&self.store);
        albums::search(&self.store, query, &artists_map)
    }

    // ── Lyric ────────────────────────────────────────

    pub fn lyric_count(&self) -> usize {
        lyrics::count(&self.store)
    }

    pub fn get_lyric(&self, id: &str) -> Option<Lyric> {
        lyrics::get(&self.store, id)
    }

    pub fn get_all_lyrics(&self) -> HashMap<String, Lyric> {
        lyrics::get_all(&self.store)
    }

    pub fn add_lyric(&self, lyric: &Lyric) -> Result<(), String> {
        lyrics::add(&self.store, lyric)
    }

    pub fn update_lyric(&self, lyric: &Lyric) -> Result<(), String> {
        lyrics::update(&self.store, lyric)
    }

    pub fn remove_lyric(&self, id: &str) -> Result<bool, String> {
        lyrics::remove(&self.store, id)
    }

    pub fn search_lyrics(&self, query: &str) -> Vec<Lyric> {
        lyrics::search(&self.store, query)
    }

    // ── Relations ────────────────────────────────────

    /// 获取歌曲的艺术家列表。
    pub fn get_artists_of_song(&self, song_id: &str) -> Vec<Artist> {
        relations::get_artists_of_song(&self.store, song_id)
    }

    /// 获取歌曲所属的专辑。
    pub fn get_album_of_song(&self, song_id: &str) -> Option<Album> {
        relations::get_album_of_song(&self.store, song_id)
    }

    /// 获取歌曲的歌词。
    pub fn get_lyric_of_song(&self, song_id: &str) -> Option<Lyric> {
        relations::get_lyric_of_song(&self.store, song_id)
    }

    /// 获取某艺术家的所有歌曲。
    pub fn get_songs_by_artist(&self, artist_id: &str) -> Vec<Song> {
        relations::get_songs_by_artist(&self.store, artist_id)
    }

    /// 获取某艺术家的所有专辑。
    pub fn get_albums_by_artist(&self, artist_id: &str) -> Vec<Album> {
        relations::get_albums_by_artist(&self.store, artist_id)
    }

    /// 获取专辑中的所有歌曲。
    pub fn get_songs_in_album(&self, album_id: &str) -> Vec<Song> {
        relations::get_songs_in_album(&self.store, album_id)
    }

    // ── Sources ──────────────────────────────────────

    /// 获取歌曲的所有来源 ID。
    pub fn get_source_ids_of_song(&self, song_id: &str) -> Vec<crate::module::music_source::types::SourceId> {
        relations::get_source_ids_of_song(&self.store, song_id)
    }

    /// 从所有实体中移除指定来源的 `SourceId`。
    ///
    /// 对每类实体（Song / Artist / Album / Lyric）：
    /// - 过滤掉 `source_name` 匹配的 `SourceId`
    /// - 若某实体的 `source_ids` 全部被移除，则删除该实体
    ///
    /// 此方法由 [`SourceCleanup`] 回调调用，参见
    /// [`SourceRegistrar::unregister`](crate::module::music_source::registrar::SourceRegistrar::unregister)。
    pub fn remove_source_from_all_entities(&self, source_name: &str) -> Result<(), String> {
        let _scope = perf::scope("library.remove_source_from_all_entities");
        // ── Songs ──（批量内存修改，最后一次 set，避免循环内重复反序列化整表）
        {
            let mut all_songs = songs::get_all(&self.store);
            let mut to_remove: Vec<String> = Vec::with_capacity(all_songs.len() / 8 + 1);
            for (id, song) in all_songs.iter_mut() {
                let before = song.source_ids.len();
                song.source_ids.retain(|sid| sid.source_name != source_name);
                if song.source_ids.len() < before && song.source_ids.is_empty() {
                    to_remove.push(id.clone());
                }
            }
            for id in &to_remove {
                all_songs.remove(id);
            }
            // 任何 source_ids 被裁剪过都写回（包括未变空只是缩短的）
            store_set_all(&self.store, songs::KEY, &all_songs)?;
        }

        // ── Artists ──
        {
            let mut all_artists = artists::get_all(&self.store);
            let mut to_remove: Vec<String> = Vec::with_capacity(all_artists.len() / 8 + 1);
            for (id, artist) in all_artists.iter_mut() {
                let before = artist.source_ids.len();
                artist.source_ids.retain(|sid| sid.source_name != source_name);
                if artist.source_ids.len() < before && artist.source_ids.is_empty() {
                    to_remove.push(id.clone());
                }
            }
            for id in &to_remove {
                all_artists.remove(id);
            }
            store_set_all(&self.store, artists::KEY, &all_artists)?;
        }

        // ── Albums ──
        {
            let mut all_albums = albums::get_all(&self.store);
            let mut to_remove: Vec<String> = Vec::with_capacity(all_albums.len() / 8 + 1);
            for (id, album) in all_albums.iter_mut() {
                let before = album.source_ids.len();
                album.source_ids.retain(|sid| sid.source_name != source_name);
                if album.source_ids.len() < before && album.source_ids.is_empty() {
                    to_remove.push(id.clone());
                }
            }
            for id in &to_remove {
                all_albums.remove(id);
            }
            store_set_all(&self.store, albums::KEY, &all_albums)?;
        }

        // ── Lyrics ──
        {
            let mut all_lyrics = lyrics::get_all(&self.store);
            let before_len = all_lyrics.len();
            all_lyrics.retain(|_, lyric| lyric.source_id.source_name != source_name);
            if all_lyrics.len() != before_len {
                store_set_all(&self.store, lyrics::KEY, &all_lyrics)?;
            }
        }

        self.save()
    }

    /// 精准移除：从歌曲中删除匹配的 SourceId，并清理因此变为空的实体。
    ///
    /// 对每首歌曲，若其 `source_ids` 中有 `(source_name, entity_id)` 匹配项，
    /// 则移除该 SourceId；若歌曲的 `source_ids` 因此变空，则删除该歌曲。
    ///
    /// 随后调用 [`cleanup_empty_entities`](Self::cleanup_empty_entities)
    /// 清理所有失去全部来源引用的艺人、专辑、歌词。
    ///
    /// 适用场景：本地来源的某个文件被删除，仅移除该文件的引用，
    /// 而不影响同一来源的其他文件。
    pub fn remove_specific_song_source_ids(
        &self,
        source_name: &str,
        entity_ids: &std::collections::HashSet<String>,
    ) -> Result<(), String> {
        let _scope = perf::scope("library.remove_specific_song_source_ids");
        if entity_ids.is_empty() {
            return Ok(());
        }

        let mut all_songs = songs::get_all(&self.store);
        let mut to_remove: Vec<String> = Vec::with_capacity(all_songs.len() / 8 + 1);
        let mut any_changed = false;

        for (id, song) in all_songs.iter_mut() {
            let before = song.source_ids.len();
            song.source_ids.retain(|sid| {
                !(sid.source_name == source_name && entity_ids.contains(&sid.entity_id))
            });
            if song.source_ids.len() < before {
                any_changed = true;
                if song.source_ids.is_empty() {
                    to_remove.push(id.clone());
                }
            }
        }
        for id in &to_remove {
            all_songs.remove(id);
        }
        if any_changed {
            store_set_all(&self.store, songs::KEY, &all_songs)?;
        }

        // 清理级联空实体
        self.cleanup_empty_entities()?;
        self.save()
    }

    /// 清理所有 `source_ids` 为空的实体（Song / Artist / Album / Lyric）。
    ///
    /// 遍历全部四类实体，删除所有失去全部来源引用的条目。
    /// 适用于：文件删除、文件夹移除后确保库中无悬挂数据。
    pub fn cleanup_empty_entities(&self) -> Result<(), String> {
        let _scope = perf::scope("library.cleanup_empty_entities");
        // Songs（批量：内存中 retain + 一次 set）
        {
            let mut all_songs = songs::get_all(&self.store);
            let before = all_songs.len();
            all_songs.retain(|_, s| !s.source_ids.is_empty());
            if all_songs.len() != before {
                store_set_all(&self.store, songs::KEY, &all_songs)?;
            }
        }
        // Artists
        {
            let mut all_artists = artists::get_all(&self.store);
            let before = all_artists.len();
            all_artists.retain(|_, a| !a.source_ids.is_empty());
            if all_artists.len() != before {
                store_set_all(&self.store, artists::KEY, &all_artists)?;
            }
        }
        // Albums
        {
            let mut all_albums = albums::get_all(&self.store);
            let before = all_albums.len();
            all_albums.retain(|_, a| !a.source_ids.is_empty());
            if all_albums.len() != before {
                store_set_all(&self.store, albums::KEY, &all_albums)?;
            }
        }
        // Lyrics
        {
            let mut all_lyrics = lyrics::get_all(&self.store);
            let before = all_lyrics.len();
            all_lyrics.retain(|_, l| !l.source_id.source_name.is_empty());
            if all_lyrics.len() != before {
                store_set_all(&self.store, lyrics::KEY, &all_lyrics)?;
            }
        }
        Ok(())
    }

    /// 批量添加歌曲（智能去重合并 + 自动初始化艺人/专辑）。
    ///
    /// 一次性加载全部数据，预建 O(1) 查重索引后逐首合并。
    /// 仅当数据实际变化时才序列化写回对应集合。
    ///
    /// 返回与输入等长的 `Vec<String>`，每个元素为对应歌曲在库中的实际存储 ID。
    pub fn add_songs_batch(&self, songs: &[Song]) -> Result<Vec<String>, String> {
        let _scope = perf::scope("library.add_songs_batch");
        let mut all_songs = songs::get_all(&self.store);
        let mut all_artists = artists::get_all(&self.store);
        let mut all_albums = albums::get_all(&self.store);

        let mut song_index = build_song_index(&all_songs);
        let mut artist_name_index = build_artist_name_index(&all_artists);
        let mut album_index = build_album_index(&all_albums);

        let mut songs_changed = false;
        let mut artists_changed = false;
        let mut albums_changed = false;
        let mut stored_ids = Vec::with_capacity(songs.len());

        for song in songs {
            let (stored_id, sc, ac, alc) = merge_or_init_song_in_memory(
                song,
                &mut all_songs,
                &mut all_artists,
                &mut all_albums,
                &mut song_index,
                &mut artist_name_index,
                &mut album_index,
            );
            stored_ids.push(stored_id);
            songs_changed |= sc;
            artists_changed |= ac;
            albums_changed |= alc;
        }

        if songs_changed {
            self.store.set("songs", &all_songs)?;
        }
        if artists_changed {
            self.store.set("artists", &all_artists)?;
        }
        if albums_changed {
            self.store.set("albums", &all_albums)?;
        }

        Ok(stored_ids)
    }

    // ── 私有辅助 ─────────────────────────────────────

}

/// 在内存中合并或插入一首歌。
///
/// 使用预建索引实现 O(1) 去重查找。索引在新建条目时会同步更新。
///
/// 返回 `(stored_id, songs_changed, artists_changed, albums_changed)`。
fn merge_or_init_song_in_memory(
    song: &Song,
    all_songs: &mut HashMap<String, Song>,
    all_artists: &mut HashMap<String, Artist>,
    all_albums: &mut HashMap<String, Album>,
    song_index: &mut HashMap<(String, Vec<String>), String>,
    artist_name_index: &mut HashMap<String, String>,
    album_index: &mut HashMap<(String, String), String>,
) -> (String, bool, bool, bool) {
    let title_lower = song.title.to_lowercase();
    let mut names_sorted: Vec<String> = song.artist_names.iter().map(|n| n.to_lowercase()).collect();
    names_sorted.sort();
    let lookup_key = (title_lower, names_sorted);

    if let Some(existing_id) = song_index.get(&lookup_key).cloned() {
        // ── 合并模式 ──
        let mut songs_changed = false;

        if let Some(existing) = all_songs.get_mut(&existing_id) {
            let sid_before = existing.source_ids.len();
            merge_source_ids(&mut existing.source_ids, &song.source_ids);
            if existing.source_ids.len() > sid_before {
                songs_changed = true;
            }
            for aid in &song.artist_ids {
                if !existing.artist_ids.contains(aid) {
                    existing.artist_ids.push(aid.clone());
                    songs_changed = true;
                }
            }
            if existing.album_id.is_none() {
                existing.album_id = song.album_id.clone();
                existing.album_title = song.album_title.clone();
                songs_changed = true;
            }
        }

        let artists_changed = merge_artists_in_memory(
            &song.artist_ids,
            &song.artist_names,
            &song.source_ids,
            all_artists,
            artist_name_index,
        );

        let mut albums_changed = false;
        if let (Some(album_id), Some(album_title)) = (&song.album_id, &song.album_title) {
            albums_changed = merge_album_in_memory(
                album_id,
                album_title,
                &song.artist_ids,
                &song.source_ids,
                &existing_id,
                all_albums,
                album_index,
            );
        }

        (existing_id, songs_changed, artists_changed, albums_changed)
    } else {
        // ── 新建模式 ──
        all_songs.insert(song.id.clone(), song.clone());
        song_index.insert(lookup_key, song.id.clone());

        let artists_changed = merge_artists_in_memory(
            &song.artist_ids,
            &song.artist_names,
            &song.source_ids,
            all_artists,
            artist_name_index,
        );

        let mut albums_changed = false;
        if let (Some(album_id), Some(album_title)) = (&song.album_id, &song.album_title) {
            albums_changed = merge_album_in_memory(
                album_id,
                album_title,
                &song.artist_ids,
                &song.source_ids,
                &song.id,
                all_albums,
                album_index,
            );
        }

        (song.id.clone(), true, artists_changed, albums_changed)
    }
}

/// 在内存中合并或创建艺人，使用名称索引 O(1) 查找。返回是否有变化。
fn merge_artists_in_memory(
    artist_ids: &[String],
    artist_names: &[String],
    song_source_ids: &[SourceId],
    all_artists: &mut HashMap<String, Artist>,
    artist_name_index: &mut HashMap<String, String>,
) -> bool {
    let artist_sids: Vec<SourceId> = song_source_ids
        .iter()
        .map(|s| s.with_entity_type(EntityType::Artist))
        .collect();
    let mut changed = false;

    for (i, artist_id) in artist_ids.iter().enumerate() {
        let artist_name = artist_names.get(i).map(|s| s.as_str()).unwrap_or("");

        if let Some(artist) = all_artists.get_mut(artist_id) {
            let sid_before = artist.source_ids.len();
            merge_source_ids(&mut artist.source_ids, &artist_sids);
            if artist.source_ids.len() > sid_before {
                changed = true;
            }
        } else if let Some(aid) = artist_name_index.get(&artist_name.to_lowercase()).cloned() {
            if let Some(artist) = all_artists.get_mut(&aid) {
                let sid_before = artist.source_ids.len();
                merge_source_ids(&mut artist.source_ids, &artist_sids);
                if artist.source_ids.len() > sid_before {
                    changed = true;
                }
            }
        } else {
            all_artists.insert(
                artist_id.clone(),
                Artist {
                    id: artist_id.clone(),
                    name: artist_name.to_string(),
                    bio: None,
                    source_ids: artist_sids.clone(),
                },
            );
            artist_name_index.insert(artist_name.to_lowercase(), artist_id.clone());
            changed = true;
        }
    }

    changed
}

/// 在内存中合并或创建专辑，使用 (title, artist_id) 索引 O(1) 查找。返回是否有变化。
fn merge_album_in_memory(
    album_id: &str,
    album_title: &str,
    artist_ids: &[String],
    song_source_ids: &[SourceId],
    song_id: &str,
    all_albums: &mut HashMap<String, Album>,
    album_index: &mut HashMap<(String, String), String>,
) -> bool {
    let album_sids: Vec<SourceId> = song_source_ids
        .iter()
        .map(|s| s.with_entity_type(EntityType::Album))
        .collect();
    let artist_id = artist_ids.first().map(|s| s.as_str()).unwrap_or("");
    let lookup_key = (album_title.to_lowercase(), artist_id.to_string());
    let mut changed = false;

    if let Some(album) = all_albums.get_mut(album_id) {
        let sid_before = album.source_ids.len();
        merge_source_ids(&mut album.source_ids, &album_sids);
        if album.source_ids.len() > sid_before {
            changed = true;
        }
        if !album.song_ids.iter().any(|s| s == song_id) {
            album.song_ids.push(song_id.to_string());
            changed = true;
        }
    } else if let Some(aid) = album_index.get(&lookup_key).cloned() {
        if let Some(album) = all_albums.get_mut(&aid) {
            let sid_before = album.source_ids.len();
            merge_source_ids(&mut album.source_ids, &album_sids);
            if album.source_ids.len() > sid_before {
                changed = true;
            }
            if !album.song_ids.iter().any(|s| s == song_id) {
                album.song_ids.push(song_id.to_string());
                changed = true;
            }
        }
    } else {
        all_albums.insert(
            album_id.to_string(),
            Album {
                id: album_id.to_string(),
                title: album_title.to_string(),
                artist_id: artist_id.to_string(),
                cover_url: None,
                song_ids: vec![song_id.to_string()],
                source_ids: album_sids,
            },
        );
        album_index.insert(lookup_key, album_id.to_string());
        changed = true;
    }

    changed
}

/// 构建歌曲去重索引：(title_lower, sorted_artist_names_lower) → song_id。
fn build_song_index(all_songs: &HashMap<String, Song>) -> HashMap<(String, Vec<String>), String> {
    let mut index = HashMap::with_capacity(all_songs.len());
    for (id, song) in all_songs {
        let title_lower = song.title.to_lowercase();
        let mut names: Vec<String> = song.artist_names.iter().map(|n| n.to_lowercase()).collect();
        names.sort();
        index.insert((title_lower, names), id.clone());
    }
    index
}

/// 构建艺人名称索引：name_lower → artist_id。
fn build_artist_name_index(all_artists: &HashMap<String, Artist>) -> HashMap<String, String> {
    let mut index = HashMap::with_capacity(all_artists.len());
    for (id, artist) in all_artists {
        index.insert(artist.name.to_lowercase(), id.clone());
    }
    index
}

/// 构建专辑索引：(title_lower, artist_id) → album_id。
fn build_album_index(all_albums: &HashMap<String, Album>) -> HashMap<(String, String), String> {
    let mut index = HashMap::with_capacity(all_albums.len());
    for (id, album) in all_albums {
        index.insert((album.title.to_lowercase(), album.artist_id.clone()), id.clone());
    }
    index
}

/// 将 `incoming` 中不存在于 `existing` 的 SourceId 追加进去（去重合并）。
///
/// 优化：existing 较大时使用 HashSet 临时索引，避免 O(n²) 扫描。
fn merge_source_ids(existing: &mut Vec<SourceId>, incoming: &[SourceId]) {
    if existing.is_empty() {
        existing.extend_from_slice(incoming);
        return;
    }
    // existing 较大（>16）时构建 HashSet 索引；小切片直接线性扫描更快
    if existing.len() > 16 {
        // 先收集待追加项（避免在迭代 incoming 时同时可变借用 existing）
        let set: std::collections::HashSet<&SourceId> =
            existing.iter().collect();
        let to_add: Vec<SourceId> = incoming
            .iter()
            .filter(|sid| !set.contains(sid))
            .cloned()
            .collect();
        existing.extend(to_add);
    } else {
        for sid in incoming {
            if !existing.contains(sid) {
                existing.push(sid.clone());
            }
        }
    }
}

// ── SourceCleanup 实现 ────────────────────────────

impl SourceCleanup for MusicLibrary {
    fn remove_source_from_all_entities(&self, source_name: &str) -> Result<(), String> {
        MusicLibrary::remove_source_from_all_entities(self, source_name)
    }
}
