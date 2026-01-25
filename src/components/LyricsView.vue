<template>
  <div class="lyrics-view" ref="lyricsContainer">
    <!-- 无歌词状态 -->
    <div v-if="!hasLyrics" class="no-lyrics">
      <span class="icon">🎵</span>
      <p>{{ noLyricsMessage }}</p>
    </div>
    
    <!-- 同步歌词显示 -->
    <div v-else-if="hasSyncedLyrics && syncedLyrics.length > 0" class="synced-lyrics">
      <div 
        v-for="(line, index) in syncedLyrics" 
        :key="index"
        :class="['lyric-line', { 
          active: currentLineIndex === index,
          past: currentLineIndex > index
        }]"
        :data-index="index"
        :data-time="line.time"
        @click="seekTo(line.time)"
      >
        <span class="time-tag" v-if="showTimeTags">{{ formatTime(line.time) }}</span>
        <span class="lyric-text">{{ line.text }}</span>
      </div>
    </div>
    
    <!-- 纯文本歌词显示 -->
    <div v-else-if="plainLyrics" class="plain-lyrics">
      <pre>{{ plainLyrics }}</pre>
    </div>
    
    <!-- 加载状态 -->
    <div v-if="isLoading" class="loading">
      <div class="spinner"></div>
      <span>加载歌词中...</span>
    </div>
  </div>
</template>

<script>
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
import { parseSyncedLyrics } from '@/api/musicSource/musicResource.js'

export default {
  name: 'LyricsView',
  props: {
    trackId: {
      type: String,
      default: null
    },
    lyricsData: {
      type: Object,
      default: () => ({
        plainLyrics: '',
        syncedLyrics: '',
        hasSyncedLyrics: false,
        hasPlainLyrics: false
      })
    },
    isPlaying: {
      type: Boolean,
      default: false
    },
    currentTime: {
      type: Number,
      default: 0
    },
    showTimeTags: {
      type: Boolean,
      default: true
    }
  },
  emits: ['seek'],
  setup(props, { emit }) {
    const lyricsContainer = ref(null)
    const isLoading = ref(false)
    const currentLineIndex = ref(-1)
    
    // 计算属性
    const plainLyrics = computed(() => props.lyricsData.plainLyrics || '')
    
    const syncedLyrics = computed(() => {
      if (!props.lyricsData.syncedLyrics) return []
      return parseSyncedLyrics(props.lyricsData.syncedLyrics)
    })
    
    const hasSyncedLyrics = computed(() => {
      return props.lyricsData.hasSyncedLyrics && syncedLyrics.value.length > 0
    })
    
    const hasPlainLyrics = computed(() => {
      return props.lyricsData.hasPlainLyrics && plainLyrics.value.trim().length > 0
    })
    
    const hasLyrics = computed(() => {
      return hasSyncedLyrics.value || hasPlainLyrics.value
    })
    
    const noLyricsMessage = computed(() => {
      if (props.trackId) {
        return '暂无歌词'
      }
      '请选择一首歌曲'
    })
    
    // 方法
    const formatTime = (seconds) => {
      const mins = Math.floor(seconds / 60)
      const secs = Math.floor(seconds % 60)
      return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`
    }
    
    const findCurrentLine = (time) => {
      if (!hasSyncedLyrics.value || syncedLyrics.value.length === 0) {
        return -1
      }
      
      const lines = syncedLyrics.value
      for (let i = lines.length - 1; i >= 0; i--) {
        if (time >= lines[i].time) {
          // 检查是否在当前行和下一行之间
          if (i === lines.length - 1 || time < lines[i + 1].time) {
            return i
          }
        }
      }
      return -1
    }
    
    const scrollToLine = (index) => {
      if (index < 0 || !lyricsContainer.value) return
      
      const lineElement = lyricsContainer.value.querySelector(`[data-index="${index}"]`)
      if (lineElement) {
        lineElement.scrollIntoView({
          behavior: 'smooth',
          block: 'center'
        })
      }
    }
    
    const seekTo = (time) => {
      emit('seek', time)
    }
    
    // 监听播放时间变化
    watch(() => props.currentTime, (newTime) => {
      if (!props.isPlaying) return
      
      const newIndex = findCurrentLine(newTime)
      if (newIndex !== currentLineIndex.value) {
        const oldIndex = currentLineIndex.value
        currentLineIndex.value = newIndex
        
        // 只有当时间往前跳或者跨越了多行时才滚动
        if (newIndex > oldIndex || newIndex === 0) {
          nextTick(() => {
            scrollToLine(newIndex)
          })
        }
      }
    })
    
    // 监听播放状态变化
    watch(() => props.isPlaying, (playing) => {
      if (!playing) {
        // 暂停时高亮当前行但不自动滚动
        currentLineIndex.value = findCurrentLine(props.currentTime)
      }
    })
    
    // 监听歌词数据变化
    watch(() => props.lyricsData, (newData) => {
      // 重置当前行索引
      currentLineIndex.value = findCurrentLine(props.currentTime)
    }, { deep: true })
    
    // 组件挂载时初始化
    onMounted(() => {
      currentLineIndex.value = findCurrentLine(props.currentTime)
    })
    
    return {
      lyricsContainer,
      isLoading,
      currentLineIndex,
      plainLyrics,
      syncedLyrics,
      hasSyncedLyrics,
      hasPlainLyrics,
      hasLyrics,
      noLyricsMessage,
      formatTime,
      seekTo
    }
  }
}
</script>

<style scoped>
.lyrics-view {
  width: 100%;
  height: 100%;
  overflow-y: auto;
  background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
  border-radius: 8px;
  padding: 20px;
  color: #fff;
}

.no-lyrics {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: #666;
  text-align: center;
}

.no-lyrics .icon {
  font-size: 48px;
  margin-bottom: 10px;
}

.synced-lyrics {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 20px 0;
}

.lyric-line {
  padding: 12px 20px;
  margin: 4px 0;
  border-radius: 8px;
  transition: all 0.3s ease;
  cursor: pointer;
  text-align: center;
  max-width: 80%;
  opacity: 0.5;
  transform: scale(0.95);
}

.lyric-line:hover {
  background: rgba(255, 255, 255, 0.1);
  opacity: 0.8;
}

.lyric-line.active {
  opacity: 1;
  transform: scale(1.05);
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  box-shadow: 0 4px 15px rgba(102, 126, 234, 0.4);
}

.lyric-line.past {
  opacity: 0.3;
}

.lyric-line .time-tag {
  display: block;
  font-size: 12px;
  color: rgba(255, 255, 255, 0.7);
  margin-bottom: 4px;
}

.lyric-line.active .time-tag {
  color: rgba(255, 255, 255, 0.9);
}

.lyric-line .lyric-text {
  font-size: 16px;
  line-height: 1.4;
}

.lyric-line.active .lyric-text {
  font-size: 18px;
  font-weight: bold;
}

.plain-lyrics {
  padding: 20px;
  white-space: pre-wrap;
  line-height: 1.8;
  color: rgba(255, 255, 255, 0.8);
  text-align: center;
}

.loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: #666;
}

.spinner {
  width: 40px;
  height: 40px;
  border: 3px solid rgba(255, 255, 255, 0.1);
  border-top-color: #667eea;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin-bottom: 10px;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

/* 滚动条样式 */
.lyrics-view::-webkit-scrollbar {
  width: 6px;
}

.lyrics-view::-webkit-scrollbar-track {
  background: rgba(255, 255, 255, 0.1);
  border-radius: 3px;
}

.lyrics-view::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.2);
  border-radius: 3px;
}

.lyrics-view::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.3);
}
</style>