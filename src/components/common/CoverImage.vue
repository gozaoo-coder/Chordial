<script setup>
import { ref, computed, watch } from 'vue';

const props = defineProps({
  src: {
    type: String,
    default: ''
  },
  alt: {
    type: String,
    default: ''
  },
  type: {
    type: String,
    default: 'album', // 'album' | 'artist' | 'track'
    validator: (value) => ['album', 'artist', 'track'].includes(value)
  }
});

const isLoading = ref(true);
const hasError = ref(false);
const isEmpty = computed(() => !props.src);

// 当 src 变化时重置状态
watch(() => props.src, () => {
  isLoading.value = true;
  hasError.value = false;
});

const handleLoad = () => {
  isLoading.value = false;
  hasError.value = false;
};

const handleError = () => {
  isLoading.value = false;
  hasError.value = true;
};
</script>

<template>
  <div class="cover-wrapper" :class="`cover-${type}`">
    <!-- 加载中状态 -->
    <div v-if="isLoading && !isEmpty" class="cover-placeholder cover-loading">
      <div class="spinner"></div>
    </div>
    
    <!-- 图片 -->
    <img
      v-if="!isEmpty"
      :src="src"
      :alt="alt"
      class="cover-image"
      :class="{ 'cover-hidden': hasError || isLoading }"
      @load="handleLoad"
      @error="handleError"
    />
    
    <!-- 空状态或错误状态 -->
    <div v-if="isEmpty || hasError" class="cover-placeholder">
      <!-- 专辑占位符 -->
      <svg v-if="type === 'album'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <circle cx="12" cy="12" r="10"/>
        <circle cx="12" cy="12" r="3"/>
      </svg>
      
      <!-- 歌手占位符 -->
      <svg v-else-if="type === 'artist'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/>
        <circle cx="12" cy="7" r="4"/>
      </svg>
      
      <!-- 歌曲占位符 -->
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
