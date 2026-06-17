/**
 * 缓存存储 API — 纯内存，支持 TTL 过期
 *
 * 数据仅存在于进程生命周期内，适合临时缓存、最近播放等场景。
 *
 * @example
 * import { cacheGet, cacheSet, cacheTtl } from '@/api/storage/cache';
 * // 缓存 10 分钟
 * await cacheSet('recent_tracks', data, cacheTtl.durationSecs(600));
 * // 永不过期
 * await cacheSet('app_state', data, cacheTtl.forever);
 */

import { invoke } from '@tauri-apps/api/core';

// ══════════════════════════════════════════════════════════════════════════════
// TTL 辅助工具
// ══════════════════════════════════════════════════════════════════════════════

/**
 * TTL 策略工厂
 *
 * @example
 * cacheTtl.forever            // → "forever"
 * cacheTtl.session            // → "session"
 * cacheTtl.durationSecs(600)  // → { duration_secs: 600 }
 */
export const cacheTtl = {
  /** 永不过期 */
  get forever() {
    return 'forever';
  },
  /** 当前进程生命周期 */
  get session() {
    return 'session';
  },
  /** 指定秒数后过期 */
  durationSecs(secs) {
    return { duration_secs: secs };
  },
};

// ══════════════════════════════════════════════════════════════════════════════
// Cache API
// ══════════════════════════════════════════════════════════════════════════════

/**
 * 读取缓存值（自动检查过期）
 * @param {string} key - 缓存键
 * @returns {Promise<any>} 缓存值
 */
export async function cacheGet(key) {
  return invoke('cache_get', { key });
}

/**
 * 写入缓存值并指定 TTL
 * @param {string} key - 缓存键
 * @param {any} value - 要缓存的值
 * @param {string|{duration_secs: number}} ttl - TTL 策略，使用 cacheTtl 辅助对象
 * @returns {Promise<void>}
 *
 * @example
 * await cacheSet('recent', data, cacheTtl.durationSecs(600));
 * await cacheSet('state', data, cacheTtl.forever);
 */
export async function cacheSet(key, value, ttl) {
  return invoke('cache_set', { key, value, ttl });
}

/**
 * 删除缓存项
 * @param {string} key - 缓存键
 * @returns {Promise<boolean>} 是否成功删除
 */
export async function cacheRemove(key) {
  return invoke('cache_remove', { key });
}

/**
 * 检查缓存项是否存在且未过期
 * @param {string} key - 缓存键
 * @returns {Promise<boolean>}
 */
export async function cacheHas(key) {
  return invoke('cache_has', { key });
}

/**
 * 获取所有未过期的缓存键
 * @returns {Promise<string[]>}
 */
export async function cacheKeys() {
  return invoke('cache_keys');
}

/**
 * 清空所有缓存（含未过期条目）
 * @returns {Promise<void>}
 */
export async function cacheClear() {
  return invoke('cache_clear');
}

/**
 * 清理所有已过期条目
 * @returns {Promise<number>} 清理数量
 */
export async function cacheClearExpired() {
  return invoke('cache_clear_expired');
}

/**
 * 续期缓存项：按给定 TTL 重置过期倒计时
 * @param {string} key - 缓存键
 * @param {string|{duration_secs: number}} ttl - 新的 TTL 策略
 * @returns {Promise<boolean>} 是否续期成功
 */
export async function cacheTouch(key, ttl) {
  return invoke('cache_touch', { key, ttl });
}
