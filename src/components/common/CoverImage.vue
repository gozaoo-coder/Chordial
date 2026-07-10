<script setup>
import { ref, computed, watch, toRef } from 'vue';
import { useCoverImage } from '@/composables/useCoverImage';

const props = defineProps({
  /** 直接传入 URL 字符串（向后兼容，优先级低于 item） */
  src: {
    type: String,
    default: ''
  },
  /** 专辑/歌手/歌曲对象（优先使用其 acquireCoverResource 方法加载封面） */
  item: {
    type: Object,
    default: null
  },
  alt: {
    type: String,
    default: ''
  },
  type: {
    type: String,
    default: 'album', // 'album' | 'artist' | 'track'
    validator: (value) => ['album', 'artist', 'track'].includes(value)
  },
  size: {
    type: String,
    default: 'medium' // 'small' | 'medium' | 'large'
  }
});

// ── 通过 useCoverImage 加载（item 模式）──
const itemRef = toRef(props, 'item');
const { coverUrl, isLoading: isComposableLoading } = useCoverImage(itemRef, props.size);

// ── 本地状态（src 模式 / 图片加载状态）──
const isImgLoading = ref(true);
const hasImgError = ref(false);

// 最终显示的 URL：item 加载的优先，其次 src
const displaySrc = computed(() => {
  if (coverUrl.value) return coverUrl.value;
  return props.src || '';
});

const isEmpty = computed(() => !displaySrc.value);

// item 变化时重置 img 状态
watch(() => props.src, () => {
  if (!coverUrl.value) {
    isImgLoading.value = true;
    hasImgError.value = false;
  }
});

// coverUrl 变化时重置 img 状态
watch(coverUrl, (newUrl) => {
  if (newUrl) {
    isImgLoading.value = true;
    hasImgError.value = false;
  }
});

const isLoading = computed(() => {
  // composable 正在后台加载
  if (itemRef.value && isComposableLoading.value) return true;
  // <img> 还在加载
  if (displaySrc.value && isImgLoading.value) return true;
  return false;
});

const handleLoad = () => {
  isImgLoading.value = false;
  hasImgError.value = false;
};

const handleError = () => {
  isImgLoading.value = false;
  hasImgError.value = true;
};
</script>

<template>
  <div class="cover-wrapper" :class="`cover-${type}`">
    <!-- 加载中 -->
    <div v-if="isLoading && !isEmpty" class="cover-placeholder cover-loading">
      <div class="spinner"></div>
    </div>

    <!-- 图片 -->
    <img
      v-if="!isEmpty"
      :src="displaySrc"
      :alt="alt"
      class="cover-image"
      :class="{ 'cover-hidden': hasImgError || (isLoading && displaySrc) }"
      loading="lazy"
      decoding="async"
      @load="handleLoad"
      @error="handleError"
    />

    <!-- 空状态 / 错误占位符 -->
    <div v-if="isEmpty || hasImgError" class="cover-placeholder">
      <!-- 专辑 -->
      <svg v-if="type === 'album'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <circle cx="12" cy="12" r="10"/>
        <circle cx="12" cy="12" r="3"/>
      </svg>
      <!-- 歌手 -->
      <svg v-else-if="type === 'artist'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/>
        <circle cx="12" cy="7" r="4"/>
      </svg>
      <!-- 歌曲 -->
      <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M9 18V5l12-2v13"/>
        <circle cx="6" cy="18" r="3"/>
        <circle cx="18" cy="16" r="3"/>
      </svg>
    </div>
  </div>
</template>

<style scoped>
.cover-wrapper {
  position: relative;
  width: 100%;
  height: 100%;
  overflow: hidden;
  background: var(--bg-tertiary, #e8e8ed);
}

.cover-album {
  border-radius: var(--radius-md, 8px);
}

.cover-artist {
  border-radius: 50%;
}

.cover-track {
  border-radius: var(--radius-sm, 6px);
}

.cover-image {
  width: 100%;
  height: 100%;
  object-fit: cover;
  opacity: 1;
  transition: opacity 0.2s ease;
}

.cover-hidden {
  opacity: 0;
  position: absolute;
}

.cover-placeholder {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
}

.cover-placeholder svg {
  width: 40%;
  height: 40%;
  color: var(--text-tertiary, #999);
}

.cover-loading {
  z-index: 1;
}

.spinner {
  width: 24px;
  height: 24px;
  border: 2px solid var(--border-color, rgba(0, 0, 0, 0.1));
  border-top-color: var(--primary-color, #667eea);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
