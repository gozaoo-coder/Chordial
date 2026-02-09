/**
 * 专辑完整信息类
 */
import { invoke } from '@tauri-apps/api/core';
import { AlbumSummary } from './AlbumSummary';
import { resourceManager } from '@/js/resourceManager';

export class Album {
  /**
   * 创建专辑实例
   * @param {Object} data - 专辑数据
   */
  constructor(data = {}) {
    this.id = data.id || '';
    this.title = data.title || '';
    // 兼容蛇形命名和驼峰命名（Tauri IPC 可能转换命名）
    this.artistId = data.artist_id || data.artistId || '';
    this.artistName = data.artist_name || data.artistName || '';
    this.year = data.year || null;
    this.genres = data.genres || [];
    this.coverData = data.cover_data || data.coverData || null;
    this.trackIds = data.track_ids || data.trackIds || [];
    this.totalDuration = data.total_duration || data.totalDuration || 0;
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
   * 获取专辑封面图片 Blob
   * 优先使用已缓存的 coverData，如果没有则调用后端 API 获取
   * @returns {Promise<Blob|null>} 封面图片 Blob 或 null
   */
  async getCoverImage() {
    // 如果已有 coverData（Base64 Data URL），转换为 Blob
    if (this.coverData && this.coverData.startsWith('data:image/')) {
      try {
        const response = await fetch(this.coverData);
        return await response.blob();
      } catch (error) {
        console.warn('Failed to convert cover data to blob:', error);
      }
    }

    // 否则调用后端 API 获取
    try {
      const result = await invoke('get_album_art', { 
        album_id: this.id, 
        size: 'large' 
      });
      
      // 处理 Tauri Response 返回的数据
      if (result && result.data) {
        if (result.data instanceof ArrayBuffer) {
          return new Blob([result.data], { type: 'image/jpeg' });
        }
        if (result.data instanceof Uint8Array) {
          return new Blob([result.data.buffer], { type: 'image/jpeg' });
        }
      }
      
      // 兼容旧版本返回格式
      if (result instanceof ArrayBuffer) {
        return new Blob([result], { type: 'image/jpeg' });
      }
      if (result instanceof Uint8Array) {
        return new Blob([result.buffer], { type: 'image/jpeg' });
      }
      if (Array.isArray(result)) {
        return new Blob([new Uint8Array(result).buffer], { type: 'image/jpeg' });
      }
    } catch (error) {
      console.error('Failed to get album cover image:', error);
    }
    
    return null;
  }

  /**
   * 获取专辑封面图片 URL（可用于 img 标签 src）
   * @returns {Promise<string>} 封面图片 URL（Data URL 或 Blob URL）
   */
  async getCoverImageUrl() {
    // 如果已有 coverData，直接返回
    if (this.coverData) {
      return this.coverData;
    }

    // 否则获取 Blob 并创建 Object URL
    const blob = await this.getCoverImage();
    if (blob) {
      return URL.createObjectURL(blob);
    }

    // 返回默认占位图或空字符串
    return '';
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
   * 获取指定尺寸的专辑封面图片
   * @param {string} size - 图片尺寸 ('small', 'medium', 'large')
   * @returns {Promise<Blob|null>} 封面图片 Blob 或 null
   */
  async getCoverImageBySize(size = 'large') {
    try {
      const result = await invoke('get_album_art', { 
        album_id: this.id, 
        size 
      });
      
      // 处理 Tauri Response 返回的数据
      if (result && result.data) {
        if (result.data instanceof ArrayBuffer) {
          return new Blob([result.data], { type: 'image/jpeg' });
        }
        if (result.data instanceof Uint8Array) {
          return new Blob([result.data.buffer], { type: 'image/jpeg' });
        }
      }
      
      // 兼容旧版本返回格式
      if (result instanceof ArrayBuffer) {
        return new Blob([result], { type: 'image/jpeg' });
      }
      if (result instanceof Uint8Array) {
        return new Blob([result.buffer], { type: 'image/jpeg' });
      }
      if (Array.isArray(result)) {
        return new Blob([new Uint8Array(result).buffer], { type: 'image/jpeg' });
      }
    } catch (error) {
      console.error(`Failed to get album cover image (${size}):`, error);
    }
    
    return null;
  }

  /**
   * 预加载专辑封面图片
   * @returns {Promise<boolean>} 是否成功加载
   */
  async preloadCoverImage() {
    try {
      const blob = await this.getCoverImage();
      return blob !== null && blob.size > 0;
    } catch (error) {
      return false;
    }
  }

  /**
   * 检查是否有封面图片
   * @returns {boolean}
   */
  hasCoverImage() {
    return !!this.coverData;
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
