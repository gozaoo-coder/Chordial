use super::models::Artist;
use crate::module::perf;
use crate::module::storage::persistent::PersistentStore;
use std::collections::HashMap;

pub const KEY: &str = "artists";

/// 获取所有艺术家。
pub fn get_all(store: &PersistentStore) -> HashMap<String, Artist> {
    let _scope = perf::scope("artists.get_all");
    store.get::<HashMap<String, Artist>>(KEY).unwrap_or_default()
}

/// 按 ID 获取单个艺术家。
///
/// 优化：仅反序列化目标条目，不反序列化整个 HashMap。
pub fn get(store: &PersistentStore, id: &str) -> Option<Artist> {
    let _scope = perf::scope("artists.get");
    store.get_entry::<Artist>(KEY, id)
}

/// 添加一个艺术家。
///
/// 优化：存在性检查用 `has_entry`（O(1) JSON 键查，不反序列化），
/// 插入用 `set_field` 仅写一条而非全量重写。
pub fn add(store: &PersistentStore, artist: &Artist) -> Result<(), String> {
    if store.has_entry(KEY, &artist.id) {
        return Err(format!("艺术家 '{}' (id={}) 已存在", artist.name, artist.id));
    }
    store.set_subkey(KEY, &artist.id, artist)
}

/// 更新一个艺术家。
///
/// 优化：存在性检查用 `has_entry`，插入用 `set_subkey`。
pub fn update(store: &PersistentStore, artist: &Artist) -> Result<(), String> {
    let _scope = perf::scope("artists.update");
    if !store.has_entry(KEY, &artist.id) {
        return Err(format!("艺术家 id={} 不存在", artist.id));
    }
    store.set_subkey(KEY, &artist.id, artist)
}

/// 删除一个艺术家。
///
/// 优化：直接 `remove_entry` 操作 JSON Object 键，
/// 避免反序列化整个 HashMap 再重写。
pub fn remove(store: &PersistentStore, id: &str) -> Result<bool, String> {
    let _scope = perf::scope("artists.remove");
    Ok(store.remove_entry(KEY, id))
}

/// 按名称模糊搜索艺术家。
///
/// 优化：JSON 层过滤，仅反序列化匹配项。
pub fn search(store: &PersistentStore, query: &str) -> Vec<Artist> {
    let _scope = perf::scope("artists.search");
    let query_lower = query.to_lowercase();
    store.get_entries_filtered::<Artist, _>(KEY, |v| {
        v.get("name")
            .and_then(|n| n.as_str())
            .map_or(false, |n| n.to_lowercase().contains(&query_lower))
    })
}

/// 按名称精确查找艺术家（忽略大小写）。
///
/// 优化：JSON 层过滤，仅反序列化匹配项。
pub fn find_by_name(store: &PersistentStore, name: &str) -> Option<Artist> {
    store
        .get_entries_filtered::<Artist, _>(KEY, |v| {
            v.get("name")
                .and_then(|n| n.as_str())
                .map_or(false, |n| n.eq_ignore_ascii_case(name))
        })
        .into_iter()
        .next()
}

/// 分页获取艺术家。
///
/// 优化：仅反序列化 [offset, offset+limit) 范围的条目。
pub fn get_page(store: &PersistentStore, offset: usize, limit: usize) -> Vec<Artist> {
    let _scope = perf::scope("artists.get_page");
    store.get_page_entries::<Artist>(KEY, offset, limit)
}

/// 获取艺术家总数。
///
/// 优化：O(1) 检查 JSON Object 键数量，不反序列化。
pub fn count(store: &PersistentStore) -> usize {
    store.count_entries(KEY)
}
