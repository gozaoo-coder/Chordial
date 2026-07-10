use super::models::Song;
use crate::module::perf;
use crate::module::storage::persistent::PersistentStore;
use std::collections::{HashMap, HashSet};

pub const KEY: &str = "songs";

/// 获取所有歌曲。
pub fn get_all(store: &PersistentStore) -> HashMap<String, Song> {
    let _scope = perf::scope("songs.get_all");
    store.get::<HashMap<String, Song>>(KEY).unwrap_or_default()
}

/// 按 ID 获取单首歌曲。
///
/// 优化：仅反序列化目标条目，不反序列化整个 HashMap。
pub fn get(store: &PersistentStore, id: &str) -> Option<Song> {
    let _scope = perf::scope("songs.get");
    store.get_entry::<Song>(KEY, id)
}

/// 查找重复歌曲：标题相同（忽略大小写）且艺人名集合相同（无序匹配）。
///
/// 返回找到的第一首匹配歌曲，若不存在则返回 `None`。
///
/// 优化：JSON 层预过滤（标题等值），仅对候选条目反序列化并做集合比较。
pub fn find_duplicate(store: &PersistentStore, title: &str, artist_names: &[String]) -> Option<Song> {
    let title_lower = title.to_lowercase();
    let names_set: HashSet<String> = artist_names.iter().map(|n| n.to_lowercase()).collect();

    // 先按标题等值过滤（不区分大小写），再在反序列化后的候选集上做集合比较。
    // 候选集通常 ≤ 5 条（同名歌曲极少），集合比较开销可忽略。
    let candidates = store.get_entries_filtered::<Song, _>(KEY, |v| {
        v.get("title")
            .and_then(|t| t.as_str())
            .map_or(false, |t| t.to_lowercase() == title_lower)
    });

    candidates.into_iter().find(|s| {
        let existing_set: HashSet<String> =
            s.artist_names.iter().map(|n| n.to_lowercase()).collect();
        existing_set == names_set
    })
}

/// 添加一首歌曲（纯插入，不做去重）。
///
/// 若 ID 已存在则返回 `Err`。推荐使用上层 [`super::library::MusicLibrary::add_song`]
/// 以获得去重合并和自动初始化艺人/专辑的能力。
///
/// 优化：存在性检查用 `has_entry`，插入用 `set_subkey`。
pub fn add(store: &PersistentStore, song: &Song) -> Result<(), String> {
    if store.has_entry(KEY, &song.id) {
        return Err(format!("歌曲 '{}' (id={}) 已存在", song.title, song.id));
    }
    store.set_subkey(KEY, &song.id, song)
}

/// 更新一首歌曲（按 ID 覆盖）。
///
/// 若歌曲不存在则返回 `Err`。
///
/// 优化：存在性检查用 `has_entry`，插入用 `set_subkey`。
pub fn update(store: &PersistentStore, song: &Song) -> Result<(), String> {
    let _scope = perf::scope("songs.update");
    if !store.has_entry(KEY, &song.id) {
        return Err(format!("歌曲 id={} 不存在", song.id));
    }
    store.set_subkey(KEY, &song.id, song)
}

/// 删除一首歌曲。
///
/// 返回 `true` 表示存在并被删除。
///
/// 优化：直接 `remove_entry` 操作 JSON Object 键。
pub fn remove(store: &PersistentStore, id: &str) -> Result<bool, String> {
    let _scope = perf::scope("songs.remove");
    Ok(store.remove_entry(KEY, id))
}

/// 按标题/艺术家名模糊搜索歌曲。
///
/// 匹配范围：歌曲标题 + 关联的艺术家名称。
///
/// 优化：JSON 层过滤，仅反序列化匹配项；艺术家名通过 artists map 查找。
pub fn search(store: &PersistentStore, query: &str, artists: &HashMap<String, super::models::Artist>) -> Vec<Song> {
    let _scope = perf::scope("songs.search");
    let query_lower = query.to_lowercase();
    store.get_entries_filtered::<Song, _>(KEY, |v| {
        let title_match = v
            .get("title")
            .and_then(|t| t.as_str())
            .map_or(false, |t| t.to_lowercase().contains(&query_lower));
        if title_match {
            return true;
        }
        // 艺术家名匹配：检查 artist_ids 数组中任一 artist 名称匹配
        v.get("artist_ids")
            .and_then(|a| a.as_array())
            .map_or(false, |arr| {
                arr.iter().any(|aid| {
                    aid.as_str()
                        .and_then(|id| artists.get(id))
                        .map(|a| a.name.to_lowercase().contains(&query_lower))
                        .unwrap_or(false)
                })
            })
    })
}

/// 分页获取歌曲。
///
/// 优化：仅反序列化 [offset, offset+limit) 范围的条目。
pub fn get_page(store: &PersistentStore, offset: usize, limit: usize) -> Vec<Song> {
    let _scope = perf::scope("songs.get_page");
    store.get_page_entries::<Song>(KEY, offset, limit)
}

/// 获取歌曲总数。
///
/// 优化：O(1) 检查 JSON Object 键数量，不反序列化。
pub fn count(store: &PersistentStore) -> usize {
    store.count_entries(KEY)
}
