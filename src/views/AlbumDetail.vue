<script setup>
import { ref, onMounted, watch, computed } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import TrackList from '../components/common/TrackList.vue';
import { getAlbum } from '../api/album';
import { getTracksByIds } from '../api/musicSource/musicResource';
import { useCoverImage } from '@/composables/useCoverImage';

const route = useRoute();
const router = useRouter();

const album = ref(null);
const tracks = ref([]);
const isLoading = ref(true);

// 使用 composable 加载封面
const { coverUrl, reload: reloadCover } = useCoverImage(album, 'large');

// 按专辑 trackIds 顺序排序的歌曲列表
const sortedTracks = computed(() => {
  if (!album.value?.trackIds?.length || !tracks.value.length) {
    return tracks.value;
  }
  // 根据 album.trackIds 的顺序排序
  const trackOrderMap = new Map(album.value.trackIds.map((id, index) => [id, index]));
  return [...tracks.value].sort((a, b) => {
    const orderA = trackOrderMap.get(a.id) ?? Infinity;
    const orderB = trackOrderMap.get(b.id) ?? Infinity;
    return orderA - orderB;
  });
});

onMounted(async () => {
  const albumId = route.params.albumId;
  if (!albumId) {
    router.push('/albums');
    return;
  }

  try {
    album.value = await getAlbum(albumId);
    // 专辑数据加载后，封面会自动加载

    // 获取专辑中的歌曲列表
    if (album.value?.trackIds?.length > 0) {
      tracks.value = await getTracksByIds(album.value.trackIds);
    }
  } catch (error) {
    console.error('Failed to load album:', error);
  } finally {
    isLoading.value = false;
  }
});

// 监听专辑变化，重新加载封面
watch(() => album.value?.id, (newId) => {
  if (newId) {
    reloadCover();
  }
});

const getCoverUrl = () => {
  // 优先使用从 ResourceManager 加载的封面
  return coverUrl.value || album.value?.coverData || '';
};

const formatYear = (year) => {
  return year || '未知年份';
};

const formatDuration = (seconds) => {
  if (!seconds) return '0:00';
  const hours = Math.floor(seconds / 3600);
  const mins = Math.floor((seconds % 3600) / 60);
  const secs = seconds % 60;
  
  if (hours > 0) {
    return `${hours}:${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
  }
  return `${mins}:${secs.toString().padStart(2, '0')}`;
};

const handleTrackSelect = (track) => {
  router.push(`/track/${track.id}`);
};

const handleTrackPlay = (track) => {
  console.log('Play track:', track);
};
</script>

<template>
  <div class="album-detail-page">
    <div v-if="isLoading" class="loading-state">
      <div class="spinner"></div>
    </div>

    <template v-else-if="album">
      <div class="album-hero">
        <div class="album-cover-wrapper">
          <img 
            v-if="getCoverUrl()" 
            :src="getCoverUrl()" 
            class="album-cover"
            :alt="album.title"
          />
          <div v-else class="album-cover-placeholder">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <circle cx="12" cy="12" r="10"/>
              <circle cx="12" cy="12" r="3"/>
            </svg>
          </div>
        </div>

        <div class="album-info">
          <span class="album-label">专辑</span>
          <h1 class="album-title">{{ album.title }}</h1>
          <div class="album-meta">
            <router-link 
              v-if="album.artistId" 
              :to="`/artists/${album.artistId}`"
              class="artist-link"
            >
              {{ album.artistName }}
            </router-link>
            <span v-else>{{ album.artistName || '未知歌手' }}</span>
            <span class="separator">·</span>
            <span>{{ formatYear(album.year) }}</span>
            <span class="separator">·</span>
            <span>{{ album.trackIds?.length || 0 }} 首歌曲</span>
            <span class="separator">·</span>
            <span>{{ formatDuration(album.totalDuration) }}</span>
          </div>

          <div class="album-actions">
            <button class="btn btn-primary btn-play">
              <svg viewBox="0 0 24 24" fill="currentColor">
                <path d="M8 5v14l11-7z"/>
              </svg>
              播放全部
            </button>
            <button class="btn btn-secondary">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z"/>
              </svg>
              收藏
            </button>
          </div>
        </div>
      </div>

      <!-- 歌曲列表 -->
      <section class="section" v-if="sortedTracks.length > 0">
        <TrackList
          :tracks="sortedTracks"
          :show-album="false"
          @select="handleTrackSelect"
          @play="handleTrackPlay"
        />
      </section>
    </template>

    <div v-else class="empty-state">
      <svg class="empty-state-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <circle cx="12" cy="12" r="10"/>
        <line x1="12" y1="8" x2="12" y2="12"/>
        <line x1="12" y1="16" x2="12.01" y2="16"/>
      </svg>
      <h3 class="empty-state-title">专辑不存在</h3>
      <p class="empty-state-desc">找不到该专辑的信息</p>
      <router-link to="/albums" class="btn btn-primary" style="margin-top: 16px;">
        返回专辑列表
      </router-link>
    </div>
  </div>
</template>

<style scoped>
.album-detail-page {
  max-width: 1200px;
  margin: 0 auto;
}

.album-hero {
  display: flex;
  gap: 32px;
  margin-bottom: 40px;
}

.album-cover-wrapper {
  flex-shrink: 0;
  width: 240px;
  height: 240px;
  border-radius: var(--radius-lg, 12px);
  overflow: hidden;
  box-shadow: var(--shadow-md, 0 4px 6px rgba(0, 0, 0, 0.1));
}

.album-cover {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.album-cover-placeholder {
  width: 100%;
  height: 100%;
  background: var(--bg-tertiary, #e8e8ed);
  display: flex;
  align-items: center;
  justify-content: center;
}

.album-cover-placeholder svg {
  width: 80px;
  height: 80px;
  color: var(--text-tertiary, #999);
}

.album-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: center;
}

.album-label {
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 1px;
  color: var(--text-secondary, #666);
  margin-bottom: 8px;
}

.album-title {
  font-size: 32px;
  font-weight: 700;
  color: var(--text-primary, #333);
  margin: 0 0 16px 0;
  line-height: 1.2;
}

.album-meta {
  font-size: 14px;
  color: var(--text-secondary, #666);
  margin-bottom: 24px;
}

.album-meta .separator {
  margin: 0 8px;
}

.artist-link {
  color: var(--primary-color, #0078d7);
  font-weight: 500;
}

.artist-link:hover {
  text-decoration: underline;
}

.album-actions {
  display: flex;
  gap: 12px;
}

.btn-play {
  padding: 12px 32px;
}

.btn-play svg {
  width: 20px;
  height: 20px;
}

.section {
  margin-bottom: 40px;
}

@media (max-width: 767px) {
  .album-hero {
    flex-direction: column;
    align-items: center;
    text-align: center;
  }

  .album-cover-wrapper {
    width: 200px;
    height: 200px;
  }

  .album-title {
    font-size: 24px;
  }

  .album-actions {
    justify-content: center;
  }
}

@media (prefers-color-scheme: dark) {
  .album-title {
    color: var(--text-primary, #f6f6f6);
  }
}
</style>
