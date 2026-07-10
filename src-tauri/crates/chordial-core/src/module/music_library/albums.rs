use super::models::Album;
use crate::module::perf;
use crate::module::storage::persistent::PersistentStore;
use std::collections::HashMap;

const KEY: &str = "albums";

/// 获取所有专辑。
pub fn get_all(store: &PersistentStore) -> HashMap<String, Album> {
    let _scope = perf::scope("albums.get_all");
    store.get::<HashMap<String, Album>>(KEY).unwrap_or_default()
}

/// 按 ID 获取单个专辑。
pub fn get(store: &PersistentStore, id: &str) -> Option<Album> {
    let _scope = perf::scope("albums.get");
    get_all(store).remove(id)
}

/// 添加一个专辑。
pub fn add(store: &PersistentStore, album: &Album) -> Result<(), String> {
    let mut albums = get_all(store);
    if albums.contains_key(&album.id) {
        return Err(format!("专辑 '{}' (id={}) 已存在", album.title, album.id));
    }
    albums.insert(album.id.clone(), album.clone());
    store.set(KEY, &albums)
}

/// 更新一个专辑。
pub fn update(store: &PersistentStore, album: &Album) -> Result<(), String> {
    let _scope = perf::scope("albums.update");
    let mut albums = get_all(store);
    if !albums.contains_key(&album.id) {
        return Err(format!("专辑 id={} 不存在", album.id));
    }
    albums.insert(album.id.clone(), album.clone());
    store.set(KEY, &albums)
}

/// 删除一个专辑。
pub fn remove(store: &PersistentStore, id: &str) -> Result<bool, String> {
    let _scope = perf::scope("albums.remove");
    let mut albums = get_all(store);
    let existed = albums.remove(id).is_some();
    if existed {
        store.set(KEY, &albums)?;
    }
    Ok(existed)
}

/// 按标题/艺术家名模糊搜索专辑。
pub fn search(
    store: &PersistentStore,
    query: &str,
    artists: &HashMap<String, super::models::Artist>,
) -> Vec<Album> {
    let _scope = perf::scope("albums.search");
    let query_lower = query.to_lowercase();
    get_all(store)
        .into_values()
        .filter(|a| {
            a.title.to_lowercase().contains(&query_lower)
                || artists
                    .get(&a.artist_id)
                    .map(|ar| ar.name.to_lowercase().contains(&query_lower))
                    .unwrap_or(false)
        })
        .collect()
}

/// 按标题 + 艺术家 ID 精确查找专辑（标题忽略大小写）。
pub fn find_by_title_and_artist(
    store: &PersistentStore,
    title: &str,
    artist_id: &str,
) -> Option<Album> {
    let title_lower = title.to_lowercase();
    get_all(store)
        .into_values()
        .find(|a| a.title.to_lowercase() == title_lower && a.artist_id == artist_id)
}

/// 分页获取专辑。
///
/// 注意：当前存储为全量 HashMap，分页仍需反序列化全部数据，
/// 但可显著减少通过 IPC 传往前端的 JSON 载荷。
pub fn get_page(store: &PersistentStore, offset: usize, limit: usize) -> Vec<Album> {
    get_all(store)
        .into_values()
        .skip(offset)
        .take(limit)
        .collect()
}

/// 获取专辑总数。
pub fn count(store: &PersistentStore) -> usize {
    get_all(store).len()
}
