/**
 * 专辑相关 API
 * 
 * 所有返回的数据都会被包装成对应的类实例
 */
import { invoke } from '@tauri-apps/api/core';
import { Album, AlbumSummary } from '@/class';

/**
 * 获取专辑完整信息
 * @param {string} albumId - 专辑 ID
 * @returns {Promise<Album>} 专辑实例
 */
export async function getAlbum(albumId) {
  const data = await invoke('get_album_info', { album_id: albumId });
  return new Album(data);
}

/**
 * 获取专辑摘要信息
 * @param {string} albumId - 专辑 ID
 * @returns {Promise<AlbumSummary>} 专辑摘要实例
 */
export async function getAlbumSummary(albumId) {
  const data = await invoke('get_album_summary', { album_id: albumId });
  return new AlbumSummary(data);
}

/**
 * 批量获取专辑信息
 * @param {string[]} albumIds - 专辑 ID 数组
 * @returns {Promise<Album[]>} 专辑实例数组
 */
export async function getAlbumsByIds(albumIds) {
  const data = await invoke('get_albums_by_ids', { album_ids: albumIds });
  return data.map(item => new Album(item));
}

/**
 * 获取所有专辑列表
 * @returns {Promise<AlbumSummary[]>} 专辑摘要列表
 */
export async function getAllAlbums() {
  const data = await invoke('get_all_albums');
  return data.map(item => new AlbumSummary(item));
}

/**
 * 获取专辑封面
 * @param {string} albumId - 专辑 ID
 * @param {string} [size='large'] - 图片尺寸 (small, medium, large)
 * @returns {Promise<Blob>} 图片 Blob
 */
export async function getAlbumArt(albumId, size = 'large') {
  const response = await invoke('get_album_art', { 
    album_id: albumId, 
    size 
  });
  return new Blob([response], { type: 'image/jpeg' });
}

/**
 * 获取专辑封面 URL
 * 如果专辑有封面数据，返回 Data URL；否则返回默认图片
 * @param {string} albumId - 专辑 ID
 * @returns {Promise<string>} 图片 URL
 */
export async function getAlbumArtUrl(albumId) {
  try {
    const album = await getAlbum(albumId);
    if (album.coverData) {
      return album.coverData;
    }
  } catch (e) {
    console.warn('Failed to get album cover:', e);
  }
  // 返回默认图片 URL 或空
  return '';
}
