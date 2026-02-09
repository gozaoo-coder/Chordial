/**
 * 单曲类
 * 包装曲目数据，提供便捷访问歌手和专辑信息的方法
 * 
 * 特性：
 * - 支持多歌手（artists 为 ArtistSummary 数组）
 * - album 为 AlbumSummary 类型
 * - 集成 BlobResource 资源管理
 */
import { invoke } from '@tauri-apps/api/core';
import { ArtistSummary } from './ArtistSummary';
import { AlbumSummary } from './AlbumSummary';
import { BlobResource, BlobResourcePool } from './BlobResource';

// Track 专用的资源池（用于管理封面图片等）
const trackResourcePool = new BlobResourcePool();

export class Track {
  /**
   * 创建单曲实例
   * @param {Object} data - 曲目数据
   */
  constructor(data = {}) {
    this.id = data.id || '';
    this.sourceId = data.source_id || '';
    this.path = data.path || '';
    this.fileName = data.file_name || '';
    this.title = data.title || '';
    this.artistId = data.artist_id || '';
    this.albumId = data.album_id || '';
    this.albumCoverData = data.album_cover_data || null;
    this.duration = data.duration || 0;
    this.format = data.format || '';
    this.fileSize = data.file_size || 0;
    this.bitrate = data.bitrate || null;
    this.sampleRate = data.sample_rate || null;
    this.channels = data.channels || null;
    this.year = data.year || null;
    this.genre = data.genre || '';
    this.composer = data.composer || '';
    this.comment = data.comment || '';
    this.lyrics = data.lyrics || '';
    this.syncedLyrics = data.synced_lyrics || '';
    this.addedAt = data.added_at || '';

    // 包装多歌手摘要信息（数组）
    this._artistSummaries = (data.artist_summaries || [])
      .map(a => new ArtistSummary(a));
    
    // 兼容旧数据：如果只有单个 artist_summary
    if (data.artist_summary && this._artistSummaries.length === 0) {
      this._artistSummaries = [new ArtistSummary(data.artist_summary)];
    }

    // 包装专辑摘要信息
    this._albumSummary = data.album_summary 
      ? new AlbumSummary(data.album_summary) 
      : null;

    // 封面资源（延迟加载）
    this._coverResource = null;
    
    // 音乐文件资源（延迟加载）
    this._audioResource = null;
  }

  /**
   * 从原始数据创建 Track 实例
   * @param {Object} data - 原始数据
   * @returns {Track} Track 实例
   */
  static fromData(data) {
    if (!data) return null;
    if (data instanceof Track) return data;
    return new Track(data);
  }

  /**
   * 批量从原始数据创建 Track 实例数组
   * @param {Object[]} dataArray - 原始数据数组
   * @returns {Track[]} Track 实例数组
   */
  static fromDataArray(dataArray) {
    if (!Array.isArray(dataArray)) return [];
    return dataArray.map(data => Track.fromData(data));
  }

  /**
   * 通过 ID 获取曲目信息
   * @param {string} trackId - 曲目 ID
   * @returns {Promise<Track>} 曲目实例
   */
  static async getById(trackId) {
    try {
      const trackData = await invoke('get_track_info', { track_id: trackId });
      return new Track(trackData);
    } catch (error) {
      console.error('Failed to get track by id:', error);
      throw error;
    }
  }

  /**
   * 获取歌手摘要数组
   * @returns {ArtistSummary[]}
   */
  get artists() {
    return this._artistSummaries;
  }

  /**
   * 获取专辑摘要
   * @returns {AlbumSummary|null}
   */
  get album() {
    return this._albumSummary;
  }

  /**
   * 获取主歌手（第一个歌手）
   * @returns {ArtistSummary|null}
   */
  get primaryArtist() {
    return this._artistSummaries[0] || null;
  }

  /**
   * 获取所有歌手名数组
   * @returns {string[]}
   */
  get artistNames() {
    return this._artistSummaries.map(a => a.name);
  }

  /**
   * 获取显示标题
   * @returns {string}
   */
  getDisplayTitle() {
    return this.title || this.fileName || '未知标题';
  }

  /**
   * 获取显示歌手名（多个歌手用 / 分隔）
   * @returns {string}
   */
  getDisplayArtist() {
    if (this._artistSummaries.length > 0) {
      return this._artistSummaries.map(a => a.name).join(' / ');
    }
    return '未知歌手';
  }

  /**
   * 获取显示专辑名
   * @returns {string}
   */
  getDisplayAlbum() {
    if (this._albumSummary) {
      return this._albumSummary.title;
    }
    return '未知专辑';
  }

  /**
   * 获取主歌手名（第一个歌手）
   * @returns {string}
   */
  getPrimaryArtistName() {
    return this.primaryArtist?.name || '未知歌手';
  }

  /**
   * 获取封面 URL（Data URL 或缓存的 URL）
   * @returns {string|null}
   */
  getCoverUrl() {
    return this.albumCoverData || this._albumSummary?.getCoverUrl() || null;
  }

  /**
   * 从 Data URL 创建封面资源
   * @param {string} key - 资源键
   * @returns {Promise<BlobResource|null>}
   * @private
   */
  async _createCoverFromDataUrl(key) {
    let resource = trackResourcePool.get(key);
    if (resource) {
      this._coverResource = resource;
      return resource;
    }

    try {
      const res = await fetch(this.albumCoverData);
      const blob = await res.blob();
      resource = trackResourcePool.add(key, blob);
      this._coverResource = resource;
      return resource;
    } catch (err) {
      console.warn('Failed to create cover resource:', err);
      return null;
    }
  }

  /**
   * 从专辑摘要获取封面资源
   * @param {string} size - 图片尺寸
   * @returns {Promise<BlobResource|null>}
   * @private
   */
  async _getCoverFromAlbumSummary(size) {
    try {
      const albumResource = await this._albumSummary.getCoverResource(size);
      return albumResource || null;
    } catch (err) {
      console.warn('Failed to get cover from album:', err);
      return null;
    }
  }

  /**
   * 从后端 API 获取封面资源
   * @param {string} size - 图片尺寸
   * @returns {Promise<BlobResource|null>}
   * @private
   */
  async _getCoverFromApi(size) {
    const key = `album-cover-${this.albumId}-${size}`;
    let resource = trackResourcePool.get(key);

    if (resource) {
      return resource;
    }

    try {
      const result = await invoke('get_album_art', {
        album_id: this.albumId,
        size
      });

      const imageData = this._extractImageData(result);
      if (!imageData) {
        return null;
      }

      const blob = new Blob([imageData], { type: 'image/jpeg' });
      resource = trackResourcePool.add(key, blob);
      return resource;
    } catch (err) {
      console.warn('Failed to get cover from album_id:', err);
      return null;
    }
  }

  /**
   * 从响应中提取图片数据
   * @param {Object|Array|Uint8Array|ArrayBuffer} result - 响应结果
   * @returns {Uint8Array|ArrayBuffer|null}
   * @private
   */
  _extractImageData(result) {
    if (Array.isArray(result)) {
      return new Uint8Array(result);
    }
    if (result instanceof Uint8Array || result instanceof ArrayBuffer) {
      return result;
    }
    if (result && result.data) {
      if (Array.isArray(result.data)) {
        return new Uint8Array(result.data);
      }
      if (result.data instanceof Uint8Array || result.data instanceof ArrayBuffer) {
        return result.data;
      }
    }
    return null;
  }

  /**
   * 获取封面 BlobResource（用于资源管理）
   * @param {string} size - 图片尺寸 ('small', 'medium', 'large')
   * @returns {Promise<BlobResource|null>}
   */
  async getCoverResource(size = 'medium') {
    if (this._coverResource) {
      return this._coverResource;
    }

    // 优先级1: 从 Data URL 创建
    if (this.albumCoverData?.startsWith('data:image/')) {
      const key = `track_cover_${this.id}`;
      return this._createCoverFromDataUrl(key);
    }

    // 优先级2: 从专辑摘要获取
    if (this._albumSummary) {
      const resource = await this._getCoverFromAlbumSummary(size);
      if (resource) return resource;
    }

    // 优先级3: 从后端 API 获取
    if (this.albumId) {
      return this._getCoverFromApi(size);
    }

    return null;
  }

  /**
   * 使用封面资源（自动管理生命周期）
   * @returns {Promise<string|null>} Blob URL 或 Data URL
   */
  async useCover() {
    const resource = await this.getCoverResource();
    if (resource) {
      return resource.use();
    }
    return this.getCoverUrl();
  }

  /**
   * 释放封面资源
   */
  releaseCover() {
    if (this._coverResource) {
      this._coverResource.release();
    }
  }

  /**
   * 获取音乐文件 BlobResource（用于资源管理）
   * @returns {Promise<BlobResource|null>} 音乐文件资源
   */
  async getAudioResource() {
    // 如果已有资源，直接返回
    if (this._audioResource) {
      return this._audioResource;
    }

    try {
      const key = `track_audio_${this.id}`;
      
      // 检查资源池是否已有
      let resource = trackResourcePool.get(key);
      if (resource) {
        this._audioResource = resource;
        return resource;
      }

      // 从后端获取音乐文件数据
      const result = await invoke('get_music_file', { track_id: this.id });
      
      // 处理 Tauri Response 返回的数据
      let audioData = null;
      if (result && result.data) {
        if (result.data instanceof ArrayBuffer) {
          audioData = result.data;
        } else if (result.data instanceof Uint8Array) {
          audioData = result.data.buffer;
        }
      } else if (result instanceof ArrayBuffer) {
        audioData = result;
      } else if (result instanceof Uint8Array) {
        audioData = result.buffer;
      } else if (Array.isArray(result)) {
        audioData = new Uint8Array(result).buffer;
      }

      if (audioData) {
        // 根据格式确定 MIME 类型
        const mimeType = this._getMimeType();
        const blob = new Blob([audioData], { type: mimeType });
        resource = trackResourcePool.add(key, blob);
        this._audioResource = resource;
        return resource;
      }
    } catch (error) {
      console.error('Failed to get audio resource:', error);
    }
    
    return null;
  }

  /**
   * 获取音乐文件的 MIME 类型
   * @returns {string}
   * @private
   */
  _getMimeType() {
    const format = this.format?.toLowerCase() || '';
    const mimeTypes = {
      'mp3': 'audio/mpeg',
      'mpeg': 'audio/mpeg',
      'wav': 'audio/wav',
      'wave': 'audio/wav',
      'flac': 'audio/flac',
      'ogg': 'audio/ogg',
      'oga': 'audio/ogg',
      'm4a': 'audio/mp4',
      'mp4': 'audio/mp4',
      'aac': 'audio/aac',
      'wma': 'audio/x-ms-wma',
      'aiff': 'audio/aiff',
      'au': 'audio/basic',
    };
    return mimeTypes[format] || 'audio/mpeg';
  }

  /**
   * 获取音乐文件 Blob URL（用于播放）
   * @returns {Promise<string|null>} Blob URL 或 null
   */
  async getAudioBlobUrl() {
    const resource = await this.getAudioResource();
    if (resource) {
      return resource.use();
    }
    return null;
  }

  /**
   * 预加载音乐文件
   * @returns {Promise<boolean>} 是否成功加载
   */
  async preloadAudio() {
    try {
      const resource = await this.getAudioResource();
      return resource !== null && resource.size > 0;
    } catch (error) {
      console.warn('Failed to preload audio:', error);
      return false;
    }
  }

  /**
   * 释放音乐文件资源
   */
  releaseAudio() {
    if (this._audioResource) {
      this._audioResource.release();
      // 注意：这里不将 _audioResource 设为 null，以便资源可以复用
    }
  }

  /**
   * 获取音频资源信息
   * @returns {Promise<Object|null>}
   */
  async getAudioInfo() {
    const resource = await this.getAudioResource();
    if (resource) {
      return {
        size: resource.size,
        type: resource.type,
        url: resource.getUrl(),
        isInUse: resource.isInUse()
      };
    }
    return null;
  }

  /**
   * 格式化时长
   * @returns {string} 格式化的时长字符串 (如 "3:45")
   */
  getFormattedDuration() {
    if (!this.duration) return '0:00';

    const hours = Math.floor(this.duration / 3600);
    const minutes = Math.floor((this.duration % 3600) / 60);
    const seconds = this.duration % 60;

    if (hours > 0) {
      return `${hours}:${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`;
    }
    return `${minutes}:${seconds.toString().padStart(2, '0')}`;
  }

  /**
   * 格式化文件大小
   * @returns {string} 格式化的文件大小 (如 "3.5 MB")
   */
  getFormattedFileSize() {
    if (this.fileSize === 0) return '0 B';

    const units = ['B', 'KB', 'MB', 'GB'];
    const k = 1024;
    const i = Math.floor(Math.log(this.fileSize) / Math.log(k));

    return parseFloat((this.fileSize / Math.pow(k, i)).toFixed(2)) + ' ' + units[i];
  }

  /**
   * 格式化比特率
   * @returns {string}
   */
  getFormattedBitrate() {
    if (!this.bitrate) return '未知';
    return `${this.bitrate} kbps`;
  }

  /**
   * 格式化采样率
   * @returns {string}
   */
  getFormattedSampleRate() {
    if (!this.sampleRate) return '未知';
    return `${(this.sampleRate / 1000).toFixed(1)} kHz`;
  }

  /**
   * 解析同步歌词
   * @returns {Array<{time: number, text: string}>}
   */
  getParsedSyncedLyrics() {
    if (!this.syncedLyrics) return [];

    try {
      const parsed = JSON.parse(this.syncedLyrics);
      if (Array.isArray(parsed)) {
        return parsed.map(line => ({
          time: (line.timestamp || line.time || 0) / 1000,
          text: line.text || ''
        }));
      }
    } catch (e) {
      console.warn('Failed to parse synced lyrics:', e);
    }

    return [];
  }

  /**
   * 是否有同步歌词
   * @returns {boolean}
   */
  hasSyncedLyrics() {
    return !!this.syncedLyrics && this.syncedLyrics.length > 0;
  }

  /**
   * 是否有纯文本歌词
   * @returns {boolean}
   */
  hasPlainLyrics() {
    return !!this.lyrics && this.lyrics.length > 0;
  }

  /**
   * 获取歌词信息对象
   * @returns {Object}
   */
  getLyricsInfo() {
    return {
      plainLyrics: this.lyrics || '',
      syncedLyrics: this.syncedLyrics || '',
      hasSyncedLyrics: this.hasSyncedLyrics(),
      hasPlainLyrics: this.hasPlainLyrics()
    };
  }

  /**
   * 获取所有歌手 ID
   * @returns {string[]}
   */
  getArtistIds() {
    return this._artistSummaries.map(a => a.id);
  }

  /**
   * 获取所有专辑 ID
   * @returns {string|null}
   */
  getAlbumId() {
    return this._albumSummary?.id || null;
  }

  /**
   * 检查是否有多个歌手
   * @returns {boolean}
   */
  hasMultipleArtists() {
    return this._artistSummaries.length > 1;
  }

  /**
   * 转换为普通对象
   * @returns {Object}
   */
  toJSON() {
    return {
      id: this.id,
      source_id: this.sourceId,
      path: this.path,
      file_name: this.fileName,
      title: this.title,
      artist_id: this.artistId,
      artist_summaries: this._artistSummaries.map(a => a.toJSON()),
      album_id: this.albumId,
      album_summary: this._albumSummary?.toJSON(),
      album_cover_data: this.albumCoverData,
      duration: this.duration,
      format: this.format,
      file_size: this.fileSize,
      bitrate: this.bitrate,
      sample_rate: this.sampleRate,
      channels: this.channels,
      year: this.year,
      genre: this.genre,
      composer: this.composer,
      comment: this.comment,
      lyrics: this.lyrics,
      synced_lyrics: this.syncedLyrics,
      added_at: this.addedAt,
    };
  }

  /**
   * 转换为用于显示的字符串
   * @returns {string}
   */
  toString() {
    return `${this.getDisplayTitle()} - ${this.getDisplayArtist()}`;
  }

  /**
   * 清理资源
   */
  destroy() {
    this.releaseCover();
    this.releaseAudio();
    this._coverResource = null;
    this._audioResource = null;
  }

  /**
   * 清理所有资源（静态方法，清理资源池）
   */
  static clearAllResources() {
    trackResourcePool.clear();
  }
}

// 导出资源池（供外部使用）
export { trackResourcePool };

export default Track;
