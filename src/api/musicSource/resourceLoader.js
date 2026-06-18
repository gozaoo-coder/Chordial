/**
 * ResourceManager 集成层
 *
 * 管理音乐资源（音频、封面）的访问。
 * 音频和图片通过 chordial:// 自定义协议流式传输，
 * 无需 Blob + createObjectURL。
 */

import { buildAudioUrl, buildImageUrl } from './chordialUrl.js';

/**
 * 获取专辑封面资源 URL。
 *
 * @param {import('@/class').SourceId} sourceId - 专辑的 SourceId
 * @returns {Promise<{url: string, release: Function}>}
 */
export async function getAlbumArtResource(sourceId) {
  const url = buildImageUrl(sourceId);
  return { url, release: () => {} };
}

/**
 * 获取音乐文件资源 URL。
 *
 * @param {import('@/class').SourceId} sourceId - 歌曲的 SourceId
 * @returns {Promise<{url: string, release: Function}>}
 */
export async function getMusicFileResource(sourceId) {
  const url = buildAudioUrl(sourceId);
  return { url, release: () => {} };
}

/**
 * 获取歌手图片资源（使用 cover_url 或 via SourceId）。
 *
 * @param {import('@/class').SourceId|null} sourceId
 * @param {string} fallbackUrl
 * @returns {Promise<{url: string, release: Function}>}
 */
export async function getArtistImageResource(sourceId, fallbackUrl = '') {
  if (sourceId) {
    return { url: buildImageUrl(sourceId), release: () => {} };
  }
  return { url: fallbackUrl, release: () => {} };
}

// ══════════════════════════════════════════════════════════════════════════════
// Preload / Release（chordial:// 协议下无需 Blob 管理，保留接口兼容性）
// ══════════════════════════════════════════════════════════════════════════════

export async function preloadAlbumArt(_sourceId) {
  // chordial:// 协议下无需预加载，浏览器按需请求
}

export async function preloadMusicFile(_sourceId) {
  // chordial:// 协议下无需预加载，浏览器按需请求
}

export function releaseAlbumArt(_sourceId) {
  // chordial:// 协议下无需释放
}

export function releaseMusicFile(_sourceId) {
  // chordial:// 协议下无需释放
}

export function clearAllResources() {
  // chordial:// 协议下无需清理 Blob URL
}

export function getResourceStats() {
  return { cachedResources: 0, referenceCounts: {} };
}

export default {
  getAlbumArtResource,
  getMusicFileResource,
  getArtistImageResource,
  preloadAlbumArt,
  preloadMusicFile,
  releaseAlbumArt,
  releaseMusicFile,
  clearAllResources,
  getResourceStats,
};
