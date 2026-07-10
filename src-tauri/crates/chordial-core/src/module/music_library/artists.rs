use super::models::Artist;
use crate::module::storage::persistent::PersistentStore;
use std::collections::HashMap;

const KEY: &str = "artists";

/// 获取所有艺术家。
pub fn get_all(store: &PersistentStore) -> HashMap<String, Artist> {
    store.get::<HashMap<String, Artist>>(KEY).unwrap_or_default()
}

/// 按 ID 获取单个艺术家。
pub fn get(store: &PersistentStore, id: &str) -> Option<Artist> {
    get_all(store).remove(id)
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
    let mut artists = get_all(store);
    if !artists.contains_key(&artist.id) {
        return Err(format!("艺术家 id={} 不存在", artist.id));
    }
    artists.insert(artist.id.clone(), artist.clone());
    store.set(KEY, &artists)
}

/// 删除一个艺术家。
pub fn remove(store: &PersistentStore, id: &str) -> Result<bool, String> {
    let mut artists = get_all(store);
    let existed = artists.remove(id).is_some();
    if existed {
        store.set(KEY, &artists)?;
    }
    Ok(existed)
}

/// 按名称模糊搜索艺术家。
pub fn search(store: &PersistentStore, query: &str) -> Vec<Artist> {
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
/// 注意：当前存储为全量 HashMap，分页仍需反序列化全部数据，
/// 但可显著减少通过 IPC 传往前端的 JSON 载荷。
pub fn get_page(store: &PersistentStore, offset: usize, limit: usize) -> Vec<Artist> {
    get_all(store)
        .into_values()
        .skip(offset)
        .take(limit)
        .collect()
}

/// 获取艺术家总数。
pub fn count(store: &PersistentStore) -> usize {
    get_all(store).len()
}
