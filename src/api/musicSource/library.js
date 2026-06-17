/**
 * 音乐库管理 API — 后端 library_* 命令
 *
 * 所有返回值通过 {@link Song|Artist|Album|Lyric} 类包装。
 */

import { invoke } from '@tauri-apps/api/core';
import { Song, Artist, Album, Lyric, ArtistSummary, AlbumSummary } from '@/class';

// ══════════════════════════════════════════════════════════════════════════════
// Persistence
// ══════════════════════════════════════════════════════════════════════════════

/** 立即将所有未落盘的修改写入磁盘 */
export async function save() {
  return invoke('library_save');
}

/** 清理所有 source_ids 为空的实体 */
export async function cleanupEmptyEntities() {
  return invoke('library_cleanup_empty_entities');
}

// ══════════════════════════════════════════════════════════════════════════════
// Song
// ══════════════════════════════════════════════════════════════════════════════

/** @returns {Promise<number>} */
export async function songCount() {
  return invoke('library_song_count');
}

/** @param {string} id @returns {Promise<Song>} */
export async function getSong(id) {
  const data = await invoke('library_get_song', { id });
  return new Song(data);
}

/** @returns {Promise<Record<string, Song>>} */
export async function getAllSongs() {
  const data = await invoke('library_get_all_songs');
  const map = {};
  for (const [id, d] of Object.entries(data)) {
    map[id] = new Song(d);
  }
  return map;
}

/** @param {string} query @returns {Promise<Song[]>} */
export async function searchSongs(query) {
  const data = await invoke('library_search_songs', { query });
  return Song.fromDataArray(data);
}

// ══════════════════════════════════════════════════════════════════════════════
// Artist
// ══════════════════════════════════════════════════════════════════════════════

/** @returns {Promise<number>} */
export async function artistCount() {
  return invoke('library_artist_count');
}

/** @param {string} id @returns {Promise<Artist>} */
export async function getArtist(id) {
  const data = await invoke('library_get_artist', { id });
  return new Artist(data);
}

/** @returns {Promise<Record<string, Artist>>} */
export async function getAllArtists() {
  const data = await invoke('library_get_all_artists');
  const map = {};
  for (const [id, d] of Object.entries(data)) {
    map[id] = new Artist(d);
  }
  return map;
}

/** @param {string} query @returns {Promise<Artist[]>} */
export async function searchArtists(query) {
  const data = await invoke('library_search_artists', { query });
  return Artist.fromDataArray(data);
}

// ══════════════════════════════════════════════════════════════════════════════
// Album
// ══════════════════════════════════════════════════════════════════════════════

/** @returns {Promise<number>} */
export async function albumCount() {
  return invoke('library_album_count');
}

/** @param {string} id @returns {Promise<Album>} */
export async function getAlbum(id) {
  const data = await invoke('library_get_album', { id });
  return new Album(data);
}

/** @returns {Promise<Record<string, Album>>} */
export async function getAllAlbums() {
  const data = await invoke('library_get_all_albums');
  const map = {};
  for (const [id, d] of Object.entries(data)) {
    map[id] = new Album(d);
  }
  return map;
}

/** @param {string} query @returns {Promise<Album[]>} */
export async function searchAlbums(query) {
  const data = await invoke('library_search_albums', { query });
  return Album.fromDataArray(data);
}

// ══════════════════════════════════════════════════════════════════════════════
// Lyric
// ══════════════════════════════════════════════════════════════════════════════

/** @returns {Promise<number>} */
export async function lyricCount() {
  return invoke('library_lyric_count');
}

/** @param {string} id @returns {Promise<Lyric>} */
export async function getLyric(id) {
  const data = await invoke('library_get_lyric', { id });
  return new Lyric(data);
}

/** @returns {Promise<Record<string, Lyric>>} */
export async function getAllLyrics() {
  const data = await invoke('library_get_all_lyrics');
  const map = {};
  for (const [id, d] of Object.entries(data)) {
    map[id] = new Lyric(d);
  }
  return map;
}

/** @param {string} query @returns {Promise<Lyric[]>} */
export async function searchLyrics(query) {
  const data = await invoke('library_search_lyrics', { query });
  return Lyric.fromDataArray(data);
}

// ══════════════════════════════════════════════════════════════════════════════
// Relations
// ══════════════════════════════════════════════════════════════════════════════

/** @param {string} songId @returns {Promise<Artist[]>} */
export async function getArtistsOfSong(songId) {
  const data = await invoke('library_get_artists_of_song', { songId });
  return Artist.fromDataArray(data);
}

/** @param {string} songId @returns {Promise<Album|null>} */
export async function getAlbumOfSong(songId) {
  const data = await invoke('library_get_album_of_song', { songId });
  return data ? new Album(data) : null;
}

/** @param {string} songId @returns {Promise<Lyric|null>} */
export async function getLyricOfSong(songId) {
  const data = await invoke('library_get_lyric_of_song', { songId });
  return data ? new Lyric(data) : null;
}

/** @param {string} artistId @returns {Promise<Song[]>} */
export async function getSongsByArtist(artistId) {
  const data = await invoke('library_get_songs_by_artist', { artistId });
  return Song.fromDataArray(data);
}

/** @param {string} artistId @returns {Promise<Album[]>} */
export async function getAlbumsByArtist(artistId) {
  const data = await invoke('library_get_albums_by_artist', { artistId });
  return Album.fromDataArray(data);
}

/** @param {string} albumId @returns {Promise<Song[]>} */
export async function getSongsInAlbum(albumId) {
  const data = await invoke('library_get_songs_in_album', { albumId });
  return Song.fromDataArray(data);
}

/** @param {string} songId @returns {Promise<SourceId[]>} */
export async function getSourceIdsOfSong(songId) {
  const data = await invoke('library_get_source_ids_of_song', { songId });
  const { SourceId } = await import('@/class');
  return (data || []).map((s) => new SourceId(s));
}

// ══════════════════════════════════════════════════════════════════════════════
// Backward compat aliases
// ══════════════════════════════════════════════════════════════════════════════

/** @deprecated use {@link searchSongs} + {@link getAllArtists} + {@link getAllAlbums} */
export async function scanAll() {
  const [songsData, artistsData, albumsData] = await Promise.all([
    invoke('library_get_all_songs'),
    invoke('library_get_all_artists'),
    invoke('library_get_all_albums'),
  ]);
  const songs = Object.values(songsData).map((d) => new Song(d));
  const artists = Object.values(artistsData).map((d) => new Artist(d));
  const albums = Object.values(albumsData).map((d) => new Album(d));
  return { sources: [], tracks: songs, artists, albums };
}

/** @deprecated use {@link searchSongs} with source filter */
export async function getCached() {
  return scanAll();
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
  searchSongs,
  artistCount,
  getArtist,
  getAllArtists,
  searchArtists,
  albumCount,
  getAlbum,
  getAllAlbums,
  searchAlbums,
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
  // deprecated
  scanAll,
  getCached,
  refreshSource,
  getSourceFromCache: refreshSource,
};

export default library;
