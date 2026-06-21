/**
 * 专辑相关 API — 后端 library_* 命令
 *
 * 所有返回值通过 {@link Album|AlbumSummary} 类包装。
 */
import { transport } from '@/api/transport';
import { Album, AlbumSummary } from '@/class';

/**
 * 获取专辑完整信息。
 * @param {string} albumId
 * @returns {Promise<Album>}
 */
export async function getAlbum(albumId) {
  const data = await transport.command('library_get_album', { id: albumId });
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
  const data = await transport.command('library_get_album', { id: albumId });
  return new AlbumSummary(data);
}

/**
 * 批量获取专辑信息。
 * @param {string[]} albumIds
 * @returns {Promise<Album[]>}
 */
export async function getAlbumsByIds(albumIds) {
  const all = await transport.command('library_get_all_albums');
  return (albumIds || [])
    .map((id) => (all[id] ? new Album(all[id]) : null))
    .filter(Boolean);
}

/**
 * 获取所有专辑列表。
 * @returns {Promise<AlbumSummary[]>}
 */
export async function getAllAlbums() {
  const data = await transport.command('library_get_all_albums');
  return Object.values(data).map((d) => new AlbumSummary(d));
}

/**
 * 获取专辑封面图片数据。
 *
 * 通过 chordial:// 协议直接从后端流式传输，无需 invoke + Blob。
 *
 * @param {string} albumId
 * @returns {Promise<ArrayBuffer|null>} 图片二进制数据
 * @deprecated 使用 {@link getAlbumArtUrl} 获取可直接使用的 URL
 */
export async function getAlbumArt(albumId) {
  // 保留兼容性：通过 chordial 协议获取
  const url = await getAlbumArtUrl(albumId);
  if (!url) return null;
  // 回退：通过 fetch 获取数据
  try {
    const response = await fetch(url);
    return await response.arrayBuffer();
  } catch (_) {
    return null;
  }
}

/**
 * 获取专辑封面 URL（直接返回 chordial:// URL）。
 *
 * @param {string} albumId
 * @returns {Promise<string>} 图片 URL
 */
export async function getAlbumArtUrl(albumId) {
  try {
    // 1. 获取专辑数据
    const albumData = await transport.command('library_get_album', { id: albumId });
    if (!albumData?.source_ids?.length) return '';

    // 2. 取第一个匹配 Album 类型的 SourceId
    const { SourceId } = await import('@/class');
    const albumSourceId = albumData.source_ids.find(
      (s) => (s.entity_type ?? s.entityType) === 'Album',
    );
    if (!albumSourceId) return '';

    // 3. 通过 chordial:// 协议返回 URL
    const { buildImageUrl } = await import('./musicSource/chordialUrl.js');
    return buildImageUrl(new SourceId(albumSourceId));
  } catch (e) {
    console.warn('获取专辑封面失败:', e);
  }
  return '';
}
