<template>
  <div ref="lyricContainerRef" class="amll-lyric-container" :style="containerStyle">
    <!-- 歌词播放器将挂载到这里 -->
  </div>
</template>

<script setup>
/**
 * AMLL 歌词播放器组件
 * 
 * 职责：独立管理 AMLL 歌词播放器的生命周期和渲染
 * 依赖：仅依赖 @applemusic-like-lyrics/core 的 LyricPlayer (DomLyricPlayer)
 * 输入：通过 props 接收歌词数据、播放时间、播放状态等
 * 输出：渲染逐词歌词动画，触发歌词行点击事件
 */
import { 
  ref, 
  computed, 
  watch, 
  onMounted, 
  onUnmounted, 
  shallowRef,
  nextTick 
} from 'vue'
import { LyricPlayer } from '@applemusic-like-lyrics/core'

const props = defineProps({
  /** 歌词行数据数组 */
  lyricLines: {
    type: Array,
    default: () => []
  },
  /** 当前播放时间（毫秒） */
  currentTime: {
    type: Number,
    default: 0
  },
  /** 是否正在播放 */
  playing: {
    type: Boolean,
    default: false
  },
  /** 是否禁用 */
  disabled: {
    type: Boolean,
    default: false
  },
  /** 歌词对齐位置 (0.0 - 1.0) */
  alignPosition: {
    type: Number,
    default: 0.5
  },
  /** 对齐锚点 ('top' | 'bottom' | 'center') */
  alignAnchor: {
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
  /** 字体大小（px） */
  fontSize: {
    type: Number,
    default: 32
  },
  /** 行高 */
  lineHeight: {
    type: Number,
    default: 1.5
  },
  /** 歌词区域宽度 */
  width: {
    type: [Number, String],
    default: '100%'
  },
  /** 歌词区域高度 */
  height: {
    type: [Number, String],
    default: '100%'
  }
})

const emit = defineEmits({
  /** 歌词播放器准备就绪 */
  ready: (player) => player !== null,
  /** 歌词行被点击 */
  lineClick: (event) => event && typeof event.lineIndex === 'number',
  /** 歌词行右键菜单 */
  lineContextMenu: (event) => event && typeof event.lineIndex === 'number',
  /** 发生错误 */
  error: (error) => error instanceof Error
})

// DOM 引用
const lyricContainerRef = ref(null)

// 使用 shallowRef 存储大型对象
const lyricPlayer = shallowRef(null)
const isReady = ref(false)

// 动画帧 ID
let animationFrameId = null
let lastTime = 0

// 容器样式
const containerStyle = computed(() => ({
  width: typeof props.width === 'number' ? `${props.width}px` : props.width,
  height: typeof props.height === 'number' ? `${props.height}px` : props.height,
  fontSize: `${props.fontSize}px`,
  lineHeight: props.lineHeight,
  opacity: props.disabled ? 0.3 : 1,
  pointerEvents: props.disabled ? 'none' : 'auto'
}))

/**
 * 初始化歌词播放器
 */
const initLyricPlayer = async () => {
  if (!lyricContainerRef.value) return

  try {
    // 创建歌词播放器实例
    const player = new LyricPlayer()
    
    // 配置播放器
    player.setEnableBlur(props.enableBlur)
    player.setEnableScale(props.enableScale)
    player.setEnableSpring(props.enableSpring)
    player.setWordFadeWidth(props.wordFadeWidth)
    player.setHidePassedLines(props.hidePassedLines)
    player.setAlignPosition(props.alignPosition)
    player.setAlignAnchor(props.alignAnchor)
    
    // 获取 DOM 元素并添加到容器
    const element = player.getElement()
    element.style.width = '100%'
    element.style.height = '100%'
    
    lyricContainerRef.value.appendChild(element)
    
    // 绑定事件
    element.addEventListener('line-click', handleLineClick)
    element.addEventListener('line-contextmenu', handleLineContextMenu)
    
    // 设置初始歌词
    if (props.lyricLines.length > 0) {
      player.setLyricLines(props.lyricLines, props.currentTime)
    }
    
    // 设置初始时间
    player.setCurrentTime(props.currentTime)
    
    // 根据播放状态启动或暂停
    if (props.playing) {
      player.resume()
      startAnimationLoop()
    } else {
      player.pause()
    }
    
    lyricPlayer.value = player
    isReady.value = true
    emit('ready', player)
  } catch (error) {
    console.error('[AMLLLyricPlayer] 初始化失败:', error)
    emit('error', error)
  }
}

/**
 * 处理歌词行点击事件
 */
const handleLineClick = (event) => {
  emit('lineClick', {
    lineIndex: event.lineIndex,
    line: event.line,
    originalEvent: event
  })
}

/**
 * 处理歌词行右键菜单事件
 */
const handleLineContextMenu = (event) => {
  emit('lineContextMenu', {
    lineIndex: event.lineIndex,
    line: event.line,
    originalEvent: event
  })
}

/**
 * 动画循环
 */
const animationLoop = (currentTime) => {
  if (!lyricPlayer.value) return
  
  const delta = lastTime ? currentTime - lastTime : 16.67
  lastTime = currentTime
  
  lyricPlayer.value.update(delta)
  
  if (props.playing) {
    animationFrameId = requestAnimationFrame(animationLoop)
  }
}

/**
 * 启动动画循环
 */
const startAnimationLoop = () => {
  if (animationFrameId) return
  lastTime = 0
  animationFrameId = requestAnimationFrame(animationLoop)
}

/**
 * 停止动画循环
 */
const stopAnimationLoop = () => {
  if (animationFrameId) {
    cancelAnimationFrame(animationFrameId)
    animationFrameId = null
  }
  lastTime = 0
}

/**
 * 销毁歌词播放器
 */
const disposeLyricPlayer = () => {
  stopAnimationLoop()
  
  if (lyricPlayer.value) {
    const element = lyricPlayer.value.getElement()
    element.removeEventListener('line-click', handleLineClick)
    element.removeEventListener('line-contextmenu', handleLineContextMenu)
    
    lyricPlayer.value.dispose()
    lyricPlayer.value = null
    isReady.value = false
  }
}

/**
 * 更新歌词数据
 */
const updateLyricLines = (lines) => {
  if (!lyricPlayer.value) return
  
  // 深拷贝歌词数据，避免引用问题
  const processedLines = lines.map(line => ({
    ...line,
    words: line.words?.map(word => ({ ...word })) || []
  }))
  
  lyricPlayer.value.setLyricLines(processedLines, props.currentTime)
}

/**
 * 更新当前时间
 */
const updateCurrentTime = (time) => {
  if (!lyricPlayer.value) return
  lyricPlayer.value.setCurrentTime(time)
}

// 监听歌词数据变化
watch(() => props.lyricLines, (newLines) => {
  updateLyricLines(newLines)
}, { deep: true })

// 监听播放时间变化
watch(() => props.currentTime, (newTime) => {
  updateCurrentTime(newTime)
})

// 监听播放状态变化
watch(() => props.playing, (isPlaying) => {
  if (!lyricPlayer.value) return
  
  if (isPlaying) {
    lyricPlayer.value.resume()
    startAnimationLoop()
  } else {
    lyricPlayer.value.pause()
    stopAnimationLoop()
  }
})

// 监听配置变化
watch(() => props.enableBlur, (enabled) => {
  if (lyricPlayer.value) {
    lyricPlayer.value.setEnableBlur(enabled)
  }
})

watch(() => props.enableScale, (enabled) => {
  if (lyricPlayer.value) {
    lyricPlayer.value.setEnableScale(enabled)
  }
})

watch(() => props.enableSpring, (enabled) => {
  if (lyricPlayer.value) {
    lyricPlayer.value.setEnableSpring(enabled)
  }
})

watch(() => props.wordFadeWidth, (width) => {
  if (lyricPlayer.value) {
    lyricPlayer.value.setWordFadeWidth(width)
  }
})

watch(() => props.hidePassedLines, (hide) => {
  if (lyricPlayer.value) {
    lyricPlayer.value.setHidePassedLines(hide)
  }
})

watch(() => props.alignPosition, (position) => {
  if (lyricPlayer.value) {
    lyricPlayer.value.setAlignPosition(position)
  }
})

watch(() => props.alignAnchor, (anchor) => {
  if (lyricPlayer.value) {
    lyricPlayer.value.setAlignAnchor(anchor)
  }
})

// 生命周期
onMounted(() => {
  nextTick(() => {
    initLyricPlayer()
  })
})

onUnmounted(() => {
  disposeLyricPlayer()
})

// 暴露方法给父组件
defineExpose({
  /** 获取歌词播放器实例 */
  getPlayer: () => lyricPlayer.value,
  /** 设置歌词数据 */
  setLyricLines: updateLyricLines,
  /** 设置当前时间 */
  setCurrentTime: updateCurrentTime,
  /** 暂停 */
  pause: () => {
    lyricPlayer.value?.pause()
    stopAnimationLoop()
  },
  /** 恢复 */
  resume: () => {
    lyricPlayer.value?.resume()
    startAnimationLoop()
  }
})
</script>

<style scoped>
.amll-lyric-container {
  position: relative;
  width: 100%;
  height: 100%;
  overflow: hidden;
  /* AMLL 样式会通过 JS 动态注入 */
}

/* 确保 AMLL 内部样式正确应用 */
.amll-lyric-container :deep(.amll-lyric-player) {
  width: 100% !important;
  height: 100% !important;
}
</style>
