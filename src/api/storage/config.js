/**
 * 配置存储 API — 自动防抖落盘
 *
 * 适合音量、主题、窗口位置等高频变更的应用设置。
 * 每次修改后 500ms 内无新修改即自动写入 config.json。
 *
 * @example
 * import { configGet, configSet } from '@/api/storage/config';
 * await configSet('theme', 'dark');
 * const theme = await configGet('theme');
 */

import { transport } from '@/api/transport';

/**
 * 读取配置项
 * @param {string} key - 配置键
 * @returns {Promise<any>} 配置值
 */
export async function configGet(key) {
  return transport.command('config_get', { key });
}

/**
 * 写入配置项（自动防抖落盘）
 * @param {string} key - 配置键
 * @param {any} value - 配置值
 * @returns {Promise<void>}
 */
export async function configSet(key, value) {
  return transport.command('config_set', { key, value });
}

/**
 * 删除配置项
 * @param {string} key - 配置键
 * @returns {Promise<boolean>} 是否成功删除
 */
export async function configRemove(key) {
  return transport.command('config_remove', { key });
}

/**
 * 检查配置项是否存在
 * @param {string} key - 配置键
 * @returns {Promise<boolean>}
 */
export async function configHas(key) {
  return transport.command('config_has', { key });
}

/**
 * 获取所有配置键
 * @returns {Promise<string[]>}
 */
export async function configKeys() {
  return transport.command('config_keys');
}

/**
 * 清空所有配置
 * @returns {Promise<void>}
 */
export async function configClear() {
  return transport.command('config_clear');
}

/**
 * 立即同步落盘（跳过防抖定时器）
 * 适合在应用退出前调用
 * @returns {Promise<void>}
 */
export async function configFlush() {
  return transport.command('config_flush');
}

/**
 * 从磁盘重新加载配置，丢弃未落盘的修改
 * @returns {Promise<void>}
 */
export async function configReload() {
  return transport.command('config_reload');
}
