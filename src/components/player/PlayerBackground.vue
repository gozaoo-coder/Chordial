<template>
  <div class="player-background" :class="{ 'has-cover': hasCover && !webglFailed }">
    <!-- AMLL 流体背景 -->
    <div v-if="hasCover && !webglFailed" class="amll-background-wrapper">
      <BackgroundRender
        :key="renderKey"
        :album="stableCoverUrl"
        :playing="!isPlaying"
        :flow-speed="flowSpeed"
        :fps="fps"
        :has-lyric="hasLyric"
        :render-scale="renderScale"
        :low-freq-volume="lowFreqVolume"
        class="amll-background"
        @error="onBackgroundError"
      />
    </div>

    <!-- WebGL 失败或加载失败时的备用背景 -->
    <div v-if="!hasCover || webglFailed" class="fallback-background">
      <div class="cover-blur-bg" :style="coverBgStyle"></div>
      <div class="gradient-bg"></div>
    </div>

    <!-- 遮罩层 -->
    <div class="background-overlay"></div>
  </div>
</template>

<script>
import { computed, ref, watch, onUnmounted } from 'vue';
import { BackgroundRender } from '@applemusic-like-lyrics/vue';

export default {
  name: 'PlayerBackground',
  components: {
    BackgroundRender
  },
  props: {
    // 封面图片 URL
    coverUrl: {
      type: String,
      default: ''
    },
    // 是否正在播放
    isPlaying: {
      type: Boolean,
      default: false
    },
    // 背景流动速度
    flowSpeed: {
      type: Number,
      default: 2
    },
    // 帧率
    fps: {
      type: Number,
      default: 30
    },
    // 是否有歌词
    hasLyric: {
      type: Boolean,
      default: true
    },
    // 渲染缩放比例
    renderScale: {
      type: Number,
      default: 0.5
    },
    // 低频音量 (0-1)
    lowFreqVolume: {
      type: Number,
      default: 1.0
    }
  },
  setup(props) {
    // 使用 ref 存储稳定的封面 URL，避免频繁更新
    const stableCoverUrl = ref('');
    // 用于强制重新渲染 BackgroundRender 的 key
    const renderKey = ref(0);
    // WebGL 是否失败
    const webglFailed = ref(false);

    // 防抖处理 coverUrl 变化
    let coverUpdateTimer = null;
    watch(() => props.coverUrl, (newUrl, oldUrl) => {
      // 清除之前的定时器
      if (coverUpdateTimer) {
        clearTimeout(coverUpdateTimer);
      }

      coverUpdateTimer = setTimeout(() => {
        if (newUrl && newUrl.length > 0) {
          // 只在 URL 真正变化时才更新
          if (newUrl !== oldUrl) {
            stableCoverUrl.value = newUrl;
            // 增加 key 强制重新渲染 BackgroundRender
            renderKey.value++;
            // 重置 WebGL 失败状态
            webglFailed.value = false;
          }
        } else {
          stableCoverUrl.value = '';
        }
      }, 100); // 100ms 防抖
    }, { immediate: true });

    // 是否有封面
    const hasCover = computed(() => {
      return !!stableCoverUrl.value && stableCoverUrl.value.length > 0;
    });

    // 备用背景样式（使用封面图片做模糊背景）
    const coverBgStyle = computed(() => {
      if (!stableCoverUrl.value) return {};
      return {
        backgroundImage: `url(${stableCoverUrl.value})`,
        backgroundSize: 'cover',
        backgroundPosition: 'center'
      };
    });

    // 处理背景渲染错误
    const onBackgroundError = (error) => {
      console.warn('BackgroundRender 错误，切换到备用背景:', error);
      webglFailed.value = true;
    };

    // 组件卸载时清理定时器
    onUnmounted(() => {
      if (coverUpdateTimer) {
        clearTimeout(coverUpdateTimer);
      }
    });

    return {
      hasCover,
      stableCoverUrl,
      renderKey,
      webglFailed,
      coverBgStyle,
      onBackgroundError
    };
  }
};
</script>

<style scoped>
.player-background {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  z-index: 0;
  overflow: hidden;
}

/* AMLL 流体背景包装器 */
.amll-background-wrapper {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  z-index: 0;
  background-color: #808080;
}

.amll-background {
  width: 100% !important;
  height: 100% !important;
}

/* 默认背景 */
.default-background {
  width: 100%;
  height: 100%;
  background: linear-gradient(135deg, #1a1a2e 0%, #16213e 50%, #0f0f23 100%);
}

/* WebGL 失败时的备用背景 */
.fallback-background {
  width: 100%;
  height: 100%;
  position: relative;
  background: linear-gradient(135deg, #1a1a2e 0%, #16213e 50%, #0f0f23 100%);
}

.cover-blur-bg {
  position: absolute;
  top: -20px;
  left: -20px;
  right: -20px;
  bottom: -20px;
  filter: blur(60px) brightness(0.7);
  transform: scale(1.2);
  opacity: 0.8;
  transition: background-image 0.5s ease;
}

.gradient-bg {
  width: 100%;
  height: 100%;
  position: relative;
  z-index: 1;
  background:
    radial-gradient(ellipse at 20% 20%, rgba(102, 126, 234, 0.15) 0%, transparent 50%),
    radial-gradient(ellipse at 80% 80%, rgba(118, 75, 162, 0.15) 0%, transparent 50%),
    radial-gradient(ellipse at 50% 50%, rgba(255, 255, 255, 0.05) 0%, transparent 70%);
  animation: gradientShift 20s ease-in-out infinite;
}

@keyframes gradientShift {
  0%, 100% {
    transform: scale3d(1, 1, 1);
    opacity: 1;
  }
  50% {
    transform: scale3d(1.05, 1.05, 1);
    opacity: 0.85;
  }
}

/* 遮罩层 */
.background-overlay {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: rgba(0, 0, 0, 0.3);
  backdrop-filter: blur(0px);
  z-index: 1;
}

/* 深色模式适配 */
@media (prefers-color-scheme: dark) {
  .default-background {
    background: linear-gradient(135deg, #0a0a12 0%, #0c1020 50%, #080810 100%);
  }
}
</style>
