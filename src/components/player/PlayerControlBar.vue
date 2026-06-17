<template>
  <div class="player-control-bar">
    <!-- 进度条 -->
    <div class="progress-section">
      <span class="time">{{ formatTime(currentTime) }}</span>
      <input
        type="range"
        class="progress-slider"
        :min="0"
        :max="duration"
        :value="currentTime"
        @input="onSeek"
      />
      <span class="time">{{ formatTime(duration) }}</span>
    </div>
    
    <!-- 控制按钮 -->
    <div class="controls-section">
      <button class="control-btn" @click="$emit('toggle-play-mode')">
        <i :class="playModeIcon"></i>
      </button>
      
      <button class="control-btn" @click="$emit('previous')">
        <i class="bi bi-skip-start-fill"></i>
      </button>
      
      <button class="control-btn play-btn" @click="togglePlay">
        <i :class="isPlaying ? 'bi bi-pause-fill' : 'bi bi-play-fill'"></i>
      </button>
      
      <button class="control-btn" @click="$emit('next')">
        <i class="bi bi-skip-end-fill"></i>
      </button>
      
      <!-- 音量控制 -->
      <div class="volume-control">
        <i :class="volumeIcon" @click="$emit('toggle-mute')"></i>
        <input
          type="range"
          class="volume-slider"
          min="0"
          max="1"
          step="0.01"
          :value="volume"
          @input="onVolumeChange"
        />
      </div>
    </div>
  </div>
</template>

<script setup>
/**
 * PlayerControlBar - 播放器控制栏
 * 
 * 提供播放控制、进度条、音量控制等功能
 */
import { computed } from 'vue'

const props = defineProps({
  currentTime: {
    type: Number,
    default: 0
  },
  duration: {
    type: Number,
    default: 0
  },
  isPlaying: {
    type: Boolean,
    default: false
  },
  volume: {
    type: Number,
    default: 0.8
  },
  playMode: {
    type: String,
    default: 'sequence'
  },
  muted: {
    type: Boolean,
    default: false
  }
})

const emit = defineEmits([
  'play',
  'pause',
  'seek',
  'volume-change',
  'toggle-play-mode',
  'previous',
  'next',
  'toggle-mute'
])

// 播放模式图标
const playModeIcon = computed(() => {
  switch (props.playMode) {
    case 'random':
      return 'bi bi-shuffle'
    case 'loop':
      return 'bi bi-repeat'
    case 'loop_one':
      return 'bi bi-repeat-1'
    default:
      return 'bi bi-arrow-right'
  }
})

// 音量图标
const volumeIcon = computed(() => {
  if (props.muted || props.volume === 0) return 'bi bi-volume-mute-fill'
  if (props.volume < 0.3) return 'bi bi-volume-off-fill'
  if (props.volume < 0.7) return 'bi bi-volume-down-fill'
  return 'bi bi-volume-up-fill'
})

// 切换播放/暂停
const togglePlay = () => {
  if (props.isPlaying) {
    emit('pause')
  } else {
    emit('play')
  }
}

// 跳转
const onSeek = (e) => {
  emit('seek', parseFloat(e.target.value))
}

// 音量变化
const onVolumeChange = (e) => {
  emit('volume-change', parseFloat(e.target.value))
}

// 格式化时间
const formatTime = (seconds) => {
  if (!seconds || isNaN(seconds)) return '0:00'
  const mins = Math.floor(seconds / 60)
  const secs = Math.floor(seconds % 60)
  return `${mins}:${secs.toString().padStart(2, '0')}`
}
</script>

<style scoped>
.player-control-bar {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  padding: 1rem;
  color: white;
}

.progress-section {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.time {
  font-size: 0.75rem;
  opacity: 0.8;
  min-width: 2.5rem;
  text-align: center;
}

.progress-slider {
  flex: 1;
  height: 4px;
  -webkit-appearance: none;
  appearance: none;
  background: rgba(255, 255, 255, 0.2);
  border-radius: 2px;
  cursor: pointer;
}

.progress-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: white;
  cursor: pointer;
}

.controls-section {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 1rem;
}

.control-btn {
  background: none;
  border: none;
  color: white;
  font-size: 1.25rem;
  cursor: pointer;
  padding: 0.5rem;
  opacity: 0.8;
  transition: opacity 0.2s;
}

.control-btn:hover {
  opacity: 1;
}

.play-btn {
  font-size: 2rem;
}

.volume-control {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-left: 1rem;
}

.volume-control i {
  font-size: 1rem;
  cursor: pointer;
}

.volume-slider {
  width: 80px;
  height: 4px;
  -webkit-appearance: none;
  appearance: none;
  background: rgba(255, 255, 255, 0.2);
  border-radius: 2px;
  cursor: pointer;
}

.volume-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: white;
  cursor: pointer;
}
</style>
