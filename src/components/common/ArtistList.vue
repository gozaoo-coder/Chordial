<script setup>
import { computed, ref } from 'vue';
import { useRouter } from 'vue-router';
import { useCoverImages } from '@/composables/useCoverImage';
import { useVirtualList } from '@/composables/useVirtualList';
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

// 使用 composable 批量加载封面
const artistsRef = computed(() => props.artists);
const { coverUrls } = useCoverImages(artistsRef, 'small');

// 计算每项高度（歌手卡片高度 = 封面高度 + 信息区域高度）
const itemHeight = computed(() => {
  // 根据屏幕宽度动态计算
  const width = window.innerWidth;
  if (width <= 480) return 180; // 小屏幕
  if (width <= 767) return 200; // 平板
  return 220; // 桌面
});

// 虚拟列表
const {
  visibleItems,
  totalHeight,
  onScroll,
  poolSize
} = useVirtualList(artistsRef, {
  itemHeight: itemHeight.value,
  bufferSize: 3,
  containerRef
});

const handleArtistClick = (artist) => {
  router.push(`/artists/${artist.id}`);
};

const getCoverUrl = (artist) => {
  // 优先使用从 ResourceManager 加载的封面
  return coverUrls.value.get(artist.id) || artist.coverData || '';
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
  <div class="artist-list-wrapper">
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
          v-for="{ item: artist, index, offsetY } in visibleItems"
          :key="artist.id"
          class="artist-card virtual-item"
          :style="{ transform: `translateY(${offsetY}px)` }"
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
  left: 0;
  right: 0;
  width: calc(20% - 16px); /* 5列布局 */
  contain: layout style paint;
}

@media (max-width: 1200px) {
  .artist-card.virtual-item {
    width: calc(25% - 15px); /* 4列布局 */
  }
}

@media (max-width: 900px) {
  .artist-card.virtual-item {
    width: calc(33.333% - 14px); /* 3列布局 */
  }
}

@media (max-width: 767px) {
  .artist-card.virtual-item {
    width: calc(33.333% - 14px); /* 3列布局 */
  }
}

@media (max-width: 480px) {
  .artist-card.virtual-item {
    width: calc(50% - 10px); /* 2列布局 */
  }
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
