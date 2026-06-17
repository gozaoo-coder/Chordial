/**
 * Tauri 后端 API 统一入口
 *
 * @example
 * import { addLocalFolder, removeLocalFolder, getFolders } from '@/api/tauri';
 * import { getSong, searchSongs, getArtist, getAlbum } from '@/api/tauri';
 * import { getMusicFile, getAlbumPicture, getLyricText } from '@/api/tauri';
 * import { configGet, configSet, storageGet, storageSet } from '@/api/tauri';
 * import { Song, Artist, Album, Lyric, SourceId } from '@/api/tauri';
 */

// ── Music Source: 文件夹管理 ────────────────────────
export {
  addLocalFolder,
  removeLocalFolder,
  remove as removeSource,
  getFolders,
  getLocalStats,
  rescanAll,
} from './musicSource/sources.js';

// ── Music Library: CRUD + 搜索 + 关系查询 ──────────
export {
  // persistence
  save as librarySave,
  cleanupEmptyEntities as libraryCleanup,
  // song
  songCount,
  getSong,
  getAllSongs,
  searchSongs,
  getSongsByArtist,
  getSongsInAlbum,
  getSourceIdsOfSong,
  // artist
  artistCount,
  getArtist,
  getAllArtists,
  searchArtists,
  getArtistsOfSong,
  // album
  albumCount,
  getAlbum,
  getAllAlbums,
  searchAlbums,
  getAlbumOfSong,
  getAlbumsByArtist,
  // lyric
  lyricCount,
  getLyric,
  getAllLyrics,
  searchLyrics,
  getLyricOfSong,
} from './musicSource/library.js';

// ── Music Resource: 大文件获取 ─────────────────────
export {
  getMusicFile,
  getSongFile,
  getAlbumPicture,
  getAlbumArt,
  getLyricText,
  getLyrics,
  parseSyncedLyrics,
  formatToLRC,
} from './musicSource/musicResource.js';

// ── Resource Loader ─────────────────────────────────
export {
  getAlbumArtResource,
  getMusicFileResource,
  getArtistImageResource,
  preloadAlbumArt,
  preloadMusicFile,
  releaseAlbumArt,
  releaseMusicFile,
  clearAllResources,
  getResourceStats,
} from './musicSource/resourceLoader.js';

// ── Config ──────────────────────────────────────────
export {
  configGet,
  configSet,
  configRemove,
  configHas,
  configKeys,
  configClear,
  configFlush,
  configReload,
} from './storage/config.js';

// ── Storage ─────────────────────────────────────────
export {
  storageGet,
  storageSet,
  storageRemove,
  storageHas,
  storageKeys,
  storageClear,
  storageSave,
  storageSetBlob,
  storageGetBlob,
  storageRemoveBlob,
  storageHasBlob,
  storageBlobKeys,
  storageClearBlobs,
} from './storage/index.js';

// ── Cache ───────────────────────────────────────────
export {
  cacheGet,
  cacheSet,
  cacheRemove,
  cacheHas,
  cacheKeys,
  cacheClear,
  cacheClearExpired,
  cacheTouch,
  cacheTtl,
  enableBlobStorage,
  isBlobStorageEnabled,
  cacheSetBlob,
  cacheGetBlob,
  cacheRemoveBlob,
  cacheHasBlob,
  cacheBlobKeys,
  cacheClearBlobs,
  cacheClearExpiredBlobs,
} from './storage/index.js';

// ── Artist / Album convenience ──────────────────────
export { getArtistImageUrl } from './artist.js';
export { getAlbumArtUrl } from './album.js';

// ── Classes ─────────────────────────────────────────
export { Song, Track, Artist, ArtistSummary, Album, AlbumSummary, Lyric, SourceId } from '@/class';
