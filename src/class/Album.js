/**
 * Album — 专辑
 *
 * 对应后端 `music_library::models::Album`。
 */
import { SourceId } from './SourceId.js';

export class Album {
  /** @param {Partial<Album>} data */
  constructor(data = {}) {
    /** 库内统一 ID */
    this.id = data.id ?? '';
    /** 专辑标题 */
    this.title = data.title ?? '未知专辑';
    /** 所属艺术家 ID */
    this.artistId = data.artist_id ?? data.artistId ?? '';
    /** 封面图片 URL */
    this.coverUrl = data.cover_url ?? data.coverUrl ?? null;
    /** 包含的歌曲 ID 列表 */
    this.songIds = data.song_ids ?? data.songIds ?? [];
    /** @deprecated 使用 songIds */
    this.trackIds = data.track_ids ?? data.trackIds ?? this.songIds;
    /** 来源引用列表 */
    this.sourceIds = (data.source_ids ?? data.sourceIds ?? []).map(
      (s) => (s instanceof SourceId ? s : new SourceId(s)),
    );

    // ── 外部注入 / 计算 ──────────────────────────
    /** 年份（外部注入，后端未提供该字段） */
    this._year = data.year ?? null;
  }

  get year() {
    return this._year;
  }

  getTrackCount() {
    return this.songIds.length;
  }

  // ── 资源方法 ────────────────────────────────────

  /**
   * 获取封面资源（用于 useCoverImage composable）。
   * 优先使用 coverUrl，其次通过第一个 SourceId 获取。
   * @param {string} _size - 图片尺寸（本地来源忽略）
   * @returns {Promise<{url: string, release: Function}|null>}
   */
  async acquireCoverResource(_size = 'medium') {
    // 优先级1: 已有 coverUrl
    if (this.coverUrl) {
      return { url: this.coverUrl, release: () => {} };
    }
    // 优先级2: 通过 SourceId 获取
    if (this.sourceIds && this.sourceIds.length > 0) {
      try {
        const { getAlbumArtResource } = await import('@/api/musicSource/resourceLoader.js');
        return await getAlbumArtResource(this.sourceIds[0]);
      } catch (e) {
        console.warn('获取专辑封面失败:', e);
      }
    }
    return null;
  }

  static fromDataArray(arr) {
    return (arr || []).map((d) => (d instanceof Album ? d : new Album(d)));
  }
}

// ══════════════════════════════════════════════════════════════════════════════
// AlbumSummary — 轻量摘要版
// ══════════════════════════════════════════════════════════════════════════════

export class AlbumSummary {
  constructor(data = {}) {
    this.id = data.id ?? '';
    this.title = data.title ?? '未知专辑';
    this.artistId = data.artist_id ?? data.artistId ?? '';
    /** 封面图片 URL */
    this.coverUrl = data.cover_url ?? data.coverUrl ?? null;
    this.songCount = data.song_count ?? data.songCount ?? (data.song_ids?.length ?? 0);
    this._year = data.year ?? null;
  }

  get year() {
    return this._year;
  }

  get trackCount() {
    return this.songCount;
  }

  getTrackCount() {
    return this.songCount;
  }

  /**
   * 获取封面资源（用于 useCoverImage composable）。
   * @param {string} _size
   * @returns {Promise<{url: string, release: Function}|null>}
   */
  async acquireCoverResource(_size = 'medium') {
    if (this.coverUrl) {
      return { url: this.coverUrl, release: () => {} };
    }
    return null;
  }

  static fromDataArray(arr) {
    return (arr || []).map((d) => new AlbumSummary(d));
  }
}
