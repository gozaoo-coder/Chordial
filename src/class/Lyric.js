/**
 * Lyric — 歌词
 *
 * 对应后端 `music_library::models::Lyric`。
 */
import { SourceId } from './SourceId.js';

export class Lyric {
  /** @param {Partial<Lyric>} data */
  constructor(data = {}) {
    /** 库内统一 ID */
    this.id = data.id ?? '';
    /** 关联的歌曲 ID */
    this.songId = data.song_id ?? data.songId ?? '';
    /** 歌词文本 */
    this.text = data.text ?? '';
    /** 来源引用 */
    this.sourceId = data.source_id
      ? data.source_id instanceof SourceId
        ? data.source_id
        : new SourceId(data.source_id)
      : data.sourceId instanceof SourceId
        ? data.sourceId
        : null;
  }

  /** 解析为逐行数组 */
  get lines() {
    return this.text ? this.text.split(/\r?\n/) : [];
  }

  /** 是否为 LRC 同步歌词 */
  get isSynced() {
    return /^\[(\d{1,2}):(\d{2})\.(\d{2,3})\]/.test(this.text);
  }

  static fromDataArray(arr) {
    return (arr || []).map((d) => (d instanceof Lyric ? d : new Lyric(d)));
  }
}
