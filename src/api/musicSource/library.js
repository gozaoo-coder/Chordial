/**
 * 音乐库管理 API
 */

import { invoke } from '@tauri-apps/api/core';

/**
 * 扫描所有已启用的源
 * @returns {Promise<Object>} 包含 sources 和 tracks 的音乐库对象
 */
export async function scanAll() {
  return invoke('scan_all_sources');
}

/**
 * 从缓存获取音乐库
 * @returns {Promise<Object|null>} 缓存的音乐库，如果不存在则返回 null
 */
export async function getCached() {
  return invoke('get_cached_library');
}

/**
 * 刷新指定源
 * @param {string} sourceId - 源 ID
 * @returns {Promise<Array>} 该源的曲目列表
 */
export async function refreshSource(sourceId) {
  return invoke('refresh_source', { source_id: sourceId });
}

/**
 * 从缓存获取指定源的曲目
 * @param {string} sourceId - 源 ID
 * @returns {Promise<Array|null>} 该源的缓存曲目列表
 */
export async function getSourceFromCache(sourceId) {
  return invoke('get_source_from_cache', { source_id: sourceId });
}

export default {
  scanAll,
  getCached,
  refreshSource,
  getSourceFromCache,
};
