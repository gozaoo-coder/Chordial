<script setup>
import { computed, ref } from 'vue';
import { useRouter } from 'vue-router';
import { useCoverImages } from '@/composables/useCoverImage';
import { useVirtualList } from '@/composables/useVirtualList';
import CoverImage from './CoverImage.vue';

const props = defineProps({
  albums: {
    type: Array,
    default: () => []
  },
  // 是否启用虚拟列表
  virtualScroll: {
    type: Boolean,
    default: true
  },
  // 每行显示的专辑数（用于计算每项高度）
  columns: {
    type: Number,
    default: 0 // 0 表示自动计算
  }
});

const router = useRouter();
const containerRef = ref(null);

// 使用 composable 批量加载封面
const albumsRef = computed(() => props.albums);
const { coverUrls } = useCoverImages(albumsRef, 'small');

// 计算每项高度（专辑卡片高度 = 封面高度 + 信息区域高度）
const itemHeight = computed(() => {
  // 根据屏幕宽度动态计算
  const width = window.innerWidth;
  if (width <= 480) return 220; // 小屏幕
  if (width <= 767) return 240; // 平板
  return 260; // 桌面
});

// 虚拟列表
const {
  visibleItems,
  totalHeight,
  onScroll,
  poolSize
} = useVirtualList(albumsRef, {
  itemHeight: itemHeight.value,
  bufferSize: 3,
  containerRef
});

const handleAlbumClick = (album) => {
  router.push(`/albums/${album.id}`);
};

const getCoverUrl = (album) => {
  // 优先使用从 ResourceManager 加载的封面
  return coverUrls.value.get(album.id) || album.coverData || '';
};

const formatYear = (year) => {
  return year || '未知年份';
};

const getTrackCount = (album) => {
  // Album 类使用 getTrackCount() 方法，AlbumSummary 类使用 trackCount 属性
  if (album.getTrackCount) {
    return album.getTrackCount();
  }
  return album.trackCount || 0;
};
</script>

<template>
  <div class="album-list-wrapper">
    <!-- 虚拟列表容器 -->
    <div
      v-if="virtualScroll && albums.length > 30"
      ref="containerRef"
      class="album-list virtual-scroll"
      @scroll="onScroll"
    >
      <!-- 可视区域 -->
      <div class="viewport" :style="{ height: totalHeight + 'px' }">
        <div
          v-for="{ item: album, index, offsetY } in visibleItems"
          :key="album.id"
          class="album-card virtual-item"
          :style="{ transform: `translateY(${offsetY}px)` }"
          @click="handleAlbumClick(album)"
        >
          <div class="album-cover-wrapper">
            <CoverImage
              :src="getCoverUrl(album)"
              :alt="album.title"
              type="album"
            />
            <div class="album-play-overlay">
              <button class="play-btn">
                <svg viewBox="0 0 24 24" fill="currentColor">
                  <path d="M8 5v14l11-7z"/>
                </svg>
              </button>
            </div>
          </div>

          <div class="album-info">
            <h3 class="album-title">{{ album.title }}</h3>
            <p class="album-artist">{{ album.artistName || '未知歌手' }}</p>
            <p class="album-year">{{ formatYear(album.year) }} · {{ getTrackCount(album) }} 首</p>
          </div>
        </div>
      </div>

      <!-- 性能指示器（开发环境显示） -->
      <div v-if="false" class="perf-indicator">
        DOM: {{ poolSize }} / {{ albums.length }}
      </div>
    </div>

    <!-- 普通网格（数据量小或禁用虚拟列表时） -->
    <div v-else class="album-list">
      <div
        v-for="album in albums"
        :key="album.id"
        class="album-card"
        @click="handleAlbumClick(album)"
      >
        <div class="album-cover-wrapper">
          <CoverImage
            :src="getCoverUrl(album)"
            :alt="album.title"
            type="album"
          />
          <div class="album-play-overlay">
            <button class="play-btn">
              <svg viewBox="0 0 24 24" fill="currentColor">
                <path d="M8 5v14l11-7z"/>
              </svg>
            </button>
          </div>
        </div>

        <div class="album-info">
          <h3 class="album-title">{{ album.title }}</h3>
          <p class="album-artist">{{ album.artistName || '未知歌手' }}</p>
          <p class="album-year">{{ formatYear(album.year) }} · {{ getTrackCount(album) }} 首</p>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.album-list-wrapper {
  width: 100%;
}

.album-list {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 28px;
}

/* 虚拟列表样式 */
.album-list.virtual-scroll {
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

.album-card {
  cursor: pointer;
  transition: transform var(--transition-spring);
}

.album-card:hover {
  transform: translateY(-6px);
}

.album-card:hover .album-play-overlay {
  opacity: 1;
}

.album-card:hover .album-cover-wrapper {
  box-shadow: var(--shadow-md);
}

/* 虚拟列表项使用绝对定位 */
.album-card.virtual-item {
  position: absolute;
  left: 0;
  right: 0;
  width: calc(25% - 18px); /* 4列布局 */
  contain: layout style paint;
}

@media (max-width: 1200px) {
  .album-card.virtual-item {
    width: calc(33.333% - 16px); /* 3列布局 */
  }
}

@media (max-width: 900px) {
  .album-card.virtual-item {
    width: calc(50% - 12px); /* 2列布局 */
  }
}

@media (max-width: 480px) {
  .album-card.virtual-item {
    width: calc(50% - 8px); /* 2列布局 */
  }
}

.album-cover-wrapper {
  position: relative;
  width: 100%;
  aspect-ratio: 1;
  border-radius: var(--radius-md);
  overflow: hidden;
  margin-bottom: 14px;
  box-shadow: var(--shadow-sm);
  transition: box-shadow var(--transition-normal);
}

.album-play-overlay {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.35);
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transition: opacity var(--transition-fast);
  backdrop-filter: blur(2px);
}

.play-btn {
  width: 52px;
  height: 52px;
  border: none;
  border-radius: 50%;
  background: var(--primary-color);
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: transform var(--transition-fast), box-shadow var(--transition-fast);
  box-shadow: var(--shadow-md);
}

.play-btn:hover {
  transform: scale(1.12);
  box-shadow: var(--shadow-primary);
}

.play-btn svg {
  width: 26px;
  height: 26px;
}

.album-info {
  padding: 0 4px;
}

.album-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0 0 5px 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  letter-spacing: -0.2px;
}

.album-artist {
  font-size: 13px;
  color: var(--text-secondary);
  margin: 0 0 3px 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  font-weight: 500;
}

.album-year {
  font-size: 12px;
  color: var(--text-tertiary);
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
  .album-list {
    grid-template-columns: repeat(2, 1fr);
    gap: 16px;
  }

  .album-title {
    font-size: 13px;
  }

  .album-artist {
    font-size: 12px;
  }

  .album-year {
    font-size: 11px;
  }
}

@media (max-width: 480px) {
  .album-list {
    grid-template-columns: repeat(2, 1fr);
    gap: 12px;
  }
}

@media (prefers-color-scheme: dark) {
  .album-title {
    color: var(--text-primary);
  }
}
</style>
