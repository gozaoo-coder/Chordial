/**
 * 缓存管理 API
 */

import { invoke } from '@tauri-apps/api/core';

/**
 * 清除所有缓存
 * @returns {Promise<void>}
 */
export async function clearAll() {
  return invoke('clear_all_cache');
}

/**
 * 获取缓存大小
 * @returns {Promise<number>} 缓存大小（字节）
 */
export async function getSize() {
  return invoke('get_cache_size');
}

export default {
  clearAll,
  getSize,
};
