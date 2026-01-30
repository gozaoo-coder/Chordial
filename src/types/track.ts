/**
 * 单曲类型定义
 */
import type { ArtistSummary } from './artist';
import type { AlbumSummary } from './album';

/**
 * 单曲元数据
 */
export interface Track {
  /** 歌曲唯一标识 */
  id: string;
  /** 所属源ID */
  source_id: string;
  /** 文件路径 */
  path: string;
  /** 文件名 */
  file_name: string;
  /** 标题 */
  title?: string;
  /** 艺术家（原始字符串，可能包含多个歌手） */
  artist?: string;
  /** 艺术家ID */
  artist_id?: string;
  /** 歌手摘要信息（用于快速展示） */
  artist_summary?: ArtistSummary;
  /** 专辑 */
  album?: string;
  /** 专辑ID */
  album_id?: string;
  /** 专辑摘要信息（用于快速展示） */
  album_summary?: AlbumSummary;
  /** 专辑封面数据 (Base64 编码) */
  album_cover_data?: string;
  /** 时长（秒） */
  duration?: number;
  /** 格式 */
  format: string;
  /** 文件大小（字节） */
  file_size: number;
  /** 比特率 (kbps) */
  bitrate?: number;
  /** 采样率 (Hz) */
  sample_rate?: number;
  /** 声道数 */
  channels?: number;
  /** 年份 */
  year?: number;
  /** 流派 */
  genre?: string;
  /** 作曲 */
  composer?: string;
  /** 备注 */
  comment?: string;
  /** 歌词（纯文本） */
  lyrics?: string;
  /** 同步歌词（JSON 格式的时间戳歌词） */
  synced_lyrics?: string;
  /** 添加时间 */
  added_at: string;
}

/**
 * 歌词信息
 */
export interface LyricsInfo {
  /** 纯文本歌词 */
  plain_lyrics: string;
  /** 同步歌词 */
  synced_lyrics: string;
  /** 是否有同步歌词 */
  has_synced_lyrics: boolean;
  /** 是否有纯文本歌词 */
  has_plain_lyrics: boolean;
}

/**
 * 解析后的同步歌词行
 */
export interface ParsedLyricLine {
  /** 时间（秒） */
  time: number;
  /** 歌词文本 */
  text: string;
}

/**
 * 将同步歌词 JSON 字符串解析为数组
 * @param syncedLyrics JSON 格式的同步歌词字符串
 * @returns 解析后的歌词行数组
 */
export function parseSyncedLyrics(syncedLyrics?: string): ParsedLyricLine[] {
  if (!syncedLyrics) {
    return [];
  }

  try {
    const parsed = JSON.parse(syncedLyrics);
    if (Array.isArray(parsed)) {
      return parsed.map((line: any) => ({
        time: (line.timestamp || line.time || 0) / 1000, // 转换为秒
        text: line.text || ''
      }));
    }
  } catch (e) {
    console.warn('Failed to parse synced lyrics:', e);
  }

  return [];
}

/**
 * 格式化文件大小
 * @param bytes 字节数
 * @returns 格式化后的字符串 (如 "3.5 MB")
 */
export function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 B';

  const units = ['B', 'KB', 'MB', 'GB'];
  const k = 1024;
  const i = Math.floor(Math.log(bytes) / Math.log(k));

  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + units[i];
}

/**
 * 格式化比特率
 * @param bitrate 比特率 (kbps)
 * @returns 格式化后的字符串 (如 "320 kbps")
 */
export function formatBitrate(bitrate?: number): string {
  if (!bitrate) return '未知';
  return `${bitrate} kbps`;
}

/**
 * 格式化采样率
 * @param sampleRate 采样率 (Hz)
 * @returns 格式化后的字符串 (如 "44.1 kHz")
 */
export function formatSampleRate(sampleRate?: number): string {
  if (!sampleRate) return '未知';
  return `${(sampleRate / 1000).toFixed(1)} kHz`;
}
