<template>
  <div class="amll-player-container" :class="{ 'has-lyric': hasLyric }">
    <!-- 背景渲染层 -->
    <AMLLBackgroundRenderer
      ref="backgroundRef"
      :album-cover="albumCover"
      :is-video="isVideo"
      :low-freq-volume="lowFreqVolume"
      :render-scale="renderScale"
      :fps="fps"
      :static-mode="staticMode"
      :flow-speed="flowSpeed"
      :paused="!playing"
      :has-lyric="hasLyric"
      :z-index="-1"
      @ready="onBackgroundReady"
      @album-loaded="onAlbumLoaded"
      @error="onBackgroundError"
    />
    
    <!-- 内容层 -->
    <div class="player-content">
      <!-- 左侧：专辑封面和歌曲信息 -->
      <div class="player-left-section" :style="leftSectionStyle">
        <slot name="left">
          <!-- 默认内容：专辑封面 -->
          <div class="album-cover-wrapper">
            <img 
              v-if="albumCover && !isVideo" 
              :src="albumCover" 
              class="album-cover"
              alt="Album Cover"
            />
            <video
              v-else-if="albumCover && isVideo"
              :src="albumCover"
              class="album-cover"
              autoplay
              loop
              muted
            />
            <div v-else class="album-cover-placeholder">
              <i class="bi bi-music-note-beamed"></i>
            </div>
          </div>
          
          <!-- 歌曲信息 -->
          <div class="track-info">
            <h2 class="track-name">{{ trackName }}</h2>
            <p class="track-artists">{{ artistsText }}</p>
            <p v-if="albumName" class="track-album">{{ albumName }}</p>
          </div>
        </slot>
      </div>
      
      <!-- 右侧：歌词显示 -->
      <div class="player-right-section" :style="rightSectionStyle">
        <AMLLLyricPlayer
          ref="lyricRef"
          :lyric-lines="processedLyricLines"
          :current-time="currentTime"
          :playing="playing"
          :disabled="!hasLyric"
          :align-position="lyricAlignPosition"
          :align-anchor="lyricAlignAnchor"
          :enable-blur="enableBlur"
          :enable-scale="enableScale"
          :enable-spring="enableSpring"
          :word-fade-width="wordFadeWidth"
          :hide-passed-lines="hidePassedLines"
          :font-size="lyricFontSize"
          @ready="onLyricReady"
          @line-click="onLyricLineClick"
          @line-context-menu="onLyricLineContextMenu"
          @error="onLyricError"
        />
      </div>
    </div>
    
    <!-- 底部控制栏插槽 -->
    <div class="player-controls-section">
      <slot name="controls" />
    </div>
  </div>
</template>

<script setup>
/**
 * AMLL 播放器容器组件
 * 
 * 职责：协调背景渲染器和歌词播放器，提供完整的播放界面布局
 * 依赖：AMLLBackgroundRenderer, AMLLLyricPlayer
 * 特点：
 *   - 纯展示组件，不直接管理播放状态
 *   - 通过 props 接收所有数据，通过 events 通知外部
 *   - 支持插槽自定义左右区域内容
 */
import { ref, computed, watch } from 'vue'
import AMLLBackgroundRenderer from './AMLLBackgroundRenderer.vue'
import AMLLLyricPlayer from './AMLLLyricPlayer.vue'

const props = defineProps({
  // === 专辑封面相关 ===
  /** 专辑封面 URL 或 HTMLImageElement */
  albumCover: {
    type: [String, Object],
    default: null
  },
  /** 是否为视频封面 */
  isVideo: {
    type: Boolean,
    default: false
  },
  
  // === 歌曲信息 ===
  /** 歌曲名称 */
  trackName: {
    type: String,
    default: 'Unknown Track'
  },
  /** 歌手列表 */
  artists: {
    type: Array,
    default: () => []
  },
  /** 专辑名称 */
  albumName: {
    type: String,
    default: ''
  },
  
  // === 播放状态 ===
  /** 是否正在播放 */
  playing: {
    type: Boolean,
    default: false
  },
  /** 当前播放时间（毫秒） */
  currentTime: {
    type: Number,
    default: 0
  },
  /** 低频音量 (0.0 - 1.0) */
  lowFreqVolume: {
    type: Number,
    default: 0
  },
  
  // === 歌词数据 ===
  /** 原始歌词数据 */
  lyricLines: {
    type: Array,
    default: () => []
  },
  
  // === 背景渲染配置 ===
  /** 渲染缩放比例 */
  renderScale: {
    type: Number,
    default: 0.75
  },
  /** 目标帧率 */
  fps: {
    type: Number,
    default: 30
  },
  /** 流动速度 */
  flowSpeed: {
    type: Number,
    default: 4
  },
  /** 是否启用静态模式 */
  staticMode: {
    type: Boolean,
    default: false
  },
  
  // === 歌词显示配置 ===
  /** 歌词对齐位置 (0.0 - 1.0) */
  lyricAlignPosition: {
    type: Number,
    default: 0.5
  },
  /** 对齐锚点 */
  lyricAlignAnchor: {
    type: String,
    default: 'center'
  },
  /** 是否启用模糊效果 */
  enableBlur: {
    type: Boolean,
    default: true
  },
  /** 是否启用缩放效果 */
  enableScale: {
    type: Boolean,
    default: true
  },
  /** 是否启用弹簧动画 */
  enableSpring: {
    type: Boolean,
    default: true
  },
  /** 文字渐变宽度 */
  wordFadeWidth: {
    type: Number,
    default: 0.5
  },
  /** 是否隐藏已播放的歌词行 */
  hidePassedLines: {
    type: Boolean,
    default: false
  },
  /** 歌词字体大小 */
  lyricFontSize: {
    type: Number,
    default: 32
  },
  
  // === 布局配置 ===
  /** 左区域宽度比例 (0.0 - 1.0) */
  leftSectionRatio: {
    type: Number,
    default: 0.4
  },
  /** 是否垂直布局（移动端） */
  verticalLayout: {
    type: Boolean,
    default: false
  }
})

const emit = defineEmits({
  /** 背景渲染器准备就绪 */
  backgroundReady: (renderer) => renderer !== null,
  /** 歌词播放器准备就绪 */
  lyricReady: (player) => player !== null,
  /** 歌词行被点击（用于跳转） */
  seekToLine: (event) => event && typeof event.lineIndex === 'number',
  /** 歌词行右键菜单 */
  lyricContextMenu: (event) => event && typeof event.lineIndex === 'number',
  /** 发生错误 */
  error: (error) => error instanceof Error
})

// 组件引用
const backgroundRef = ref(null)
const lyricRef = ref(null)

// 内部状态
const isBackgroundReady = ref(false)
const isLyricReady = ref(false)

// 计算属性
const hasLyric = computed(() => props.lyricLines.length > 0)

const artistsText = computed(() => {
  if (Array.isArray(props.artists)) {
    return props.artists.map(a => typeof a === 'string' ? a : a.name).join(', ')
  }
  return String(props.artists)
})

/**
 * 处理歌词数据，确保格式正确
 */
const processedLyricLines = computed(() => {
  if (!props.lyricLines || props.lyricLines.length === 0) {
    return []
  }
  
  return props.lyricLines.map(line => ({
    words: line.words || [],
    startTime: line.startTime || 0,
    endTime: line.endTime || 0,
    translatedLyric: line.translatedLyric || '',
    romanLyric: line.romanLyric || '',
    isBG: line.isBG || false,
    isDuet: line.isDuet || false
  }))
})

// 布局样式
const leftSectionStyle = computed(() => {
  if (props.verticalLayout) {
    return {
      width: '100%',
      height: '40%'
    }
  }
  return {
    width: `${props.leftSectionRatio * 100}%`,
    height: '100%'
  }
})

const rightSectionStyle = computed(() => {
  if (props.verticalLayout) {
    return {
      width: '100%',
      height: '60%'
    }
  }
  return {
    width: `${(1 - props.leftSectionRatio) * 100}%`,
    height: '100%'
  }
})

// 事件处理
const onBackgroundReady = (renderer) => {
  isBackgroundReady.value = true
  emit('backgroundReady', renderer)
}

const onLyricReady = (player) => {
  isLyricReady.value = true
  emit('lyricReady', player)
}

const onAlbumLoaded = () => {
  // 专辑封面加载完成
}

const onLyricLineClick = (event) => {
  emit('seekToLine', event)
}

const onLyricLineContextMenu = (event) => {
  emit('lyricContextMenu', event)
}

const onBackgroundError = (error) => {
  console.error('[AMLLPlayerContainer] 背景渲染错误:', error)
  emit('error', error)
}

const onLyricError = (error) => {
  console.error('[AMLLPlayerContainer] 歌词渲染错误:', error)
  emit('error', error)
}

// 暴露方法给父组件
defineExpose({
  /** 获取背景渲染器实例 */
  getBackgroundRenderer: () => backgroundRef.value?.getRenderer(),
  /** 获取歌词播放器实例 */
  getLyricPlayer: () => lyricRef.value?.getPlayer(),
  /** 设置专辑封面 */
  setAlbum: (cover) => backgroundRef.value?.setAlbum(cover),
  /** 设置歌词数据 */
  setLyricLines: (lines) => lyricRef.value?.setLyricLines(lines),
  /** 设置当前时间 */
  setCurrentTime: (time) => lyricRef.value?.setCurrentTime(time),
  /** 暂停 */
  pause: () => {
    backgroundRef.value?.pause()
    lyricRef.value?.pause()
  },
  /** 恢复 */
  resume: () => {
    backgroundRef.value?.resume()
    lyricRef.value?.resume()
  }
})
</script>

<style scoped>
.amll-player-container {
  position: relative;
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: linear-gradient(180deg, rgba(0,0,0,0.3) 0%, rgba(0,0,0,0.5) 100%);
}

.player-content {
  flex: 1;
  display: flex;
  flex-direction: row;
  padding: 2rem;
  gap: 2rem;
  overflow: hidden;
}

/* 垂直布局 */
.amll-player-container.vertical .player-content {
  flex-direction: column;
  padding: 1rem;
  gap: 1rem;
}

/* 左侧区域 */
.player-left-section {
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  gap: 1.5rem;
  z-index: 1;
}

.album-cover-wrapper {
  width: 60%;
  max-width: 300px;
  aspect-ratio: 1;
  border-radius: 12px;
  overflow: hidden;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
}

.album-cover {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.album-cover-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  font-size: 3rem;
}

.track-info {
  text-align: center;
  color: white;
  text-shadow: 0 2px 4px rgba(0, 0, 0, 0.3);
}

.track-name {
  font-size: 1.5rem;
  font-weight: 600;
  margin: 0 0 0.5rem 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 100%;
}

.track-artists {
  font-size: 1rem;
  opacity: 0.8;
  margin: 0 0 0.25rem 0;
}

.track-album {
  font-size: 0.875rem;
  opacity: 0.6;
  margin: 0;
}

/* 右侧区域 */
.player-right-section {
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: center;
  z-index: 1;
  min-height: 0;
}

/* 底部控制栏 */
.player-controls-section {
  position: relative;
  z-index: 2;
  padding: 1rem 2rem;
  background: linear-gradient(transparent, rgba(0,0,0,0.3));
}

/* 无歌词时的样式 */
.amll-player-container:not(.has-lyric) .player-right-section {
  opacity: 0;
  pointer-events: none;
}

.amll-player-container:not(.has-lyric) .player-left-section {
  width: 100% !important;
}

/* 响应式布局 */
@media (max-width: 768px) {
  .player-content {
    flex-direction: column;
    padding: 1rem;
    gap: 1rem;
  }
  
  .player-left-section,
  .player-right-section {
    width: 100% !important;
    height: 50% !important;
  }
  
  .album-cover-wrapper {
    width: 50%;
    max-width: 200px;
  }
  
  .track-name {
    font-size: 1.25rem;
  }
  
  .track-artists {
    font-size: 0.875rem;
  }
}
</style>
