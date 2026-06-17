/**
 * Artist — 艺术家
 *
 * 对应后端 `music_library::models::Artist`。
 */
import { SourceId } from './SourceId.js';

export class Artist {
  /** @param {Partial<Artist>} data */
  constructor(data = {}) {
    /** 库内统一 ID */
    this.id = data.id ?? '';
    /** 艺术家名称 */
    this.name = data.name ?? '未知艺术家';
    /** 简介 */
    this.bio = data.bio ?? null;
    /** 来源引用列表 */
    this.sourceIds = (data.source_ids ?? data.sourceIds ?? []).map(
      (s) => (s instanceof SourceId ? s : new SourceId(s)),
    );

    // ── 外部注入的汇总数据 ──────────────────────
    /** @type {number} 关联歌曲数量（由调用方注入） */
    this._trackCount = data.track_count ?? data.trackCount ?? 0;
    /** @type {number} 关联专辑数量（由调用方注入） */
    this._albumCount = data.album_count ?? data.albumCount ?? 0;

    // ── 向后兼容：旧 API 中 Artist 携带子实体 ID 列表 ──
    /** @deprecated 使用 @/api/musicSource/library.getSongsByArtist */
    this.trackIds = data.track_ids ?? data.trackIds ?? data.song_ids ?? data.songIds ?? [];
    /** @deprecated 使用 @/api/musicSource/library.getAlbumsByArtist */
    this.albumIds = data.album_ids ?? data.albumIds ?? [];
  }

  // ── 资源方法 ────────────────────────────────────

  /**
   * 获取封面资源（用于 useCoverImage composable）。
   * 通过第一个 SourceId 获取艺术家图片。
   * @param {string} _size - 图片尺寸（本地来源忽略）
   * @returns {Promise<{url: string, release: Function}|null>}
   */
  async acquireCoverResource(_size = 'medium') {
    if (this.sourceIds && this.sourceIds.length > 0) {
      try {
        const { getArtistImageResource } = await import('@/api/musicSource/resourceLoader.js');
        return await getArtistImageResource(this.sourceIds[0]);
      } catch (e) {
        console.warn('获取艺术家封面失败:', e);
      }
    }
    return null;
  }

  // ── 汇总查询 ────────────────────────────────────

  getTrackCount() {
    return this._trackCount;
  }

  getAlbumCount() {
    return this._albumCount;
  }

  // ── 工厂 ────────────────────────────────────────

  static fromDataArray(arr) {
    return (arr || []).map((d) => (d instanceof Artist ? d : new Artist(d)));
  }
}

// ══════════════════════════════════════════════════════════════════════════════
// ArtistSummary — 轻量摘要版
// ══════════════════════════════════════════════════════════════════════════════

export class ArtistSummary {
  constructor(data = {}) {
    this.id = data.id ?? '';
    this.name = data.name ?? '未知艺术家';
    this._trackCount = data.track_count ?? data.trackCount ?? 0;
    this._albumCount = data.album_count ?? data.albumCount ?? 0;
  }

  get trackCount() {
    return this._trackCount;
  }

  get albumCount() {
    return this._albumCount;
  }

  getTrackCount() {
    return this._trackCount;
  }

  getAlbumCount() {
    return this._albumCount;
  }

  /**
   * 获取封面资源（用于 useCoverImage composable）。
   * ArtistSummary 不含封面信息，始终返回 null。
   * @returns {Promise<null>}
   */
  async acquireCoverResource(_size = 'medium') {
    return null;
  }

  static fromDataArray(arr) {
    return (arr || []).map(
      (d) => new ArtistSummary(d),
    );
  }
}
