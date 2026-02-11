<script setup>
import { computed, ref, onMounted, onUnmounted } from 'vue';
import CoverImage from './CoverImage.vue';

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
  
  // 过滤出有封面的专辑
  const albumsWithCover = props.albums.filter(album => album.coverData);
  
  if (albumsWithCover.length === 0) return [];
  
  // 随机打乱并选取
  const shuffled = shuffleArray(albumsWithCover);
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

// 动画偏移量
const offsetX = ref(0);
const offsetY = ref(0);
let animationId = null;

// 缓慢漂移动画
const startAnimation = () => {
  if (!props.animationEnabled) return;
  
  let startTime = null;
  const duration = 60000; // 60秒一个周期
  
  const animate = (timestamp) => {
    if (!startTime) startTime = timestamp;
    const elapsed = timestamp - startTime;
    const progress = (elapsed % duration) / duration;
    
    // 缓慢的正弦波动
    offsetX.value = Math.sin(progress * Math.PI * 2) * 10; // ±10px
    offsetY.value = Math.cos(progress * Math.PI * 2) * 8;  // ±8px
    
    animationId = requestAnimationFrame(animate);
  };
  
  animationId = requestAnimationFrame(animate);
};

const stopAnimation = () => {
  if (animationId) {
    cancelAnimationFrame(animationId);
    animationId = null;
  }
};

onMounted(() => {
  startAnimation();
});

onUnmounted(() => {
  stopAnimation();
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

// 动画样式
const animationStyle = computed(() => {
  if (!props.animationEnabled) return {};
  return {
    transform: `translate(${offsetX.value}px, ${offsetY.value}px) scale(1.1)`
  };
});
</script>

<template>
  <div class="album-collage-background">
    <!-- 多专辑封面网格 -->
    <div 
      v-if="hasEnoughCovers" 
      class="collage-grid"
      :style="gridStyle"
    >
      <div 
        v-for="(album, index) in backgroundAlbums" 
        :key="`${album.id}-${index}`"
        class="collage-item"
        :style="animationStyle"
      >
        <CoverImage
          :src="album.coverData"
          :alt="album.title"
          type="album"
          class="collage-cover"
        />
      </div>
    </div>
    
    <!-- 备用背景 - 当专辑不足时 -->
    <div v-else class="fallback-gradient"></div>
    
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
  transform: scale(1.1);
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
  backdrop-filter: blur(40px) saturate(1.2);
  background: rgba(0, 0, 0, 0.2);
  z-index: 1;
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
  animation: gradientShift 15s ease infinite;
}

@keyframes gradientShift {
  0% { background-position: 0% 50%; }
  50% { background-position: 100% 50%; }
  100% { background-position: 0% 50%; }
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

/* 减少动画偏好 */
@media (prefers-reduced-motion: reduce) {
  .collage-item {
    transform: none !important;
  }
  
  .fallback-gradient {
    animation: none;
  }
}
</style>
