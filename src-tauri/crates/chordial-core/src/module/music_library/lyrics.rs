use super::models::Lyric;
use crate::module::perf;
use crate::module::storage::persistent::PersistentStore;
use std::collections::HashMap;

pub const KEY: &str = "lyrics";

/// 获取所有歌词。
pub fn get_all(store: &PersistentStore) -> HashMap<String, Lyric> {
    let _scope = perf::scope("lyrics.get_all");
    store.get::<HashMap<String, Lyric>>(KEY).unwrap_or_default()
}

/// 按 ID 获取单条歌词。
///
/// 优化：仅反序列化目标条目，不反序列化整个 HashMap。
pub fn get(store: &PersistentStore, id: &str) -> Option<Lyric> {
    let _scope = perf::scope("lyrics.get");
    store.get_entry::<Lyric>(KEY, id)
}

/// 添加一条歌词。
///
/// 优化：存在性检查用 `has_entry`，插入用 `set_subkey`。
pub fn add(store: &PersistentStore, lyric: &Lyric) -> Result<(), String> {
    if store.has_entry(KEY, &lyric.id) {
        return Err(format!("歌词 id={} 已存在", lyric.id));
    }
    store.set_subkey(KEY, &lyric.id, lyric)
}

/// 更新一条歌词。
///
/// 优化：存在性检查用 `has_entry`，插入用 `set_subkey`。
pub fn update(store: &PersistentStore, lyric: &Lyric) -> Result<(), String> {
    let _scope = perf::scope("lyrics.update");
    if !store.has_entry(KEY, &lyric.id) {
        return Err(format!("歌词 id={} 不存在", lyric.id));
    }
    store.set_subkey(KEY, &lyric.id, lyric)
}

/// 删除一条歌词。
///
/// 优化：直接 `remove_entry` 操作 JSON Object 键。
pub fn remove(store: &PersistentStore, id: &str) -> Result<bool, String> {
    let _scope = perf::scope("lyrics.remove");
    Ok(store.remove_entry(KEY, id))
}

/// 按歌曲 ID 获取歌词。
///
/// 优化：JSON 层字符串字段等值匹配，仅反序列化匹配项（~0.1ms）。
/// 旧实现 `get_all + find` 反序列化全部歌词 = 数毫秒。
pub fn get_by_song(store: &PersistentStore, song_id: &str) -> Option<Lyric> {
    store
        .get_entries_by_str_field::<Lyric>(KEY, "song_id", song_id)
        .into_iter()
        .next()
}

/// 按歌词文本模糊搜索。
///
/// 优化：JSON 层过滤，仅反序列化匹配项。
pub fn search(store: &PersistentStore, query: &str) -> Vec<Lyric> {
    let _scope = perf::scope("lyrics.search");
    let query_lower = query.to_lowercase();
    store.get_entries_filtered::<Lyric, _>(KEY, |v| {
        v.get("text")
            .and_then(|t| t.as_str())
            .map_or(false, |t| t.to_lowercase().contains(&query_lower))
    })
}

/// 获取歌词总数。
///
/// 优化：O(1) 检查 JSON Object 键数量，不反序列化。
pub fn count(store: &PersistentStore) -> usize {
    store.count_entries(KEY)
}
