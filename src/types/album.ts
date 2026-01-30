/**
 * 专辑类型定义
 */

/**
 * 专辑完整信息
 */
export interface Album {
  /** 专辑唯一标识 */
  id: string;
  /** 专辑名稱 */
  title: string;
  /** 歌手ID */
  artist_id: string;
  /** 歌手名稱 */
  artist_name: string;
  /** 发行年份 */
  year?: number;
  /** 流派列表 */
  genres: string[];
  /** 封面图片数据 (Base64 Data URL) */
  cover_data?: string;
  /** 歌曲ID列表 (按曲目顺序) */
  track_ids: string[];
  /** 总时长（秒） */
  total_duration: number;
}

/**
 * 专辑摘要信息
 * 用于列表展示等场景，减少数据传输
 */
export interface AlbumSummary {
  /** 专辑唯一标识 */
  id: string;
  /** 专辑名稱 */
  title: string;
  /** 歌手ID */
  artist_id: string;
  /** 歌手名稱 */
  artist_name: string;
  /** 封面图片数据 (Base64 Data URL) */
  cover_data?: string;
  /** 发行年份 */
  year?: number;
  /** 歌曲数量 */
  track_count: number;
}

/**
 * 将秒数格式化为时长字符串
 * @param seconds 秒数
 * @returns 格式化的时长字符串 (如 "3:45")
 */
export function formatDuration(seconds: number): string {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const secs = seconds % 60;

  if (hours > 0) {
    return `${hours}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
  }
  return `${minutes}:${secs.toString().padStart(2, '0')}`;
}

/**
 * 获取专辑总时长的格式化字符串
 * @param album 专辑对象
 * @returns 格式化的时长字符串
 */
export function getAlbumDurationText(album: Album): string {
  return formatDuration(album.total_duration);
}
