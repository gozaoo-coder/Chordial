/**
 * 歌手类型定义
 */

/**
 * 歌手完整信息
 */
export interface Artist {
  /** 歌手唯一标识 */
  id: string;
  /** 歌手名稱 */
  name: string;
  /** 歌手简介 */
  bio?: string;
  /** 流派列表 */
  genres: string[];
  /** 封面图片数据 (Base64 Data URL) */
  cover_data?: string;
  /** 专辑ID列表 */
  album_ids: string[];
  /** 歌曲ID列表 */
  track_ids: string[];
}

/**
 * 歌手摘要信息
 * 用于列表展示等场景，减少数据传输
 */
export interface ArtistSummary {
  /** 歌手唯一标识 */
  id: string;
  /** 歌手名稱 */
  name: string;
  /** 封面图片数据 (Base64 Data URL) */
  cover_data?: string;
  /** 专辑数量 */
  album_count: number;
  /** 歌曲数量 */
  track_count: number;
}

/**
 * 解析可能包含多个歌手的字符串
 * 支持 "/" 和 "&" 作为分隔符
 * @param artistStr 歌手字符串
 * @returns 歌手名稱数组
 */
export function parseArtists(artistStr: string): string[] {
  if (!artistStr || artistStr.trim() === '') {
    return [];
  }

  // 先尝试 "/" 分隔符
  let parts: string[];
  if (artistStr.includes('/')) {
    parts = artistStr.split('/');
  } else if (artistStr.includes('&')) {
    parts = artistStr.split('&');
  } else {
    parts = [artistStr];
  }

  return parts
    .map(s => normalizeArtistName(s))
    .filter(s => s !== '');
}

/**
 * 规范化歌手名稱
 * @param name 原始名稱
 * @returns 规范化后的名稱
 */
export function normalizeArtistName(name: string): string {
  return name.trim().replace(/\s+/g, ' ');
}

/**
 * 获取主歌手名稱（第一个歌手）
 * @param artistStr 歌手字符串
 * @returns 主歌手名稱
 */
export function getPrimaryArtist(artistStr: string): string {
  const artists = parseArtists(artistStr);
  return artists[0] || '未知歌手';
}
