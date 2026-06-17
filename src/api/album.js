/**
 * 专辑相关 API — 后端 library_* 命令
 *
 * 所有返回值通过 {@link Album|AlbumSummary} 类包装。
 */
import { invoke } from '@tauri-apps/api/core';
import { Album, AlbumSummary } from '@/class';

/**
 * 获取专辑完整信息。
 * @param {string} albumId
 * @returns {Promise<Album>}
 */
export async function getAlbum(albumId) {
  const data = await invoke('library_get_album', { id: albumId });
  return new Album({
    ...data,
    track_ids: data.song_ids ?? data.songIds ?? [],
  });
}

/**
 * 获取专辑摘要信息。
 * @param {string} albumId
 * @returns {Promise<AlbumSummary>}
 */
export async function getAlbumSummary(albumId) {
  const data = await invoke('library_get_album', { id: albumId });
  return new AlbumSummary(data);
}

/**
 * 批量获取专辑信息。
 * @param {string[]} albumIds
 * @returns {Promise<Album[]>}
 */
export async function getAlbumsByIds(albumIds) {
  const all = await invoke('library_get_all_albums');
  return (albumIds || [])
    .map((id) => (all[id] ? new Album(all[id]) : null))
    .filter(Boolean);
}

/**
 * 获取所有专辑列表。
 * @returns {Promise<AlbumSummary[]>}
 */
export async function getAllAlbums() {
  const data = await invoke('library_get_all_albums');
  return Object.values(data).map((d) => new AlbumSummary(d));
}

/**
 * 获取专辑封面图片数据。
 *
 * 注意：需要通过来源中的 SourceId 调用 `get_album_picture` 后端命令。
 * 本函数作为快捷入口，接收专辑 ID 后查找其第一个 SourceId 并请求图片。
 *
 * @param {string} albumId
 * @returns {Promise<ArrayBuffer|null>} 图片二进制数据
 */
export async function getAlbumArt(albumId) {
  // 1. 获取专辑数据
  const albumData = await invoke('library_get_album', { id: albumId });
  if (!albumData?.source_ids?.length) return null;

  // 2. 取第一个匹配 Album 类型的 SourceId
  const { SourceId } = await import('@/class');
  const albumSourceId = albumData.source_ids.find(
    (s) => (s.entity_type ?? s.entityType) === 'Album',
  );
  if (!albumSourceId) return null;

  // 3. 通过 SourceId 请求图片
  const { getAlbumPicture } = await import('./musicSource/musicResource.js');
  return getAlbumPicture(new SourceId(albumSourceId));
}

/**
 * 获取专辑封面 URL（返回 Object URL）。
 *
 * @param {string} albumId
 * @returns {Promise<string>} 图片 URL
 */
export async function getAlbumArtUrl(albumId) {
  try {
    const data = await getAlbumArt(albumId);
    if (data) {
      const blob = new Blob([data], { type: 'image/jpeg' });
      return URL.createObjectURL(blob);
    }
  } catch (e) {
    console.warn('获取专辑封面失败:', e);
  }
  return '';
}
