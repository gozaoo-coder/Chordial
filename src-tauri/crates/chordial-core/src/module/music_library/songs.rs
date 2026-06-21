use super::models::Song;
use crate::module::storage::persistent::PersistentStore;
use std::collections::{HashMap, HashSet};

const KEY: &str = "songs";

/// 获取所有歌曲。
pub fn get_all(store: &PersistentStore) -> HashMap<String, Song> {
    store.get::<HashMap<String, Song>>(KEY).unwrap_or_default()
}

/// 按 ID 获取单首歌曲。
pub fn get(store: &PersistentStore, id: &str) -> Option<Song> {
    get_all(store).remove(id)
}

/// 查找重复歌曲：标题相同（忽略大小写）且艺人名集合相同（无序匹配）。
///
/// 返回找到的第一首匹配歌曲，若不存在则返回 `None`。
pub fn find_duplicate(store: &PersistentStore, title: &str, artist_names: &[String]) -> Option<Song> {
    let title_lower = title.to_lowercase();
    let names_set: HashSet<String> = artist_names.iter().map(|n| n.to_lowercase()).collect();

    get_all(store).into_values().find(|s| {
        s.title.to_lowercase() == title_lower
            && {
                let existing_set: HashSet<String> =
                    s.artist_names.iter().map(|n| n.to_lowercase()).collect();
                existing_set == names_set
            }
    })
}

/// 添加一首歌曲（纯插入，不做去重）。
///
/// 若 ID 已存在则返回 `Err`。推荐使用上层 [`super::library::MusicLibrary::add_song`]
/// 以获得去重合并和自动初始化艺人/专辑的能力。
pub fn add(store: &PersistentStore, song: &Song) -> Result<(), String> {
    let mut songs = get_all(store);
    if songs.contains_key(&song.id) {
        return Err(format!("歌曲 '{}' (id={}) 已存在", song.title, song.id));
    }
    songs.insert(song.id.clone(), song.clone());
    store.set(KEY, &songs)
}

/// 更新一首歌曲（按 ID 覆盖）。
///
/// 若歌曲不存在则返回 `Err`。
pub fn update(store: &PersistentStore, song: &Song) -> Result<(), String> {
    let mut songs = get_all(store);
    if !songs.contains_key(&song.id) {
        return Err(format!("歌曲 id={} 不存在", song.id));
    }
    songs.insert(song.id.clone(), song.clone());
    store.set(KEY, &songs)
}

/// 删除一首歌曲。
///
/// 返回 `true` 表示存在并被删除。
pub fn remove(store: &PersistentStore, id: &str) -> Result<bool, String> {
    let mut songs = get_all(store);
    let existed = songs.remove(id).is_some();
    if existed {
        store.set(KEY, &songs)?;
    }
    Ok(existed)
}

/// 按标题/艺术家名模糊搜索歌曲。
///
/// 匹配范围：歌曲标题 + 关联的艺术家名称。
pub fn search(store: &PersistentStore, query: &str, artists: &HashMap<String, super::models::Artist>) -> Vec<Song> {
    let query_lower = query.to_lowercase();
    get_all(store)
        .into_values()
        .filter(|s| {
            s.title.to_lowercase().contains(&query_lower)
                || s.artist_ids.iter().any(|aid| {
                    artists
                        .get(aid)
                        .map(|a| a.name.to_lowercase().contains(&query_lower))
                        .unwrap_or(false)
                })
        })
        .collect()
}

/// 获取歌曲总数。
pub fn count(store: &PersistentStore) -> usize {
    get_all(store).len()
}
