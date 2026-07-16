<template>
  <transition :css="false" @enter="onEnter" @leave="onLeave">
    <div v-if="pendingRequests.length" class="p2p-overlay">
      <div class="p2p-dialog">
        <div class="p2p-header">
          <div class="p2p-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="3"/>
              <path d="M12 1v6m0 10v6M4.22 4.22l4.24 4.24m7.08 7.08l4.24 4.24M1 12h6m10 0h6M4.22 19.78l4.24-4.24m7.08-7.08l4.24-4.24"/>
            </svg>
          </div>
          <div class="p2p-title">
            <h3>P2P 匹配请求</h3>
            <p>以下 Chordial 实例请求共享您的音乐库</p>
          </div>
        </div>

        <div class="p2p-list">
          <div v-for="req in pendingRequests" :key="req.request_id" class="p2p-card">
            <div class="p2p-card-info">
              <div class="p2p-card-name">{{ req.peer_name }}</div>
              <div class="p2p-card-addr">{{ req.peer_addr }}</div>
            </div>
            <div class="p2p-card-actions">
              <button class="p2p-btn reject" :disabled="busy[req.request_id]" @click="onReject(req.request_id)">拒绝</button>
              <button class="p2p-btn accept" :disabled="busy[req.request_id]" @click="onAccept(req.request_id)">同意</button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </transition>

  <!-- Toast 提示 -->
  <div class="p2p-toasts">
    <TransitionGroup name="p2p-toast">
      <div v-for="t in toasts" :key="t.id" :class="['p2p-toast', `kind-${t.kind}`]" @click="dismissToast(t.id)">
        <div class="p2p-toast-title">{{ t.title }}</div>
        <div v-if="t.body" class="p2p-toast-body">{{ t.body }}</div>
      </div>
    </TransitionGroup>
  </div>
</template>

<script setup>
import { reactive } from 'vue';
import { animate } from 'animejs';
import { ANIME_PRESETS } from '@/utils/animePresets.js';
import { useP2pEvents } from '@/composables/useP2pEvents.js';

const { pendingRequests, toasts, accept, reject, dismissToast } = useP2pEvents();
const busy = reactive({});

// 弹窗入场：遮罩 fadeIn，内容 scaleIn
function onEnter(el, done) {
  const dialog = el.querySelector('.p2p-dialog');
  animate(el, { ...ANIME_PRESETS.fadeIn });
  animate(dialog, { ...ANIME_PRESETS.scaleIn, onComplete: done });
}

// 弹窗退场：遮罩 fadeOut，内容 scaleOut
function onLeave(el, done) {
  const dialog = el.querySelector('.p2p-dialog');
  animate(el, { ...ANIME_PRESETS.fadeOut });
  animate(dialog, { ...ANIME_PRESETS.scaleOut, onComplete: done });
}

const onAccept = async (id) => {
  busy[id] = true;
  try { await accept(id); } finally { delete busy[id]; }
};
const onReject = async (id) => {
  busy[id] = true;
  try { await reject(id); } finally { delete busy[id]; }
};
</script>

<style scoped>
.p2p-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.45);
  backdrop-filter: blur(6px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
}

.p2p-dialog {
  width: min(92vw, 440px);
  max-height: 80vh;
  overflow-y: auto;
  background: var(--bg-secondary, #fff);
  border-radius: 16px;
  padding: 24px;
  box-shadow: 0 12px 48px rgba(0, 0, 0, 0.25);
}

.p2p-header {
  display: flex;
  align-items: center;
  gap: 14px;
  margin-bottom: 20px;
}

.p2p-icon {
  width: 44px;
  height: 44px;
  flex-shrink: 0;
  border-radius: 12px;
  background: var(--primary-color, #0078d7);
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
}

.p2p-icon svg {
  width: 24px;
  height: 24px;
}

.p2p-title h3 {
  margin: 0 0 4px;
  font-size: 17px;
  font-weight: 600;
  color: var(--text-primary, #333);
}

.p2p-title p {
  margin: 0;
  font-size: 13px;
  color: var(--text-secondary, #666);
}

.p2p-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.p2p-card {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px;
  border-radius: 12px;
  background: var(--bg-primary, #f7f7f7);
  border: 1px solid var(--border-color, #e8e8e8);
}

.p2p-card-info {
  min-width: 0;
  flex: 1;
}

.p2p-card-name {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary, #333);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.p2p-card-addr {
  font-size: 12px;
  color: var(--text-secondary, #666);
  font-family: ui-monospace, monospace;
  margin-top: 2px;
}

.p2p-card-actions {
  display: flex;
  gap: 8px;
  flex-shrink: 0;
}

.p2p-btn {
  padding: 7px 14px;
  font-size: 13px;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
}

.p2p-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.p2p-btn.accept {
  background: var(--primary-color, #0078d7);
  color: #fff;
}

.p2p-btn.accept:hover:not(:disabled) {
  background: var(--primary-color-dark, #005a9e);
}

.p2p-btn.reject {
  background: transparent;
  color: var(--text-secondary, #666);
  border: 1px solid var(--border-color, #ccc);
}

.p2p-btn.reject:hover:not(:disabled) {
  background: var(--bg-secondary, #eee);
}

/* Toasts */
.p2p-toasts {
  position: fixed;
  right: 20px;
  bottom: 20px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  z-index: 10000;
  pointer-events: none;
}

.p2p-toast {
  pointer-events: auto;
  min-width: 240px;
  max-width: 360px;
  padding: 12px 16px;
  border-radius: 10px;
  background: var(--bg-secondary, #fff);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
  border-left: 3px solid var(--primary-color, #0078d7);
  cursor: pointer;
}

.p2p-toast.kind-success { border-left-color: #28a745; }
.p2p-toast.kind-warn { border-left-color: #ffc107; }
.p2p-toast.kind-info { border-left-color: #17a2b8; }

.p2p-toast-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary, #333);
}

.p2p-toast-body {
  font-size: 12px;
  color: var(--text-secondary, #666);
  margin-top: 2px;
}

/* Toast Transitions（保留 CSS，属于状态过渡） */
.p2p-toast-enter-active,
.p2p-toast-leave-active {
  transition: all 0.3s ease;
}
.p2p-toast-enter-from {
  opacity: 0;
  transform: translateX(40px);
}
.p2p-toast-leave-to {
  opacity: 0;
  transform: translateX(40px);
}

@media (prefers-color-scheme: dark) {
  .p2p-dialog,
  .p2p-toast {
    background: var(--bg-secondary, #2a2a2a);
  }
  .p2p-card {
    background: var(--bg-primary, #1f1f1f);
    border-color: var(--border-color, #3a3a3a);
  }
  .p2p-title h3,
  .p2p-card-name,
  .p2p-toast-title {
    color: var(--text-primary, #f0f0f0);
  }
}
</style>
