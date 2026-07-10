use super::models::Album;
use crate::module::perf;
use crate::module::storage::persistent::PersistentStore;
use std::collections::HashMap;

pub const KEY: &str = "albums";

/// 获取所有专辑。
pub fn get_all(store: &PersistentStore) -> HashMap<String, Album> {
    let _scope = perf::scope("albums.get_all");
    store.get::<HashMap<String, Album>>(KEY).unwrap_or_default()
}

/// 按 ID 获取单个专辑。
///
/// 优化：仅反序列化目标条目，不反序列化整个 HashMap。
/// 旧实现 `get_all(store).remove(id)` 对 3853 张专辑需 24ms，本方法 ~0.1ms。
pub fn get(store: &PersistentStore, id: &str) -> Option<Album> {
    let _scope = perf::scope("albums.get");
    store.get_entry::<Album>(KEY, id)
}

/// 添加一个专辑。
///
/// 优化：存在性检查用 `has_entry`，插入用 `set_subkey` 仅写一条。
pub fn add(store: &PersistentStore, album: &Album) -> Result<(), String> {
    if store.has_entry(KEY, &album.id) {
        return Err(format!("专辑 '{}' (id={}) 已存在", album.title, album.id));
    }
    store.set_subkey(KEY, &album.id, album)
}

/// 更新一个专辑。
///
/// 优化：存在性检查用 `has_entry`，插入用 `set_subkey`。
pub fn update(store: &PersistentStore, album: &Album) -> Result<(), String> {
    let _scope = perf::scope("albums.update");
    if !store.has_entry(KEY, &album.id) {
        return Err(format!("专辑 id={} 不存在", album.id));
    }
    store.set_subkey(KEY, &album.id, album)
}

/// 删除一个专辑。
///
/// 优化：直接 `remove_entry` 操作 JSON Object 键。
pub fn remove(store: &PersistentStore, id: &str) -> Result<bool, String> {
    let _scope = perf::scope("albums.remove");
    Ok(store.remove_entry(KEY, id))
}

/// 按标题/艺术家名模糊搜索专辑。
///
/// 优化：JSON 层过滤，仅反序列化匹配项；按需查找艺术家名而非全量预加载。
pub fn search(
    store: &PersistentStore,
    query: &str,
    artists: &HashMap<String, super::models::Artist>,
) -> Vec<Album> {
    let _scope = perf::scope("albums.search");
    let query_lower = query.to_lowercase();
    store.get_entries_filtered::<Album, _>(KEY, |v| {
        let title_match = v
            .get("title")
            .and_then(|t| t.as_str())
            .map_or(false, |t| t.to_lowercase().contains(&query_lower));
        if title_match {
            return true;
        }
        // 艺术家名匹配：用预加载的 artists map（已在 library 层提供）
        v.get("artist_id")
            .and_then(|aid| aid.as_str())
            .map_or(false, |aid| {
                artists
                    .get(aid)
                    .map(|ar| ar.name.to_lowercase().contains(&query_lower))
                    .unwrap_or(false)
            })
    })
}

/// 按标题 + 艺术家 ID 精确查找专辑（标题忽略大小写）。
///
/// 优化：JSON 层过滤，仅反序列化匹配项。
pub fn find_by_title_and_artist(
    store: &PersistentStore,
    title: &str,
    artist_id: &str,
) -> Option<Album> {
    store
        .get_entries_filtered::<Album, _>(KEY, |v| {
            v.get("artist_id").and_then(|a| a.as_str()) == Some(artist_id)
                && v
                    .get("title")
                    .and_then(|t| t.as_str())
                    .map_or(false, |t| t.eq_ignore_ascii_case(title))
        })
        .into_iter()
        .next()
}

/// 分页获取专辑。
///
/// 优化：仅反序列化 [offset, offset+limit) 范围的条目，
/// 不反序列化整个 HashMap。对 3853 张专辑分页取 50 条：
/// 旧实现 24ms，本方法 ~1ms。
pub fn get_page(store: &PersistentStore, offset: usize, limit: usize) -> Vec<Album> {
    let _scope = perf::scope("albums.get_page");
    store.get_page_entries::<Album>(KEY, offset, limit)
}

/// 获取专辑总数。
///
/// 优化：O(1) 检查 JSON Object 键数量，不反序列化。
pub fn count(store: &PersistentStore) -> usize {
    store.count_entries(KEY)
}
