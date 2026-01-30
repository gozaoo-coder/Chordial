/**
 * 单曲类
 * 包装曲目数据，提供便捷访问歌手和专辑信息的方法
 */
import { invoke } from '@tauri-apps/api/core';
import { ArtistSummary } from './ArtistSummary';
import { AlbumSummary } from './AlbumSummary';

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
    this.artist = data.artist || '';
    this.artistId = data.artist_id || '';
    this.album = data.album || '';
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

    // 包装歌手摘要信息
    this._artistSummary = data.artist_summary
      ? new ArtistSummary(data.artist_summary)
      : null;

    // 包装专辑摘要信息
    this._albumSummary = data.album_summary
      ? new AlbumSummary(data.album_summary)
      : null;
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
   * 获取歌手摘要信息
   * @returns {ArtistSummary|null}
   */
  get artistSummary() {
    return this._artistSummary;
  }

  /**
   * 获取专辑摘要信息
   * @returns {AlbumSummary|null}
   */
  get albumSummary() {
    return this._albumSummary;
  }

  /**
   * 获取显示标题
   * @returns {string}
   */
  getDisplayTitle() {
    return this.title || this.fileName || '未知标题';
  }

  /**
   * 获取显示歌手名
   * @returns {string}
   */
  getDisplayArtist() {
    // 优先使用摘要中的歌手名
    if (this._artistSummary) {
      return this._artistSummary.name;
    }
    return this.artist || '未知歌手';
  }

  /**
   * 获取显示专辑名
   * @returns {string}
   */
  getDisplayAlbum() {
    // 优先使用摘要中的专辑名
    if (this._albumSummary) {
      return this._albumSummary.title;
    }
    return this.album || '未知专辑';
  }

  /**
   * 解析可能包含多个歌手的字符串
   * @returns {string[]} 歌手名数组
   */
  getParsedArtists() {
    if (!this.artist) return [];

    let parts;
    if (this.artist.includes('/')) {
      parts = this.artist.split('/');
    } else if (this.artist.includes('&')) {
      parts = this.artist.split('&');
    } else {
      parts = [this.artist];
    }

    return parts.map(s => s.trim()).filter(s => s !== '');
  }

  /**
   * 获取主歌手名（第一个歌手）
   * @returns {string}
   */
  getPrimaryArtist() {
    const artists = this.getParsedArtists();
    return artists[0] || '未知歌手';
  }

  /**
   * 获取封面 URL
   * @returns {string|null}
   */
  getCoverUrl() {
    return this.albumCoverData || this._albumSummary?.getCoverUrl() || null;
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
      artist: this.artist,
      artist_id: this.artistId,
      artist_summary: this._artistSummary?.toJSON(),
      album: this.album,
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
}

export default Track;
