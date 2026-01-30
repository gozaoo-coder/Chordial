/**
 * 歌手相关 API
 * 
 * 所有返回的数据都会被包装成对应的类实例
 */
import { invoke } from '@tauri-apps/api/core';
import { Artist, ArtistSummary } from '@/class';

/**
 * 获取歌手完整信息
 * @param {string} artistId - 歌手 ID
 * @returns {Promise<Artist>} 歌手实例
 */
export async function getArtist(artistId) {
  const data = await invoke('get_artist_info', { artist_id: artistId });
  return new Artist(data);
}

/**
 * 获取歌手摘要信息
 * @param {string} artistId - 歌手 ID
 * @returns {Promise<ArtistSummary>} 歌手摘要实例
 */
export async function getArtistSummary(artistId) {
  const data = await invoke('get_artist_summary', { artist_id: artistId });
  return new ArtistSummary(data);
}

/**
 * 批量获取歌手信息
 * @param {string[]} artistIds - 歌手 ID 数组
 * @returns {Promise<Artist[]>} 歌手实例数组
 */
export async function getArtistsByIds(artistIds) {
  const data = await invoke('get_artists_by_ids', { artist_ids: artistIds });
  return data.map(item => new Artist(item));
}

/**
 * 获取所有歌手列表
 * @returns {Promise<ArtistSummary[]>} 歌手摘要列表
 */
export async function getAllArtists() {
  const data = await invoke('get_all_artists');
  return data.map(item => new ArtistSummary(item));
}

/**
 * 获取歌手图片
 * @param {string} artistId - 歌手 ID
 * @returns {Promise<Blob>} 图片 Blob
 */
export async function getArtistImage(artistId) {
  const response = await invoke('get_artist_image', { artist_id: artistId });
  return new Blob([response], { type: 'image/jpeg' });
}

/**
 * 获取歌手图片 URL
 * 如果歌手有封面数据，返回 Data URL；否则返回默认图片
 * @param {string} artistId - 歌手 ID
 * @returns {Promise<string>} 图片 URL
 */
export async function getArtistImageUrl(artistId) {
  try {
    const artist = await getArtist(artistId);
    if (artist.coverData) {
      return artist.coverData;
    }
  } catch (e) {
    console.warn('Failed to get artist cover:', e);
  }
  // 返回默认图片 URL 或空
  return '';
}
