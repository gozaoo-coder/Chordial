<template>
  <div class="player-header">
    <button class="back-btn" @click="onBack" title="返回">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M19 12H5M12 19l-7-7 7-7"/>
      </svg>
    </button>
    <div class="header-title">{{ title }}</div>
    <button class="settings-btn" @click="onSettings" title="设置">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="3"/>
        <path d="M12 1v6m0 6v6m4.22-10.22l4.24-4.24M6.34 6.34L2.1 2.1m17.8 17.8l-4.24-4.24M6.34 17.66l-4.24 4.24M23 12h-6m-6 0H1m20.24-4.24l-4.24 4.24M6.34 6.34l-4.24-4.24"/>
      </svg>
    </button>
  </div>
</template>

<script>
export default {
  name: 'PlayerHeader',
  props: {
    title: {
      type: String,
      default: '正在播放'
    }
  },
  emits: ['back', 'settings'],
  setup(props, { emit }) {
    const onBack = () => emit('back');
    const onSettings = () => emit('settings');

    return {
      onBack,
      onSettings
    };
  }
};
</script>

<style scoped>
.player-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: clamp(12px, 1.5vh, 18px) clamp(16px, 2vw, 28px);
  background: linear-gradient(to bottom, rgba(0,0,0,0.5) 0%, transparent 100%);
}

.back-btn,
.settings-btn {
  width: clamp(36px, 4.5vmin, 44px);
  height: clamp(36px, 4.5vmin, 44px);
  border: none;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 50%;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  transition: all 0.2s;
  backdrop-filter: blur(10px);
}

.back-btn:hover,
.settings-btn:hover {
  background: rgba(255, 255, 255, 0.2);
  transform: scale(1.05);
}

.back-btn svg,
.settings-btn svg {
  width: clamp(18px, 2.2vmin, 22px);
  height: clamp(18px, 2.2vmin, 22px);
}

.header-title {
  font-size: clamp(14px, 1.8vh, 17px);
  font-weight: 500;
  opacity: 0.9;
  color: white;
}

@media (max-width: 767px) {
  .player-header {
    padding: clamp(10px, 1.2vh, 14px) clamp(12px, 3vw, 18px);
  }

  .back-btn,
  .settings-btn {
    width: clamp(32px, 10vw, 40px);
    height: clamp(32px, 10vw, 40px);
  }

  .back-btn svg,
  .settings-btn svg {
    width: clamp(16px, 5vw, 20px);
    height: clamp(16px, 5vw, 20px);
  }
}
</style>
