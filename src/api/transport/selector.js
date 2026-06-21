/**
 * Transport 选择器 — 按运行环境自动选择传输方式。
 *
 * | 环境 | 检测条件 | Transport |
 * |------|---------|-----------|
 * | Tauri | `window.__TAURI_INTERNALS__` 存在 | invokeTransport |
 * | Web   | 其他（纯浏览器） | httpTransport |
 *
 * 可通过 `setTransportMode('invoke'|'http')` 运行时切换。
 * Web 模式下，服务器地址从 `import.meta.env.VITE_CHORDIAL_SERVER`
 * 或 `localStorage` 读取，默认 `http://localhost:7878`。
 */

import * as invokeTransport from './invokeTransport.js';
import * as httpTransport from './httpTransport.js';

/** @type {'auto'|'invoke'|'http'} */
let _mode = 'auto';

/** @type {import('./base.js').Transport|null} */
let _current = null;

/**
 * 检测当前环境是否在 Tauri 运行时内。
 * @returns {boolean}
 */
function isTauri() {
  return !!(window.__TAURI_INTERNALS__ || window.__TAURI__);
}

/**
 * 读取 Web 模式下的服务器地址。
 */
function resolveHttpBaseUrl() {
  // 1. Vite 环境变量
  if (import.meta.env?.VITE_CHORDIAL_SERVER) {
    return import.meta.env.VITE_CHORDIAL_SERVER;
  }
  // 2. localStorage 配置
  try {
    const stored = localStorage.getItem('chordial_server_url');
    if (stored) return stored;
  } catch (_) { /* localStorage 不可用 */ }
  // 3. 默认
  return 'http://localhost:7878';
}

/**
 * 获取当前 transport 实例。
 *
 * @returns {import('./base.js').Transport}
 */
export function getTransport() {
  if (_current && _mode !== 'auto') {
    return _current;
  }

  // 自动检测
  if (_mode === 'auto') {
    if (isTauri()) {
      _current = invokeTransport;
    } else {
      _current = httpTransport;
      httpTransport.setBaseUrl(resolveHttpBaseUrl());
    }
  } else if (_mode === 'invoke') {
    _current = invokeTransport;
  } else if (_mode === 'http') {
    _current = httpTransport;
    httpTransport.setBaseUrl(resolveHttpBaseUrl());
  }

  return _current;
}

/**
 * 运行时切换传输模式（供调试/配置页使用）。
 *
 * @param {'invoke'|'http'} mode
 */
export function setTransportMode(mode) {
  _mode = mode;
  _current = null; // 强制下次 getTransport() 重新解析
  getTransport();   // 立即解析
}

/**
 * 获取当前传输模式名称。
 * @returns {string}
 */
export function getTransportMode() {
  if (_mode === 'auto') {
    return isTauri() ? 'invoke' : 'http';
  }
  return _mode;
}

export default { getTransport, setTransportMode, getTransportMode };
