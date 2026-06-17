<template>
  <div ref="containerRef" class="amll-background-container" :style="containerStyle">
    <canvas ref="canvasRef" class="amll-background-canvas" />
  </div>
</template>

<script setup>
/**
 * AMLL 背景渲染组件
 * 
 * 职责：独立管理 AMLL 背景渲染器的生命周期
 * 依赖：仅依赖 @applemusic-like-lyrics/core 的 BackgroundRender 和 MeshGradientRenderer
 * 输入：通过 props 接收专辑封面、播放状态等数据
 * 输出：渲染动态网格渐变背景
 */
import { ref, computed, watch, onMounted, onUnmounted, shallowRef } from 'vue'
import { BackgroundRender, MeshGradientRenderer } from '@applemusic-like-lyrics/core'

const props = defineProps({
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
  /** 低频音量 (0.0 - 1.0)，影响背景脉动效果 */
  lowFreqVolume: {
    type: Number,
    default: 0
  },
  /** 渲染缩放比例 (0.1 - 1.0) */
  renderScale: {
    type: Number,
    default: 0.75
  },
  /** 目标帧率 */
  fps: {
    type: Number,
    default: 30
  },
  /** 是否启用静态模式（节省性能） */
  staticMode: {
    type: Boolean,
    default: false
  },
  /** 流动速度 */
  flowSpeed: {
    type: Number,
    default: 4
  },
  /** 是否暂停动画 */
  paused: {
    type: Boolean,
    default: false
  },
  /** 是否有歌词（影响背景活跃度） */
  hasLyric: {
    type: Boolean,
    default: true
  },
  /** z-index 层级 */
  zIndex: {
    type: Number,
    default: -1
  }
})

const emit = defineEmits({
  /** 背景渲染器准备就绪 */
  ready: (renderer) => renderer !== null,
  /** 专辑封面加载完成 */
  albumLoaded: () => true,
  /** 发生错误 */
  error: (error) => error instanceof Error
})

// DOM 引用
const containerRef = ref(null)
const canvasRef = ref(null)

// 使用 shallowRef 存储大型对象，避免深度响应式带来的性能问题
const bgRenderer = shallowRef(null)
const isReady = ref(false)

// 容器样式
const containerStyle = computed(() => ({
  zIndex: props.zIndex
}))

/**
 * 初始化背景渲染器
 */
const initRenderer = () => {
  if (!canvasRef.value) return

  try {
    // 创建渲染器实例
    const renderer = BackgroundRender.new(MeshGradientRenderer)
    
    // 配置渲染器
    renderer.setRenderScale(props.renderScale)
    renderer.setFPS(props.fps)
    renderer.setFlowSpeed(props.flowSpeed)
    renderer.setStaticMode(props.staticMode)
    renderer.setHasLyric(props.hasLyric)
    
    // 将 canvas 移动到我们的容器中
    const canvas = renderer.getElement()
    canvas.style.position = 'absolute'
    canvas.style.top = '0'
    canvas.style.left = '0'
    canvas.style.width = '100%'
    canvas.style.height = '100%'
    
    // 清空并添加 canvas
    if (canvasRef.value) {
      canvasRef.value.replaceWith(canvas)
    }
    
    bgRenderer.value = renderer
    isReady.value = true
    emit('ready', renderer)
    
    // 如果有初始封面，立即加载
    if (props.albumCover) {
      loadAlbumCover(props.albumCover)
    }
  } catch (error) {
    console.error('[AMLLBackgroundRenderer] 初始化失败:', error)
    emit('error', error)
  }
}

/**
 * 加载专辑封面
 */
const loadAlbumCover = async (source) => {
  if (!bgRenderer.value || !source) return

  try {
    await bgRenderer.value.setAlbum(source, props.isVideo)
    emit('albumLoaded')
  } catch (error) {
    console.error('[AMLLBackgroundRenderer] 加载封面失败:', error)
    emit('error', error)
  }
}

/**
 * 销毁渲染器
 */
const disposeRenderer = () => {
  if (bgRenderer.value) {
    bgRenderer.value.dispose()
    bgRenderer.value = null
    isReady.value = false
  }
}

// 监听专辑封面变化
watch(() => props.albumCover, (newCover) => {
  if (newCover && bgRenderer.value) {
    loadAlbumCover(newCover)
  }
}, { immediate: false })

// 监听播放状态变化
watch(() => props.paused, (isPaused) => {
  if (!bgRenderer.value) return
  
  if (isPaused) {
    bgRenderer.value.pause()
  } else {
    bgRenderer.value.resume()
  }
})

// 监听低频音量变化
watch(() => props.lowFreqVolume, (volume) => {
  if (bgRenderer.value) {
    bgRenderer.value.setLowFreqVolume(volume)
  }
}, { immediate: false })

// 监听配置变化
watch(() => props.renderScale, (scale) => {
  if (bgRenderer.value) {
    bgRenderer.value.setRenderScale(scale)
  }
})

watch(() => props.fps, (fps) => {
  if (bgRenderer.value) {
    bgRenderer.value.setFPS(fps)
  }
})

watch(() => props.staticMode, (enabled) => {
  if (bgRenderer.value) {
    bgRenderer.value.setStaticMode(enabled)
  }
})

watch(() => props.flowSpeed, (speed) => {
  if (bgRenderer.value) {
    bgRenderer.value.setFlowSpeed(speed)
  }
})

watch(() => props.hasLyric, (hasLyric) => {
  if (bgRenderer.value) {
    bgRenderer.value.setHasLyric(hasLyric)
  }
})

// 生命周期
onMounted(() => {
  initRenderer()
})

onUnmounted(() => {
  disposeRenderer()
})

// 暴露方法给父组件
defineExpose({
  /** 获取渲染器实例 */
  getRenderer: () => bgRenderer.value,
  /** 手动设置专辑封面 */
  setAlbum: loadAlbumCover,
  /** 暂停动画 */
  pause: () => bgRenderer.value?.pause(),
  /** 恢复动画 */
  resume: () => bgRenderer.value?.resume()
})
</script>

<style scoped>
.amll-background-container {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  overflow: hidden;
  pointer-events: none;
}

.amll-background-canvas {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
}
</style>
