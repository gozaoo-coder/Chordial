/**
 * HTTP Transport — Web 服务器形式（网络 REST API）。
 *
 * 前端通过 HTTP 请求与 chordial-server 通信，可在纯浏览器环境运行（无需 Tauri）。
 * 媒体流通过 HTTP 端点传输（支持 Range / 206 Partial Content）。
 *
 * @implements {import('./base.js').Transport}
 */

/** @type {string} */
let _baseUrl = 'http://localhost:7878';

/**
 * 设置服务器基础 URL。
 * @param {string} url
 */
export function setBaseUrl(url) {
  _baseUrl = url.replace(/\/+$/, '');
}

/**
 * 获取当前服务器基础 URL。
 * @returns {string}
 */
export function getBaseUrl() {
  return _baseUrl;
}

/**
 * 调用后端命令（通过 HTTP POST /rpc）。
 *
 * @param {string} name - 命令名称
 * @param {object} [args={}] - 命令参数
 * @returns {Promise<any>}
 */
export async function command(name, args = {}) {
  const response = await fetch(`${_baseUrl}/rpc`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ name, args }),
  });

  if (!response.ok) {
    const text = await response.text();
    throw new Error(text || `HTTP ${response.status}: ${response.statusText}`);
  }

  return response.json();
}

/**
 * 标准 base64 → base64url（无填充）
 */
function toBase64Url(str) {
  const base64 = btoa(unescape(encodeURIComponent(str)));
  return base64.replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

/**
 * 构建媒体流 URL（HTTP 端点）。
 *
 * @param {'audio'|'image'|'lyric'} type
 * @param {string} sourceName
 * @param {string} entityId
 * @returns {string}
 */
export function streamUrl(type, sourceName, entityId) {
  const sn = toBase64Url(sourceName);
  const eid = toBase64Url(entityId);
  return `${_baseUrl}/${type}/${sn}/${eid}`;
}

export default { command, streamUrl, setBaseUrl, getBaseUrl };
