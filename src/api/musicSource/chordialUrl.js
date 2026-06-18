/**
 * chordial:// 自定义协议 URL 构建工具。
 *
 * 使用 Tauri 的 register_asynchronous_uri_scheme_protocol 注册的 chordial 协议，
 * 后端直接流式传输文件，前端无需 invoke + Blob + createObjectURL，
 * 浏览器原生处理流式传输和 Range 请求（用于 seeking）。
 *
 * URL 格式（自动适配平台）:
 *   macOS / Linux: chordial://localhost/<type>/<sn>/<eid>
 *   Windows:        http://chordial.localhost/<type>/<sn>/<eid>
 *
 * base64url 编码规则: + → -, / → _, 无 = 填充。
 */

/**
 * 检测当前是否为 Windows 平台
 */
function isWindows() {
  // Tauri 环境下 navigator.userAgent 会包含 "Windows"
  return navigator.platform?.toLowerCase().includes('win')
      || navigator.userAgent?.toLowerCase().includes('windows');
}

/**
 * 获取 chordial 协议的基础 URL
 */
function getBaseUrl() {
  if (isWindows()) {
    return 'http://chordial.localhost';
  }
  return 'chordial://localhost';
}

/**
 * 标准 base64 → base64url（无填充）
 */
function toBase64Url(str) {
  // btoa 处理 UTF-8 字符串需要先编码
  const base64 = btoa(unescape(encodeURIComponent(str)));
  return base64.replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

/**
 * 构建 chordial:// 协议 URL。
 *
 * @param {'audio'|'image'|'lyric'} type - 资源类型
 * @param {string} sourceName - 来源名称（如 "local"）
 * @param {string} entityId - 来源内部的实体 ID（如文件路径）
 * @returns {string} chordial:// URL
 */
export function buildChordialUrl(type, sourceName, entityId) {
  const sn = toBase64Url(sourceName);
  const eid = toBase64Url(entityId);
  return `${getBaseUrl()}/${type}/${sn}/${eid}`;
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
