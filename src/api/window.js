/**
 * 窗口控制 API
 *
 * 基于 Tauri v2 内置 Window API，无需手写 Rust 命令。
 * 窗口位置/尺寸由 tauri-plugin-window-state 自动持久化。
 *
 * @example
 * import { setWindowSize, getWindowPosition } from '@/api/window';
 * await setWindowSize(1280, 720);
 * const { x, y } = await getWindowPosition();
 */

import { getCurrentWindow } from '@tauri-apps/api/window';

// ══════════════════════════════════════════════════════════════════════════════
// 置顶
// ══════════════════════════════════════════════════════════════════════════════

/**
 * 切换窗口置顶状态
 * @returns {Promise<boolean>} 切换后的置顶状态
 */
export async function toggleAlwaysOnTop() {
  const win = getCurrentWindow();
  const current = await win.isAlwaysOnTop();
  await win.setAlwaysOnTop(!current);
  return !current;
}

// ══════════════════════════════════════════════════════════════════════════════
// 窗口生命周期
// ══════════════════════════════════════════════════════════════════════════════

/**
 * 关闭窗口
 */
export async function closeWindow() {
  await getCurrentWindow().close();
}

/**
 * 最小化窗口
 */
export async function minimizeWindow() {
  await getCurrentWindow().minimize();
}

/**
 * 切换窗口最大化状态
 * @returns {Promise<boolean>} 切换后的最大化状态
 */
export async function toggleMaximize() {
  const win = getCurrentWindow();
  const maximized = await win.isMaximized();
  await win.toggleMaximize();
  return !maximized;
}

// ══════════════════════════════════════════════════════════════════════════════
// 位置
// ══════════════════════════════════════════════════════════════════════════════

/**
 * 获取窗口位置
 * @returns {Promise<{x: number, y: number}>} 窗口坐标
 */
export async function getWindowPosition() {
  const pos = await getCurrentWindow().innerPosition();
  return { x: pos.x, y: pos.y };
}

/**
 * 设置窗口位置
 * @param {number} x - X坐标
 * @param {number} y - Y坐标
 */
export async function setWindowPosition(x, y) {
  await getCurrentWindow().setPosition({ x, y });
}

// ══════════════════════════════════════════════════════════════════════════════
// 尺寸
// ══════════════════════════════════════════════════════════════════════════════

/**
 * 获取窗口尺寸
 * @returns {Promise<{width: number, height: number}>} 窗口尺寸
 */
export async function getWindowSize() {
  const size = await getCurrentWindow().innerSize();
  return { width: size.width, height: size.height };
}

/**
 * 设置窗口尺寸
 * @param {number} width - 窗口宽度
 * @param {number} height - 窗口高度
 */
export async function setWindowSize(width, height) {
  await getCurrentWindow().setSize({ width, height });
}

// ══════════════════════════════════════════════════════════════════════════════
// 窗口状态持久化（tauri-plugin-window-state）
// ══════════════════════════════════════════════════════════════════════════════

/**
 * 初始化窗口状态管理（自动恢复上次的尺寸/位置/最大化状态）
 * 应在应用入口调用一次。
 */
export async function initWindowState() {
  const { restoreStateCurrent } = await import('@tauri-apps/plugin-window-state');
  await restoreStateCurrent();
}

/**
 * 手动保存当前窗口状态
 */
export async function saveWindowState() {
  const { saveWindowState: saveFn } = await import('@tauri-apps/plugin-window-state');
  await saveFn();
}

// 默认导出
export default {
  toggleAlwaysOnTop,
  closeWindow,
  minimizeWindow,
  toggleMaximize,
  getWindowPosition,
  setWindowPosition,
  getWindowSize,
  setWindowSize,
  initWindowState,
  saveWindowState,
};
