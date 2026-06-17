/**
 * Blob Cache API — 磁盘文件存储 + 内存 TTL 过期
 *
 * 数据以文件形式保存在磁盘上，TTL 元数据保存在内存中。
 * 必须先调用 `enableBlobStorage(dir)` 启用。
 *
 * @example
 * import { enableBlobStorage, cacheSetBlob, cacheGetBlob, cacheTtl } from '@/api/storage/cacheBlob';
 * await enableBlobStorage('/path/to/cache_dir');
 * const bytes = new Uint8Array([1, 2, 3]);
 * await cacheSetBlob('album_cover_123', bytes, cacheTtl.durationSecs(600));
 * const data = await cacheGetBlob('album_cover_123'); // → Uint8Array
 */

import { invoke } from '@tauri-apps/api/core';
import { cacheTtl } from './cache.js';

/** 启用 Blob 缓存磁盘存储 */
export async function enableBlobStorage(dir) {
  return invoke('cache_enable_blob_storage', { dir });
}

/** 检查 Blob 缓存是否已启用 */
export async function isBlobStorageEnabled() {
  return invoke('cache_blob_storage_enabled');
}

/**
 * 写入二进制数据到 Blob 缓存。
 * @param {string} key
 * @param {Uint8Array|ArrayBuffer|number[]} data
 * @param {string|object} ttl - TTL 策略（同 cacheTtl）
 */
export async function cacheSetBlob(key, data, ttl) {
  const bytes = data instanceof ArrayBuffer
    ? Array.from(new Uint8Array(data))
    : data instanceof Uint8Array
      ? Array.from(data)
      : data;
  return invoke('cache_set_blob', { key, data: bytes, ttl });
}

/**
 * 读取 Blob 缓存的二进制数据。
 * @param {string} key
 * @returns {Promise<Uint8Array>}
 */
export async function cacheGetBlob(key) {
  const result = await invoke('cache_get_blob', { key });
  if (result instanceof Uint8Array) return result;
  if (Array.isArray(result)) return new Uint8Array(result);
  throw new Error(`无效的 Blob 响应格式: ${typeof result}`);
}

/** 删除 Blob 缓存项（含磁盘文件） */
export async function cacheRemoveBlob(key) {
  return invoke('cache_remove_blob', { key });
}

/** 检查 Blob 缓存项是否存在且未过期 */
export async function cacheHasBlob(key) {
  return invoke('cache_has_blob', { key });
}

/** 获取所有未过期的 Blob 缓存 key */
export async function cacheBlobKeys() {
  return invoke('cache_blob_keys');
}

/** 清空所有 Blob 缓存（含磁盘文件） */
export async function cacheClearBlobs() {
  return invoke('cache_clear_blobs');
}

/** 清理所有已过期的 Blob 缓存条目，返回清理数量 */
export async function cacheClearExpiredBlobs() {
  return invoke('cache_clear_expired_blobs');
}
