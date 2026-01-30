/**
 * 歌手摘要类
 * 用于列表展示等场景，提供获取完整信息的方法
 */
import { invoke } from '@tauri-apps/api/core';
import { Artist } from './Artist';

export class ArtistSummary {
  /**
   * 创建歌手摘要实例
   * @param {Object} data - 歌手摘要数据
   */
  constructor(data = {}) {
    this.id = data.id || '';
    this.name = data.name || '';
    this.coverData = data.cover_data || null;
    this.albumCount = data.album_count || 0;
    this.trackCount = data.track_count || 0;
  }

  /**
   * 从原始数据创建 ArtistSummary 实例
   * @param {Object} data - 原始数据
   * @returns {ArtistSummary} ArtistSummary 实例
   */
  static fromData(data) {
    if (!data) return null;
    if (data instanceof ArtistSummary) return data;
    return new ArtistSummary(data);
  }

  /**
   * 获取完整歌手信息
   * 调用后端 API 获取完整数据并返回 Artist 实例
   * @returns {Promise<Artist>} 完整歌手信息
   */
  async getFullInfo() {
    try {
      const artistData = await invoke('get_artist_info', { artist_id: this.id });
      return new Artist(artistData);
    } catch (error) {
      console.error('Failed to get artist full info:', error);
      throw error;
    }
  }

  /**
   * 获取歌手封面 URL
   * @returns {string|null} 封面 URL 或 null
   */
  getCoverUrl() {
    return this.coverData || null;
  }

  /**
   * 获取歌手统计信息文本
   * @returns {string} 统计信息文本
   */
  getStatsText() {
    const parts = [];
    if (this.albumCount > 0) {
      parts.push(`${this.albumCount} 张专辑`);
    }
    if (this.trackCount > 0) {
      parts.push(`${this.trackCount} 首歌曲`);
    }
    return parts.join(' · ') || '暂无歌曲';
  }

  /**
   * 转换为普通对象
   * @returns {Object} 普通对象
   */
  toJSON() {
    return {
      id: this.id,
      name: this.name,
      cover_data: this.coverData,
      album_count: this.albumCount,
      track_count: this.trackCount,
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

export default ArtistSummary;
