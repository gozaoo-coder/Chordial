/**
 * 音乐资源获取 API — 后端 get_* 命令
 *
 * 大文件通过 Tauri 的 raw payload 传输（Vec<u8> / String）。
 * 前端以 `source_id_json` 的形式传入 {@link SourceId} 的 JSON 序列化字符串。
 *
 * 链路：
 *   front → tauri → get_song_file(source_id_json)
 *     → trait MusicSource::song_file_get → Vec<u8> → 前端
 */

import { invoke } from '@tauri-apps/api/core';
import { SourceId } from '@/class';

// ══════════════════════════════════════════════════════════════════════════════
// Helpers
// ══════════════════════════════════════════════════════════════════════════════

/** 将 SourceId 实例或普通对象序列化为后端所需的 JSON 字符串 */
function serializeSourceId(sourceId) {
  const obj = sourceId instanceof SourceId ? sourceId : new SourceId(sourceId);
  return JSON.stringify(obj.toJSON());
}

/** 处理 Tauri 返回的二进制数据（可能是 Uint8Array 或 number[]） */
function toArrayBuffer(data) {
  if (data instanceof ArrayBuffer) return data;
  if (data instanceof Uint8Array) return data.buffer.slice(data.byteOffset, data.byteOffset + data.byteLength);
  if (Array.isArray(data)) return new Uint8Array(data).buffer;
  throw new Error(`无效的二进制响应格式: ${typeof data}`);
}

// ══════════════════════════════════════════════════════════════════════════════
// Resource Commands
// ══════════════════════════════════════════════════════════════════════════════

/**
 * 获取歌曲的音频文件数据。
 *
 * @param {SourceId|object} sourceId - 歌曲的来源标识
 * @returns {Promise<ArrayBuffer>} 音频文件的原始字节数据
 */
export async function getMusicFile(sourceId) {
  const result = await invoke('get_song_file', {
    sourceIdJson: serializeSourceId(sourceId),
  });
  return toArrayBuffer(result);
}

/** @deprecated 使用 {@link getMusicFile} */
export const getSongFile = getMusicFile;

/**
 * 获取专辑的封面图片数据。
 *
 * @param {SourceId|object} sourceId - 专辑的来源标识
 * @returns {Promise<ArrayBuffer>} 图片文件的原始字节数据（JPEG / PNG）
 */
export async function getAlbumPicture(sourceId) {
  const result = await invoke('get_album_picture', {
    sourceIdJson: serializeSourceId(sourceId),
  });
  return toArrayBuffer(result);
}

/** @deprecated 旧名称，使用 {@link getAlbumPicture} */
export const getAlbumArt = getAlbumPicture;

/**
 * 获取歌曲的歌词文本。
 *
 * @param {SourceId|object} sourceId - 歌词的来源标识
 * @returns {Promise<string>} 歌词原始文本（LRC 或纯文本）
 */
export async function getLyricText(sourceId) {
  return invoke('get_lyric_text', {
    sourceIdJson: serializeSourceId(sourceId),
  });
}

/** @deprecated 旧名称，使用 {@link getLyricText} */
export const getLyrics = getLyricText;

// ══════════════════════════════════════════════════════════════════════════════
// Utility: LRC parse / format
// ══════════════════════════════════════════════════════════════════════════════

/**
 * 解析同步歌词（支持 JSON 和 LRC 两种格式）。
 *
 * @param {string} content - 歌词内容
 * @returns {{time: number, text: string}[]} 解析后的同步歌词数组
 */
export function parseSyncedLyrics(content) {
  if (!content) return [];

  // 1. 尝试 JSON
  try {
    const data = JSON.parse(content);
    if (Array.isArray(data)) {
      return data.map((line) => ({
        time: line.timestamp / 1000,
        text: line.text,
      }));
    }
  } catch (_) {
    // 非 JSON，尝试 LRC
  }

  // 2. LRC 格式 [mm:ss.xx]歌词 或 [mm:ss.xxx]歌词
  const lyrics = [];
  const lines = content.split(/\r?\n/);
  const timeRegex = /\[(\d{1,2}):(\d{2})\.(\d{2,3})\]/;

  for (const line of lines) {
    const trimmed = line.trim();
    if (!trimmed) continue;

    const matches = [...trimmed.matchAll(timeRegex)];
    if (matches.length === 0) continue;

    const lastMatch = matches[matches.length - 1];
    const text = trimmed.substring(lastMatch.index + lastMatch[0].length).trim();
    if (!text) continue;

    for (const m of matches) {
      const minutes = parseInt(m[1], 10);
      const seconds = parseInt(m[2], 10);
      const msPart = m[3];
      const ms = msPart.length === 2 ? parseInt(msPart, 10) * 10 : parseInt(msPart, 10);
      lyrics.push({ time: minutes * 60 + seconds + ms / 1000, text });
    }
  }

  return lyrics.sort((a, b) => a.time - b.time);
}

/**
 * 将同步歌词格式化为 LRC 文本。
 *
 * @param {{time: number, text: string}[]} syncedLyrics
 * @returns {string} LRC 格式文本
 */
export function formatToLRC(syncedLyrics) {
  if (!syncedLyrics?.length) return '';
  return syncedLyrics
    .map((line) => {
      const mins = Math.floor(line.time / 60);
      const secs = Math.floor(line.time % 60);
      const ms = Math.floor((line.time % 1) * 100);
      return `[${String(mins).padStart(2, '0')}:${String(secs).padStart(2, '0')}.${String(ms).padStart(3, '0')}]${line.text}`;
    })
    .join('\n');
}

// ══════════════════════════════════════════════════════════════════════════════
// Deprecated stubs (前端不再有 getTrackInfo / getTracksByIds / getPlaylistInfo)
// ══════════════════════════════════════════════════════════════════════════════

/** @deprecated 使用 {@link module:src/api/musicSource/library.getSong} */
export async function getTrackInfo(trackId) {
  const { getSong } = await import('./library.js');
  return getSong(trackId);
}

/** @deprecated 使用多次 {@link module:src/api/musicSource/library.getSong} */
export async function getTracksByIds(trackIds) {
  if (!trackIds?.length) return [];
  const { getSong } = await import('./library.js');
  return Promise.all(trackIds.map((id) => getSong(id)));
}

/** @deprecated 使用 {@link module:src/api/artist.getArtist} */
export async function getArtistImage() {
  throw new Error('getArtistImage 已移除 —— 请通过 SourceId + getAlbumPicture 或 Album/Artist 数据中的 cover_url 获取图片');
}

/** @deprecated 后端未实现 */
export async function getPlaylistInfo() {
  throw new Error('getPlaylistInfo 后端未实现');
}

export default {
  getMusicFile,
  getSongFile,
  getAlbumPicture,
  getAlbumArt,
  getLyricText,
  getLyrics,
  parseSyncedLyrics,
  formatToLRC,
  getTrackInfo,
  getTracksByIds,
  getArtistImage,
  getPlaylistInfo,
};
