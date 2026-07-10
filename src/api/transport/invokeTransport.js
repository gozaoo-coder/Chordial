/**
 * Tauri Invoke Transport — 库调用形式（进程内通信）。
 *
 * 前端通过 Tauri IPC `invoke` 调用后端命令，全程进程内，无网络开销。
 * 媒体流通过 chordial:// 自定义协议传输（浏览器原生 Range 支持）。
 *
 * @implements {import('./base.js').Transport}
 */

import { invoke } from '@tauri-apps/api/core';
import { perf } from '@/utils/performanceMonitor.js';

/**
 * 调用后端命令（Tauri invoke 直通）。
 *
 * @param {string} name - 命令名称
 * @param {object} [args={}] - 命令参数
 * @returns {Promise<any>}
 */
export async function command(name, args = {}) {
  return perf.measureAsync(`transport.${name}`, invoke(name, args));
}

// ── 媒体 URL 构建（复用 chordialUrl.js 逻辑）───────────────

// 平台检测缓存：运行时平台不会变化，避免每次 streamUrl 都做字符串扫描
const _isWindows = (() => {
  return navigator.platform?.toLowerCase().includes('win')
      || navigator.userAgent?.toLowerCase().includes('windows');
})();

// chordial 协议基础 URL 缓存
const _baseUrl = _isWindows ? 'http://chordial.localhost' : 'chordial://localhost';

/**
 * 检测当前是否为 Windows 平台（返回缓存值）
 */
function isWindows() {
  return _isWindows;
}

/**
 * 获取 chordial 协议的基础 URL（返回缓存值）
 */
function getBaseUrl() {
  return _baseUrl;
}

/**
 * 标准 base64 → base64url（无填充）
 */
function toBase64Url(str) {
  const base64 = btoa(unescape(encodeURIComponent(str)));
  return base64.replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

/**
 * 构建媒体流 URL（chordial:// 自定义协议）。
 *
 * @param {'audio'|'image'|'lyric'} type
 * @param {string} sourceName
 * @param {string} entityId
 * @returns {string}
 */
export function streamUrl(type, sourceName, entityId) {
  const sn = toBase64Url(sourceName);
  const eid = toBase64Url(entityId);
  return `${getBaseUrl()}/${type}/${sn}/${eid}`;
}

export default { command, streamUrl };
