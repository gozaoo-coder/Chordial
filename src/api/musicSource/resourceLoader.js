/**
 * ResourceManager 集成层
 * 管理音乐资源（图片、音频文件）的内存引用和生命周期
 */

import { resourceManager } from '@/js/resourceManager.js';

/**
 * 获取专辑图片资源
 * @param {string} albumId - 专辑 ID
 * @param {string} size - 图片尺寸
 * @returns {Promise<{url: string, release: Function}>}
 */
export async function getAlbumArtResource(albumId, size = 'medium') {
  const key = `album_art_${albumId}_${size}`;
  
  return resourceManager.getResource(key, async () => {
    const { getAlbumArt } = await import('./musicResource.js');
    const data = await getAlbumArt(albumId, size);
    return data;
  });
}

/**
 * 获取音乐文件资源
 * @param {string} trackId - 曲目 ID
 * @returns {Promise<{url: string, release: Function}>}
 */
export async function getMusicFileResource(trackId) {
  const key = `music_file_${trackId}`;
  
  return resourceManager.getResource(key, async () => {
    const { getMusicFile } = await import('./musicResource.js');
    const data = await getMusicFile(trackId);
    return data;
  });
}

/**
 * 获取歌手图片资源
 * @param {string} artistId - 歌手 ID
 * @returns {Promise<{url: string, release: Function}>}
 */
export async function getArtistImageResource(artistId) {
  const key = `artist_image_${artistId}`;
  
  return resourceManager.getResource(key, async () => {
    const { getArtistImage } = await import('./musicResource.js');
    const data = await getArtistImage(artistId);
    return data;
  });
}

/**
 * 预加载专辑图片
 * @param {string} albumId - 专辑 ID
 * @param {string} size - 图片尺寸
 */
export async function preloadAlbumArt(albumId, size = 'medium') {
  const key = `album_art_${albumId}_${size}`;
  
  resourceManager.preload(key, async () => {
    const { getAlbumArt } = await import('./musicResource.js');
    return getAlbumArt(albumId, size);
  });
}

/**
 * 预加载音乐文件
 * @param {string} trackId - 曲目 ID
 */
export async function preloadMusicFile(trackId) {
  const key = `music_file_${trackId}`;
  
  resourceManager.preload(key, async () => {
    const { getMusicFile } = await import('./musicResource.js');
    return getMusicFile(trackId);
  });
}

/**
 * 释放专辑图片资源
 * @param {string} albumId - 专辑 ID
 * @param {string} size - 图片尺寸
 */
export function releaseAlbumArt(albumId, size = 'medium') {
  const key = `album_art_${albumId}_${size}`;
  const resource = resourceManager.cache.get(key);
  if (resource) {
    resourceManager._releaseResource(key);
  }
}

/**
 * 释放音乐文件资源
 * @param {string} trackId - 曲目 ID
 */
export function releaseMusicFile(trackId) {
  const key = `music_file_${trackId}`;
  const resource = resourceManager.cache.get(key);
  if (resource) {
    resourceManager._releaseResource(key);
  }
}

/**
 * 清理所有音乐资源
 */
export function clearAllResources() {
  resourceManager.clear();
}

/**
 * 获取资源使用统计
 * @returns {Object} 资源统计信息
 */
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