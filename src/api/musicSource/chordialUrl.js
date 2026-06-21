/**
 * chordial:// 自定义协议 URL 构建工具。
 *
 * 委托给 transport.streamUrl() 以确保双传输模式下 URL 格式一致。
 *
 * - Tauri 模式：chordial://localhost/<type>/<sn>/<eid> (或 Windows http://chordial.localhost/...)
 * - HTTP 模式：http://<host>:<port>/<type>/<sn>/<eid>
 *
 * base64url 编码规则: + → -, / → _, 无 = 填充。
 */

import { transport } from '@/api/transport';

/**
 * 构建 chordial:// 协议 URL。
 *
 * @param {'audio'|'image'|'lyric'} type - 资源类型
 * @param {string} sourceName - 来源名称（如 "local"）
 * @param {string} entityId - 来源内部的实体 ID（如文件路径）
 * @returns {string}
 */
export function buildChordialUrl(type, sourceName, entityId) {
  return transport.streamUrl(type, sourceName, entityId);
}

/**
 * 从 SourceId 对象构建音频 chordial:// URL。
 *
 * @param {import('@/class').SourceId} sourceId
 * @returns {string}
 */
export function buildAudioUrl(sourceId) {
  return buildChordialUrl('audio', sourceId.sourceName, sourceId.entityId);
}

/**
 * 从 SourceId 对象构建图片 chordial:// URL。
 *
 * @param {import('@/class').SourceId} sourceId
 * @returns {string}
 */
export function buildImageUrl(sourceId) {
  return buildChordialUrl('image', sourceId.sourceName, sourceId.entityId);
}

/**
 * 从 SourceId 对象构建歌词 chordial:// URL。
 *
 * @param {import('@/class').SourceId} sourceId
 * @returns {string}
 */
export function buildLyricUrl(sourceId) {
  return buildChordialUrl('lyric', sourceId.sourceName, sourceId.entityId);
}

export default {
  buildChordialUrl,
  buildAudioUrl,
  buildImageUrl,
  buildLyricUrl,
};
