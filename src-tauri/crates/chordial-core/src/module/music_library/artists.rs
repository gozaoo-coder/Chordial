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
pub fn add(store: &PersistentStore, artist: &Artist) -> Result<(), String> {
    let mut artists = get_all(store);
    if artists.contains_key(&artist.id) {
        return Err(format!("艺术家 '{}' (id={}) 已存在", artist.name, artist.id));
    }
    artists.insert(artist.id.clone(), artist.clone());
    store.set(KEY, &artists)
}

/// 更新一个艺术家。
pub fn update(store: &PersistentStore, artist: &Artist) -> Result<(), String> {
    let _scope = perf::scope("artists.update");
    let mut artists = get_all(store);
    if !artists.contains_key(&artist.id) {
        return Err(format!("艺术家 id={} 不存在", artist.id));
    }
    artists.insert(artist.id.clone(), artist.clone());
    store.set(KEY, &artists)
}

/// 删除一个艺术家。
pub fn remove(store: &PersistentStore, id: &str) -> Result<bool, String> {
    let _scope = perf::scope("artists.remove");
    let mut artists = get_all(store);
    let existed = artists.remove(id).is_some();
    if existed {
        store.set(KEY, &artists)?;
    }
    Ok(existed)
}

/// 按名称模糊搜索艺术家。
pub fn search(store: &PersistentStore, query: &str) -> Vec<Artist> {
    let _scope = perf::scope("artists.search");
    let query_lower = query.to_lowercase();
    get_all(store)
        .into_values()
        .filter(|a| a.name.to_lowercase().contains(&query_lower))
        .collect()
}

/// 按名称精确查找艺术家（忽略大小写）。
pub fn find_by_name(store: &PersistentStore, name: &str) -> Option<Artist> {
    let name_lower = name.to_lowercase();
    get_all(store)
        .into_values()
        .find(|a| a.name.to_lowercase() == name_lower)
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
