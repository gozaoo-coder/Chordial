<template>
  <div class="player-progress">
    <!-- 音质标签 - 在进度条上方居中 -->
    <div class="quality-wrapper" v-if="quality">
      <div class="quality-badge">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3"/>
        </svg>
        <span>{{ quality }}</span>
      </div>
    </div>

    <!-- 进度条 -->
    <div
      class="progress-bar"
      @click="onProgressClick"
      @mousedown="onProgressMouseDown"
      ref="progressBarRef"
    >
      <div class="progress-track">
        <div
          class="progress-fill"
          :style="{ width: progressPercent + '%' }"
        ></div>
        <div
          class="progress-handle"
          :style="{ left: progressPercent + '%' }"
          :class="{ dragging: isDragging }"
        ></div>
      </div>
    </div>

    <!-- 时间显示 - 在进度条下方两侧 -->
    <div class="time-row">
      <span class="time current">{{ formattedCurrentTime }}</span>
      <span class="time duration">{{ formattedDuration }}</span>
    </div>
  </div>
</template>

<script>
import { ref, computed } from 'vue';

export default {
  name: 'PlayerProgress',
  props: {
    currentTime: {
      type: Number,
      default: 0
    },
    duration: {
      type: Number,
      default: 0
    },
    progressPercent: {
      type: Number,
      default: 0
    },
    quality: {
      type: String,
      default: ''
    }
  },
  emits: ['seek', 'seek-to-percent'],
  setup(props, { emit }) {
    const progressBarRef = ref(null);
    const isDragging = ref(false);

    // 格式化时间
    const formatTime = (seconds) => {
      if (!seconds || isNaN(seconds)) return '0:00';
      const mins = Math.floor(seconds / 60);
      const secs = Math.floor(seconds % 60);
      return `${mins}:${secs.toString().padStart(2, '0')}`;
    };

    const formattedCurrentTime = computed(() => formatTime(props.currentTime));
    const formattedDuration = computed(() => formatTime(props.duration));

    // 进度条点击
    const onProgressClick = (e) => {
      if (!progressBarRef.value) return;
      const rect = progressBarRef.value.getBoundingClientRect();
      const percent = (e.clientX - rect.left) / rect.width;
      emit('seek-to-percent', percent * 100);
    };

    // 进度条拖拽
    const onProgressMouseDown = (e) => {
      isDragging.value = true;
      document.addEventListener('mousemove', onProgressMouseMove);
      document.addEventListener('mouseup', onProgressMouseUp);
    };

    const onProgressMouseMove = (e) => {
      if (!isDragging.value || !progressBarRef.value) return;
      const rect = progressBarRef.value.getBoundingClientRect();
      const percent = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
      emit('seek-to-percent', percent * 100);
    };

    const onProgressMouseUp = () => {
      isDragging.value = false;
      document.removeEventListener('mousemove', onProgressMouseMove);
      document.removeEventListener('mouseup', onProgressMouseUp);
    };

    return {
      progressBarRef,
      isDragging,
      formattedCurrentTime,
      formattedDuration,
      onProgressClick,
      onProgressMouseDown
    };
  }
};
</script>

<style scoped>
.player-progress {
  display: flex;
  flex-direction: column;
  gap: clamp(4px, 0.8vh, 8px);
  width: 100%;
}

/* 音质标签 - 居中 */
.quality-wrapper {
  display: flex;
  justify-content: center;
  width: 100%;
}

.quality-badge {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: clamp(2px, 0.4vh, 4px) clamp(6px, 0.8vw, 10px);
  background: rgba(255, 255, 255, 0.12);
  border-radius: clamp(4px, 0.6vmin, 6px);
  font-size: clamp(10px, 1.2vh, 12px);
  color: rgba(255, 255, 255, 0.85);
  backdrop-filter: blur(10px);
  font-weight: 500;
}

.quality-badge svg {
  width: clamp(10px, 1.2vh, 12px);
  height: clamp(10px, 1.2vh, 12px);
}

/* 进度条 */
.progress-bar {
  width: 100%;
  height: clamp(16px, 2vh, 20px);
  display: flex;
  align-items: center;
  cursor: pointer;
  position: relative;
}

.progress-track {
  width: 100%;
  height: clamp(4px, 0.6vh, 5px);
  background: rgba(255, 255, 255, 0.2);
  border-radius: clamp(2px, 0.4vmin, 3px);
  position: relative;
  overflow: visible;
}

.progress-fill {
  height: 100%;
  background: white;
  border-radius: clamp(2px, 0.4vmin, 3px);
  transition: width 0.1s linear;
}

.progress-handle {
  position: absolute;
  top: 50%;
  width: clamp(10px, 1.2vh, 12px);
  height: clamp(10px, 1.2vh, 12px);
  background: white;
  border-radius: 50%;
  transform: translate(-50%, -50%);
  opacity: 0;
  transition: opacity 0.2s, transform 0.2s;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
}

.progress-bar:hover .progress-handle,
.progress-handle.dragging {
  opacity: 1;
}

/* 时间显示行 */
.time-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
}

.time {
  font-size: clamp(10px, 1.2vh, 12px);
  color: rgba(255, 255, 255, 0.6);
  font-variant-numeric: tabular-nums;
  min-width: clamp(30px, 3vw, 36px);
}

.time.current {
  text-align: left;
}

.time.duration {
  text-align: right;
}

@media (max-width: 767px) {
  .player-progress {
    gap: clamp(3px, 0.6vh, 4px);
  }

  .time {
    font-size: clamp(9px, 1.1vh, 11px);
    min-width: clamp(28px, 3vw, 32px);
  }

  .quality-badge {
    font-size: clamp(9px, 1vh, 10px);
    padding: clamp(2px, 0.3vh, 3px) clamp(4px, 0.6vw, 6px);
  }

  .quality-badge svg {
    width: clamp(9px, 1vh, 10px);
    height: clamp(9px, 1vh, 10px);
  }
}
</style>
