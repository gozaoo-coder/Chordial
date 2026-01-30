/**
 * 类型定义统一导出
 */

export * from './artist';
export * from './album';
export * from './track';

// 音乐库类型
export interface MusicLibrary {
  /** 所有音乐源 */
  sources: SourceConfig[];
  /** 所有歌曲的元数据 */
  tracks: Track[];
  /** 所有歌手信息 */
  artists: Artist[];
  /** 所有专辑信息 */
  albums: Album[];
}

// 音乐源配置
export interface SourceConfig {
  /** 源唯一标识 */
  id: string;
  /** 源类型 */
  source_type: SourceType;
  /** 源路径 (本地文件夹) 或 URL (网盘) */
  path: string;
  /** 源名称 (可自定义) */
  name: string;
  /** 是否启用 */
  enabled: boolean;
  /** 选项 */
  options: SourceOptions;
  /** 创建时间 */
  created_at: string;
  /** 最后扫描时间 */
  last_scanned_at?: string;
}

// 源类型
export type SourceType = 'LocalFolder' | 'WebDisk';

// 源选项
export interface SourceOptions {
  /** 是否递归扫描 */
  recursive: boolean;
  /** 包含的文件扩展名 */
  extensions: string[];
  /** 排除的模式 */
  exclude_patterns: string[];
}

// 导入其他类型
import type { Artist } from './artist';
import type { Album } from './album';
import type { Track } from './track';
