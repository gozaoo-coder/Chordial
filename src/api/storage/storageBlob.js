/**
 * Blob Storage API — 持久化二进制文件存储
 *
 * 文件保存在 `storage.json` 同级的 `blobs/` 目录下，写入立即落盘。
 * 适合图片、歌词等需要持久保存的二进制数据。
 *
 * @example
 * import { storageSetBlob, storageGetBlob } from '@/api/storage/storageBlob';
 * const bytes = new Uint8Array([...]);
 * await storageSetBlob('artist_cover_456', bytes);
 * const data = await storageGetBlob('artist_cover_456'); // → Uint8Array
 */

import { invoke } from '@tauri-apps/api/core';

/**
 * 写入二进制数据到持久化 Blob 存储（立即落盘）。
 * @param {string} key
 * @param {Uint8Array|ArrayBuffer|number[]} data
 */
export async function storageSetBlob(key, data) {
  const bytes = data instanceof ArrayBuffer
    ? Array.from(new Uint8Array(data))
    : data instanceof Uint8Array
      ? Array.from(data)
      : data;
  return invoke('storage_set_blob', { key, data: bytes });
}

/**
 * 读取持久化 Blob 存储的二进制数据。
 * @param {string} key
 * @returns {Promise<Uint8Array>}
 */
export async function storageGetBlob(key) {
  const result = await invoke('storage_get_blob', { key });
  if (result instanceof Uint8Array) return result;
  if (Array.isArray(result)) return new Uint8Array(result);
  throw new Error(`无效的 Blob 响应格式: ${typeof result}`);
}

/** 删除持久化 Blob 存储项 */
export async function storageRemoveBlob(key) {
  return invoke('storage_remove_blob', { key });
}

/** 检查持久化 Blob 存储项是否存在 */
export async function storageHasBlob(key) {
  return invoke('storage_has_blob', { key });
}

/** 获取所有持久化 Blob 存储的 key 列表 */
export async function storageBlobKeys() {
  return invoke('storage_blob_keys');
}

/** 清空所有持久化 Blob 存储项 */
export async function storageClearBlobs() {
  return invoke('storage_clear_blobs');
}
