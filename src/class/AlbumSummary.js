/**
 * 专辑摘要类
 * 用于列表展示等场景，提供获取完整信息的方法
 */
import { invoke } from '@tauri-apps/api/core';
import { Album } from './Album';
import { resourceManager } from '@/js/resourceManager';

export class AlbumSummary {
  /**
   * 创建专辑摘要实例
   * @param {Object} data - 专辑摘要数据
   */
  constructor(data = {}) {
    this.id = data.id || '';
    this.title = data.title || '';
    // 兼容蛇形命名和驼峰命名（Tauri IPC 可能转换命名）
    this.artistId = data.artist_id || data.artistId || '';
    this.artistName = data.artist_name || data.artistName || '';
    // coverData 不再从缓存读取，改为按需加载
    this.coverData = null;
    this.year = data.year || null;
    this.trackCount = data.track_count || data.trackCount || data.track_ids?.length || data.trackIds?.length || 0;
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
   * @deprecated 使用 getCoverImageUrl 或 getCoverResource 代替
   */
  getCoverUrl() {
    return this.coverData || null;
  }

  /**
   * 获取专辑封面资源（使用 ResourceManager 管理）
   * @param {string} size - 图片尺寸 ('small', 'medium', 'large')
   * @returns {Promise<{url: string, release: Function}>} 资源对象，使用完后调用 release()
   */
  async getCoverResource(size = 'medium') {
    const key = `album-cover-${this.id}-${size}`;
    return resourceManager.getResource(key, async () => {
      const result = await invoke('get_album_art', { 
        album_id: this.id, 
        size 
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
   * 获取专辑封面图片 URL（可用于 img 标签 src）
   * 使用 ResourceManager 缓存，需要手动释放
   * @param {string} size - 图片尺寸 ('small', 'medium', 'large')
   * @returns {Promise<string>} 封面图片 URL（Blob URL）
   */
  async getCoverImageUrl(size = 'medium') {
    const resource = await this.getCoverResource(size);
    return resource.url;
  }

  /**
   * 获取专辑封面图片 Blob
   * @param {string} size - 图片尺寸 ('small', 'medium', 'large')
   * @returns {Promise<Blob|null>} 封面图片 Blob 或 null
   */
  async getCoverImage(size = 'medium') {
    try {
      const key = `album-cover-${this.id}-${size}`;
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
      console.error('Failed to get album cover image:', error);
      return null;
    }
  }

  /**
   * 预加载专辑封面图片
   * @param {string} size - 图片尺寸 ('small', 'medium', 'large')
   * @returns {Promise<boolean>} 是否成功加载
   */
  async preloadCoverImage(size = 'medium') {
    try {
      const key = `album-cover-${this.id}-${size}`;
      resourceManager.preload(key, async () => {
        const result = await invoke('get_album_art', { 
          album_id: this.id, 
          size 
        });
        
        if (result && result.data) {
          if (result.data instanceof ArrayBuffer) {
            return result.data;
          }
          if (result.data instanceof Uint8Array) {
            return result.data.buffer;
          }
        }
        if (result instanceof ArrayBuffer) {
          return result;
        }
        if (result instanceof Uint8Array) {
          return result.buffer;
        }
        if (Array.isArray(result)) {
          return new Uint8Array(result).buffer;
        }
        throw new Error('Invalid cover data format');
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
    const key = `album-cover-${this.id}-${size}`;
    if (resourceManager.has(key)) {
      resourceManager.releaseResource(key);
    }
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
