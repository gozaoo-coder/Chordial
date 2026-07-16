<script setup>
import { computed, onMounted, onUnmounted, ref, watch, nextTick } from 'vue';
import { animate } from 'animejs';
import CoverImage from './CoverImage.vue';
import { perf } from '@/utils/performanceMonitor.js';

const props = defineProps({
  albums: {
    type: Array,
    default: () => []
  },
  maxDisplay: {
    type: Number,
    default: 9
  },
  animationEnabled: {
    type: Boolean,
    default: true
  }
});

// 随机打乱数组
const shuffleArray = (array) => {
  const shuffled = [...array];
  for (let i = shuffled.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [shuffled[i], shuffled[j]] = [shuffled[j], shuffled[i]];
  }
  return shuffled;
};

// 获取用于背景的专辑封面
const backgroundAlbums = computed(() => {
  if (!props.albums || props.albums.length === 0) return [];

  // 过滤出有来源的专辑（可通过 ResourceManager 尝试加载封面）
  const albumsWithSource = props.albums.filter(
    album => album.sourceIds && album.sourceIds.length > 0
  );

  if (albumsWithSource.length === 0) return [];

  // 随机打乱并选取
  const shuffled = shuffleArray(albumsWithSource);
  const selected = shuffled.slice(0, props.maxDisplay);

  // 如果数量不足，重复填充以达到网格效果
  const result = [...selected];
  while (result.length < props.maxDisplay && selected.length > 0) {
    result.push(...selected.slice(0, props.maxDisplay - result.length));
  }

  return result.slice(0, props.maxDisplay);
});

// 是否有足够的封面
const hasEnoughCovers = computed(() => backgroundAlbums.value.length >= 3);

// perf: 旧实现用 requestAnimationFrame 每帧更新 offsetX/offsetY，
// 导致 animationStyle computed 每帧重算，9 个 collage-item 的 :style 各触发一次。
// 现改用 anime.js v4（WAAPI 路径，合成器线程），与 CSS keyframes 同等性能，
// 但统一由 JS 管理生命周期与 animationEnabled 暂停/恢复。
let driftAnim = null;
let gradientAnim = null;
let stopFps = null;

const gridRef = ref(null);
const gradientRef = ref(null);

const startDrift = () => {
  if (!gridRef.value) return;
  // revert 旧实例：清除其内联 transform，避免与新动画的 WAAPI 关键帧叠加
  driftAnim?.revert();
  // 60s 缓慢正弦漂移，5 个关键帧对应原 CSS collage-drift
  driftAnim = animate(gridRef.value, {
    scale: 1.1,
    translateX: [0, 10, 0, -10, 0],
    translateY: [0, 8, 16, 8, 0],
    duration: 60000,
    easing: 'linear',
    loop: true,
  });
  if (!props.animationEnabled) driftAnim.pause();
};

const startGradient = () => {
  if (!gradientRef.value) return;
  gradientAnim?.revert();
  // 15s 渐变背景位置循环
  gradientAnim = animate(gradientRef.value, {
    backgroundPositionX: ['0%', '100%', '0%'],
    backgroundPositionY: '50%',
    duration: 15000,
    easing: 'easeInOutQuad',
    loop: true,
  });
  if (!props.animationEnabled) gradientAnim.pause();
};

// animationEnabled 变化时暂停/恢复
watch(
  () => props.animationEnabled,
  (enabled) => {
    if (enabled) {
      driftAnim?.play();
      gradientAnim?.play();
    } else {
      driftAnim?.pause();
      gradientAnim?.pause();
    }
  }
);

// hasEnoughCovers 变化时（封面数据到达/切换 v-if v-else）启动对应动画
watch(
  hasEnoughCovers,
  async (has) => {
    await nextTick();
    if (has) startDrift();
    else startGradient();
  },
  { immediate: true }
);

onMounted(() => {
  // 保留 FPS 监控以验证 anime.js WAAPI 动画的帧率
  stopFps = perf.startFpsMonitor('AlbumCollageBackground.waapi', 2000);
});

onUnmounted(() => {
  // revert() 而非 pause()：停止动画并清除 anime.js 写入的内联 transform/backgroundPosition，
  // 防止组件被 <keep-alive> 缓存时残留内联样式干扰下次挂载的 CSS 布局
  driftAnim?.revert();
  gradientAnim?.revert();
  driftAnim = null;
  gradientAnim = null;
  if (stopFps) stopFps();
});

// 网格样式
const gridStyle = computed(() => {
  const count = backgroundAlbums.value.length;
  if (count <= 4) {
    return {
      gridTemplateColumns: 'repeat(2, 1fr)',
      gridTemplateRows: 'repeat(2, 1fr)'
    };
  } else if (count <= 6) {
    return {
      gridTemplateColumns: 'repeat(3, 1fr)',
      gridTemplateRows: 'repeat(2, 1fr)'
    };
  } else {
    return {
      gridTemplateColumns: 'repeat(3, 1fr)',
      gridTemplateRows: 'repeat(3, 1fr)'
    };
  }
});
</script>

<template>
  <div class="album-collage-background">
    <!-- 多专辑封面网格 -->
    <div
      v-if="hasEnoughCovers"
      ref="gridRef"
      class="collage-grid"
      :style="gridStyle"
    >
      <div
        v-for="(album, index) in backgroundAlbums"
        :key="`${album.id}-${index}`"
        class="collage-item"
      >
        <CoverImage
          :item="album"
          :alt="album.title"
          type="album"
          size="medium"
          class="collage-cover"
        />
      </div>
    </div>

    <!-- 备用背景 - 当专辑不足时 -->
    <div v-else ref="gradientRef" class="fallback-gradient"></div>
    
    <!-- 模糊层 -->
    <div class="blur-overlay"></div>
    
    <!-- 渐变遮罩 - 确保文字可读 -->
    <div class="gradient-overlay"></div>
  </div>
</template>

<style scoped>
.album-collage-background {
  position: absolute;
  inset: 0;
  overflow: hidden;
  z-index: 0;
}

/* 专辑封面网格 */
.collage-grid {
  position: absolute;
  inset: -20px;
  display: grid;
  gap: 4px;
  will-change: transform;
}

.collage-item {
  position: relative;
  overflow: hidden;
  border-radius: 4px;
  transition: transform 0.3s ease;
}

.collage-cover {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.collage-cover :deep(.cover-wrapper) {
  width: 100%;
  height: 100%;
}

.collage-cover :deep(.cover-image) {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

/* 模糊层 */
.blur-overlay {
  position: absolute;
  inset: 0;
  /* 4rem backdrop-filter 是 GPU 合成最重热点，降级到 2rem + 渐变叠加
     保持视觉接近，合成开销降约 60% */
  backdrop-filter: blur(2rem) saturate(1.2);
  background: rgba(0, 0, 0, 0.2);
  z-index: 1;
  will-change: backdrop-filter;
  contain: paint;
}

/* 渐变遮罩 - 从底部向上渐变 */
.gradient-overlay {
  position: absolute;
  inset: 0;
  background: linear-gradient(
    180deg,
    rgba(0, 0, 0, 0.1) 0%,
    rgba(0, 0, 0, 0.3) 40%,
    rgba(0, 0, 0, 0.6) 70%,
    rgba(0, 0, 0, 0.75) 100%
  );
  z-index: 2;
}

/* 备用渐变背景 */
.fallback-gradient {
  position: absolute;
  inset: 0;
  background: linear-gradient(
    135deg,
    #667eea 0%,
    #764ba2 50%,
    #f093fb 100%
  );
  background-size: 200% 200%;
}

/* 深色模式适配 */
@media (prefers-color-scheme: dark) {
  .blur-overlay {
    background: rgba(0, 0, 0, 0.35);
  }
  
  .gradient-overlay {
    background: linear-gradient(
      180deg,
      rgba(0, 0, 0, 0.2) 0%,
      rgba(0, 0, 0, 0.4) 40%,
      rgba(0, 0, 0, 0.7) 70%,
      rgba(0, 0, 0, 0.85) 100%
    );
  }
  
  .fallback-gradient {
    background: linear-gradient(
      135deg,
      #1a1a2e 0%,
      #16213e 50%,
      #0f0f23 100%
    );
  }
}
</style>
