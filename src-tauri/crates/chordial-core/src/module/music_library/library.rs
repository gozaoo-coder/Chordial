use super::{albums, artists, lyrics, models::*, relations, songs};
use crate::module::music_source::registrar::SourceCleanup;
use crate::module::music_source::types::{EntityType, SourceId};
use crate::module::storage::persistent::PersistentStore;
use std::collections::HashMap;
use std::path::PathBuf;

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

    // ── Song ─────────────────────────────────────────

    pub fn song_count(&self) -> usize {
        songs::count(&self.store)
    }

    pub fn get_song(&self, id: &str) -> Option<Song> {
        songs::get(&self.store, id)
    }

    pub fn get_all_songs(&self) -> HashMap<String, Song> {
        songs::get_all(&self.store)
    }

    /// 添加一首歌曲（智能去重合并 + 自动初始化艺人/专辑）。
    ///
    /// # 行为
    ///
    /// - **去重检测**：若库中已存在标题 + 艺人名集合完全相同的歌曲，则进入合并模式。
    /// - **合并模式**：将新 song 的 `source_ids` 合并到已有歌曲、艺人、专辑的 `source_ids` 中；
    ///   同时将 song.id 加入专辑的 `song_ids`。
    /// - **新建模式**：直接插入歌曲，并为尚未存在的艺人、专辑自动创建初始数据
    ///   （名称取自 `song.artist_names` / `song.album_title`，来源 ID 由歌曲的 source_ids 派生）。
    pub fn add_song(&self, song: &Song) -> Result<(), String> {
        // 1. 查找重复歌曲
        if let Some(mut existing) =
            songs::find_duplicate(&self.store, &song.title, &song.artist_names)
        {
            // ── 合并模式 ──

            // a) 合并 source_ids + artist_ids 到已有歌曲
            merge_source_ids(&mut existing.source_ids, &song.source_ids);
            for aid in &song.artist_ids {
                if !existing.artist_ids.contains(aid) {
                    existing.artist_ids.push(aid.clone());
                }
            }
            if existing.album_id.is_none() {
                existing.album_id = song.album_id.clone();
                existing.album_title = song.album_title.clone();
            }
            songs::update(&self.store, &existing)?;

            // b) 合并到艺人
            self.merge_or_init_artists(
                &song.artist_ids,
                &song.artist_names,
                &song.source_ids,
            )?;

            // c) 合并到专辑
            if let (Some(album_id), Some(album_title)) =
                (&song.album_id, &song.album_title)
            {
                let artist_id = song.artist_ids.first().map(|s| s.as_str()).unwrap_or("");
                self.merge_or_init_album(
                    album_id,
                    album_title,
                    artist_id,
                    &song.source_ids,
                    &existing.id,
                )?;
            }
        } else {
            // ── 新建模式 ──

            // a) 插入歌曲
            songs::add(&self.store, song)?;

            // b) 确保艺人存在
            self.merge_or_init_artists(
                &song.artist_ids,
                &song.artist_names,
                &song.source_ids,
            )?;

            // c) 确保专辑存在
            if let (Some(album_id), Some(album_title)) =
                (&song.album_id, &song.album_title)
            {
                let artist_id = song.artist_ids.first().map(|s| s.as_str()).unwrap_or("");
                self.merge_or_init_album(
                    album_id,
                    album_title,
                    artist_id,
                    &song.source_ids,
                    &song.id,
                )?;
            }
        }

        Ok(())
    }

    pub fn update_song(&self, song: &Song) -> Result<(), String> {
        songs::update(&self.store, song)
    }

    pub fn remove_song(&self, id: &str) -> Result<bool, String> {
        songs::remove(&self.store, id)
    }

    pub fn search_songs(&self, query: &str) -> Vec<Song> {
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

    pub fn get_all_artists(&self) -> HashMap<String, Artist> {
        artists::get_all(&self.store)
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

    pub fn get_all_albums(&self) -> HashMap<String, Album> {
        albums::get_all(&self.store)
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
        // ── Songs ──
        {
            let mut all_songs = songs::get_all(&self.store);
            let mut to_remove: Vec<String> = Vec::new();
            for (id, song) in all_songs.iter_mut() {
                let before = song.source_ids.len();
                song.source_ids.retain(|sid| sid.source_name != source_name);
                if song.source_ids.len() < before {
                    if song.source_ids.is_empty() {
                        to_remove.push(id.clone());
                    } else {
                        songs::update(&self.store, song)?;
                    }
                }
            }
            for id in &to_remove {
                let _ = songs::remove(&self.store, id);
            }
        }

        // ── Artists ──
        {
            let mut all_artists = artists::get_all(&self.store);
            let mut to_remove: Vec<String> = Vec::new();
            for (id, artist) in all_artists.iter_mut() {
                let before = artist.source_ids.len();
                artist.source_ids.retain(|sid| sid.source_name != source_name);
                if artist.source_ids.len() < before {
                    if artist.source_ids.is_empty() {
                        to_remove.push(id.clone());
                    } else {
                        artists::update(&self.store, artist)?;
                    }
                }
            }
            for id in &to_remove {
                let _ = artists::remove(&self.store, id);
            }
        }

        // ── Albums ──
        {
            let mut all_albums = albums::get_all(&self.store);
            let mut to_remove: Vec<String> = Vec::new();
            for (id, album) in all_albums.iter_mut() {
                let before = album.source_ids.len();
                album.source_ids.retain(|sid| sid.source_name != source_name);
                if album.source_ids.len() < before {
                    if album.source_ids.is_empty() {
                        to_remove.push(id.clone());
                    } else {
                        albums::update(&self.store, album)?;
                    }
                }
            }
            for id in &to_remove {
                let _ = albums::remove(&self.store, id);
            }
        }

        // ── Lyrics ──
        {
            let all_lyrics = lyrics::get_all(&self.store);
            let mut to_remove: Vec<String> = Vec::new();
            for (id, lyric) in all_lyrics.iter() {
                if lyric.source_id.source_name == source_name {
                    to_remove.push(id.clone());
                }
            }
            for id in &to_remove {
                let _ = lyrics::remove(&self.store, id);
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
        if entity_ids.is_empty() {
            return Ok(());
        }

        let all_songs = songs::get_all(&self.store);
        let mut to_update: Vec<Song> = Vec::new();
        let mut to_remove: Vec<String> = Vec::new();

        for (id, mut song) in all_songs {
            let before = song.source_ids.len();
            song.source_ids.retain(|sid| {
                !(sid.source_name == source_name && entity_ids.contains(&sid.entity_id))
            });
            if song.source_ids.len() < before {
                if song.source_ids.is_empty() {
                    to_remove.push(id);
                } else {
                    to_update.push(song);
                }
            }
        }

        for song in &to_update {
            songs::update(&self.store, song)?;
        }
        for id in &to_remove {
            let _ = songs::remove(&self.store, id);
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
        // Songs
        {
            let all_songs = songs::get_all(&self.store);
            let empty_ids: Vec<String> = all_songs
                .iter()
                .filter(|(_, s)| s.source_ids.is_empty())
                .map(|(id, _)| id.clone())
                .collect();
            for id in &empty_ids {
                let _ = songs::remove(&self.store, id);
            }
        }
        // Artists
        {
            let all_artists = artists::get_all(&self.store);
            let empty_ids: Vec<String> = all_artists
                .iter()
                .filter(|(_, a)| a.source_ids.is_empty())
                .map(|(id, _)| id.clone())
                .collect();
            for id in &empty_ids {
                let _ = artists::remove(&self.store, id);
            }
        }
        // Albums
        {
            let all_albums = albums::get_all(&self.store);
            let empty_ids: Vec<String> = all_albums
                .iter()
                .filter(|(_, a)| a.source_ids.is_empty())
                .map(|(id, _)| id.clone())
                .collect();
            for id in &empty_ids {
                let _ = albums::remove(&self.store, id);
            }
        }
        // Lyrics
        {
            let all_lyrics = lyrics::get_all(&self.store);
            let empty_ids: Vec<String> = all_lyrics
                .iter()
                .filter(|(_, l)| l.source_id.source_name.is_empty())
                .map(|(id, _)| id.clone())
                .collect();
            for id in &empty_ids {
                let _ = lyrics::remove(&self.store, id);
            }
        }
        Ok(())
    }

    // ── 私有辅助 ─────────────────────────────────────

    /// 合并或初始化一组艺人：按 ID 查找 → 按名称查找 → 新建。
    fn merge_or_init_artists(
        &self,
        artist_ids: &[String],
        artist_names: &[String],
        song_source_ids: &[SourceId],
    ) -> Result<(), String> {
        let artist_sids: Vec<SourceId> = song_source_ids
            .iter()
            .map(|s| s.with_entity_type(EntityType::Artist))
            .collect();

        for (i, artist_id) in artist_ids.iter().enumerate() {
            let artist_name = artist_names.get(i).map(|s| s.as_str()).unwrap_or("");

            if let Some(mut artist) = artists::get(&self.store, artist_id) {
                // 按 ID 找到 → 合并 source_ids
                merge_source_ids(&mut artist.source_ids, &artist_sids);
                artists::update(&self.store, &artist)?;
            } else if let Some(mut artist) = artists::find_by_name(&self.store, artist_name) {
                // 按名称找到（不同 ID）→ 合并 source_ids
                merge_source_ids(&mut artist.source_ids, &artist_sids);
                artists::update(&self.store, &artist)?;
            } else {
                // 不存在 → 新建
                let artist = Artist {
                    id: artist_id.clone(),
                    name: artist_name.to_string(),
                    bio: None,
                    source_ids: artist_sids.clone(),
                };
                artists::add(&self.store, &artist)?;
            }
        }
        Ok(())
    }

    /// 合并或初始化专辑：按 ID 查找 → 按标题+艺人查找 → 新建。
    fn merge_or_init_album(
        &self,
        album_id: &str,
        album_title: &str,
        artist_id: &str,
        song_source_ids: &[SourceId],
        song_id: &str,
    ) -> Result<(), String> {
        let album_sids: Vec<SourceId> = song_source_ids
            .iter()
            .map(|s| s.with_entity_type(EntityType::Album))
            .collect();

        if let Some(mut album) = albums::get(&self.store, album_id) {
            merge_source_ids(&mut album.source_ids, &album_sids);
            if !album.song_ids.contains(&song_id.to_string()) {
                album.song_ids.push(song_id.to_string());
            }
            albums::update(&self.store, &album)?;
        } else if let Some(mut album) =
            albums::find_by_title_and_artist(&self.store, album_title, artist_id)
        {
            merge_source_ids(&mut album.source_ids, &album_sids);
            if !album.song_ids.contains(&song_id.to_string()) {
                album.song_ids.push(song_id.to_string());
            }
            albums::update(&self.store, &album)?;
        } else {
            let album = Album {
                id: album_id.to_string(),
                title: album_title.to_string(),
                artist_id: artist_id.to_string(),
                cover_url: None,
                song_ids: vec![song_id.to_string()],
                source_ids: album_sids,
            };
            albums::add(&self.store, &album)?;
        }
        Ok(())
    }
}

/// 将 `incoming` 中不存在于 `existing` 的 SourceId 追加进去（去重合并）。
fn merge_source_ids(existing: &mut Vec<SourceId>, incoming: &[SourceId]) {
    for sid in incoming {
        if !existing.contains(sid) {
            existing.push(sid.clone());
        }
    }
}

// ── SourceCleanup 实现 ────────────────────────────

impl SourceCleanup for MusicLibrary {
    fn remove_source_from_all_entities(&self, source_name: &str) -> Result<(), String> {
        MusicLibrary::remove_source_from_all_entities(self, source_name)
    }
}
