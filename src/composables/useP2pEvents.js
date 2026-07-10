/**
 * P2P 事件桥接 composable。
 *
 * 监听 Tauri 'p2p-event' 事件（由 chordial-tauri lib.rs 从 core mpsc 转发），
 * 维护：
 * - pendingRequests：待用户确认的入站匹配请求队列（驱动 P2pMatchDialog）
 * - toasts：peer 连接/断开/出站匹配结果的可消失提示
 *
 * 收到 MatchRequested 时同时触发 OS 通知（tauri-plugin-notification）。
 *
 * @example
 * import { useP2pEvents } from '@/composables/useP2pEvents';
 * const { pendingRequests, toasts, accept, reject, dismissToast } = useP2pEvents();
 */
import { ref, onUnmounted } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { p2pApi } from '@/api/p2p.js';
import { platformIsTauri } from '@/composables/usePlatform.js';

// 模块级单例 — 多组件共享同一份状态
const pendingRequests = ref([]);
const toasts = ref([]);
let unlistenFn = null;
let started = false;
let startPromise = null;

async function sendOsNotification(title, body) {
  if (!platformIsTauri()) return;
  try {
    const { isPermissionGranted, requestPermission, sendNotification } =
      await import('@tauri-apps/plugin-notification');
    let granted = await isPermissionGranted();
    if (!granted) {
      const perm = await requestPermission();
      granted = perm === 'granted';
    }
    if (granted) {
      await sendNotification({ title, body });
    }
  } catch (e) {
    console.warn('[p2p] OS 通知发送失败:', e);
  }
}

function pushToast(kind, title, body) {
  const id = `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
  toasts.value.push({ id, kind, title, body });
  setTimeout(() => dismissToast(id), 5000);
}

function dismissToast(id) {
  const idx = toasts.value.findIndex((t) => t.id === id);
  if (idx >= 0) toasts.value.splice(idx, 1);
}

function handleEvent(payload) {
  const evt = payload?.event ?? payload;
  if (!evt || typeof evt !== 'object') return;
  switch (evt.kind) {
    case 'match_requested': {
      pendingRequests.value.push({
        request_id: evt.request_id,
        peer_addr: evt.peer_addr,
        peer_name: evt.peer_name,
      });
      sendOsNotification('Chordial 匹配请求', `${evt.peer_name} (${evt.peer_addr}) 请求共享资源`);
      break;
    }
    case 'peer_connected': {
      pushToast('success', 'P2P 已连接', `${evt.peer_name} 已连接，权限：${evt.permission === 'editable' ? '可编辑' : '只读'}`);
      break;
    }
    case 'peer_disconnected': {
      pushToast('info', 'P2P 已断开', `${evt.peer_id} 断开：${evt.reason || '未知'}`);
      break;
    }
    case 'match_result': {
      pushToast(
        evt.accepted ? 'success' : 'warn',
        evt.accepted ? '匹配成功' : '匹配被拒',
        evt.reason || (evt.accepted ? '对方已同意共享' : '对方拒绝了请求'),
      );
      break;
    }
    case 'match_code_rotated': {
      // 匹配码自动轮换；提示用户更新展示
      pushToast('info', '匹配码已更新', `新匹配码：${evt.new_code}`);
      break;
    }
    case 'trusted_auto_connected': {
      pushToast('success', '可信设备已连接', `${evt.peer_name} (${evt.addr}) 已自动连接`);
      break;
    }
    default:
      console.warn('[p2p] 未知事件:', evt);
  }
}

async function start() {
  if (started) return startPromise;
  started = true;
  startPromise = (async () => {
    if (!platformIsTauri()) return;
    try {
      unlistenFn = await listen('p2p-event', (e) => handleEvent(e.payload));
    } catch (e) {
      console.error('[p2p] 监听 p2p-event 失败:', e);
    }
  })();
  return startPromise;
}

async function accept(requestId) {
  try {
    await p2pApi.respondMatch(requestId, true);
  } finally {
    const idx = pendingRequests.value.findIndex((r) => r.request_id === requestId);
    if (idx >= 0) pendingRequests.value.splice(idx, 1);
  }
}

async function reject(requestId) {
  try {
    await p2pApi.respondMatch(requestId, false);
  } finally {
    const idx = pendingRequests.value.findIndex((r) => r.request_id === requestId);
    if (idx >= 0) pendingRequests.value.splice(idx, 1);
  }
}

export function useP2pEvents() {
  if (!started) start();
  return {
    pendingRequests,
    toasts,
    accept,
    reject,
    dismissToast,
  };
}

export function shutdownP2pEvents() {
  if (unlistenFn) {
    unlistenFn();
    unlistenFn = null;
  }
  started = false;
  startPromise = null;
}
