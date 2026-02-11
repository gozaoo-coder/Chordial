<template>
  <div class="player-controls">
    <!-- 播放模式 -->
    <button
      class="control-btn mode-btn"
      @click="onToggleMode"
      :title="playModeText"
    >
      <svg v-if="playMode === 'sequence'" viewBox="0 0 24 24" fill="currentColor">
        <path d="M4 4h16v16H4z" style="display:none"/>
        <path d="M3 6h18M3 12h18M3 18h18"/>
      </svg>
      <svg v-else-if="playMode === 'random'" viewBox="0 0 24 24" fill="currentColor">
        <path d="M16 3h5v5M4 20L21 3M21 16v5h-5M15 15l6 6M4 4l5 5"/>
      </svg>
      <svg v-else-if="playMode === 'loop'" viewBox="0 0 24 24" fill="currentColor">
        <path d="M17 2l4 4-4 4M3 12V9a4 4 0 014-4h13M7 22l-4-4 4-4M21 12v3a4 4 0 01-4 4H4"/>
      </svg>
      <svg v-else-if="playMode === 'loop_one'" viewBox="0 0 24 24" fill="currentColor">
        <path d="M17 2l4 4-4 4M3 12V9a4 4 0 014-4h13M7 22l-4-4 4-4M21 12v3a4 4 0 01-4 4H4"/>
        <text x="12" y="15" text-anchor="middle" font-size="8" fill="currentColor">1</text>
      </svg>
    </button>

    <!-- 上一首 -->
    <button
      class="control-btn previous-btn"
      @click="onPrevious"
      :disabled="!canPlayPrevious"
      title="上一首"
    >
      <svg viewBox="0 0 24 24" fill="currentColor">
        <path d="M6 6h2v12H6zm3.5 6l8.5 6V6z"/>
      </svg>
    </button>

    <!-- 播放/暂停 -->
    <button
      class="control-btn play-btn"
      @click="onTogglePlay"
      :title="isPlaying ? '暂停' : '播放'"
    >
      <svg v-if="!isPlaying" viewBox="0 0 24 24" fill="currentColor">
        <path d="M8 5v14l11-7z"/>
      </svg>
      <svg v-else viewBox="0 0 24 24" fill="currentColor">
        <path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z"/>
      </svg>
    </button>

    <!-- 下一首 -->
    <button
      class="control-btn next-btn"
      @click="onNext"
      :disabled="!canPlayNext"
      title="下一首"
    >
      <svg viewBox="0 0 24 24" fill="currentColor">
        <path d="M6 18l8.5-6L6 6v12zM16 6v12h2V6h-2z"/>
      </svg>
    </button>

    <!-- 歌词按钮 -->
    <button
      class="control-btn lyrics-btn"
      @click="onToggleLyrics"
      :class="{ active: showLyrics }"
      title="歌词"
    >
      <svg viewBox="0 0 24 24" fill="currentColor">
        <path d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
      </svg>
    </button>

    <!-- Automix 按钮 -->
    <button
      class="control-btn automix-btn"
      @click="onToggleAutomix"
      :class="{ active: automixEnabled }"
      title="智能混音"
    >
      <svg viewBox="0 0 24 24" fill="currentColor">
        <path d="M12 3v10.55c-.59-.34-1.27-.55-2-.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z"/>
      </svg>
    </button>
  </div>
</template>

<script>
import { computed } from 'vue';

export default {
  name: 'PlayerControls',
  props: {
    isPlaying: {
      type: Boolean,
      default: false
    },
    playMode: {
      type: String,
      default: 'sequence'
    },
    canPlayPrevious: {
      type: Boolean,
      default: true
    },
    canPlayNext: {
      type: Boolean,
      default: true
    },
    showLyrics: {
      type: Boolean,
      default: false
    },
    automixEnabled: {
      type: Boolean,
      default: false
    }
  },
  emits: ['toggle-play', 'previous', 'next', 'toggle-mode', 'toggle-lyrics', 'toggle-automix'],
  setup(props, { emit }) {
    const playModeText = computed(() => {
      const modeTexts = {
        'sequence': '顺序播放',
        'random': '随机播放',
        'loop': '列表循环',
        'loop_one': '单曲循环'
      };
      return modeTexts[props.playMode] || '顺序播放';
    });

    const onTogglePlay = () => emit('toggle-play');
    const onPrevious = () => emit('previous');
    const onNext = () => emit('next');
    const onToggleMode = () => emit('toggle-mode');
    const onToggleLyrics = () => emit('toggle-lyrics');
    const onToggleAutomix = () => emit('toggle-automix');

    return {
      playModeText,
      onTogglePlay,
      onPrevious,
      onNext,
      onToggleMode,
      onToggleLyrics,
      onToggleAutomix
    };
  }
};
</script>

<style scoped>
.player-controls {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: clamp(12px, 1.5vw, 24px);
  flex-wrap: nowrap;
  flex-shrink: 0;
}

.control-btn {
  width: clamp(40px, 5vmin, 52px);
  height: clamp(40px, 5vmin, 52px);
  border: none;
  background: rgba(255, 255, 255, 0.08);
  border-radius: 50%;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  transition: all 0.2s ease;
  backdrop-filter: blur(10px);
}

.control-btn:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.2);
  transform: scale(1.05);
}

.control-btn:active:not(:disabled) {
  transform: scale(0.95);
}

.control-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.control-btn svg {
  width: clamp(18px, 2.2vmin, 24px);
  height: clamp(18px, 2.2vmin, 24px);
}

/* 播放按钮 - 突出显示 */
.play-btn {
  width: clamp(56px, 7vmin, 72px);
  height: clamp(56px, 7vmin, 72px);
  background: white;
  color: #1a1a1a;
}

.play-btn:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.95);
  transform: scale(1.08);
}

.play-btn svg {
  width: clamp(24px, 3vmin, 32px);
  height: clamp(24px, 3vmin, 32px);
}

/* 小按钮 */
.mode-btn,
.lyrics-btn {
  width: clamp(32px, 4vmin, 40px);
  height: clamp(32px, 4vmin, 40px);
}

.mode-btn svg,
.lyrics-btn svg {
  width: clamp(16px, 2vmin, 20px);
  height: clamp(16px, 2vmin, 20px);
}

.lyrics-btn.active {
  color: #0078d7;
  background: rgba(0, 120, 215, 0.15);
}

/* Automix 按钮 */
.automix-btn.active {
  color: #667eea;
  background: rgba(102, 126, 234, 0.15);
}

@media (max-width: 767px) {
  .player-controls {
    gap: clamp(10px, 3vw, 16px);
  }

  .control-btn {
    width: clamp(40px, 12vw, 52px);
    height: clamp(40px, 12vw, 52px);
  }

  .control-btn svg {
    width: clamp(18px, 5vw, 24px);
    height: clamp(18px, 5vw, 24px);
  }

  .play-btn {
    width: clamp(56px, 16vw, 72px);
    height: clamp(56px, 16vw, 72px);
  }

  .play-btn svg {
    width: clamp(24px, 7vw, 32px);
    height: clamp(24px, 7vw, 32px);
  }

  .mode-btn,
  .lyrics-btn,
  .automix-btn {
    display: none;
  }
}
</style>
