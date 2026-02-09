import { invoke } from '@tauri-apps/api/core';

/**
 * 窗口控制 API
 * 提供窗口控制相关的功能封装
 */

/**
 * 切换窗口置顶状态
 * @returns {Promise<boolean>} 切换后的置顶状态
 */
export async function toggleAlwaysOnTop() {
  return await invoke('toggle_always_on_top');
}

/**
 * 关闭窗口
 */
export async function closeWindow() {
  await invoke('close_window');
}

/**
 * 最小化窗口
 */
export async function minimizeWindow() {
  await invoke('minimize_window');
}

/**
 * 切换窗口最大化状态
 * @returns {Promise<boolean>} 切换后的最大化状态
 */
export async function toggleMaximize() {
  return await invoke('toggle_maximize');
}

/**
 * 获取窗口位置
 * @returns {Promise<{x: number, y: number}>} 窗口坐标
 */
export async function getWindowPosition() {
  const [x, y] = await invoke('get_window_position');
  return { x, y };
}

/**
 * 设置窗口位置
 * @param {number} x - X坐标
 * @param {number} y - Y坐标
 */
export async function setWindowPosition(x, y) {
  await invoke('set_window_position', { x, y });
}

/**
 * 获取窗口尺寸
 * @returns {Promise<{width: number, height: number}>} 窗口尺寸
 */
export async function getWindowSize() {
  const [width, height] = await invoke('get_window_size');
  return { width, height };
}

/**
 * 设置窗口尺寸
 * @param {number} width - 窗口宽度
 * @param {number} height - 窗口高度
 */
export async function setWindowSize(width, height) {
  await invoke('set_window_size', { width, height });
}

// 默认导出所有方法
export default {
  toggleAlwaysOnTop,
  closeWindow,
  minimizeWindow,
  toggleMaximize,
  getWindowPosition,
  setWindowPosition,
  getWindowSize,
  setWindowSize,
};
