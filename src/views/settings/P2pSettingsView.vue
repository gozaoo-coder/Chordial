<template>
  <div ref="rootRef" class="p2p-view">
    <div class="section-header">
      <h2 class="section-title">P2P 资源共享</h2>
      <p class="section-subtitle">在局域网内与其他 Chordial 实例共享音乐库</p>
    </div>

    <!-- 服务控制 -->
    <div class="settings-section">
      <h3 class="block-title"><i class="bi bi-power"></i>服务</h3>

      <div class="setting-item">
        <div class="setting-info">
          <label class="setting-label">启用共享服务</label>
          <span class="setting-desc">启动 P2P 服务器，允许其他 Chordial 实例连接</span>
        </div>
        <div class="setting-control">
          <label class="toggle">
            <input type="checkbox" v-model="p2pEnabled" @change="onToggleP2p" />
            <span class="toggle-slider"></span>
          </label>
        </div>
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <label class="setting-label">允许广播发现</label>
          <span class="setting-desc">局域网内其他 Chordial 可发现本机；关闭后只能通过 IP:端口+匹配码连接</span>
        </div>
        <div class="setting-control">
          <label class="toggle">
            <input type="checkbox" v-model="p2pBroadcast" @change="onToggleBroadcast" :disabled="!p2pEnabled" />
            <span class="toggle-slider"></span>
          </label>
        </div>
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <label class="setting-label">本机共享权限</label>
          <span class="setting-desc">决定对方拿到本库的权限；默认只读</span>
        </div>
        <div class="setting-control">
          <select v-model="p2pPermission" @change="onPermissionChange" :disabled="!p2pEnabled" class="select">
            <option value="readonly">仅可读</option>
            <option value="editable">可编辑</option>
          </select>
        </div>
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <label class="setting-label">本机监听地址</label>
          <span class="setting-desc">将此地址告知对方以建立连接</span>
        </div>
        <div class="setting-control">
          <code v-if="p2pStatus.listening" class="match-code">{{ p2pStatus.listen_addr }}</code>
          <span v-else class="setting-value">未启动</span>
        </div>
      </div>
    </div>

    <!-- 匹配码 + 二维码 -->
    <div class="settings-section">
      <h3 class="block-title"><i class="bi bi-qr-code"></i>匹配码 / 二维码</h3>

      <div class="setting-item">
        <div class="setting-info">
          <label class="setting-label">当前匹配码</label>
          <span class="setting-desc">
            6 位数字，每 60 秒自动轮换防撞码。
            <span v-if="p2pStatus.listening && rotateCountdown > 0" class="countdown">
              {{ rotateCountdown }}s 后更新
            </span>
          </span>
        </div>
        <div class="setting-control">
          <code v-if="p2pStatus.match_code" class="match-code big">{{ p2pStatus.match_code }}</code>
          <span v-else class="setting-value">未启动</span>
          <button class="action-btn" @click="onRegenerateCode" :disabled="!p2pEnabled">重新生成</button>
        </div>
      </div>

      <div v-if="p2pEnabled && qrDataUrl" class="qr-area">
        <div class="qr-side">
          <img :src="qrDataUrl" alt="匹配二维码" class="qr-img" />
          <p class="qr-tip">让对方扫描此二维码即可建立连接</p>
        </div>
        <div class="qr-side">
          <button class="action-btn primary" @click="showScanner = true">
            <i class="bi bi-camera"></i> 扫描对方二维码
          </button>
          <p class="qr-tip">使用摄像头扫描对方展示的匹配二维码</p>
        </div>
      </div>
    </div>

    <!-- 局域网扫描结果 -->
    <div class="settings-section">
      <h3 class="block-title">
        <i class="bi bi-broadcast"></i>局域网扫描结果
        <button class="action-btn ghost refresh-btn" @click="refreshStatus" :disabled="!p2pEnabled">
          <i class="bi bi-arrow-clockwise"></i>
        </button>
      </h3>

      <div v-if="!p2pEnabled" class="empty-hint">先启用共享服务</div>
      <div v-else-if="!discoveredPeers.length" class="empty-hint">未发现局域网设备，请确认对方已开启广播</div>
      <div v-else class="peer-list">
        <div v-for="peer in discoveredPeers" :key="peerKey(peer)" class="peer-row">
          <div class="peer-info">
            <div class="peer-name">
              {{ peer.name }}
              <span v-if="isTrusted(peer)" class="badge trusted">已信任</span>
              <span v-else-if="isConnected(peer)" class="badge connected">已连接</span>
            </div>
            <div class="peer-meta">{{ peer.addr }}:{{ peer.port }}</div>
          </div>
          <div class="peer-actions">
            <button v-if="isTrusted(peer)" class="action-btn ghost" disabled>自动连接</button>
            <button v-else-if="isConnected(peer)" class="action-btn ghost" disabled>已连接</button>
            <button v-else class="action-btn primary" @click="onConnectDiscovered(peer)">连接</button>
          </div>
        </div>
      </div>
    </div>

    <!-- 可信设备 -->
    <div class="settings-section">
      <h3 class="block-title"><i class="bi bi-shield-check"></i>可信设备</h3>
      <p class="block-desc">匹配同意过的设备会自动重连，无需再次确认</p>

      <div v-if="!trustedDevices.length" class="empty-hint">暂无可信设备</div>
      <div v-else class="peer-list">
        <div v-for="d in trustedDevices" :key="d.instance_id" class="peer-row">
          <div class="peer-info">
            <div class="peer-name">{{ d.name }}</div>
            <div class="peer-meta">
              {{ d.addr }} · {{ d.permission === 'editable' ? '可编辑' : '只读' }}
              · {{ formatTime(d.added_at) }}
            </div>
          </div>
          <div class="peer-actions">
            <button class="action-btn danger" @click="onRemoveTrusted(d.instance_id)">移除</button>
          </div>
        </div>
      </div>
    </div>

    <!-- 已连接 peer -->
    <div class="settings-section">
      <h3 class="block-title"><i class="bi bi-people"></i>已连接</h3>
      <div v-if="!connectedPeers.length" class="empty-hint">暂无已连接的对端</div>
      <div v-else class="peer-list">
        <div v-for="peer in connectedPeers" :key="peer.id" class="peer-row">
          <div class="peer-info">
            <div class="peer-name">{{ peer.name }}</div>
            <div class="peer-meta">{{ peer.addr }} · {{ peer.permission === 'editable' ? '可编辑' : '只读' }}</div>
          </div>
          <div class="peer-actions">
            <button class="action-btn danger" @click="onDisconnect(peer.id)">断开</button>
          </div>
        </div>
      </div>
    </div>

    <!-- 手动连接 -->
    <div class="settings-section">
      <h3 class="block-title"><i class="bi bi-link-45deg"></i>手动匹配</h3>
      <div class="setting-item col">
        <div class="setting-info">
          <label class="setting-label">主动发起匹配</label>
          <span class="setting-desc">输入对方的 IP:端口 与匹配码以发起握手</span>
        </div>
        <div class="setting-control wrap">
          <input v-model="remoteAddr" placeholder="192.168.1.10:58008" class="text-input" :disabled="!p2pEnabled" />
          <input v-model="remoteCode" placeholder="匹配码" maxlength="6" class="text-input code-input" :disabled="!p2pEnabled" />
          <button class="action-btn primary" @click="onRequestMatch" :disabled="!p2pEnabled || !remoteAddr || !remoteCode">连接</button>
        </div>
      </div>
    </div>

    <!-- 二维码扫描对话框 -->
    <div v-if="showScanner" class="scanner-overlay" @click.self="closeScanner">
      <div class="scanner-box">
        <div class="scanner-head">
          <h4>扫描二维码</h4>
          <button class="action-btn ghost" @click="closeScanner"><i class="bi bi-x-lg"></i></button>
        </div>
        <video ref="videoEl" class="scanner-video" autoplay playsinline muted></video>
        <div class="scanner-foot">
          <p v-if="scannerError" class="scanner-error">{{ scannerError }}</p>
          <p v-else class="scanner-hint">将对方二维码对准摄像头</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted, watch, nextTick, useTemplateRef } from 'vue';
import QRCode from 'qrcode';
import jsQR from 'jsqr';
import { p2pApi } from '@/api/p2p.js';
import { useP2pEvents } from '@/composables/useP2pEvents.js';
import { useAnime } from '@/composables/useAnime.js';

const { toasts } = useP2pEvents();

const rootRef = useTemplateRef('root');
const { run } = useAnime(() => rootRef.value);

const p2pEnabled = ref(false);
const p2pBroadcast = ref(false);
const p2pPermission = ref('readonly');
const p2pStatus = ref({
  listening: false,
  listen_addr: '',
  match_code: '',
  permission: 'readonly',
  broadcast_enabled: false,
  peers: [],
  discovered: [],
  trusted_devices: [],
  instance_id: '',
  match_code_rotates_at: 0,
});
const remoteAddr = ref('');
const remoteCode = ref('');

// 二维码生成
const qrDataUrl = ref('');
// 二维码扫描
const showScanner = ref(false);
const videoEl = ref(null);
const scannerError = ref('');
let scanStream = null;
let scanRaf = null;
let scanCanvas = null;

// 轮换倒计时
const nowSec = ref(Math.floor(Date.now() / 1000));
const rotateCountdown = computed(() =>
  p2pStatus.value.match_code_rotates_at
    ? Math.max(0, p2pStatus.value.match_code_rotates_at - nowSec.value)
    : 0,
);

const discoveredPeers = computed(() => p2pStatus.value.discovered || []);
const trustedDevices = computed(() => p2pStatus.value.trusted_devices || []);
const connectedPeers = computed(() => p2pStatus.value.peers || []);

const trustedIds = computed(() => new Set(trustedDevices.value.map((d) => d.instance_id)));
const connectedNames = computed(() => new Set(connectedPeers.value.map((p) => p.name)));

function isTrusted(peer) {
  return !!(peer.instance_id && trustedIds.value.has(peer.instance_id));
}
function isConnected(peer) {
  return connectedNames.value.has(peer.name);
}
function peerKey(peer) {
  return peer.instance_id || `${peer.addr}:${peer.port}`;
}

function formatTime(sec) {
  if (!sec) return '';
  try {
    const d = new Date(sec * 1000);
    return `${d.getMonth() + 1}/${d.getDate()}`;
  } catch {
    return '';
  }
}

// ── 状态刷新 ────────────────────────────────
const refreshStatus = async () => {
  try {
    p2pStatus.value = await p2pApi.status();
    p2pEnabled.value = p2pStatus.value.listening;
    p2pBroadcast.value = p2pStatus.value.broadcast_enabled;
    p2pPermission.value = p2pStatus.value.permission || 'readonly';
  } catch (e) {
    console.error('P2P 状态查询失败:', e);
  }
};

// ── 控制 ────────────────────────────────────
const onToggleP2p = async () => {
  try {
    if (p2pEnabled.value) {
      await p2pApi.startServer({ broadcast: p2pBroadcast.value, permission: p2pPermission.value });
    } else {
      await p2pApi.stopServer();
      p2pBroadcast.value = false;
    }
    await refreshStatus();
  } catch (e) {
    console.error('P2P 切换失败:', e);
    p2pEnabled.value = !p2pEnabled.value;
  }
};

const onToggleBroadcast = async () => {
  try {
    await p2pApi.setBroadcast(p2pBroadcast.value);
    await refreshStatus();
  } catch (e) {
    console.error('切换广播失败:', e);
    p2pBroadcast.value = !p2pBroadcast.value;
  }
};

const onPermissionChange = async () => {
  try {
    await p2pApi.setPermission(p2pPermission.value);
  } catch (e) {
    console.error('设置权限失败:', e);
  }
};

const onRegenerateCode = async () => {
  try {
    await p2pApi.regenerateMatchCode();
    await refreshStatus();
  } catch (e) {
    console.error('重新生成匹配码失败:', e);
  }
};

const onRequestMatch = async () => {
  try {
    await p2pApi.requestMatch(remoteAddr.value, remoteCode.value);
    remoteAddr.value = '';
    remoteCode.value = '';
    await refreshStatus();
  } catch (e) {
    console.error('发起匹配失败:', e);
    alert('匹配失败: ' + e);
  }
};

const onConnectDiscovered = async (peer) => {
  // 信任设备会自动连接，这里仅处理未信任设备
  const addr = `${peer.addr}:${peer.port}`;
  try {
    await p2pApi.requestMatch(addr, peer.match_code);
    await refreshStatus();
  } catch (e) {
    console.error('连接发现设备失败:', e);
    alert('连接失败: ' + e);
  }
};

const onDisconnect = async (peerId) => {
  try {
    await p2pApi.disconnectPeer(peerId);
    await refreshStatus();
  } catch (e) {
    console.error('断开失败:', e);
  }
};

const onRemoveTrusted = async (instanceId) => {
  try {
    await p2pApi.removeTrusted(instanceId);
    await refreshStatus();
  } catch (e) {
    console.error('移除可信设备失败:', e);
  }
};

// ── 二维码生成 ──────────────────────────────
const regenerateQr = async () => {
  if (!p2pEnabled.value) {
    qrDataUrl.value = '';
    return;
  }
  try {
    const payload = await p2pApi.getMatchPayload();
    const text = JSON.stringify({
      v: 1,
      iid: payload.instance_id,
      name: payload.name,
      addr: payload.listen_addr,
      port: payload.port,
      code: payload.match_code,
    });
    qrDataUrl.value = await QRCode.toDataURL(text, { width: 240, margin: 2 });
  } catch (e) {
    console.error('二维码生成失败:', e);
    qrDataUrl.value = '';
  }
};

// 监听 match_code 变化时重新生成二维码
watch(() => p2pStatus.value.match_code, () => {
  regenerateQr();
});
watch(p2pEnabled, () => {
  regenerateQr();
});

// ── 二维码扫描 ──────────────────────────────
const startScanner = async () => {
  scannerError.value = '';
  if (!navigator.mediaDevices || !navigator.mediaDevices.getUserMedia) {
    scannerError.value = '当前环境不支持摄像头';
    return;
  }
  try {
    scanStream = await navigator.mediaDevices.getUserMedia({
      video: { facingMode: 'environment' },
    });
    await nextTick();
    if (videoEl.value) {
      videoEl.value.srcObject = scanStream;
      videoEl.value.play();
      scanLoop();
    }
  } catch (e) {
    scannerError.value = `摄像头启动失败: ${e?.message || e}`;
  }
};

const scanLoop = () => {
  if (!showScanner.value || !videoEl.value) return;
  const video = videoEl.value;
  if (video.readyState === video.HAVE_ENOUGH_DATA) {
    const w = video.videoWidth;
    const h = video.videoHeight;
    if (!scanCanvas) scanCanvas = document.createElement('canvas');
    scanCanvas.width = w;
    scanCanvas.height = h;
    const ctx = scanCanvas.getContext('2d', { willReadFrequently: true });
    ctx.drawImage(video, 0, 0, w, h);
    const imgData = ctx.getImageData(0, 0, w, h);
    const result = jsQR(imgData.data, imgData.width, imgData.height, {
      inversionAttempts: 'attemptOnce',
    });
    if (result && result.data) {
      handleScannedPayload(result.data);
      return;
    }
  }
  scanRaf = requestAnimationFrame(scanLoop);
};

const handleScannedPayload = async (raw) => {
  let payload;
  try {
    payload = JSON.parse(raw);
  } catch {
    // 兼容纯文本 "addr:port|code"
    const parts = raw.split(/[|,]/);
    if (parts.length >= 2) {
      payload = { addr: parts[0], code: parts[1] };
    } else {
      scannerError.value = '无法识别的二维码内容';
      scanRaf = requestAnimationFrame(scanLoop);
      return;
    }
  }
  const addr = payload.addr || (payload.iid ? null : null);
  const code = payload.code || payload.match_code;
  if (!addr || !code) {
    scannerError.value = '二维码缺少地址或匹配码';
    scanRaf = requestAnimationFrame(scanLoop);
    return;
  }
  closeScanner();
  try {
    await p2pApi.requestMatch(addr, code);
    await refreshStatus();
  } catch (e) {
    console.error('二维码匹配失败:', e);
    alert('匹配失败: ' + e);
  }
};

const closeScanner = () => {
  showScanner.value = false;
  if (scanRaf) {
    cancelAnimationFrame(scanRaf);
    scanRaf = null;
  }
  if (scanStream) {
    scanStream.getTracks().forEach((t) => t.stop());
    scanStream = null;
  }
  if (videoEl.value) videoEl.value.srcObject = null;
  scannerError.value = '';
};

watch(showScanner, (v) => {
  if (v) startScanner();
  else closeScanner();
});

// ── 生命周期 ────────────────────────────────
let statusTimer = null;
let nowTimer = null;

onMounted(async () => {
  // 设置区块为静态结构，挂载即触入场动画；异步数据只填充值
  run(({ animate, stagger, presets }) => {
    animate('.section-header', { ...presets.fadeIn });
    animate('.settings-section', { ...presets.fadeInUp, delay: stagger(70) });
  });
  await refreshStatus();
  await regenerateQr();
  statusTimer = setInterval(refreshStatus, 3000);
  nowTimer = setInterval(() => {
    nowSec.value = Math.floor(Date.now() / 1000);
  }, 1000);
});

onUnmounted(() => {
  if (statusTimer) clearInterval(statusTimer);
  if (nowTimer) clearInterval(nowTimer);
  closeScanner();
});
</script>

<style scoped>
.p2p-view {
  max-width: 720px;
  margin: 0 auto;
}

.section-header {
  margin-bottom: 24px;
}

.section-title {
  font-size: 24px;
  font-weight: 800;
  margin: 0 0 6px;
  color: var(--text-primary, #333);
}

.section-subtitle {
  font-size: 13px;
  color: var(--text-secondary, #666);
  margin: 0;
}

.settings-section {
  background: var(--bg-secondary, #fff);
  border-radius: 16px;
  padding: 20px 24px;
  margin-bottom: 16px;
  border: 1px solid var(--border-light, #e8e8e8);
}

.block-title {
  font-size: 15px;
  font-weight: 700;
  margin: 0 0 16px;
  color: var(--text-primary, #333);
  display: flex;
  align-items: center;
  gap: 8px;
}

.block-title i {
  color: var(--primary-color, #0078d7);
}

.block-desc {
  font-size: 12px;
  color: var(--text-secondary, #666);
  margin: -10px 0 14px;
}

.refresh-btn {
  margin-left: auto;
  padding: 4px 8px;
}

.setting-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 0;
  border-bottom: 1px solid var(--border-light, #e8e8e8);
}

.setting-item:last-child {
  border-bottom: none;
}

.setting-item.col {
  flex-direction: column;
  align-items: flex-start;
  gap: 10px;
}

.setting-info {
  flex: 1;
  min-width: 0;
}

.setting-label {
  display: block;
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary, #333);
  margin-bottom: 3px;
}

.setting-desc {
  display: block;
  font-size: 12px;
  color: var(--text-secondary, #666);
}

.countdown {
  color: var(--primary-color, #0078d7);
  font-weight: 600;
}

.setting-control {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-shrink: 0;
}

.setting-control.wrap {
  flex-wrap: wrap;
  width: 100%;
  justify-content: flex-end;
}

.setting-value {
  font-size: 13px;
  color: var(--text-secondary, #666);
}

.match-code {
  font-family: ui-monospace, monospace;
  font-size: 14px;
  background: var(--bg-primary, #f5f5f5);
  border: 1px solid var(--border-color, #e8e8e8);
  padding: 6px 12px;
  border-radius: 6px;
  color: var(--text-primary, #333);
  letter-spacing: 1px;
}

.match-code.big {
  font-size: 20px;
  font-weight: 700;
  letter-spacing: 4px;
  padding: 8px 16px;
}

.toggle {
  position: relative;
  display: inline-block;
  width: 44px;
  height: 22px;
}

.toggle input {
  opacity: 0;
  width: 0;
  height: 0;
}

.toggle-slider {
  position: absolute;
  cursor: pointer;
  inset: 0;
  background-color: var(--border-color, #ccc);
  transition: 0.3s;
  border-radius: 22px;
}

.toggle-slider:before {
  position: absolute;
  content: "";
  height: 16px;
  width: 16px;
  left: 3px;
  bottom: 3px;
  background-color: white;
  transition: 0.3s;
  border-radius: 50%;
}

.toggle input:checked + .toggle-slider {
  background-color: var(--primary-color, #0078d7);
}

.toggle input:checked + .toggle-slider:before {
  transform: translateX(22px);
}

.toggle input:disabled + .toggle-slider {
  opacity: 0.4;
  cursor: not-allowed;
}

.select {
  padding: 7px 28px 7px 10px;
  font-size: 13px;
  border: 1px solid var(--border-color, #e8e8e8);
  border-radius: 8px;
  background: var(--bg-primary, #fff);
  color: var(--text-primary, #333);
  cursor: pointer;
  outline: none;
  min-width: 110px;
  appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 12 12'%3E%3Cpath fill='%23666' d='M6 8L1 3h10z'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 10px center;
}

.select:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.text-input {
  padding: 7px 10px;
  font-size: 13px;
  border: 1px solid var(--border-color, #e8e8e8);
  border-radius: 8px;
  background: var(--bg-primary, #fff);
  color: var(--text-primary, #333);
  outline: none;
  min-width: 180px;
}

.text-input:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.code-input {
  min-width: 90px;
  font-family: ui-monospace, monospace;
  letter-spacing: 2px;
}

.action-btn {
  padding: 7px 14px;
  border: 1px solid transparent;
  border-radius: 8px;
  background: var(--bg-hover, #eaeaea);
  color: var(--text-primary, #333);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s ease;
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.action-btn:hover:not(:disabled) {
  background: var(--bg-active, #dcdcdc);
}

.action-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.action-btn.primary {
  background: var(--primary-color, #0078d7);
  color: #fff;
}

.action-btn.primary:hover:not(:disabled) {
  background: var(--primary-color-dark, #005a9e);
}

.action-btn.ghost {
  background: transparent;
  border-color: var(--border-color, #e8e8e8);
}

.action-btn.danger {
  background: #dc3545;
  color: #fff;
}

.action-btn.danger:hover:not(:disabled) {
  background: #c82333;
}

/* 二维码区 */
.qr-area {
  display: flex;
  gap: 24px;
  margin-top: 16px;
  padding: 16px;
  border-radius: 12px;
  background: var(--bg-primary, #f5f5f5);
  border: 1px dashed var(--border-color, #e8e8e8);
  flex-wrap: wrap;
}

.qr-side {
  flex: 1;
  min-width: 200px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
}

.qr-img {
  width: 200px;
  height: 200px;
  border-radius: 8px;
  background: #fff;
}

.qr-tip {
  font-size: 12px;
  color: var(--text-secondary, #666);
  margin: 0;
  text-align: center;
}

/* 列表 */
.peer-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.peer-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px;
  border-radius: 10px;
  background: var(--bg-primary, #f5f5f5);
  border: 1px solid var(--border-color, #e8e8e8);
}

.peer-info {
  flex: 1;
  min-width: 0;
}

.peer-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary, #333);
  display: flex;
  align-items: center;
  gap: 8px;
}

.peer-meta {
  font-size: 12px;
  color: var(--text-secondary, #666);
  margin-top: 2px;
}

.peer-actions {
  flex-shrink: 0;
}

.badge {
  font-size: 10px;
  padding: 2px 8px;
  border-radius: 10px;
  font-weight: 600;
}

.badge.trusted {
  background: rgba(40, 167, 69, 0.15);
  color: #28a745;
}

.badge.connected {
  background: rgba(0, 120, 215, 0.15);
  color: var(--primary-color, #0078d7);
}

.empty-hint {
  font-size: 13px;
  color: var(--text-tertiary, #999);
  padding: 16px 0;
  text-align: center;
}

/* 扫描对话框 */
.scanner-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.7);
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
}

.scanner-box {
  width: 90%;
  max-width: 480px;
  background: var(--bg-secondary, #fff);
  border-radius: 16px;
  overflow: hidden;
}

.scanner-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border-light, #e8e8e8);
}

.scanner-head h4 {
  margin: 0;
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary, #333);
}

.scanner-video {
  width: 100%;
  display: block;
  background: #000;
  max-height: 60vh;
  object-fit: cover;
}

.scanner-foot {
  padding: 10px 16px;
}

.scanner-hint {
  font-size: 12px;
  color: var(--text-secondary, #666);
  margin: 0;
  text-align: center;
}

.scanner-error {
  font-size: 12px;
  color: #dc3545;
  margin: 0;
  text-align: center;
}

@media (max-width: 767px) {
  .setting-item {
    flex-direction: column;
    align-items: flex-start;
    gap: 10px;
  }

  .setting-control {
    width: 100%;
    justify-content: space-between;
  }

  .qr-area {
    flex-direction: column;
  }
}
</style>
