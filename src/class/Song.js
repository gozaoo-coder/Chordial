/**
 * Song / Track — 歌曲
 *
 * 对应后端 `music_library::models::Song`。
 * 别名 `Track` 用于兼容现有组件。
 */
import { SourceId } from './SourceId.js';

export class Song {
  /** @param {Partial<Song>} data 后端 JSON 或普通对象 */
  constructor(data = {}) {
    /** 库内统一 ID */
    this.id = data.id ?? '';
    /** 歌曲标题 */
    this.title = data.title ?? '未知歌曲';
    /** 艺人名称列表 */
    this.artistNames = data.artist_names ?? data.artistNames ?? [];
    /** 专辑名称 */
    this.albumTitle = data.album_title ?? data.albumTitle ?? null;
    /** 时长（秒） */
    this.duration = data.duration ?? null;
    /** 关联的艺术家 ID 列表 */
    this.artistIds = data.artist_ids ?? data.artistIds ?? [];
    /** 关联的专辑 ID */
    this.albumId = data.album_id ?? data.albumId ?? null;
    /** 关联的歌词 ID */
    this.lyricId = data.lyric_id ?? data.lyricId ?? null;
    /** 来源引用列表 */
    this.sourceIds = (data.source_ids ?? data.sourceIds ?? []).map(
      (s) => (s instanceof SourceId ? s : new SourceId(s)),
    );
  }

  // ── 显示辅助 ────────────────────────────────────

  /** 艺人名称（逗号连接） */
  get artist() {
    return this.artistNames.length > 0
      ? this.artistNames.join(', ')
      : '未知歌手';
  }

  getDisplayArtist() {
    return this.artist;
  }

  /** 主要艺人，如果有 */
  get primaryArtist() {
    if (this.artistIds.length > 0) {
      return {
        id: this.artistIds[0],
        name: this.artistNames[0] ?? '未知歌手',
      };
    }
    return null;
  }

  // ── 资源方法 ────────────────────────────────────

  /**
   * 获取音频 Blob URL（通过第一个 SourceId 从后端拉取）
   * @returns {Promise<string|null>}
   */
  async getAudioBlobUrl() {
    if (!this.sourceIds || this.sourceIds.length === 0) return null;
    const { getMusicFileResource } = await import('@/api/musicSource/resourceLoader.js');
    try {
      const { url } = await getMusicFileResource(this.sourceIds[0]);
      return url;
    } catch (e) {
      console.warn('获取音频 Blob URL 失败:', e);
      return null;
    }
  }

  /**
   * 释放音频 Blob URL（通过第一个 SourceId）
   */
  releaseAudio() {
    if (!this.sourceIds || this.sourceIds.length === 0) return;
    import('@/api/musicSource/resourceLoader.js').then(({ releaseMusicFile }) => {
      releaseMusicFile(this.sourceIds[0]);
    }).catch(() => {});
  }

  /**
   * 获取封面资源（用于 useCoverImage composable）。
   * 通过第一个 SourceId 获取（本地来源会查找歌曲所在目录的封面文件）。
   * @param {string} _size - 图片尺寸（本地来源忽略）
   * @returns {Promise<{url: string, release: Function}|null>}
   */
  async acquireCoverResource(_size = 'medium') {
    if (this.sourceIds && this.sourceIds.length > 0) {
      try {
        const { getAlbumArtResource } = await import('@/api/musicSource/resourceLoader.js');
        return await getAlbumArtResource(this.sourceIds[0]);
      } catch (e) {
        console.warn('获取歌曲封面失败:', e);
      }
    }
    return null;
  }

  /**
   * 获取歌词信息（同步/异步兼容，优先从缓存）
   * @returns {Promise<{plainLyrics: string, syncedLyrics: string, hasSyncedLyrics: boolean, hasPlainLyrics: boolean}>}
   */
  async getLyricsInfo() {
    const result = {
      plainLyrics: '',
      syncedLyrics: '',
      hasSyncedLyrics: false,
      hasPlainLyrics: false
    };

    try {
      let lyricText = '';

      if (this.lyricId) {
        const { getLyricOfSong } = await import('@/api/musicSource/library.js');
        const lyric = await getLyricOfSong(this.id);

        if (lyric && lyric.text) {
          lyricText = lyric.text;
        } else if (lyric && lyric.sourceId) {
          const { getLyricText } = await import('@/api/musicSource/musicResource.js');
          lyricText = await getLyricText(lyric.sourceId) || '';
        }
      }

      if (!lyricText) return result;

      // 检测是否为同步歌词 (LRC 格式)
      if (/^\[(\d{1,2}):(\d{2})\.(\d{2,3})\]/.test(lyricText)) {
        const { parseSyncedLyrics, formatToLRC } = await import('@/api/musicSource/musicResource.js');
        const syncedLines = parseSyncedLyrics(lyricText);
        result.syncedLyrics = formatToLRC(syncedLines);
        result.hasSyncedLyrics = true;
      } else {
        result.plainLyrics = lyricText;
        result.hasPlainLyrics = true;
      }

      return result;
    } catch (error) {
      console.warn('获取歌词信息失败:', error);
      return result;
    }
  }

  // ── 工厂方法 ────────────────────────────────────

  /** 批量创建 */
  static fromDataArray(arr) {
    return (arr || []).map((d) => (d instanceof Song ? d : new Song(d)));
  }

  /** 条件插值: 当字段缺失时从后端 JSON 更新 */
  patch(data) {
    if (data.id !== undefined) this.id = data.id;
    if (data.title !== undefined) this.title = data.title;
    if (data.artist_names !== undefined) this.artistNames = data.artist_names;
    if (data.album_title !== undefined) this.albumTitle = data.album_title;
    if (data.duration !== undefined) this.duration = data.duration;
    if (data.artist_ids !== undefined) this.artistIds = data.artist_ids;
    if (data.album_id !== undefined) this.albumId = data.album_id;
    if (data.lyric_id !== undefined) this.lyricId = data.lyric_id;
    if (data.source_ids !== undefined)
      this.sourceIds = data.source_ids.map((s) =>
        s instanceof SourceId ? s : new SourceId(s),
      );
    return this;
  }
}

/** @deprecated 使用 Song */
export { Song as Track };
