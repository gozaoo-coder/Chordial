<script setup>
import { computed } from 'vue';
import { useRouter } from 'vue-router';
import { useCoverImages } from '@/composables/useCoverImage';
import CoverImage from './CoverImage.vue';

const props = defineProps({
  albums: {
    type: Array,
    default: () => []
  }
});

const router = useRouter();

// 使用 composable 批量加载封面
const albumsRef = computed(() => props.albums);
const { coverUrls } = useCoverImages(albumsRef, 'small');

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
  <div class="album-list">
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
</template>

<style scoped>
.album-list {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 24px;
}

.album-card {
  cursor: pointer;
  transition: transform 0.2s ease;
}

.album-card:hover {
  transform: translateY(-4px);
}

.album-card:hover .album-play-overlay {
  opacity: 1;
}

.album-cover-wrapper {
  position: relative;
  width: 100%;
  aspect-ratio: 1;
  border-radius: var(--radius-md, 8px);
  overflow: hidden;
  margin-bottom: 12px;
  box-shadow: var(--shadow-sm, 0 1px 2px rgba(0, 0, 0, 0.05));
}

.album-play-overlay {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transition: opacity 0.2s ease;
}

.play-btn {
  width: 48px;
  height: 48px;
  border: none;
  border-radius: 50%;
  background: var(--primary-color, #667eea);
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: transform 0.2s ease;
}

.play-btn:hover {
  transform: scale(1.1);
}

.play-btn svg {
  width: 24px;
  height: 24px;
}

.album-info {
  padding: 0 4px;
}

.album-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary, #333);
  margin: 0 0 4px 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.album-artist {
  font-size: 13px;
  color: var(--text-secondary, #666);
  margin: 0 0 2px 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.album-year {
  font-size: 12px;
  color: var(--text-tertiary, #999);
  margin: 0;
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
    color: var(--text-primary, #f6f6f6);
  }
}
</style>
