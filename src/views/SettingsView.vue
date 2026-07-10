<template>
  <div class="settings-view">
    <div class="settings-header">
      <h1 class="settings-title">设置</h1>
      <p class="settings-subtitle">自定义您的音乐播放体验</p>
    </div>

    <div class="settings-content">
      <!-- 播放器设置 -->
      <div class="settings-section">
        <h2 class="section-title">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M9 18V5l12-2v13"/>
            <circle cx="6" cy="18" r="3"/>
            <circle cx="18" cy="16" r="3"/>
          </svg>
          播放器设置
        </h2>

        <div class="setting-item">
          <div class="setting-info">
            <label class="setting-label">默认音量</label>
            <span class="setting-desc">设置播放器启动时的默认音量</span>
          </div>
          <div class="setting-control">
            <input
              type="range"
              min="0"
              max="100"
              v-model="defaultVolume"
              class="slider"
            />
            <span class="setting-value">{{ defaultVolume }}%</span>
          </div>
        </div>

        <div class="setting-item">
          <div class="setting-info">
            <label class="setting-label">自动播放</label>
            <span class="setting-desc">选择歌曲后自动开始播放</span>
          </div>
          <div class="setting-control">
            <label class="toggle">
              <input type="checkbox" v-model="autoPlay" />
              <span class="toggle-slider"></span>
            </label>
          </div>
        </div>

        <div class="setting-item">
          <div class="setting-info">
            <label class="setting-label">默认播放模式</label>
            <span class="setting-desc">选择播放列表的默认播放模式</span>
          </div>
          <div class="setting-control">
            <select v-model="defaultPlayMode" class="select">
              <option value="sequence">顺序播放</option>
              <option value="random">随机播放</option>
              <option value="loop">列表循环</option>
              <option value="loop_one">单曲循环</option>
            </select>
          </div>
        </div>
      </div>

      <!-- P2P 资源共享 -->
      <div class="settings-section">
        <h2 class="section-title">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="3"/>
            <path d="M12 1v6m0 10v6M4.22 4.22l4.24 4.24m7.08 7.08l4.24 4.24M1 12h6m10 0h6M4.22 19.78l4.24-4.24m7.08-7.08l4.24-4.24"/>
          </svg>
          P2P 资源共享
        </h2>

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
            <span class="setting-desc">开启后局域网内其他 Chordial 可发现本机；关闭后只能通过 IP:端口+匹配码连接</span>
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
            <span class="setting-desc">决定对方拿到本库的权限。默认只读</span>
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
            <span class="setting-desc">将此地址与匹配码告知对方以建立连接</span>
          </div>
          <div class="setting-control">
            <code v-if="p2pStatus.listening" class="match-code">{{ p2pStatus.listen_addr }}</code>
            <span v-else class="setting-value">未启动</span>
          </div>
        </div>

        <div class="setting-item">
          <div class="setting-info">
            <label class="setting-label">匹配码</label>
            <span class="setting-desc">6 位数字，对方连接时需提供；可随时重新生成</span>
          </div>
          <div class="setting-control">
            <code v-if="p2pStatus.match_code" class="match-code">{{ p2pStatus.match_code }}</code>
            <span v-else class="setting-value">未启动</span>
            <button class="cache-btn" @click="onRegenerateCode" :disabled="!p2pEnabled">重新生成</button>
          </div>
        </div>

        <div class="setting-item">
          <div class="setting-info">
            <label class="setting-label">主动发起匹配</label>
            <span class="setting-desc">输入对方的 IP:端口 与匹配码以发起握手</span>
          </div>
          <div class="setting-control p2p-connect">
            <input v-model="remoteAddr" placeholder="192.168.1.10:58008" class="text-input" :disabled="!p2pEnabled" />
            <input v-model="remoteCode" placeholder="匹配码" maxlength="6" class="text-input code-input" :disabled="!p2pEnabled" />
            <button class="cache-btn" @click="onRequestMatch" :disabled="!p2pEnabled || !remoteAddr || !remoteCode">连接</button>
          </div>
        </div>

        <div v-if="p2pStatus.peers && p2pStatus.peers.length" class="peer-list">
          <div class="setting-item" v-for="peer in p2pStatus.peers" :key="peer.id">
            <div class="setting-info">
              <label class="setting-label">{{ peer.name }} ({{ peer.addr }})</label>
              <span class="setting-desc">权限：{{ peer.permission === 'editable' ? '可编辑' : '只读' }}</span>
            </div>
            <div class="setting-control">
              <button class="cache-btn danger" @click="onDisconnect(peer.id)">断开</button>
            </div>
          </div>
        </div>
      </div>

      <!-- 关于 -->
      <div class="settings-section">
        <h2 class="section-title">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10"/>
            <line x1="12" y1="16" x2="12" y2="12"/>
            <line x1="12" y1="8" x2="12.01" y2="8"/>
          </svg>
          关于
        </h2>

        <div class="about-content">
          <div class="app-info">
            <h3 class="app-name">Chordial</h3>
            <p class="app-version">版本 0.1.0</p>
            <p class="app-description">
              一个现代化的本地音乐播放器，支持多种音频格式和歌词显示。
            </p>
          </div>

          <div class="credits">
            <p>使用了以下开源项目：</p>
            <ul>
              <li>AMLL (Apple Music Like Lyrics) - 歌词显示组件</li>
              <li>Vue 3 - 前端框架</li>
              <li>Tauri - 桌面应用框架</li>
              <li>Rust - 后端开发语言</li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import { ref, watch, onMounted, onUnmounted } from 'vue';
import { p2pApi } from '@/api/p2p.js';

export default {
  name: 'SettingsView',
  setup() {
    // 播放器设置
    const defaultVolume = ref(80);
    const autoPlay = ref(true);
    const defaultPlayMode = ref('sequence');

    // P2P 设置
    const p2pEnabled = ref(false);
    const p2pBroadcast = ref(false);
    const p2pPermission = ref('readonly');
    const p2pStatus = ref({ listening: false, listen_addr: '', match_code: '', peers: [] });
    const remoteAddr = ref('');
    const remoteCode = ref('');

    const saveSettings = () => {
      const settings = {
        defaultVolume: defaultVolume.value,
        autoPlay: autoPlay.value,
        defaultPlayMode: defaultPlayMode.value,
      };
      localStorage.setItem('chordial_settings', JSON.stringify(settings));
    };

    const loadSettings = () => {
      try {
        const saved = localStorage.getItem('chordial_settings');
        if (saved) {
          const s = JSON.parse(saved);
          defaultVolume.value = s.defaultVolume ?? 80;
          autoPlay.value = s.autoPlay ?? true;
          defaultPlayMode.value = s.defaultPlayMode ?? 'sequence';
        }
      } catch (e) {
        console.error('加载设置失败:', e);
      }
    };

    // ── P2P 控制 ────────────────────────────────
    const refreshStatus = async () => {
      try {
        p2pStatus.value = await p2pApi.status();
      } catch (e) {
        console.error('P2P 状态查询失败:', e);
      }
    };

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

    const onDisconnect = async (peerId) => {
      try {
        await p2pApi.disconnectPeer(peerId);
        await refreshStatus();
      } catch (e) {
        console.error('断开连接失败:', e);
      }
    };

    watch([defaultVolume, autoPlay, defaultPlayMode], saveSettings, { deep: true });

    let statusTimer = null;
    onMounted(async () => {
      loadSettings();
      await refreshStatus();
      p2pEnabled.value = p2pStatus.value.listening;
      statusTimer = setInterval(refreshStatus, 3000);
    });

    onUnmounted(() => {
      if (statusTimer) clearInterval(statusTimer);
    });

    return {
      defaultVolume,
      autoPlay,
      defaultPlayMode,
      p2pEnabled,
      p2pBroadcast,
      p2pPermission,
      p2pStatus,
      remoteAddr,
      remoteCode,
      onToggleP2p,
      onToggleBroadcast,
      onPermissionChange,
      onRegenerateCode,
      onRequestMatch,
      onDisconnect,
    };
  }
};
</script>

<style scoped>
.settings-view {
  max-width: 800px;
  margin: 0 auto;
  padding: 24px;
}

.settings-header {
  margin-bottom: 32px;
}

.settings-title {
  font-size: 32px;
  font-weight: 700;
  color: var(--text-primary, #333);
  margin: 0 0 8px 0;
}

.settings-subtitle {
  font-size: 16px;
  color: var(--text-secondary, #666);
  margin: 0;
}

.settings-content {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.settings-section {
  background: var(--bg-secondary, #fff);
  border-radius: 16px;
  padding: 24px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
}

.section-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary, #333);
  margin: 0 0 20px 0;
  display: flex;
  align-items: center;
  gap: 10px;
}

.section-title svg {
  width: 24px;
  height: 24px;
  color: var(--primary-color, #0078d7);
}

.setting-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 0;
  border-bottom: 1px solid var(--border-color, #e8e8e8);
}

.setting-item:last-child {
  border-bottom: none;
}

.setting-info {
  flex: 1;
  min-width: 0;
}

.setting-label {
  display: block;
  font-size: 15px;
  font-weight: 500;
  color: var(--text-primary, #333);
  margin-bottom: 4px;
}

.setting-desc {
  display: block;
  font-size: 13px;
  color: var(--text-secondary, #666);
}

.setting-control {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-shrink: 0;
}

.setting-value {
  font-size: 14px;
  color: var(--text-secondary, #666);
  min-width: 50px;
  text-align: right;
}

/* 滑块样式 */
.slider {
  width: 120px;
  height: 4px;
  -webkit-appearance: none;
  appearance: none;
  background: var(--border-color, #e8e8e8);
  border-radius: 2px;
  outline: none;
  cursor: pointer;
}

.slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 16px;
  height: 16px;
  background: var(--primary-color, #0078d7);
  border-radius: 50%;
  cursor: pointer;
  transition: transform 0.2s;
}

.slider::-webkit-slider-thumb:hover {
  transform: scale(1.2);
}

.slider::-moz-range-thumb {
  width: 16px;
  height: 16px;
  background: var(--primary-color, #0078d7);
  border-radius: 50%;
  cursor: pointer;
  border: none;
}

/* 切换开关 */
.toggle {
  position: relative;
  display: inline-block;
  width: 48px;
  height: 24px;
}

.toggle input {
  opacity: 0;
  width: 0;
  height: 0;
}

.toggle-slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: var(--border-color, #ccc);
  transition: 0.3s;
  border-radius: 24px;
}

.toggle-slider:before {
  position: absolute;
  content: "";
  height: 18px;
  width: 18px;
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
  transform: translateX(24px);
}

.toggle input:disabled + .toggle-slider {
  opacity: 0.4;
  cursor: not-allowed;
}

/* 下拉选择 */
.select {
  padding: 8px 32px 8px 12px;
  font-size: 14px;
  border: 1px solid var(--border-color, #e8e8e8);
  border-radius: 8px;
  background: var(--bg-primary, #fff);
  color: var(--text-primary, #333);
  cursor: pointer;
  outline: none;
  min-width: 120px;
  appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 12 12'%3E%3Cpath fill='%23666' d='M6 8L1 3h10z'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 12px center;
}

.select:focus {
  border-color: var(--primary-color, #0078d7);
}

.select:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* 文本输入 */
.text-input {
  padding: 8px 12px;
  font-size: 14px;
  border: 1px solid var(--border-color, #e8e8e8);
  border-radius: 8px;
  background: var(--bg-primary, #fff);
  color: var(--text-primary, #333);
  outline: none;
  min-width: 180px;
}

.text-input:focus {
  border-color: var(--primary-color, #0078d7);
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

.p2p-connect {
  flex-wrap: wrap;
  justify-content: flex-end;
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

.peer-list {
  margin-top: 8px;
  border-top: 1px dashed var(--border-color, #e8e8e8);
  padding-top: 8px;
}

/* 关于部分 */
.about-content {
  padding: 8px 0;
}

.app-info {
  margin-bottom: 24px;
}

.app-name {
  font-size: 24px;
  font-weight: 700;
  color: var(--text-primary, #333);
  margin: 0 0 4px 0;
}

.app-version {
  font-size: 14px;
  color: var(--text-secondary, #666);
  margin: 0 0 12px 0;
}

.app-description {
  font-size: 14px;
  color: var(--text-secondary, #666);
  line-height: 1.6;
  margin: 0;
}

.credits {
  font-size: 14px;
  color: var(--text-secondary, #666);
}

.credits p {
  margin: 0 0 8px 0;
}

.credits ul {
  margin: 0;
  padding-left: 20px;
}

.credits li {
  margin-bottom: 4px;
}

/* 缓存按钮 */
.cache-btn {
  padding: 8px 16px;
  border: none;
  border-radius: 8px;
  background: var(--primary-color, #0078d7);
  color: white;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.cache-btn:hover:not(:disabled) {
  background: var(--primary-color-dark, #005a9e);
}

.cache-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.cache-btn.danger {
  background: #dc3545;
}

.cache-btn.danger:hover:not(:disabled) {
  background: #c82333;
}

/* 深色模式 */
@media (prefers-color-scheme: dark) {
  .settings-title,
  .section-title,
  .setting-label {
    color: var(--text-primary, #f0f0f0);
  }

  .settings-subtitle,
  .setting-desc,
  .setting-value,
  .app-version,
  .app-description,
  .credits {
    color: var(--text-secondary, #aaa);
  }

  .settings-section {
    background: var(--bg-secondary, #3a3a3a);
  }

  .setting-item {
    border-color: var(--border-color, #4a4a4a);
  }

  .slider {
    background: var(--border-color, #4a4a4a);
  }

  .toggle-slider {
    background-color: var(--border-color, #4a4a4a);
  }

  .select,
  .text-input {
    background-color: var(--bg-primary, #2a2a2a);
    border-color: var(--border-color, #4a4a4a);
    color: var(--text-primary, #f0f0f0);
  }

  .match-code {
    background: var(--bg-primary, #2a2a2a);
    border-color: var(--border-color, #4a4a4a);
    color: var(--text-primary, #f0f0f0);
  }
}

/* 移动端适配 */
@media (max-width: 767px) {
  .settings-view {
    padding: 16px;
  }

  .settings-title {
    font-size: 24px;
  }

  .settings-section {
    padding: 16px;
  }

  .setting-item {
    flex-direction: column;
    align-items: flex-start;
    gap: 12px;
  }

  .setting-control {
    width: 100%;
    justify-content: space-between;
  }

  .slider {
    flex: 1;
  }
}
</style>
