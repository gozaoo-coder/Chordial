/**
 * 音乐库管理 API — 后端 library_* 命令
 *
 * 所有返回值通过 {@link Song|Artist|Album|Lyric} 类包装。
 */

import { transport } from '@/api/transport';
import { Song, Artist, Album, Lyric, ArtistSummary, AlbumSummary } from '@/class';

// ══════════════════════════════════════════════════════════════════════════════
// Persistence
// ══════════════════════════════════════════════════════════════════════════════

/** 立即将所有未落盘的修改写入磁盘 */
export async function save() {
  return transport.command('library_save');
}

/** 清理所有 source_ids 为空的实体 */
export async function cleanupEmptyEntities() {
  return transport.command('library_cleanup_empty_entities');
}

// ══════════════════════════════════════════════════════════════════════════════
// Song
// ══════════════════════════════════════════════════════════════════════════════

/** @returns {Promise<number>} */
export async function songCount() {
  return transport.command('library_song_count');
}

/** @param {string} id @returns {Promise<Song>} */
export async function getSong(id) {
  const data = await transport.command('library_get_song', { id });
  return new Song(data);
}

/** @returns {Promise<Record<string, Song>>} */
export async function getAllSongs() {
  const data = await transport.command('library_get_all_songs');
  const map = {};
  for (const [id, d] of Object.entries(data)) {
    map[id] = new Song(d);
  }
  return map;
}

/**
 * 分页获取歌曲，减少 IPC 载荷。
 * @param {number} offset
 * @param {number} limit
 * @returns {Promise<{songs: Song[], total: number}>}
 */
export async function getSongsPage(offset = 0, limit = 50) {
  const [songsData, total] = await Promise.all([
    transport.command('library_get_songs_page', { offset, limit }),
    transport.command('library_song_count'),
  ]);
  return {
    songs: Song.fromDataArray(songsData),
    total,
  };
}

/** @param {string} query @returns {Promise<Song[]>} */
export async function searchSongs(query) {
  const data = await transport.command('library_search_songs', { query });
  return Song.fromDataArray(data);
}

// ══════════════════════════════════════════════════════════════════════════════
// Artist
// ══════════════════════════════════════════════════════════════════════════════

/** @returns {Promise<number>} */
export async function artistCount() {
  return transport.command('library_artist_count');
}

/** @param {string} id @returns {Promise<Artist>} */
export async function getArtist(id) {
  const data = await transport.command('library_get_artist', { id });
  return new Artist(data);
}

/** @returns {Promise<Record<string, Artist>>} */
export async function getAllArtists() {
  const data = await transport.command('library_get_all_artists');
  const map = {};
  for (const [id, d] of Object.entries(data)) {
    map[id] = new Artist(d);
  }
  return map;
}

/**
 * 分页获取艺术家。
 * @param {number} offset
 * @param {number} limit
 * @returns {Promise<{artists: Artist[], total: number}>}
 */
export async function getArtistsPage(offset = 0, limit = 50) {
  const [artistsData, total] = await Promise.all([
    transport.command('library_get_artists_page', { offset, limit }),
    transport.command('library_artist_count'),
  ]);
  return {
    artists: Artist.fromDataArray(artistsData),
    total,
  };
}

/** @param {string} query @returns {Promise<Artist[]>} */
export async function searchArtists(query) {
  const data = await transport.command('library_search_artists', { query });
  return Artist.fromDataArray(data);
}

// ══════════════════════════════════════════════════════════════════════════════
// Album
// ══════════════════════════════════════════════════════════════════════════════

/** @returns {Promise<number>} */
export async function albumCount() {
  return transport.command('library_album_count');
}

/** @param {string} id @returns {Promise<Album>} */
export async function getAlbum(id) {
  const data = await transport.command('library_get_album', { id });
  return new Album(data);
}

/** @returns {Promise<Record<string, Album>>} */
export async function getAllAlbums() {
  const data = await transport.command('library_get_all_albums');
  const map = {};
  for (const [id, d] of Object.entries(data)) {
    map[id] = new Album(d);
  }
  return map;
}

/**
 * 分页获取专辑。
 * @param {number} offset
 * @param {number} limit
 * @returns {Promise<{albums: Album[], total: number}>}
 */
export async function getAlbumsPage(offset = 0, limit = 50) {
  const [albumsData, total] = await Promise.all([
    transport.command('library_get_albums_page', { offset, limit }),
    transport.command('library_album_count'),
  ]);
  return {
    albums: Album.fromDataArray(albumsData),
    total,
  };
}

// ── homeStats 缓存 ────────────────────────────────────────────────────────────
let _homeStatsCache = null;
let _homeStatsCacheTime = 0;
const HOME_STATS_CACHE_TTL = 30_000; // 30 秒内复用

/**
 * 获取首页所需数据 — 一次轻量 IPC 调用替代三次全量加载。
 * 内置 30 秒 TTL 缓存，避免每次导航到首页都重新请求。
 * @param {boolean} [force=false] 设为 true 强制跳过缓存
 * @returns {Promise<{stats: {tracks: number, artists: number, albums: number}, recentTracks: Song[], featuredArtists: Artist[], recentAlbums: Album[]}>}
 */
export async function homeStats(force = false) {
  const now = Date.now();
  if (!force && _homeStatsCache && (now - _homeStatsCacheTime) < HOME_STATS_CACHE_TTL) {
    return _homeStatsCache;
  }
  const data = await transport.command('library_get_home_stats');
  _homeStatsCache = {
    stats: data.stats,
    recentTracks: Song.fromDataArray(data.recentTracks || []),
    featuredArtists: Artist.fromDataArray(data.featuredArtists || []),
    recentAlbums: Album.fromDataArray(data.recentAlbums || []),
  };
  _homeStatsCacheTime = now;
  return _homeStatsCache;
}

/** @param {string} query @returns {Promise<Album[]>} */
export async function searchAlbums(query) {
  const data = await transport.command('library_search_albums', { query });
  return Album.fromDataArray(data);
}

// ══════════════════════════════════════════════════════════════════════════════
// 统一搜索 — 基于 Rust trigram 倒排索引的跨类型快速子串搜索
// ══════════════════════════════════════════════════════════════════════════════

/**
 * 统一搜索 — 同时搜索歌曲、艺术家、专辑，支持按类型与来源过滤。
 *
 * 后端基于 Rust trigram 倒排索引，10k 条目查询约 0.3ms。
 * 索引首次查询时构建并缓存，写操作自动失效。
 *
 * @param {object} opts
 * @param {string} opts.query - 搜索关键词（大小写不敏感子串匹配）
 * @param {'song'|'artist'|'album'|null} [opts.entityType=null] - 限定实体类型，null 全搜
 * @param {string|null} [opts.sourceName=null] - 限定来源名称，null 不限制
 * @param {number|null} [opts.limitPerType=null] - 每类实体最多返回多少条
 * @returns {Promise<{songs: Song[], artists: Artist[], albums: Album[]}>}
 */
export async function search({ query, entityType = null, sourceName = null, limitPerType = null }) {
  const args = { query };
  if (entityType) args.entity_type = entityType;
  if (sourceName) args.source_name = sourceName;
  if (limitPerType != null) args.limit_per_type = limitPerType;
  const data = await transport.command('library_search', args);
  return {
    songs: Song.fromDataArray(data.songs || []),
    artists: Artist.fromDataArray(data.artists || []),
    albums: Album.fromDataArray(data.albums || []),
  };
}

// ══════════════════════════════════════════════════════════════════════════════
// Lyric
// ══════════════════════════════════════════════════════════════════════════════

/** @returns {Promise<number>} */
export async function lyricCount() {
  return transport.command('library_lyric_count');
}

/** @param {string} id @returns {Promise<Lyric>} */
export async function getLyric(id) {
  const data = await transport.command('library_get_lyric', { id });
  return new Lyric(data);
}

/** @returns {Promise<Record<string, Lyric>>} */
export async function getAllLyrics() {
  const data = await transport.command('library_get_all_lyrics');
  const map = {};
  for (const [id, d] of Object.entries(data)) {
    map[id] = new Lyric(d);
  }
  return map;
}

/** @param {string} query @returns {Promise<Lyric[]>} */
export async function searchLyrics(query) {
  const data = await transport.command('library_search_lyrics', { query });
  return Lyric.fromDataArray(data);
}

// ══════════════════════════════════════════════════════════════════════════════
// Relations
// ══════════════════════════════════════════════════════════════════════════════

/** @param {string} songId @returns {Promise<Artist[]>} */
export async function getArtistsOfSong(songId) {
  const data = await transport.command('library_get_artists_of_song', { songId });
  return Artist.fromDataArray(data);
}

/** @param {string} songId @returns {Promise<Album|null>} */
export async function getAlbumOfSong(songId) {
  const data = await transport.command('library_get_album_of_song', { songId });
  return data ? new Album(data) : null;
}

/** @param {string} songId @returns {Promise<Lyric|null>} */
export async function getLyricOfSong(songId) {
  const data = await transport.command('library_get_lyric_of_song', { songId });
  return data ? new Lyric(data) : null;
}

/** @param {string} artistId @returns {Promise<Song[]>} */
export async function getSongsByArtist(artistId) {
  const data = await transport.command('library_get_songs_by_artist', { artistId });
  return Song.fromDataArray(data);
}

/** @param {string} artistId @returns {Promise<Album[]>} */
export async function getAlbumsByArtist(artistId) {
  const data = await transport.command('library_get_albums_by_artist', { artistId });
  return Album.fromDataArray(data);
}

/** @param {string} albumId @returns {Promise<Song[]>} */
export async function getSongsInAlbum(albumId) {
  const data = await transport.command('library_get_songs_in_album', { albumId });
  return Song.fromDataArray(data);
}

/** @param {string} songId @returns {Promise<SourceId[]>} */
export async function getSourceIdsOfSong(songId) {
  const data = await transport.command('library_get_source_ids_of_song', { songId });
  const { SourceId } = await import('@/class');
  return (data || []).map((s) => new SourceId(s));
}

// ══════════════════════════════════════════════════════════════════════════════
// Memory cache — avoids re-fetching the entire library on every navigation
// ══════════════════════════════════════════════════════════════════════════════

let _cache = null;
let _cacheTime = 0;
const CACHE_TTL = 30_000; // 30 秒内重复导航不重新加载

/**
 * 使缓存失效。在数据变更（添加/删除歌曲、重新扫描等）后调用。
 */
export function invalidateCache() {
  _cache = null;
  _cacheTime = 0;
  _homeStatsCache = null;
  _homeStatsCacheTime = 0;
}

// ══════════════════════════════════════════════════════════════════════════════
// Backward compat aliases
// ══════════════════════════════════════════════════════════════════════════════

/** @deprecated use {@link searchSongs} + {@link getAllArtists} + {@link getAllAlbums} */
export async function scanAll() {
  const [songsData, artistsData, albumsData] = await Promise.all([
    transport.command('library_get_all_songs'),
    transport.command('library_get_all_artists'),
    transport.command('library_get_all_albums'),
  ]);
  const songs = Object.values(songsData).map((d) => new Song(d));
  const artists = Object.values(artistsData).map((d) => new Artist(d));
  const albums = Object.values(albumsData).map((d) => new Album(d));
  return { sources: [], tracks: songs, artists, albums };
}

/**
 * 获取缓存的库数据。
 *
 * 30 秒内重复调用会直接返回缓存结果，避免每次页面切换都全量
 * 加载 921 首歌曲 + 全部艺术家 + 全部专辑。
 *
 * @param {boolean} [force=false] 设为 true 强制跳过缓存
 * @deprecated use {@link searchSongs} with source filter
 */
export async function getCached(force = false) {
  const now = Date.now();
  if (!force && _cache && (now - _cacheTime) < CACHE_TTL) {
    return _cache;
  }
  _cache = await scanAll();
  _cacheTime = now;
  return _cache;
}

/** @deprecated use {@link rescanAll} from sources.js */
export async function refreshSource(_sourceId) {
  const { rescanAll } = await import('./sources.js');
  const result = await rescanAll();
  return Song.fromDataArray([]);
}

export const library = {
  save,
  cleanupEmptyEntities,
  songCount,
  getSong,
  getAllSongs,
  getSongsPage,
  searchSongs,
  artistCount,
  getArtist,
  getAllArtists,
  getArtistsPage,
  searchArtists,
  albumCount,
  getAlbum,
  getAllAlbums,
  getAlbumsPage,
  searchAlbums,
  search,
  homeStats,
  lyricCount,
  getLyric,
  getAllLyrics,
  searchLyrics,
  getArtistsOfSong,
  getAlbumOfSong,
  getLyricOfSong,
  getSongsByArtist,
  getAlbumsByArtist,
  getSongsInAlbum,
  getSourceIdsOfSong,
  invalidateCache,
  // deprecated
  scanAll,
  getCached,
  refreshSource,
  getSourceFromCache: refreshSource,
};

export default library;
