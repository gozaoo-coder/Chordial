/**
 * 专辑完整信息类
 */
import { invoke } from '@tauri-apps/api/core';
import { AlbumSummary } from './AlbumSummary';

export class Album {
  /**
   * 创建专辑实例
   * @param {Object} data - 专辑数据
   */
  constructor(data = {}) {
    this.id = data.id || '';
    this.title = data.title || '';
    this.artistId = data.artist_id || '';
    this.artistName = data.artist_name || '';
    this.year = data.year || null;
    this.genres = data.genres || [];
    this.coverData = data.cover_data || null;
    this.trackIds = data.track_ids || [];
    this.totalDuration = data.total_duration || 0;
  }

  /**
   * 从原始数据创建 Album 实例
   * @param {Object} data - 原始数据
   * @returns {Album} Album 实例
   */
  static fromData(data) {
    if (!data) return null;
    if (data instanceof Album) return data;
    return new Album(data);
  }

  /**
   * 从摘要信息获取完整专辑信息
   * @param {AlbumSummary} summary - 专辑摘要
   * @returns {Promise<Album>} 完整专辑信息
   */
  static async fromSummary(summary) {
    if (!summary) return null;
    return await summary.getFullInfo();
  }

  /**
   * 通过 ID 获取专辑信息
   * @param {string} albumId - 专辑 ID
   * @returns {Promise<Album>} 专辑实例
   */
  static async getById(albumId) {
    try {
      const albumData = await invoke('get_album_info', { album_id: albumId });
      return new Album(albumData);
    } catch (error) {
      console.error('Failed to get album by id:', error);
      throw error;
    }
  }

  /**
   * 批量通过 ID 获取专辑信息
   * @param {string[]} albumIds - 专辑 ID 数组
   * @returns {Promise<Album[]>} 专辑实例数组
   */
  static async getByIds(albumIds) {
    try {
      const albumsData = await invoke('get_albums_by_ids', { album_ids: albumIds });
      return albumsData.map(data => new Album(data));
    } catch (error) {
      console.error('Failed to get albums by ids:', error);
      throw error;
    }
  }

  /**
   * 获取所有专辑列表
   * @returns {Promise<AlbumSummary[]>} 专辑摘要列表
   */
  static async getAll() {
    try {
      const summariesData = await invoke('get_all_albums');
      return summariesData.map(data => new AlbumSummary(data));
    } catch (error) {
      console.error('Failed to get all albums:', error);
      throw error;
    }
  }

  /**
   * 获取歌曲数量
   * @returns {number}
   */
  getTrackCount() {
    return this.trackIds.length;
  }

  /**
   * 获取专辑封面 URL
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
   * 获取显示标题（包含年份）
   * @returns {string}
   */
  getDisplayTitle() {
    if (this.year) {
      return `${this.title} (${this.year})`;
    }
    return this.title;
  }

  /**
   * 获取艺术家显示名
   * @returns {string}
   */
  getArtistDisplayName() {
    return this.artistName || '未知艺术家';
  }

  /**
   * 格式化总时长
   * @returns {string} 格式化的时长字符串 (如 "45:30")
   */
  getFormattedDuration() {
    const hours = Math.floor(this.totalDuration / 3600);
    const minutes = Math.floor((this.totalDuration % 3600) / 60);
    const seconds = this.totalDuration % 60;

    if (hours > 0) {
      return `${hours}:${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`;
    }
    return `${minutes}:${seconds.toString().padStart(2, '0')}`;
  }

  /**
   * 生成摘要信息
   * @returns {AlbumSummary}
   */
  toSummary() {
    return new AlbumSummary({
      id: this.id,
      title: this.title,
      artist_id: this.artistId,
      artist_name: this.artistName,
      cover_data: this.coverData,
      year: this.year,
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
      title: this.title,
      artist_id: this.artistId,
      artist_name: this.artistName,
      year: this.year,
      genres: this.genres,
      cover_data: this.coverData,
      track_ids: this.trackIds,
      total_duration: this.totalDuration,
    };
  }

  /**
   * 转换为用于显示的字符串
   * @returns {string}
   */
  toString() {
    return this.title;
  }
}

export default Album;
