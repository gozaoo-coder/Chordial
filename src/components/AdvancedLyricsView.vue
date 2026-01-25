<template>
  <div class="advanced-lyrics-view" ref="lyricsContainer">
    <div class="lyrics-content" :style="contentStyle">
      <div 
        v-for="(line, index) in lyricsLines" 
        :key="index"
        class="lyric-line"
        :class="{
          'current': index === currentLineIndex,
          'past': index < currentLineIndex,
          'future': index > currentLineIndex,
          'bg-line': line.is_bg,
          'duet-line': line.is_duet
        }"
        :style="getLineStyle(line, index)"
        @click="seekToLine(line)"
      >
        <!-- 主歌词 -->
        <div class="lyric-text" :style="textStyle">
          <span 
            v-for="(word, wordIndex) in line.words" 
            :key="wordIndex"
            class="lyric-word"
            :class="{
              'highlighted': isWordHighlighted(word, index),
              'played': isWordPlayed(word, index)
            }"
            :style="getWordStyle(word, index, wordIndex)"
          >
            {{ word.word }}
          </span>
        </div>
        
        <!-- 翻译歌词 -->
        <div v-if="line.translated_lyric" class="translated-lyric" :style="translatedStyle">
          {{ line.translated_lyric }}
        </div>
        
        <!-- 音译歌词 -->
        <div v-if="line.roman_lyric" class="roman-lyric" :style="romanStyle">
          {{ line.roman_lyric }}
        </div>
      </div>
    </div>
    
    <!-- 间奏指示器 -->
    <div v-if="isInterlude" class="interlude-indicator">
      <div class="interlude-dots">
        <div v-for="i in 3" :key="i" class="dot" :class="{ active: interludeDotIndex === i - 1 }"></div>
      </div>
    </div>
  </div>
</template>

<script>
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'

export default {
  name: 'AdvancedLyricsView',
  props: {
    lyricsData: {
      type: Object,
      default: () => ({
        lines: [],
        metadata: null
      })
    },
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
    theme: {
      type: String,
      default: 'default',
      validator: (value) => ['default', 'dark', 'light'].includes(value)
    },
    fontSize: {
      type: Number,
      default: 18
    },
    lineHeight: {
      type: Number,
      default: 1.6
    },
    showTranslation: {
      type: Boolean,
      default: true
    },
    showRoman: {
      type: Boolean,
      default: true
    },
    enableAnimation: {
      type: Boolean,
      default: true
    }
  },
  emits: ['seek', 'line-change'],
  setup(props, { emit }) {
    const lyricsContainer = ref(null)
    const currentLineIndex = ref(-1)
    const interludeDotIndex = ref(0)
    let interludeInterval = null

    // 计算属性
    const lyricsLines = computed(() => {
      return props.lyricsData.lines || []
    })

    const contentStyle = computed(() => ({
      fontSize: `${props.fontSize}px`,
      lineHeight: props.lineHeight,
      padding: '20px 0',
      minHeight: '100%',
      display: 'flex',
      flexDirection: 'column',
      justifyContent: 'center'
    }))

    const textStyle = computed(() => ({
      color: getThemeColor('primary'),
      transition: props.enableAnimation ? 'all 0.3s ease' : 'none'
    }))

    const translatedStyle = computed(() => ({
      color: getThemeColor('secondary'),
      fontSize: `${props.fontSize * 0.8}px`,
      marginTop: '4px',
      opacity: props.showTranslation ? 1 : 0
    }))

    const romanStyle = computed(() => ({
      color: getThemeColor('tertiary'),
      fontSize: `${props.fontSize * 0.7}px`,
      marginTop: '2px',
      opacity: props.showRoman ? 1 : 0
    }))

    const isInterlude = computed(() => {
      if (currentLineIndex.value < 0 || currentLineIndex.value >= lyricsLines.value.length) {
        return false
      }
      const currentLine = lyricsLines.value[currentLineIndex.value]
      const nextLine = lyricsLines.value[currentLineIndex.value + 1]
      
      if (!nextLine) return false
      
      const gap = nextLine.start_time - currentLine.end_time
      return gap > 3000 // 超过3秒认为是间奏
    })

    // 主题颜色获取
    function getThemeColor(type) {
      const themes = {
        default: {
          primary: '#ffffff',
          secondary: '#cccccc',
          tertiary: '#999999',
          background: 'rgba(0, 0, 0, 0.8)',
          highlight: '#ff6b6b'
        },
        dark: {
          primary: '#ffffff',
          secondary: '#cccccc',
          tertiary: '#999999',
          background: 'rgba(0, 0, 0, 0.9)',
          highlight: '#ff6b6b'
        },
        light: {
          primary: '#333333',
          secondary: '#666666',
          tertiary: '#999999',
          background: 'rgba(255, 255, 255, 0.9)',
          highlight: '#ff6b6b'
        }
      }
      return themes[props.theme][type] || themes.default[type]
    }

    // 样式计算函数
    function getLineStyle(line, index) {
      const isCurrent = index === currentLineIndex.value
      const isPast = index < currentLineIndex.value
      
      let opacity = 0.4
      let transform = 'scale(1)'
      let fontWeight = 'normal'
      
      if (isCurrent) {
        opacity = 1
        transform = 'scale(1.05)'
        fontWeight = 'bold'
      } else if (isPast) {
        opacity = 0.2
        transform = 'scale(0.95)'
      }
      
      return {
        opacity,
        transform,
        fontWeight,
        margin: '12px 0',
        padding: '8px 16px',
        borderRadius: '8px',
        backgroundColor: isCurrent ? 'rgba(255, 255, 255, 0.1)' : 'transparent',
        transition: props.enableAnimation ? 'all 0.3s ease' : 'none',
        cursor: 'pointer',
        userSelect: 'none'
      }
    }

    function getWordStyle(word, lineIndex, wordIndex) {
      if (lineIndex !== currentLineIndex.value) {
        return {}
      }
      
      const progress = getWordProgress(word)
      const isHighlighted = isWordHighlighted(word, lineIndex)
      
      return {
        color: isHighlighted ? getThemeColor('highlight') : getThemeColor('primary'),
        textShadow: isHighlighted ? '0 0 10px rgba(255, 107, 107, 0.5)' : 'none',
        transition: props.enableAnimation ? 'all 0.2s ease' : 'none'
      }
    }

    // 歌词逻辑函数
    function updateCurrentLine() {
      const timeMs = props.currentTime * 1000 // 转换为毫秒
      let newIndex = -1
      
      for (let i = 0; i < lyricsLines.value.length; i++) {
        const line = lyricsLines.value[i]
        if (timeMs >= line.start_time && timeMs <= line.end_time) {
          newIndex = i
          break
        }
      }
      
      if (newIndex !== currentLineIndex.value) {
        const oldIndex = currentLineIndex.value
        currentLineIndex.value = newIndex
        emit('line-change', { oldIndex, newIndex, line: lyricsLines.value[newIndex] })
        
        // 滚动到当前行
        nextTick(() => {
          scrollToCurrentLine()
        })
      }
    }

    function scrollToCurrentLine() {
      if (!lyricsContainer.value || currentLineIndex.value < 0) return
      
      const currentLineElement = lyricsContainer.value.children[0]?.children[currentLineIndex.value]
      if (currentLineElement) {
        currentLineElement.scrollIntoView({
          behavior: props.enableAnimation ? 'smooth' : 'auto',
          block: 'center'
        })
      }
    }

    function seekToLine(line) {
      if (line && line.start_time !== undefined) {
        const timeInSeconds = line.start_time / 1000
        emit('seek', timeInSeconds)
      }
    }

    function isWordHighlighted(word, lineIndex) {
      if (lineIndex !== currentLineIndex.value) return false
      
      const currentTimeMs = props.currentTime * 1000
      return currentTimeMs >= word.start_time && currentTimeMs <= word.end_time
    }

    function isWordPlayed(word, lineIndex) {
      if (lineIndex !== currentLineIndex.value) return false
      
      const currentTimeMs = props.currentTime * 1000
      return currentTimeMs > word.end_time
    }

    function getWordProgress(word) {
      const currentTimeMs = props.currentTime * 1000
      const wordDuration = word.end_time - word.start_time
      const elapsed = currentTimeMs - word.start_time
      
      return Math.max(0, Math.min(1, elapsed / wordDuration))
    }

    // 间奏动画
    function startInterludeAnimation() {
      if (interludeInterval) {
        clearInterval(interludeInterval)
      }
      
      interludeInterval = setInterval(() => {
        interludeDotIndex.value = (interludeDotIndex.value + 1) % 3
      }, 500)
    }

    function stopInterludeAnimation() {
      if (interludeInterval) {
        clearInterval(interludeInterval)
        interludeInterval = null
      }
    }

    // 监听器
    watch(() => props.currentTime, () => {
      updateCurrentLine()
    })

    watch(isInterlude, (newVal) => {
      if (newVal && props.isPlaying) {
        startInterludeAnimation()
      } else {
        stopInterludeAnimation()
      }
    })

    watch(() => props.isPlaying, (newVal) => {
      if (newVal && isInterlude.value) {
        startInterludeAnimation()
      } else {
        stopInterludeAnimation()
      }
    })

    // 生命周期
    onMounted(() => {
      updateCurrentLine()
    })

    onUnmounted(() => {
      stopInterludeAnimation()
    })

    return {
      lyricsContainer,
      lyricsLines,
      currentLineIndex,
      interludeDotIndex,
      contentStyle,
      textStyle,
      translatedStyle,
      romanStyle,
      isInterlude,
      getLineStyle,
      getWordStyle,
      seekToLine,
      isWordHighlighted,
      isWordPlayed
    }
  }
}
</script>

<style scoped>
.advanced-lyrics-view {
  position: relative;
  width: 100%;
  height: 100%;
  overflow-y: auto;
  background: linear-gradient(135deg, rgba(0, 0, 0, 0.9) 0%, rgba(20, 20, 40, 0.9) 100%);
  backdrop-filter: blur(10px);
  border-radius: 12px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
}

.lyrics-content {
  font-family: 'SF Pro Display', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  font-weight: 400;
  letter-spacing: 0.5px;
}

.lyric-line {
  position: relative;
  text-align: center;
  will-change: transform, opacity;
  backface-visibility: hidden;
}

.lyric-line:hover {
  background-color: rgba(255, 255, 255, 0.05);
}

.lyric-text {
  display: inline-block;
  position: relative;
}

.lyric-word {
  display: inline-block;
  position: relative;
  will-change: color, text-shadow;
}

.translated-lyric,
.roman-lyric {
  text-align: center;
  font-style: italic;
  will-change: opacity;
}

.interlude-indicator {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  z-index: 10;
}

.interlude-dots {
  display: flex;
  gap: 8px;
}

.dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.3);
  transition: all 0.3s ease;
}

.dot.active {
  background: #ff6b6b;
  transform: scale(1.2);
  box-shadow: 0 0 10px rgba(255, 107, 107, 0.5);
}

/* 滚动条样式 */
.advanced-lyrics-view::-webkit-scrollbar {
  width: 6px;
}

.advanced-lyrics-view::-webkit-scrollbar-track {
  background: transparent;
}

.advanced-lyrics-view::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.2);
  border-radius: 3px;
}

.advanced-lyrics-view::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.3);
}

/* 响应式设计 */
@media (max-width: 768px) {
  .advanced-lyrics-view {
    border-radius: 8px;
  }
  
  .lyrics-content {
    padding: 16px 0;
  }
}

/* 动画效果 */
@keyframes fadeInUp {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.lyric-line {
  animation: fadeInUp 0.5s ease forwards;
}

.lyric-line:nth-child(odd) {
  animation-delay: 0.1s;
}

.lyric-line:nth-child(even) {
  animation-delay: 0.2s;
}
</style>