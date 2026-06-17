/**
 * 存储 API 统一入口
 *
 * 三层存储职责：
 * - config  — 应用配置，自动防抖落盘（音量、主题等）
 * - storage — 持久化数据，手动落盘（播放列表、乐库等）
 * - cache   — 内存缓存，TTL 过期（临时数据、最近播放等）
 *
 * 二进制 Blob 存储：
 * - storageBlob — 持久化二进制文件（图片、音频缓存等）
 * - cacheBlob   — 带 TTL 的二进制磁盘缓存
 *
 * @example
 * import { configGet, configSet, configFlush } from '@/api/storage';
 * import { storageGet, storageSet, storageSave } from '@/api/storage';
 * import { cacheGet, cacheSet, cacheTtl } from '@/api/storage';
 * import { storageSetBlob, storageGetBlob } from '@/api/storage';
 */

// Config
export {
  configGet,
  configSet,
  configRemove,
  configHas,
  configKeys,
  configClear,
  configFlush,
  configReload,
} from './config.js';

// Storage
export {
  storageGet,
  storageSet,
  storageRemove,
  storageHas,
  storageKeys,
  storageClear,
  storageSave,
} from './storage.js';

// Cache
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
} from './cache.js';

// Blob Cache — 磁盘文件 + 内存 TTL
export {
  enableBlobStorage,
  isBlobStorageEnabled,
  cacheSetBlob,
  cacheGetBlob,
  cacheRemoveBlob,
  cacheHasBlob,
  cacheBlobKeys,
  cacheClearBlobs,
  cacheClearExpiredBlobs,
} from './cacheBlob.js';

// Blob Storage — 持久化二进制文件
export {
  storageSetBlob,
  storageGetBlob,
  storageRemoveBlob,
  storageHasBlob,
  storageBlobKeys,
  storageClearBlobs,
} from './storageBlob.js';
