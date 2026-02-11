/**
 * 类型类统一导出
 * 
 * 这些类包装了从后端获取的数据，提供类型安全和便捷方法
 */

export { Artist } from './Artist';
export { ArtistSummary } from './ArtistSummary';
export { Album } from './Album';
export { AlbumSummary } from './AlbumSummary';
export { Track, getTrackResourcePool } from './Track';
export { BlobResource, BlobResourcePool, globalResourceManager } from './BlobResource';
