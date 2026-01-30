/**
 * 音乐库管理 API
 * 
 * 所有返回的数据都会被包装成对应的类实例
 */

import { invoke } from '@tauri-apps/api/core';
import { Track, Artist, Album, ArtistSummary, AlbumSummary } from '@/class';

/**
 * 扫描所有已启用的源
 * @returns {Promise<{sources: SourceConfig[], tracks: Track[], artists: Artist[], albums: Album[]}>} 音乐库对象
 */
export async function scanAll() {
  const result = await invoke('scan_all_sources');
  return {
    sources: result.sources || [],
    tracks: Track.fromDataArray(result.tracks || []),
    artists: (result.artists || []).map(data => new Artist(data)),
    albums: (result.albums || []).map(data => new Album(data)),
  };
}

/**
 * 从缓存获取音乐库
 * @returns {Promise<{sources: SourceConfig[], tracks: Track[], artists: ArtistSummary[], albums: AlbumSummary[]}|null>} 缓存的音乐库
 */
export async function getCached() {
  const result = await invoke('get_cached_library');
  if (!result) return null;

  return {
    sources: result.sources || [],
    tracks: Track.fromDataArray(result.tracks || []),
    artists: (result.artists || []).map(data => new ArtistSummary(data)),
    albums: (result.albums || []).map(data => new AlbumSummary(data)),
  };
}

/**
 * 刷新指定源
 * @param {string} sourceId - 源 ID
 * @returns {Promise<Track[]>} 该源的曲目列表（Track 实例数组）
 */
export async function refreshSource(sourceId) {
  const result = await invoke('refresh_source', { source_id: sourceId });
  return Track.fromDataArray(result || []);
}

/**
 * 从缓存获取指定源的曲目
 * @param {string} sourceId - 源 ID
 * @returns {Promise<Track[]|null>} 该源的缓存曲目列表（Track 实例数组）
 */
export async function getSourceFromCache(sourceId) {
  const result = await invoke('get_source_from_cache', { source_id: sourceId });
  if (!result) return null;
  return Track.fromDataArray(result);
}

export default {
  scanAll,
  getCached,
  refreshSource,
  getSourceFromCache,
};
