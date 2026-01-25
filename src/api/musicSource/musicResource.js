/**
 * 音乐资源获取 API
 * 使用 Tauri IPC Response 传递二进制大文件
 */

import { invoke } from '@tauri-apps/api/core';

/**
 * 获取音乐完整信息
 * @param {string} trackId - 曲目 ID
 * @returns {Promise<Object>} 音乐的完整信息
 */
export async function getTrackInfo(trackId) {
  return invoke('get_track_info', { track_id: trackId });
}

/**
 * 获取专辑图片（使用二进制响应）
 * @param {string} albumId - 专辑 ID
 * @param {string} size - 图片尺寸 ('small', 'medium', 'large')
 * @returns {Promise<ArrayBuffer>} 图片二进制数据
 */
export async function getAlbumArt(albumId, size = 'medium') {
  const result = await invoke('get_album_art', { album_id: albumId, size });
  // 处理 Tauri Response 返回的数据
  if (result && result.data) {
    // 如果返回的是 ArrayBuffer
    if (result.data instanceof ArrayBuffer) {
      return result.data;
    }
    // 如果返回的是 Uint8Array
    if (result.data instanceof Uint8Array) {
      return result.data.buffer;
    }
    // 如果返回的是普通的对象
    return new TextEncoder().encode(JSON.stringify(result.data)).buffer;
  }
  throw new Error('无效的响应格式');
}

/**
 * 获取音乐文件（使用二进制响应）
 * @param {string} trackId - 曲目 ID
 * @returns {Promise<ArrayBuffer>} 音乐文件二进制数据
 */
export async function getMusicFile(trackId) {
  const result = await invoke('get_music_file', { track_id: trackId });
  // 处理 Tauri Response 返回的数据
  if (result && result.data) {
    if (result.data instanceof ArrayBuffer) {
      return result.data;
    }
    if (result.data instanceof Uint8Array) {
      return result.data.buffer;
    }
    return new TextEncoder().encode(JSON.stringify(result.data)).buffer;
  }
  throw new Error('无效的响应格式');
}

/**
 * 获取歌手图片
 * @param {string} artistId - 歌手 ID
 * @returns {Promise<ArrayBuffer>} 歌手图片二进制数据
 */
export async function getArtistImage(artistId) {
  const result = await invoke('get_artist_image', { artist_id: artistId });
  if (result && result.data) {
    if (result.data instanceof ArrayBuffer) {
      return result.data;
    }
    if (result.data instanceof Uint8Array) {
      return result.data.buffer;
    }
    return new TextEncoder().encode(JSON.stringify(result.data)).buffer;
  }
  throw new Error('无效的响应格式');
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
 * 解析同步歌词 JSON 字符串
 * @param {string} jsonString - JSON 字符串
 * @returns {Array} 解析后的同步歌词数组
 */
export function parseSyncedLyrics(jsonString) {
  if (!jsonString) return [];
  
  try {
    const data = JSON.parse(jsonString);
    if (Array.isArray(data)) {
      return data.map(line => ({
        time: line.timestamp / 1000,  // 转换为秒
        text: line.text
      }));
    }
  } catch (e) {
    console.error('解析同步歌词失败:', e);
  }
  
  return [];
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
  getAlbumArt,
  getMusicFile,
  getArtistImage,
  getLyrics,
  getPlaylistInfo,
};