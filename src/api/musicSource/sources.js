/**
 * 音乐源管理 API — 后端 local_* 命令
 *
 * 当前仅支持本地来源（LocalMusicSource），网盘来源尚未实现。
 */

import { transport } from '@/api/transport';

// ══════════════════════════════════════════════════════════════════════════════
// Local Folder Management
// ══════════════════════════════════════════════════════════════════════════════

/**
 * 添加本地音乐文件夹。
 *
 * 后端操作：
 * 1. 持久化文件夹路径
 * 2. 全量扫描音频文件并导入 MusicLibrary
 * 3. 启动文件系统监听
 *
 * @param {string} path - 文件夹绝对路径
 * @returns {Promise<{added: boolean, path: string, files_found: number, indexed: number, errors: string[]}>}
 */
export async function addLocalFolder(path) {
  return transport.command('local_add_folder', { path });
}

/**
 * 移除本地音乐文件夹。
 *
 * 后端操作：
 * 1. 注销文件夹下所有音频文件的 SourceId（级联清理空实体）
 * 2. 从文件夹管理器移除
 * 3. 持久化
 *
 * @param {string} path - 文件夹绝对路径
 * @returns {Promise<{removed: boolean, path: string, cleaned_files: number}>}
 */
export async function removeLocalFolder(path) {
  return transport.command('local_remove_folder', { path });
}

/** 别名 —— 保持向后兼容 */
export { removeLocalFolder as remove };

/**
 * 获取已添加的本地文件夹列表。
 * @returns {Promise<string[]>} 文件夹路径数组
 */
export async function getFolders() {
  return transport.command('local_get_folders');
}

/** 别名 —— 保持向后兼容 */
export { getFolders as getAll };

/**
 * 获取本地来源索引统计。
 * @returns {Promise<{folder_count: number, indexed_files: number}>}
 */
export async function getLocalStats() {
  return transport.command('local_stats');
}

/**
 * 手动重新扫描所有文件夹（调试用）。
 * @returns {Promise<{indexed: number, folders_scanned: number}>}
 */
export async function rescanAll() {
  return transport.command('local_rescan');
}

// ══════════════════════════════════════════════════════════════════════════════
// Source enable / disable — 暂未实现
// ══════════════════════════════════════════════════════════════════════════════

/**
 * 设置源启用状态（当前 stub，后端尚未实现非本地来源开关）。
 * @deprecated
 */
export async function setEnabled(_id, _enabled) {
  throw new Error('setEnabled 暂未实现 — 本地来源是 must-source，不允许禁用');
}

/**
 * 添加网盘来源（后端尚未实现）。
 * @deprecated 后端未实现网盘源
 */
export async function addWebDisk(_url, _username, _password) {
  throw new Error('网盘来源后端尚未实现');
}

/**
 * 添加 WebDAV 来源（后端尚未实现）。
 * @deprecated 后端未实现 WebDAV 源
 */
export async function addWebDev(_apiBaseUrl, _name, _apiKey, _authToken) {
  throw new Error('WebDAV 来源后端尚未实现');
}
