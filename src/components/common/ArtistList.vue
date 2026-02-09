<script setup>
import { computed } from 'vue';
import { useRouter } from 'vue-router';
import { useCoverImages } from '@/composables/useCoverImage';
import CoverImage from './CoverImage.vue';

const props = defineProps({
  artists: {
    type: Array,
    default: () => []
  }
});

const router = useRouter();

// 使用 composable 批量加载封面
const artistsRef = computed(() => props.artists);
const { coverUrls } = useCoverImages(artistsRef, 'small');

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
  <div class="artist-list">
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
</template>

<style scoped>
.artist-list {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
  gap: 20px;
}

.artist-card {
  cursor: pointer;
  transition: transform 0.2s ease;
}

.artist-card:hover {
  transform: translateY(-4px);
}

.artist-cover-wrapper {
  position: relative;
  width: 100%;
  aspect-ratio: 1;
  border-radius: 50%;
  overflow: hidden;
  margin-bottom: 12px;
}

.artist-info {
  text-align: center;
}

.artist-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary, #333);
  margin: 0 0 4px 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.artist-meta {
  font-size: 12px;
  color: var(--text-secondary, #666);
  margin: 0;
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
    color: var(--text-primary, #f6f6f6);
  }
}
</style>
