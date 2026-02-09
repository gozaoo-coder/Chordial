<template>
  <div class="player-volume">
    <!-- 音量按钮 -->
    <button
      class="volume-btn"
      @click="onToggleMute"
      :title="muted ? '取消静音' : '静音'"
    >
      <svg v-if="muted || volume === 0" viewBox="0 0 24 24" fill="currentColor">
        <path d="M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z"/>
        <path d="M17 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2"/>
      </svg>
      <svg v-else-if="volume < 0.5" viewBox="0 0 24 24" fill="currentColor">
        <path d="M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z"/>
        <path d="M15.536 8.464a5 5 0 010 7.072"/>
      </svg>
      <svg v-else viewBox="0 0 24 24" fill="currentColor">
        <path d="M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z"/>
        <path d="M15.536 8.464a5 5 0 010 7.072m2.828-9.9a9 9 0 010 12.728"/>
      </svg>
    </button>

    <!-- 音量滑块 -->
    <div
      class="volume-slider"
      @click="onVolumeClick"
      @mousedown="onVolumeMouseDown"
      ref="volumeSliderRef"
    >
      <div class="volume-track">
        <div
          class="volume-fill"
          :style="{ width: (muted ? 0 : volume * 100) + '%' }"
        ></div>
        <div
          class="volume-handle"
          :style="{ left: (muted ? 0 : volume * 100) + '%' }"
        ></div>
      </div>
    </div>
  </div>
</template>

<script>
import { ref } from 'vue';

export default {
  name: 'PlayerVolume',
  props: {
    volume: {
      type: Number,
      default: 1
    },
    muted: {
      type: Boolean,
      default: false
    }
  },
  emits: ['set-volume', 'toggle-mute'],
  setup(props, { emit }) {
    const volumeSliderRef = ref(null);
    const isDragging = ref(false);

    const onToggleMute = () => emit('toggle-mute');

    const onVolumeClick = (e) => {
      if (!volumeSliderRef.value) return;
      const rect = volumeSliderRef.value.getBoundingClientRect();
      const percent = (e.clientX - rect.left) / rect.width;
      emit('set-volume', percent);
    };

    const onVolumeMouseDown = (e) => {
      isDragging.value = true;
      document.addEventListener('mousemove', onVolumeMouseMove);
      document.addEventListener('mouseup', onVolumeMouseUp);
    };

    const onVolumeMouseMove = (e) => {
      if (!isDragging.value || !volumeSliderRef.value) return;
      const rect = volumeSliderRef.value.getBoundingClientRect();
      const percent = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
      emit('set-volume', percent);
    };

    const onVolumeMouseUp = () => {
      isDragging.value = false;
      document.removeEventListener('mousemove', onVolumeMouseMove);
      document.removeEventListener('mouseup', onVolumeMouseUp);
    };

    return {
      volumeSliderRef,
      onToggleMute,
      onVolumeClick,
      onVolumeMouseDown
    };
  }
};
</script>

<style scoped>
.player-volume {
  display: flex;
  align-items: center;
  gap: clamp(6px, 0.8vw, 10px);
  width: 100%;
}

.volume-btn {
  width: clamp(32px, 4vmin, 40px);
  height: clamp(32px, 4vmin, 40px);
  border: none;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 50%;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  transition: all 0.2s ease;
  backdrop-filter: blur(10px);
  flex-shrink: 0;
}

.volume-btn:hover {
  background: rgba(255, 255, 255, 0.2);
}

.volume-btn svg {
  width: clamp(16px, 2vmin, 20px);
  height: clamp(16px, 2vmin, 20px);
}

.volume-slider {
  flex: 1;
  height: clamp(16px, 2vh, 20px);
  display: flex;
  align-items: center;
  cursor: pointer;
}

.volume-track {
  width: 100%;
  height: clamp(3px, 0.5vh, 4px);
  background: rgba(255, 255, 255, 0.2);
  border-radius: clamp(1px, 0.3vmin, 2px);
  position: relative;
  overflow: visible;
}

.volume-fill {
  height: 100%;
  background: white;
  border-radius: 2px;
  transition: width 0.1s;
}

.volume-handle {
  position: absolute;
  top: 50%;
  width: clamp(8px, 1vh, 10px);
  height: clamp(8px, 1vh, 10px);
  background: white;
  border-radius: 50%;
  transform: translate(-50%, -50%);
  opacity: 0;
  transition: opacity 0.2s;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
}

.volume-slider:hover .volume-handle {
  opacity: 1;
}

@media (max-width: 767px) {
  .player-volume {
    display: none;
  }
}
</style>
