/**
 * 缓存管理 API
 *
 * 请使用 {@link module:src/api/storage/cache} 中的真实缓存 API。
 * 本文件仅保留兼容性占位。
 */

export { cacheGet, cacheSet, cacheRemove, cacheHas, cacheKeys, cacheClear, cacheClearExpired, cacheTouch, cacheTtl } from '../storage/cache.js';

/**
 * @deprecated 使用 {@link module:src/api/storage/cache.cacheClear} 或 {@link module:src/api/storage/cache.cacheClearExpired}
 */
export async function clearAll() {
  const { cacheClearExpired } = await import('../storage/cache.js');
  return cacheClearExpired();
}

/**
 * @deprecated Blob 缓存大小查询后端未单独实现。
 * 可使用 {@link module:src/api/storage/cache.cacheBlobKeys} 查看条目数。
 */
export async function getSize() {
  const { cacheBlobKeys } = await import('../storage/cache.js');
  const keys = await cacheBlobKeys();
  return keys.length;
}

export default {
  clearAll,
  getSize,
};
