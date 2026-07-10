/**
 * P2P 资源共享 API — 后端 p2p_* 命令封装。
 *
 * 对应 chordial-core 的 P2pManager，负责两台 Chordial 实例之间的
 * music_library 互共享。所有方法透传 Tauri 命令错误。
 *
 * @example
 * import { p2pApi } from '@/api/p2p.js';
 * await p2pApi.startServer({ broadcast: true, permission: 'readonly' });
 * const status = await p2pApi.status();
 */
import { transport } from '@/api/transport';

/**
 * 查询当前 P2P 服务状态。
 * @returns {Promise<{
 *   listening: boolean,
 *   listen_addr: string,
 *   match_code: string,
 *   broadcast: boolean,
 *   permission: 'readonly'|'editable',
 *   peers: Array<{id: string, name: string, addr: string, permission: 'readonly'|'editable'}>
 * }>}
 */
export async function status() {
  return transport.command('p2p_status');
}

/**
 * 启动 P2P 服务器。
 * @param {Object} opts
 * @param {boolean} opts.broadcast - 是否开启局域网广播发现
 * @param {'readonly'|'editable'} opts.permission - 本机共享权限
 */
export async function startServer({ broadcast, permission }) {
  return transport.command('p2p_start_server', { broadcast, permission });
}

/**
 * 停止 P2P 服务器，断开所有已连接的 peer。
 */
export async function stopServer() {
  return transport.command('p2p_stop_server');
}

/**
 * 主动向远端发起握手。
 * @param {string} addr - 形如 "192.168.1.10:58008"
 * @param {string} matchCode - 6 位匹配码
 */
export async function requestMatch(addr, matchCode) {
  return transport.command('p2p_request_match', { addr, matchCode });
}

/**
 * 响应来自远端的匹配请求（由前端匹配对话框触发）。
 * @param {string} requestId - P2pEvent.MatchRequested 携带的 request_id
 * @param {boolean} accepted - 是否同意
 */
export async function respondMatch(requestId, accepted) {
  return transport.command('p2p_respond_match', { requestId, accepted });
}

/**
 * 主动断开某个 peer。
 * @param {string} peerId
 */
export async function disconnectPeer(peerId) {
  return transport.command('p2p_disconnect_peer', { peerId });
}

/**
 * 修改本机共享权限（影响新连接；已连接 peer 维持握手时权限）。
 * @param {'readonly'|'editable'} permission
 */
export async function setPermission(permission) {
  return transport.command('p2p_set_permission', { permission });
}

/**
 * 开关局域网广播发现。
 * @param {boolean} enabled
 */
export async function setBroadcast(enabled) {
  return transport.command('p2p_set_broadcast', { enabled });
}

/**
 * 重新生成 6 位匹配码。
 */
export async function regenerateMatchCode() {
  return transport.command('p2p_regenerate_match_code');
}

// ── 可信设备管理 ─────────────────────────────────────

/**
 * 列出所有可信设备（用户曾同意过的 peer，会自动重连）。
 * @returns {Promise<Array<{instance_id: string, name: string, addr: string, permission: string, added_at: number}>>}
 */
export async function listTrusted() {
  return transport.command('p2p_list_trusted');
}

/**
 * 手动添加可信设备。
 * @param {{instance_id: string, name: string, addr: string, permission: 'readonly'|'editable', added_at?: number}} device
 */
export async function addTrusted(device) {
  return transport.command('p2p_add_trusted', { device });
}

/**
 * 移除可信设备。
 * @param {string} instanceId
 */
export async function removeTrusted(instanceId) {
  return transport.command('p2p_remove_trusted', { instanceId });
}

/**
 * 获取匹配载荷（用于生成二维码）。
 * @returns {Promise<{instance_id: string, name: string, port: number, match_code: string, listen_addr: string}>}
 */
export async function getMatchPayload() {
  return transport.command('p2p_get_match_payload');
}

export const p2pApi = {
  status,
  startServer,
  stopServer,
  requestMatch,
  respondMatch,
  disconnectPeer,
  setPermission,
  setBroadcast,
  regenerateMatchCode,
  listTrusted,
  addTrusted,
  removeTrusted,
  getMatchPayload,
};

export default p2pApi;
