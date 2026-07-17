<template>
  <Teleport to="body">
    <div ref="root" class="player-view" tabindex="0" @keydown="onKeydown">
      <!-- AMLL React 播放器挂载点 -->
      <div ref="amllContainer" class="amll-container" />
      <!-- 关闭按钮（浮动在 AMLL 之上） -->
      <button class="close-btn" @click="handleClose" title="关闭">
        <i class="bi bi-chevron-down"></i>
      </button>
    </div>
  </Teleport>
</template>

<script setup>
import { ref, onMounted, onBeforeUnmount } from 'vue'
import { createRoot } from 'react-dom/client'
import { createElement } from 'react'
import { AmllPlayer } from '@/amll/AmllPlayer.jsx'
import { useAmllBridge } from '@/amll/useAmllBridge.js'
import { PlayerStore } from '@/stores/player.js'

const root = ref(null)
const amllContainer = ref(null)
let reactRoot = null

// 创建 Jotai store 并建立 Vue ↔ React 双向同步
const store = useAmllBridge()

function handleClose() {
  PlayerStore.closePlayerView()
}

function onKeydown(e) {
  if (e.key === 'Escape') {
    e.preventDefault()
    handleClose()
  } else if (e.key === ' ') {
    e.preventDefault()
    PlayerStore.togglePlay()
  }
}

onMounted(() => {
  // 挂载 React AMLL 播放器
  reactRoot = createRoot(amllContainer.value)
  reactRoot.render(createElement(AmllPlayer, { store }))

  // 标记过渡完成
  PlayerStore.setPlayerViewTransitioning(false)

  // 聚焦以接收键盘事件
  root.value?.focus()
})

onBeforeUnmount(() => {
  reactRoot?.unmount()
  reactRoot = null
})
</script>

<style scoped>
.player-view {
  position: fixed;
  inset: 0;
  z-index: 300;
  display: flex;
  flex-direction: column;
  background: #000;
  outline: none;
  overflow: hidden;
  animation: player-enter 0.4s ease-out;
}

.amll-container {
  flex: 1;
  min-height: 0;
  width: 100%;
  height: 100%;
  position: relative;
  overflow: hidden;
}

.close-btn {
  position: fixed;
  top: max(12px, env(safe-area-inset-top, 0px));
  left: 50%;
  transform: translateX(-50%);
  z-index: 310;
  width: 36px;
  height: 36px;
  border-radius: 50%;
  border: none;
  background: rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(12px);
  color: rgba(255, 255, 255, 0.8);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: background 0.15s, transform 0.15s;
  font-size: 1.1rem;
}

.close-btn:hover {
  background: rgba(255, 255, 255, 0.2);
}

.close-btn:active {
  transform: translateX(-50%) scale(0.92);
}

@keyframes player-enter {
  from {
    opacity: 0;
    transform: translateY(100%);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@media (min-width: 768px) {
  .close-btn {
    left: 24px;
    transform: none;
    top: 24px;
  }

  .close-btn:hover {
    transform: scale(1.05);
  }

  .close-btn:active {
    transform: scale(0.92);
  }
}
</style>
