/**
 * 存储 API 统一入口
 *
 * 三层存储职责：
 * - config  — 应用配置，自动防抖落盘（音量、主题等）
 * - storage — 持久化数据，手动落盘（播放列表、乐库等）
 * - cache   — 内存缓存，TTL 过期（临时数据、最近播放等）
 *
 * @example
 * import { configGet, configSet, configFlush } from '@/api/storage';
 * import { storageGet, storageSet, storageSave } from '@/api/storage';
 * import { cacheGet, cacheSet, cacheTtl } from '@/api/storage';
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
