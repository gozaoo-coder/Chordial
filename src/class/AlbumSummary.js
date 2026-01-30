/**
 * 专辑摘要类
 * 用于列表展示等场景，提供获取完整信息的方法
 */
import { invoke } from '@tauri-apps/api/core';
import { Album } from './Album';

export class AlbumSummary {
  /**
   * 创建专辑摘要实例
   * @param {Object} data - 专辑摘要数据
   */
  constructor(data = {}) {
    this.id = data.id || '';
    this.title = data.title || '';
    this.artistId = data.artist_id || '';
    this.artistName = data.artist_name || '';
    this.coverData = data.cover_data || null;
    this.year = data.year || null;
    this.trackCount = data.track_count || 0;
  }

  /**
   * 从原始数据创建 AlbumSummary 实例
   * @param {Object} data - 原始数据
   * @returns {AlbumSummary} AlbumSummary 实例
   */
  static fromData(data) {
    if (!data) return null;
    if (data instanceof AlbumSummary) return data;
    return new AlbumSummary(data);
  }

  /**
   * 获取完整专辑信息
   * 调用后端 API 获取完整数据并返回 Album 实例
   * @returns {Promise<Album>} 完整专辑信息
   */
  async getFullInfo() {
    try {
      const albumData = await invoke('get_album_info', { album_id: this.id });
      return new Album(albumData);
    } catch (error) {
      console.error('Failed to get album full info:', error);
      throw error;
    }
  }

  /**
   * 获取专辑封面 URL
   * @returns {string|null} 封面 URL 或 null
   */
  getCoverUrl() {
    return this.coverData || null;
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
   * 获取统计信息文本
   * @returns {string}
   */
  getStatsText() {
    if (this.trackCount > 0) {
      return `${this.trackCount} 首歌曲`;
    }
    return '暂无歌曲';
  }

  /**
   * 获取艺术家显示名
   * @returns {string}
   */
  getArtistDisplayName() {
    return this.artistName || '未知艺术家';
  }

  /**
   * 转换为普通对象
   * @returns {Object} 普通对象
   */
  toJSON() {
    return {
      id: this.id,
      title: this.title,
      artist_id: this.artistId,
      artist_name: this.artistName,
      cover_data: this.coverData,
      year: this.year,
      track_count: this.trackCount,
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

export default AlbumSummary;
