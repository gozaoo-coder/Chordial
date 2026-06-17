/**
 * ResourceManager 集成层
 *
 * 管理音乐资源（音频、封面）的内存引用和生命周期。
 * 适配重构后的 API：使用 SourceId 定位资源。
 */

import { resourceManager } from '@/js/resourceManager.js';

/**
 * 获取专辑封面资源。
 *
 * @param {import('@/class').SourceId} sourceId - 专辑的 SourceId
 * @returns {Promise<{url: string, release: Function}>}
 */
export async function getAlbumArtResource(sourceId) {
  const key = `album_picture_${sourceId.sourceName}_${sourceId.entityId}`;
  return resourceManager.getResource(key, async () => {
    const { getAlbumPicture } = await import('./musicResource.js');
    return getAlbumPicture(sourceId);
  });
}

/**
 * 获取音乐文件资源。
 *
 * @param {import('@/class').SourceId} sourceId - 歌曲的 SourceId
 * @returns {Promise<{url: string, release: Function}>}
 */
export async function getMusicFileResource(sourceId) {
  const key = `music_file_${sourceId.sourceName}_${sourceId.entityId}`;
  return resourceManager.getResource(key, async () => {
    const { getMusicFile } = await import('./musicResource.js');
    return getMusicFile(sourceId);
  });
}

/**
 * 获取歌手图片资源（使用 cover_url 或 via SourceId）。
 *
 * @param {import('@/class').SourceId|null} sourceId
 * @param {string} fallbackUrl
 * @returns {Promise<{url: string, release: Function}>}
 */
export async function getArtistImageResource(sourceId, fallbackUrl = '') {
  const key = `artist_image_${sourceId?.sourceName ?? 'fallback'}_${sourceId?.entityId ?? 'none'}`;
  return resourceManager.getResource(key, async () => {
    if (sourceId) {
      try {
        const { getAlbumPicture } = await import('./musicResource.js');
        return getAlbumPicture(sourceId);
      } catch (_) {
        /* 静默降级 */
      }
    }
    return fallbackUrl;
  });
}

// ══════════════════════════════════════════════════════════════════════════════
// Preload / Release
// ══════════════════════════════════════════════════════════════════════════════

export async function preloadAlbumArt(sourceId) {
  const key = `album_picture_${sourceId.sourceName}_${sourceId.entityId}`;
  resourceManager.preload(key, async () => {
    const { getAlbumPicture } = await import('./musicResource.js');
    return getAlbumPicture(sourceId);
  });
}

export async function preloadMusicFile(sourceId) {
  const key = `music_file_${sourceId.sourceName}_${sourceId.entityId}`;
  resourceManager.preload(key, async () => {
    const { getMusicFile } = await import('./musicResource.js');
    return getMusicFile(sourceId);
  });
}

export function releaseAlbumArt(sourceId) {
  const key = `album_picture_${sourceId.sourceName}_${sourceId.entityId}`;
  if (resourceManager.getCachedResource(key)) {
    resourceManager.releaseResource(key);
  }
}

export function releaseMusicFile(sourceId) {
  const key = `music_file_${sourceId.sourceName}_${sourceId.entityId}`;
  if (resourceManager.getCachedResource(key)) {
    resourceManager.releaseResource(key);
  }
}

export function clearAllResources() {
  resourceManager.clear();
}

export function getResourceStats() {
  return {
    cachedResources: resourceManager.cache.size,
    referenceCounts: Object.fromEntries(resourceManager.referenceCount),
  };
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
