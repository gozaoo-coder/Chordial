use super::models::Lyric;
use crate::module::perf;
use crate::module::storage::persistent::PersistentStore;
use std::collections::HashMap;

const KEY: &str = "lyrics";

/// 获取所有歌词。
pub fn get_all(store: &PersistentStore) -> HashMap<String, Lyric> {
    let _scope = perf::scope("lyrics.get_all");
    store.get::<HashMap<String, Lyric>>(KEY).unwrap_or_default()
}

/// 按 ID 获取单条歌词。
pub fn get(store: &PersistentStore, id: &str) -> Option<Lyric> {
    let _scope = perf::scope("lyrics.get");
    get_all(store).remove(id)
}

/// 添加一条歌词。
pub fn add(store: &PersistentStore, lyric: &Lyric) -> Result<(), String> {
    let mut lyrics = get_all(store);
    if lyrics.contains_key(&lyric.id) {
        return Err(format!("歌词 id={} 已存在", lyric.id));
    }
    lyrics.insert(lyric.id.clone(), lyric.clone());
    store.set(KEY, &lyrics)
}

/// 更新一条歌词。
pub fn update(store: &PersistentStore, lyric: &Lyric) -> Result<(), String> {
    let _scope = perf::scope("lyrics.update");
    let mut lyrics = get_all(store);
    if !lyrics.contains_key(&lyric.id) {
        return Err(format!("歌词 id={} 不存在", lyric.id));
    }
    lyrics.insert(lyric.id.clone(), lyric.clone());
    store.set(KEY, &lyrics)
}

/// 删除一条歌词。
pub fn remove(store: &PersistentStore, id: &str) -> Result<bool, String> {
    let _scope = perf::scope("lyrics.remove");
    let mut lyrics = get_all(store);
    let existed = lyrics.remove(id).is_some();
    if existed {
        store.set(KEY, &lyrics)?;
    }
    Ok(existed)
}

/// 按歌曲 ID 获取歌词。
pub fn get_by_song(store: &PersistentStore, song_id: &str) -> Option<Lyric> {
    get_all(store)
        .into_values()
        .find(|l| l.song_id == song_id)
}

/// 按歌词文本模糊搜索。
pub fn search(store: &PersistentStore, query: &str) -> Vec<Lyric> {
    let _scope = perf::scope("lyrics.search");
    let query_lower = query.to_lowercase();
    get_all(store)
        .into_values()
        .filter(|l| l.text.to_lowercase().contains(&query_lower))
        .collect()
}

/// 获取歌词总数。
pub fn count(store: &PersistentStore) -> usize {
    get_all(store).len()
}
