/**
 * Tauri 后端 API 统一入口
 * 
 * 推荐使用方式:
 * import { addLocalFolder, scanAll } from '@/api/musicSource';
 * 或
 * import * as musicApi from '@/api/musicSource';
 * 
 * 新增类型化 API:
 * import { getArtist, getAlbum } from '@/api/artist';
 * import { getAlbum, getAllAlbums } from '@/api/album';
 * import { Artist, Album, Track } from '@/class';
 */

// 音乐源相关 API
export * from './musicSource/index.js';

// 歌手相关 API（返回类实例）
export * from './artist.js';

// 专辑相关 API（返回类实例）
export * from './album.js';

// 类型类导出（方便直接使用）
export { Artist, ArtistSummary, Album, AlbumSummary, Track } from '@/class';
