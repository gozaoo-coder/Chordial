<script setup>
import { computed, ref, onMounted, onUnmounted, watch, nextTick, useTemplateRef } from 'vue';
import { useRouter } from 'vue-router';
import { useCoverImages } from '@/composables/useCoverImage';
import { useVirtualList } from '@/composables/useVirtualList';
import { useAnime } from '@/composables/useAnime.js';
import CoverImage from './CoverImage.vue';

const props = defineProps({
  artists: {
    type: Array,
    default: () => []
  },
  // 是否启用虚拟列表
  virtualScroll: {
    type: Boolean,
    default: true
  }
});

const router = useRouter();
const containerRef = ref(null);

// anime.js 动画作用域（限定到组件根节点）
const rootRef = useTemplateRef('root');
const { run } = useAnime(() => rootRef.value);

// 使用 composable 批量加载封面
const artistsRef = computed(() => props.artists);
const { coverUrls } = useCoverImages(artistsRef, 'small');

// 根据容器宽度动态计算列数
const containerWidth = ref(window.innerWidth);
const columns = computed(() => {
  const w = containerWidth.value;
  if (w <= 480) return 2;   // 小屏幕 2 列
  if (w <= 767) return 3;   // 平板竖屏 3 列
  if (w <= 900) return 3;   // 平板横屏 3 列
  if (w <= 1200) return 4;  // 中等屏幕 4 列
  return 5;                 // 大屏幕 5 列
});

// 动态计算行高：卡片内容宽度 = 封面正方形高度 + 间距 + 文字区域 + 行间距
const itemHeight = computed(() => {
  const w = containerWidth.value;
  const cols = columns.value;
  // 卡片像素宽度 → 减去左右 padding (8px*2) → 封面正方形边长
  // + margin-bottom (14px) + 文字区域 (~42px) + 行间距 (20px)
  const cardWidth = w / cols;
  const coverSize = cardWidth - 16; // padding: 0 8px
  return coverSize + 14 + 42 + 20;
});

// 虚拟列表（网格模式）
const {
  visibleItems,
  totalHeight,
  onScroll,
  poolSize
} = useVirtualList(artistsRef, {
  itemHeight: itemHeight.value,
  bufferSize: 3,
  containerRef,
  columns: columns.value,
});

// 监听容器宽度变化
let resizeTimer = null;
const handleResize = () => {
  clearTimeout(resizeTimer);
  resizeTimer = setTimeout(() => {
    containerWidth.value = window.innerWidth;
  }, 100);
};

// --- 动画（anime.js v4）---
// 列表项错峰入场：listItemEnter + stagger。
// 虚拟列表项用 top/left/width 定位（非 transform），故 translateY 入场安全；
// 动画完成后清除内联 transform 恢复 CSS hover 的 translateY(-6px) 上浮。
function playEnter() {
  run(({ animate, stagger, presets }) => {
    const root = rootRef.value;
    if (!root) return;
    const items = root.querySelectorAll('.artist-card');
    if (!items.length) return;
    animate(items, {
      ...presets.listItemEnter,
      delay: stagger(50),
      onComplete: () => {
        // 清除内联 transform，恢复 CSS :hover 的 translateY(-6px) 上浮效果
        rootRef.value?.querySelectorAll('.artist-card').forEach((el) => {
          el.style.transform = '';
        });
      },
    });
  });
}

onMounted(() => {
  window.addEventListener('resize', handleResize);
  playEnter();
});

// 数据异步到达后触发入场动画
watch(() => props.artists, () => nextTick(playEnter), { flush: 'post' });

onUnmounted(() => {
  window.removeEventListener('resize', handleResize);
  if (resizeTimer) clearTimeout(resizeTimer);
});

const handleArtistClick = (artist) => {
  router.push(`/artists/${artist.id}`);
};

const getCoverUrl = (artist) => {
  // 优先使用从 ResourceManager 加载的封面
  return coverUrls.value.get(artist.id) || '';
};

const getTrackCount = (artist) => {
  // 优先使用属性（ArtistSummary），其次使用方法（Artist）
  if (typeof artist.trackCount === 'number') {
    return artist.trackCount;
  }
  if (typeof artist.getTrackCount === 'function') {
    return artist.getTrackCount();
  }
  return 0;
};

const getAlbumCount = (artist) => {
  // 优先使用属性（ArtistSummary），其次使用方法（Artist）
  if (typeof artist.albumCount === 'number') {
    return artist.albumCount;
  }
  if (typeof artist.getAlbumCount === 'function') {
    return artist.getAlbumCount();
  }
  return 0;
};
</script>

<template>
  <div ref="root" class="artist-list-wrapper">
    <!-- 虚拟列表容器 -->
    <div
      v-if="virtualScroll && artists.length > 30"
      ref="containerRef"
      class="artist-list virtual-scroll"
      @scroll="onScroll"
    >
      <!-- 可视区域 -->
      <div class="viewport" :style="{ height: totalHeight + 'px' }">
        <div
          v-for="{ item: artist, index, offsetY, offsetX, width } in visibleItems"
          :key="artist.id"
          class="artist-card virtual-item"
          :style="{
            left: offsetX + '%',
            top: offsetY + 'px',
            width: width + '%'
          }"
          @click="handleArtistClick(artist)"
        >
          <div class="artist-cover-wrapper">
            <CoverImage
              :src="getCoverUrl(artist)"
              :alt="artist.name"
              type="artist"
            />
          </div>

          <div class="artist-info">
            <h3 class="artist-name">{{ artist.name }}</h3>
            <p class="artist-meta">
              {{ getTrackCount(artist) }} 首歌曲 · {{ getAlbumCount(artist) }} 张专辑
            </p>
          </div>
        </div>
      </div>

      <!-- 性能指示器（开发环境显示） -->
      <div v-if="false" class="perf-indicator">
        DOM: {{ poolSize }} / {{ artists.length }}
      </div>
    </div>

    <!-- 普通网格（数据量小或禁用虚拟列表时） -->
    <div v-else class="artist-list">
      <div
        v-for="artist in artists"
        :key="artist.id"
        class="artist-card"
        @click="handleArtistClick(artist)"
      >
        <div class="artist-cover-wrapper">
          <CoverImage
            :src="getCoverUrl(artist)"
            :alt="artist.name"
            type="artist"
          />
        </div>

        <div class="artist-info">
          <h3 class="artist-name">{{ artist.name }}</h3>
          <p class="artist-meta">
            {{ getTrackCount(artist) }} 首歌曲 · {{ getAlbumCount(artist) }} 张专辑
          </p>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.artist-list-wrapper {
  width: 100%;
}

.artist-list {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
  gap: 24px;
}

/* 虚拟列表样式 */
.artist-list.virtual-scroll {
  position: relative;
  overflow-y: auto;
  overflow-x: hidden;
  max-height: 600px;
  -webkit-overflow-scrolling: touch;
  display: block; /* 覆盖 grid 布局 */
}

.viewport {
  position: relative;
  width: 100%;
}

.artist-card {
  cursor: pointer;
  transition: transform var(--transition-spring);
}

.artist-card:hover {
  transform: translateY(-6px);
}

.artist-card:hover .artist-cover-wrapper {
  box-shadow: var(--shadow-md);
}

/* 虚拟列表项使用绝对定位 */
.artist-card.virtual-item {
  position: absolute;
  top: 0;
  left: 0;
  contain: layout style paint;
  padding: 0 8px;
  box-sizing: border-box;
}

/* 虚拟列表项悬停时上浮（不影响定位） */
.artist-card.virtual-item:hover {
  transform: translateY(-6px);
  z-index: 1;
}

.artist-cover-wrapper {
  position: relative;
  width: 100%;
  aspect-ratio: 1;
  border-radius: 50%;
  overflow: hidden;
  margin-bottom: 14px;
  box-shadow: var(--shadow-sm);
  transition: box-shadow var(--transition-normal);
}

.artist-info {
  text-align: center;
}

.artist-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0 0 5px 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  letter-spacing: -0.2px;
}

.artist-meta {
  font-size: 12px;
  color: var(--text-secondary);
  margin: 0;
  font-weight: 500;
}

/* 性能指示器 */
.perf-indicator {
  position: fixed;
  bottom: 80px;
  right: 20px;
  background: rgba(0, 0, 0, 0.7);
  color: #fff;
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 12px;
  font-family: monospace;
  pointer-events: none;
  z-index: 100;
}

@media (max-width: 767px) {
  .artist-list {
    grid-template-columns: repeat(3, 1fr);
    gap: 12px;
  }

  .artist-name {
    font-size: 13px;
  }

  .artist-meta {
    font-size: 11px;
  }
}

@media (max-width: 480px) {
  .artist-list {
    grid-template-columns: repeat(2, 1fr);
  }
}

@media (prefers-color-scheme: dark) {
  .artist-name {
    color: var(--text-primary);
  }
}
</style>
