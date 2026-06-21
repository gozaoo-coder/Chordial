/**
 * 持久化存储 API — 手动落盘
 *
 * 适合播放列表、乐库索引等大批量数据。
 * 写入仅修改内存缓存，需显式调用 `storageSave()` 持久化到 storage.json。
 *
 * @example
 * import { storageGet, storageSet, storageSave } from '@/api/storage/storage';
 * await storageSet('playlist', playlistData);
 * await storageSave();
 */

import { transport } from '@/api/transport';

/**
 * 读取键值
 * @param {string} key - 键名
 * @returns {Promise<any>} 存储的值
 */
export async function storageGet(key) {
  return transport.command('storage_get', { key });
}

/**
 * 写入键值（仅修改内存缓存，不落盘）
 * @param {string} key - 键名
 * @param {any} value - 要存储的值
 * @returns {Promise<void>}
 */
export async function storageSet(key, value) {
  return transport.command('storage_set', { key, value });
}

/**
 * 删除键
 * @param {string} key - 键名
 * @returns {Promise<boolean>} 是否成功删除
 */
export async function storageRemove(key) {
  return transport.command('storage_remove', { key });
}

/**
 * 检查键是否存在
 * @param {string} key - 键名
 * @returns {Promise<boolean>}
 */
export async function storageHas(key) {
  return transport.command('storage_has', { key });
}

/**
 * 获取所有键
 * @returns {Promise<string[]>}
 */
export async function storageKeys() {
  return transport.command('storage_keys');
}

/**
 * 清空所有数据（仅修改内存缓存）
 * @returns {Promise<void>}
 */
export async function storageClear() {
  return transport.command('storage_clear');
}

/**
 * 将内存数据持久化到磁盘
 * @returns {Promise<void>}
 */
export async function storageSave() {
  return transport.command('storage_save');
}
