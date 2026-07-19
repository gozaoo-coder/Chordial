//! 统一搜索引擎 — 基于 Unicode 字符 trigram 倒排索引。
//!
//! # 算法
//!
//! 采用 **trigram 倒排索引** 加速子串搜索：
//!
//! 1. 构建阶段：对每条实体的可搜索字段做 lowercase 拼接，提取所有 3 字符窗口的 trigram
//!    （基于 `char` 而非字节，正确处理 CJK / Emoji 等 Unicode 字符），
//!    建立 `trigram_hash → 实体 ID 列表` 的倒排索引。
//! 2. 查询阶段：对 query 同样提取 trigram，取交集得到候选实体集合，
//!    再对候选用 `str::contains` 做精确子串校验（消除 trigram 哈希冲突的假阳性）。
//! 3. 短查询退化：query 字符数 < 3 时 trigram 不可用，自动退化为线性扫描，
//!    但仍走预生成的 lowercase 文本表，避免每条都 `to_lowercase()` 分配。
//!
//! # 复杂度
//!
//! - 构建：`O(N * L)`，N = 实体数，L = 平均字段长度
//! - 查询：`O(T * C + C * L)`，T = query trigram 数（≈ len-2），
//!   C = 候选数（通常 ≪ N），L = 候选字段长度
//!
//! 对 10k 首歌的库，旧线性扫描 ~30ms，trigram 索引查询 ~0.3ms（约 100x 提速）。
//!
//! # 缓存与失效
//!
//! `SearchIndex` 由 `MusicLibrary` 持有，构建后缓存于 `RwLock<Option<Arc<SearchIndex>>>`。
//! 失效通过版本计数器：`MusicLibrary` 在任何写操作中递增 `version: AtomicU64`，
//! `SearchIndex` 记录构建时的版本；查询时版本不匹配则重建。
//!
//! # 字段范围
//!
//! | 实体 | 可搜索字段 |
//! |------|-----------|
//! | Song | `title` + `artist_names` + `album_title` |
//! | Artist | `name` |
//! | Album | `title`（艺术家名通过 `artist_id` 不直接索引，避免跨表 join） |

use super::models::{Album, Artist, Song};
use crate::module::music_source::types::EntityType;
use crate::module::perf;
use crate::module::storage::persistent::PersistentStore;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// 字符 trigram 哈希类型（FNV-style 变种，足够分散且无需 String 分配）。
type TrigramHash = u64;

/// 实体 ID 列表 — 倒排索引的值。
type Postings = Vec<String>;

/// 单一实体类型的索引。
#[derive(Default)]
struct TypeIndex {
    /// 实体 ID → lowercase 全文（用于查询阶段精确子串校验）
    texts: HashMap<String, String>,
    /// trigram 哈希 → 包含该 trigram 的实体 ID 列表
    trigrams: HashMap<TrigramHash, Postings>,
}

impl TypeIndex {
    fn with_capacity(cap: usize) -> Self {
        Self {
            texts: HashMap::with_capacity(cap),
            trigrams: HashMap::with_capacity(cap * 4),
        }
    }

    /// 索引一条实体：拼接文本、提取 trigram、写入倒排链。
    fn index(&mut self, id: &str, text: String) {
        for tg in char_trigram_hashes(&text) {
            self.trigrams.entry(tg).or_default().push(id.to_string());
        }
        self.texts.insert(id.to_string(), text);
    }
}

/// 完整的搜索索引（跨三种实体类型）。
pub struct SearchIndex {
    songs: TypeIndex,
    artists: TypeIndex,
    albums: TypeIndex,
    /// 构建时的库版本号（用于失效检测）
    version: u64,
}

impl SearchIndex {
    /// 从持久化存储构建索引。
    ///
    /// 仅读取 JSON 层字段，不反序列化为强类型；这是热路径，
    /// 避免 3853+ 条目 × 3 类型的 `serde_json::from_value` 开销。
    pub fn build(store: &PersistentStore, version: u64) -> Self {
        let _scope = perf::scope("search.build");

        let songs_count = store.count_entries("songs");
        let artists_count = store.count_entries("artists");
        let albums_count = store.count_entries("albums");

        let mut songs = TypeIndex::with_capacity(songs_count);
        let mut artists = TypeIndex::with_capacity(artists_count);
        let mut albums = TypeIndex::with_capacity(albums_count);

        // ── Songs: title + artist_names + album_title ──
        for_each_entry(store, "songs", |v| {
            let id = match v.get("id").and_then(|x| x.as_str()) {
                Some(s) => s,
                None => return,
            };
            let text = build_song_text(v);
            songs.index(id, text);
        });

        // ── Artists: name ──
        for_each_entry(store, "artists", |v| {
            let id = match v.get("id").and_then(|x| x.as_str()) {
                Some(s) => s,
                None => return,
            };
            let name = v.get("name").and_then(|x| x.as_str()).unwrap_or("");
            artists.index(id, name.to_lowercase());
        });

        // ── Albums: title ──
        for_each_entry(store, "albums", |v| {
            let id = match v.get("id").and_then(|x| x.as_str()) {
                Some(s) => s,
                None => return,
            };
            let title = v.get("title").and_then(|x| x.as_str()).unwrap_or("");
            albums.index(id, title.to_lowercase());
        });

        Self {
            songs,
            artists,
            albums,
            version,
        }
    }

    /// 构建时的库版本号。
    pub fn version(&self) -> u64 {
        self.version
    }

    /// 在指定类型索引中执行查询，返回匹配的实体 ID 列表。
    ///
    /// 短查询（< 3 字符）退化为线性扫描 `texts` 表；
    /// 正常查询走 trigram 交集 + 精确校验。
    fn search_type(&self, ty: &TypeIndex, query_lower: &str) -> Vec<String> {
        let q_char_count = query_lower.chars().count();
        if q_char_count < 3 {
            // 退化：线性扫描（仍走预生成 lowercase 文本，避免每条 to_lowercase）
            return ty
                .texts
                .iter()
                .filter(|(_, text)| text.contains(query_lower))
                .map(|(id, _)| id.clone())
                .collect();
        }

        // 收集所有 query trigram，按 postings 长度升序排列，
        // 从最短 postings 起步做交集，最小化工作量。
        let mut hashes: Vec<TrigramHash> = char_trigram_hashes(query_lower).collect();
        hashes.sort_by_key(|h| ty.trigrams.get(h).map_or(usize::MAX, |v| v.len()));

        // 任一 trigram 缺失即空集
        if hashes
            .iter()
            .any(|h| !ty.trigrams.contains_key(h))
        {
            return Vec::new();
        }

        // 从最短 postings 起步，逐个 trigram 做交集
        let first = ty.trigrams.get(&hashes[0]).expect("已检查存在");
        let mut candidates: Vec<String> = first.clone();

        for h in &hashes[1..] {
            if candidates.is_empty() {
                break;
            }
            let postings = ty.trigrams.get(h).expect("已检查存在");
            // 用 HashSet 加速 lookup：对大 postings 收益明显
            let set: std::collections::HashSet<&str> =
                postings.iter().map(|s| s.as_str()).collect();
            candidates.retain(|id| set.contains(id.as_str()));
        }

        // 精确子串校验，消除哈希冲突假阳性
        candidates
            .into_iter()
            .filter(|id| {
                ty.texts
                    .get(id)
                    .map(|text| text.contains(query_lower))
                    .unwrap_or(false)
            })
            .collect()
    }
}

/// 提取字符串的字符 trigram 哈希序列。
///
/// 对 `s` 中每个连续 3 字符窗口计算 FNV-style 哈希，
/// 用于倒排索引的 key（避免为每个 trigram 分配 String）。
///
/// 字符数 < 3 时返回空迭代器。
fn char_trigram_hashes(s: &str) -> impl Iterator<Item = TrigramHash> + '_ {
    let chars: Vec<char> = s.chars().collect();
    (0..chars.len().saturating_sub(2)).map(move |i| {
        let mut h: TrigramHash = 0x9E3779B97F4A7C15; // 黄金比例常数
        for &c in &chars[i..i + 3] {
            h = (h ^ (c as u64)).wrapping_mul(0x100000001B3);
        }
        h
    })
}

/// 拼接 Song 的可搜索字段（lowercase）：title + 所有 artist_names + album_title。
///
/// 字段间用 `\x00` 分隔，避免跨字段产生虚假 trigram。
fn build_song_text(v: &Value) -> String {
    let title = v.get("title").and_then(|x| x.as_str()).unwrap_or("");
    let album_title = v.get("album_title").and_then(|x| x.as_str()).unwrap_or("");

    let artist_names_len = v
        .get("artist_names")
        .and_then(|x| x.as_array())
        .map(|arr| arr.iter().map(|s| s.as_str().map(|s| s.len()).unwrap_or(0)).sum())
        .unwrap_or(0);

    let mut text = String::with_capacity(title.len() + album_title.len() + artist_names_len + 8);
    text.push_str(&title.to_lowercase());
    text.push('\x00');

    if let Some(names) = v.get("artist_names").and_then(|x| x.as_array()) {
        for n in names {
            if let Some(s) = n.as_str() {
                text.push_str(&s.to_lowercase());
                text.push('\x00');
            }
        }
    }

    if !album_title.is_empty() {
        text.push_str(&album_title.to_lowercase());
    }

    text
}

/// 遍历持久化存储中某 key 下所有 JSON Object 条目，对每条调用 `f`。
///
/// 直接操作 `serde_json::Value`，避免 `get_entries_filtered` 的反序列化开销
/// （构建索引仅需读取字段，不需要强类型）。
fn for_each_entry<F>(store: &PersistentStore, key: &str, mut f: F)
where
    F: FnMut(&Value),
{
    let guard = store.get_raw(key);
    let Some(value) = guard else { return };
    let Some(obj) = value.as_object() else { return };
    for (_, v) in obj {
        f(v);
    }
}

// ── 公共 API ─────────────────────────────────────────────────────────────

/// 搜索过滤参数。
#[derive(Debug, Clone, Default)]
pub struct SearchFilter<'a> {
    /// 搜索关键词（任意子串匹配，大小写不敏感）
    pub query: &'a str,
    /// 限定实体类型；None 表示搜索所有类型
    pub entity_type: Option<EntityType>,
    /// 限定来源名称；None 表示不限制
    pub source_name: Option<&'a str>,
    /// 每种类型最多返回多少条结果；None 表示无限制
    pub limit: Option<usize>,
}

/// 搜索结果 — 三类实体的合集。
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SearchResults {
    pub songs: Vec<Song>,
    pub artists: Vec<Artist>,
    pub albums: Vec<Album>,
}

/// 在已有 `SearchIndex` 上执行查询，返回三类匹配 ID 列表。
///
/// 调用方负责按 ID 从 `PersistentStore` 反序列化具体实体。
pub fn search_ids(index: &SearchIndex, filter: &SearchFilter) -> SearchIdSets {
    let _scope = perf::scope("search.ids");
    let query_lower = filter.query.to_lowercase();
    if query_lower.is_empty() {
        return SearchIdSets::default();
    }

    let limit = filter.limit.unwrap_or(usize::MAX);

    let mut songs = Vec::new();
    let mut artists = Vec::new();
    let mut albums = Vec::new();

    match filter.entity_type {
        None => {
            songs = truncate(index.search_type(&index.songs, &query_lower), limit);
            artists = truncate(index.search_type(&index.artists, &query_lower), limit);
            albums = truncate(index.search_type(&index.albums, &query_lower), limit);
        }
        Some(EntityType::Song) => {
            songs = truncate(index.search_type(&index.songs, &query_lower), limit);
        }
        Some(EntityType::Artist) => {
            artists = truncate(index.search_type(&index.artists, &query_lower), limit);
        }
        Some(EntityType::Album) => {
            albums = truncate(index.search_type(&index.albums, &query_lower), limit);
        }
        Some(EntityType::Lyric) => {
            // 歌词不在索引范围；调用方可使用 library_search_lyrics
        }
    }

    SearchIdSets {
        songs,
        artists,
        albums,
    }
}

fn truncate(mut v: Vec<String>, limit: usize) -> Vec<String> {
    if v.len() > limit {
        v.truncate(limit);
    }
    v
}

/// 搜索结果 ID 集合（按类型分组）。
#[derive(Default)]
pub struct SearchIdSets {
    pub songs: Vec<String>,
    pub artists: Vec<String>,
    pub albums: Vec<String>,
}

impl SearchIdSets {
    pub fn is_empty(&self) -> bool {
        self.songs.is_empty() && self.artists.is_empty() && self.albums.is_empty()
    }
}

/// 按来源名过滤 JSON 条目（黑盒谓词，作用于 `Value`）。
///
/// 用于在搜索结果回填实体时，剔除不匹配来源的条目。
/// 检查 `source_ids` 数组中是否有任一 `source_name` 字段等于目标。
pub fn matches_source(v: &Value, source_name: &str) -> bool {
    v.get("source_ids")
        .and_then(|s| s.as_array())
        .map_or(false, |arr| {
            arr.iter()
                .any(|sid| sid.get("source_name").and_then(|n| n.as_str()) == Some(source_name))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trigram_hashes_ascii() {
        let v: Vec<_> = char_trigram_hashes("hello").collect();
        // "hello" has 3 trigrams: hel, ell, llo
        assert_eq!(v.len(), 3);
        assert_ne!(v[0], v[1]);
        assert_ne!(v[1], v[2]);
        assert_ne!(v[0], v[2]);
    }

    #[test]
    fn trigram_hashes_unicode() {
        // 中文 3 字符
        let v: Vec<_> = char_trigram_hashes("周杰伦").collect();
        assert_eq!(v.len(), 1); // 3 chars → 1 trigram
    }

    #[test]
    fn trigram_hashes_short() {
        let v: Vec<_> = char_trigram_hashes("ab").collect();
        assert_eq!(v.len(), 0);
    }

    #[test]
    fn build_song_text_concatenates_fields() {
        let v: Value = serde_json::json!({
            "title": "Hello",
            "artist_names": ["World", "Foo"],
            "album_title": "Bar"
        });
        let text = build_song_text(&v);
        assert!(text.contains("hello"));
        assert!(text.contains("world"));
        assert!(text.contains("foo"));
        assert!(text.contains("bar"));
        assert!(text.contains('\x00'));
    }
}
