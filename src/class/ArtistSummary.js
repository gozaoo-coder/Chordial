/**
 * 歌手摘要类
 * 用于列表展示等场景，提供获取完整信息的方法
 */
import { invoke } from '@tauri-apps/api/core';
import { Artist } from './Artist';
import { resourceManager } from '@/js/resourceManager';

export class ArtistSummary {
  /**
   * 创建歌手摘要实例
   * @param {Object} data - 歌手摘要数据
   */
  constructor(data = {}) {
    this.id = data.id || '';
    this.name = data.name || '';
    // 兼容蛇形命名和驼峰命名（Tauri IPC 可能转换命名）
    // coverData 不再从缓存读取，改为按需加载
    this.coverData = null;
    this.albumCount = data.album_count || data.albumCount || data.album_ids?.length || data.albumIds?.length || 0;
    this.trackCount = data.track_count || data.trackCount || data.track_ids?.length || data.trackIds?.length || 0;
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
   * @deprecated 使用 getCoverImageUrl 或 getCoverResource 代替
   */
  getCoverUrl() {
    return this.coverData || null;
  }

  /**
   * 获取歌手封面资源（使用 ResourceManager 管理）
   * @param {string} size - 图片尺寸 ('small', 'medium', 'large')
   * @returns {Promise<{url: string, release: Function}>} 资源对象，使用完后调用 release()
   */
  async getCoverResource(size = 'medium') {
    const key = `artist-cover-${this.id}-${size}`;
    return resourceManager.getResource(key, async () => {
      // 调用后端 get_artist_image API 获取歌手封面
      const result = await invoke('get_artist_image', { 
        artist_id: this.id
      });
      
      // Tauri v2 返回的是 Uint8Array 数组
      if (Array.isArray(result)) {
        return new Uint8Array(result);
      }
      
      // 如果已经是 Uint8Array 或 ArrayBuffer，直接返回
      if (result instanceof Uint8Array || result instanceof ArrayBuffer) {
        return result;
      }
      
      // 如果 result 有 data 属性（某些 Tauri 版本）
      if (result && result.data) {
        if (Array.isArray(result.data)) {
          return new Uint8Array(result.data);
        }
        if (result.data instanceof Uint8Array || result.data instanceof ArrayBuffer) {
          return result.data;
        }
      }
      
      throw new Error('Invalid cover data format');
    });
  }

  /**
   * 获取歌手封面图片 URL（可用于 img 标签 src）
   * 使用 ResourceManager 缓存，需要手动释放
   * @param {string} size - 图片尺寸 ('small', 'medium', 'large')
   * @returns {Promise<string>} 封面图片 URL（Blob URL）
   */
  async getCoverImageUrl(size = 'medium') {
    try {
      const resource = await this.getCoverResource(size);
      return resource.url;
    } catch (error) {
      // 如果后端 API 未实现，返回空字符串
      return '';
    }
  }

  /**
   * 获取歌手封面图片 Blob
   * @param {string} size - 图片尺寸 ('small', 'medium', 'large')
   * @returns {Promise<Blob|null>} 封面图片 Blob 或 null
   */
  async getCoverImage(size = 'medium') {
    try {
      const key = `artist-cover-${this.id}-${size}`;
      // 先检查缓存中是否已有该资源
      const cachedResource = resourceManager.getCachedResource(key);
      if (cachedResource) {
        return cachedResource.blob || null;
      }
      // 如果没有缓存，获取资源并返回 blob
      await this.getCoverResource(size);
      const newResource = resourceManager.getCachedResource(key);
      return newResource?.blob || null;
    } catch (error) {
      return null;
    }
  }

  /**
   * 预加载歌手封面图片
   * @param {string} size - 图片尺寸 ('small', 'medium', 'large')
   * @returns {Promise<boolean>} 是否成功加载
   */
  async preloadCoverImage(size = 'medium') {
    try {
      const key = `artist-cover-${this.id}-${size}`;
      resourceManager.preload(key, async () => {
        // TODO: 后端需要添加 get_artist_cover API
        throw new Error('Artist cover API not implemented yet');
      });
      return true;
    } catch (error) {
      return false;
    }
  }

  /**
   * 释放封面资源引用
   * @param {string} size - 图片尺寸 ('small', 'medium', 'large')
   */
  releaseCoverResource(size = 'medium') {
    const key = `artist-cover-${this.id}-${size}`;
    if (resourceManager.has(key)) {
      resourceManager.releaseResource(key);
    }
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
