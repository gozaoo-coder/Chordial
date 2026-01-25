/**
 * 音乐源管理 API
 */

import { invoke } from '@tauri-apps/api/core';

/**
 * 添加本地音乐源
 * @param {string} path - 文件夹路径
 * @param {boolean} recursive - 是否递归扫描子文件夹
 * @returns {Promise<Object>} 创建的源配置
 */
export async function addLocalFolder(path, recursive = true) {
  return invoke('add_local_source', { path, recursive });
}

/**
 * 添加网盘源
 * @param {string} url - 网盘 URL (webdev://...)
 * @param {string} [username] - 用户名（可选）
 * @param {string} [password] - 密码（可选）
 * @returns {Promise<Object>} 创建的源配置
 */
export async function addWebDisk(url, username = null, password = null) {
  return invoke('add_web_disk_source', { url, username, password });
}

/**
 * 移除音乐源
 * @param {string} id - 源 ID
 * @returns {Promise<boolean>} 是否成功
 */
export async function remove(id) {
  return invoke('remove_source', { id });
}

/**
 * 获取所有音乐源
 * @returns {Promise<Array>} 所有源配置列表
 */
export async function getAll() {
  return invoke('get_all_sources');
}

/**
 * 设置源启用状态
 * @param {string} id - 源 ID
 * @param {boolean} enabled - 是否启用
 * @returns {Promise<boolean>} 操作结果
 */
export async function setEnabled(id, enabled) {
  return invoke('set_source_enabled', { id, enabled });
}

export default {
  addLocalFolder,
  addWebDisk,
  remove,
  getAll,
  setEnabled,
};
