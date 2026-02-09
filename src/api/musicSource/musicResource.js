/**
 * 音乐资源获取 API
 * 使用 Tauri IPC Response 传递二进制大文件
 * 
 * 所有返回的数据都会被包装成对应的类实例
 */

import { invoke } from '@tauri-apps/api/core';
import { Track } from '@/class';

/**
 * 处理二进制响应数据
 * 统一处理 Tauri 返回的各种二进制数据格式
 * @param {Object|ArrayBuffer|Uint8Array} result - Tauri 响应结果
 * @returns {ArrayBuffer} 处理后的 ArrayBuffer
 * @throws {Error} 当响应格式无效时抛出
 */
function processBinaryResponse(result) {
  // 处理包含 data 属性的响应对象
  if (result && result.data) {
    if (result.data instanceof ArrayBuffer) {
      return result.data;
    }
    if (result.data instanceof Uint8Array) {
      return result.data.buffer;
    }
    if (Array.isArray(result.data)) {
      return new Uint8Array(result.data).buffer;
    }
  }

  // 直接返回的 ArrayBuffer
  if (result instanceof ArrayBuffer) {
    return result;
  }

  // 直接返回的 Uint8Array
  if (result instanceof Uint8Array) {
    return result.buffer;
  }

  // 数组格式（JSON 序列化的二进制数据）
  if (Array.isArray(result)) {
    return new Uint8Array(result).buffer;
  }

  throw new Error(`无效的响应格式: ${typeof result}`);
}

/**
 * 获取音乐完整信息
 * @param {string} trackId - 曲目 ID
 * @returns {Promise<Track>} 音乐的完整信息（Track 实例）
 */
export async function getTrackInfo(trackId) {
  const data = await invoke('get_track_info', { track_id: trackId });
  return new Track(data);
}

/**
 * 批量获取曲目信息
 * @param {string[]} trackIds - 曲目 ID 数组
 * @returns {Promise<Track[]>} 曲目实例数组
 */
export async function getTracksByIds(trackIds) {
  if (!trackIds || trackIds.length === 0) {
    return [];
  }
  const data = await invoke('get_tracks_by_ids', { track_ids: trackIds });
  return data.map(item => new Track(item));
}

/**
 * 获取专辑图片（使用二进制响应）
 * @param {string} albumId - 专辑 ID
 * @param {string} size - 图片尺寸 ('small', 'medium', 'large')
 * @returns {Promise<ArrayBuffer>} 图片二进制数据
 */
export async function getAlbumArt(albumId, size = 'medium') {
  const result = await invoke('get_album_art', { album_id: albumId, size });
  return processBinaryResponse(result);
}

/**
 * 获取音乐文件（使用二进制响应）
 * @param {string} trackId - 曲目 ID
 * @returns {Promise<ArrayBuffer>} 音乐文件二进制数据
 */
export async function getMusicFile(trackId) {
  const result = await invoke('get_music_file', { track_id: trackId });
  return processBinaryResponse(result);
}

/**
 * 获取歌手图片
 * @param {string} artistId - 歌手 ID
 * @returns {Promise<ArrayBuffer>} 歌手图片二进制数据
 */
export async function getArtistImage(artistId) {
  const result = await invoke('get_artist_image', { artist_id: artistId });
  return processBinaryResponse(result);
}

/**
 * 获取歌词文件
 * @param {string} trackId - 曲目 ID
 * @returns {Promise<Object>} 歌词信息对象
 */
export async function getLyrics(trackId) {
  const result = await invoke('get_lyrics', { track_id: trackId });
  
  // 处理返回的 JSON 对象
  if (typeof result === 'object' && result !== null) {
    return {
      plainLyrics: result.plain_lyrics || '',
      syncedLyrics: result.synced_lyrics || '',
      hasSyncedLyrics: result.has_synced_lyrics || false,
      hasPlainLyrics: result.has_plain_lyrics || false
    };
  }
  
  // 兼容旧版本返回字符串的情况
  return {
    plainLyrics: result || '',
    syncedLyrics: '',
    hasSyncedLyrics: false,
    hasPlainLyrics: !!result
  };
}

/**
 * 解析同步歌词
 * 支持 JSON 格式和 LRC 文本格式 [mm:ss.xx]歌词内容
 * @param {string} content - 歌词内容（JSON 字符串或 LRC 格式文本）
 * @returns {Array} 解析后的同步歌词数组
 */
export function parseSyncedLyrics(content) {
  if (!content) return [];

  // 首先尝试解析为 JSON 格式
  try {
    const data = JSON.parse(content);
    if (Array.isArray(data)) {
      return data.map(line => ({
        time: line.timestamp / 1000,  // 转换为秒
        text: line.text
      }));
    }
  } catch (e) {
    // 不是 JSON，继续尝试 LRC 格式
  }

  // 解析 LRC 格式: [mm:ss.xx]歌词内容 或 [mm:ss.xxx]歌词内容
  const lyrics = [];
  const lines = content.split(/\r?\n/);
  // 支持 [mm:ss.xx] 或 [mm:ss.xxx] 格式，分钟可以是1位或2位
  const timeRegex = /\[(\d{1,2}):(\d{2})\.(\d{2,3})\]/g;

  for (const line of lines) {
    const trimmed = line.trim();
    if (!trimmed) continue;

    // 重置正则表达式的 lastIndex
    timeRegex.lastIndex = 0;

    // 查找所有时间标签
    const timeMatches = [...trimmed.matchAll(timeRegex)];

    if (timeMatches.length > 0) {
      // 获取最后一个时间标签后的文本
      const lastMatch = timeMatches[timeMatches.length - 1];
      const textStartIndex = lastMatch.index + lastMatch[0].length;
      const text = trimmed.substring(textStartIndex).trim();

      // 为每个时间标签创建一个歌词条目
      for (const match of timeMatches) {
        const minutes = parseInt(match[1], 10);
        const seconds = parseInt(match[2], 10);
        const msPart = match[3];
        // 处理毫秒：2位是百分之一秒，3位是毫秒
        const milliseconds = msPart.length === 2
          ? parseInt(msPart, 10) * 10
          : parseInt(msPart, 10);
        const time = (minutes * 60 + seconds) + milliseconds / 1000;

        if (text) {
          lyrics.push({ time, text });
        }
      }
    }
  }

  return lyrics.sort((a, b) => a.time - b.time);
}

/**
 * 格式化歌词为 LRC 格式
 * @param {Array} syncedLyrics - 同步歌词数组
 * @returns {string} LRC 格式的歌词
 */
export function formatToLRC(syncedLyrics) {
  if (!syncedLyrics || syncedLyrics.length === 0) return '';
  
  return syncedLyrics.map(line => {
    const minutes = Math.floor(line.time / 60);
    const seconds = Math.floor(line.time % 60);
    const millis = Math.floor((line.time % 1) * 100);
    return `[${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}.${millis.toString().padStart(3, '0')}]${line.text}`;
  }).join('\n');
}

/**
 * 获取播放列表详情
 * @param {string} playlistId - 播放列表 ID
 * @returns {Promise<Object>} 播放列表详情
 */
export async function getPlaylistInfo(playlistId) {
  return invoke('get_playlist_info', { playlist_id: playlistId });
}

export default {
  getTrackInfo,
  getTracksByIds,
  getAlbumArt,
  getMusicFile,
  getArtistImage,
  getLyrics,
  getPlaylistInfo,
};
