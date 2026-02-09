<script setup>
import { ref, onMounted } from 'vue';
import {
  toggleAlwaysOnTop,
  closeWindow,
  minimizeWindow,
  toggleMaximize,
} from '@/api/window.js';

const emit = defineEmits(['toggle-always-on-top', 'close', 'minimize', 'maximize']);

const isAlwaysOnTop = ref(false);
const isMaximized = ref(false);

async function handleToggleAlwaysOnTop() {
  isAlwaysOnTop.value = await toggleAlwaysOnTop();
  emit('toggle-always-on-top', isAlwaysOnTop.value);
}

async function handleCloseWindow() {
  await closeWindow();
  emit('close');
}

async function handleMinimizeWindow() {
  await minimizeWindow();
  emit('minimize');
}

async function handleToggleMaximize() {
  isMaximized.value = await toggleMaximize();
  emit('maximize', isMaximized.value);
}

onMounted(async () => {
  // 可以在这里获取初始窗口状态
});
</script>

<template>
  <div class="window-controls">
    <!-- 置顶按钮 -->
    <button
      class="window-btn"
      :class="{ active: isAlwaysOnTop }"
      aria-label="切换窗口置顶"
      title="置顶窗口"
      @click="handleToggleAlwaysOnTop"
    >
      <i :class="isAlwaysOnTop ? 'bi bi-pin-fill' : 'bi bi-pin'"></i>
    </button>

    <!-- 分隔线 -->
    <div class="divider"></div>

    <!-- 最小化按钮 -->
    <button
      class="window-btn"
      aria-label="最小化窗口"
      title="最小化"
      @click="handleMinimizeWindow"
    >
      <i class="bi bi-dash-lg"></i>
    </button>

    <!-- 最大化/还原按钮 -->
    <button
      class="window-btn"
      aria-label="最大化/还原窗口"
      :title="isMaximized ? '还原' : '最大化'"
      @click="handleToggleMaximize"
    >
      <i :class="isMaximized ? 'bi bi-fullscreen-exit' : 'bi bi-fullscreen'"></i>
    </button>

    <!-- 关闭按钮 -->
    <button
      class="window-btn close-btn"
      aria-label="关闭窗口"
      title="关闭"
      @click="handleCloseWindow"
    >
      <i class="bi bi-x-lg"></i>
    </button>
  </div>
</template>

<style scoped>
.window-controls {
  display: flex;
  align-items: center;
  gap: 8px;
  -webkit-app-region: no-drag;
}

.divider {
  width: 1px;
  height: 20px;
  background: var(--border-light);
  margin: 0 4px;
}

.window-btn {
  width: 36px;
  height: 36px;
  border: none;
  border-radius: 50%;
  background-color: rgba(15, 15, 15, 0.05);
  color: var(--text-secondary);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.2s ease;
  -webkit-app-region: no-drag;
}

.window-btn:hover {
  background-color: rgba(15, 15, 15, 0.08);
  color: var(--text-primary);
}

.window-btn:active {
  transform: scale(0.95);
}

.window-btn.active {
  background-color: #0078d7;
  color: #fff;
}

.window-btn.active:hover {
  background-color: #006cbd;
}

.window-btn i {
  font-size: 14px;
}

.close-btn:hover {
  background-color: #e81123 !important;
  color: #fff !important;
}

/* 深色模式适配 */
@media (prefers-color-scheme: dark) {
  .window-btn {
    background-color: rgba(255, 255, 255, 0.1);
    color: var(--text-secondary);
  }

  .window-btn:hover {
    background-color: rgba(255, 255, 255, 0.15);
    color: var(--text-primary);
  }

  .divider {
    background: var(--border-light);
  }
}
</style>
