/**
 * 歌手完整信息类
 */
import { invoke } from '@tauri-apps/api/core';
import { ArtistSummary } from './ArtistSummary';

export class Artist {
  /**
   * 创建歌手实例
   * @param {Object} data - 歌手数据
   */
  constructor(data = {}) {
    this.id = data.id || '';
    this.name = data.name || '';
    this.bio = data.bio || '';
    this.genres = data.genres || [];
    this.coverData = data.cover_data || null;
    this.albumIds = data.album_ids || [];
    this.trackIds = data.track_ids || [];
  }

  /**
   * 从原始数据创建 Artist 实例
   * @param {Object} data - 原始数据
   * @returns {Artist} Artist 实例
   */
  static fromData(data) {
    if (!data) return null;
    if (data instanceof Artist) return data;
    return new Artist(data);
  }

  /**
   * 从摘要信息获取完整歌手信息
   * @param {ArtistSummary} summary - 歌手摘要
   * @returns {Promise<Artist>} 完整歌手信息
   */
  static async fromSummary(summary) {
    if (!summary) return null;
    return await summary.getFullInfo();
  }

  /**
   * 通过 ID 获取歌手信息
   * @param {string} artistId - 歌手 ID
   * @returns {Promise<Artist>} 歌手实例
   */
  static async getById(artistId) {
    try {
      const artistData = await invoke('get_artist_info', { artist_id: artistId });
      return new Artist(artistData);
    } catch (error) {
      console.error('Failed to get artist by id:', error);
      throw error;
    }
  }

  /**
   * 批量通过 ID 获取歌手信息
   * @param {string[]} artistIds - 歌手 ID 数组
   * @returns {Promise<Artist[]>} 歌手实例数组
   */
  static async getByIds(artistIds) {
    try {
      const artistsData = await invoke('get_artists_by_ids', { artist_ids: artistIds });
      return artistsData.map(data => new Artist(data));
    } catch (error) {
      console.error('Failed to get artists by ids:', error);
      throw error;
    }
  }

  /**
   * 获取所有歌手列表
   * @returns {Promise<ArtistSummary[]>} 歌手摘要列表
   */
  static async getAll() {
    try {
      const summariesData = await invoke('get_all_artists');
      return summariesData.map(data => new ArtistSummary(data));
    } catch (error) {
      console.error('Failed to get all artists:', error);
      throw error;
    }
  }

  /**
   * 获取专辑数量
   * @returns {number}
   */
  getAlbumCount() {
    return this.albumIds.length;
  }

  /**
   * 获取歌曲数量
   * @returns {number}
   */
  getTrackCount() {
    return this.trackIds.length;
  }

  /**
   * 获取歌手封面 URL
   * @returns {string|null}
   */
  getCoverUrl() {
    return this.coverData || null;
  }

  /**
   * 获取流派文本
   * @returns {string}
   */
  getGenresText() {
    return this.genres.join(' / ') || '未知流派';
  }

  /**
   * 生成摘要信息
   * @returns {ArtistSummary}
   */
  toSummary() {
    return new ArtistSummary({
      id: this.id,
      name: this.name,
      cover_data: this.coverData,
      album_count: this.getAlbumCount(),
      track_count: this.getTrackCount(),
    });
  }

  /**
   * 转换为普通对象
   * @returns {Object}
   */
  toJSON() {
    return {
      id: this.id,
      name: this.name,
      bio: this.bio,
      genres: this.genres,
      cover_data: this.coverData,
      album_ids: this.albumIds,
      track_ids: this.trackIds,
    };
  }

  /**
   * 转换为用于显示的字符串
   * @returns {string}
   */
  toString() {
    return this.name;
  }
}

export default Artist;
