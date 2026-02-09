<script setup>
import { ref, onMounted, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import AlbumList from '../components/common/AlbumList.vue';
import TrackList from '../components/common/TrackList.vue';
import { getArtist } from '../api/artist';
import { getAlbumsByIds } from '../api/album';
import { getTracksByIds } from '../api/musicSource/musicResource';
import { useCoverImage } from '@/composables/useCoverImage';

const route = useRoute();
const router = useRouter();

const artist = ref(null);
const isLoading = ref(true);

// 使用 composable 加载封面
const { coverUrl, reload: reloadCover } = useCoverImage(artist, 'large');

onMounted(async () => {
  const artistId = route.params.artistId;
  if (!artistId) {
    router.push('/artists');
    return;
  }

  try {
    artist.value = await getArtist(artistId);

    // 加载歌手的专辑和歌曲详情
    if (artist.value) {
      const [albums, tracks] = await Promise.all([
        artist.value.albumIds?.length > 0 ? getAlbumsByIds(artist.value.albumIds) : Promise.resolve([]),
        artist.value.trackIds?.length > 0 ? getTracksByIds(artist.value.trackIds) : Promise.resolve([])
      ]);
      artist.value.albums = albums;
      artist.value.tracks = tracks;
    }
  } catch (error) {
    console.error('Failed to load artist:', error);
  } finally {
    isLoading.value = false;
  }
});

// 监听歌手变化，重新加载封面
watch(() => artist.value?.id, (newId) => {
  if (newId) {
    reloadCover();
  }
});

const getCoverUrl = () => {
  // 优先使用从 ResourceManager 加载的封面
  return coverUrl.value || artist.value?.coverData || '';
};

const handleTrackSelect = (track) => {
  router.push(`/track/${track.id}`);
};

const handleTrackPlay = (track) => {
  console.log('Play track:', track);
};
</script>

<template>
  <div class="artist-detail-page">
    <div v-if="isLoading" class="loading-state">
      <div class="spinner"></div>
    </div>

    <template v-else-if="artist">
      <div class="artist-hero">
        <div class="artist-cover-wrapper">
          <img 
            v-if="getCoverUrl()" 
            :src="getCoverUrl()" 
            class="artist-cover"
            :alt="artist.name"
          />
          <div v-else class="artist-cover-placeholder">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/>
              <circle cx="12" cy="7" r="4"/>
            </svg>
          </div>
        </div>

        <div class="artist-info">
          <span class="artist-label">歌手</span>
          <h1 class="artist-name">{{ artist.name }}</h1>
          <div class="artist-meta">
            <span>{{ artist.trackIds?.length || 0 }} 首歌曲</span>
            <span class="separator">·</span>
            <span>{{ artist.albumIds?.length || 0 }} 张专辑</span>
          </div>
          <p v-if="artist.bio" class="artist-bio">{{ artist.bio }}</p>

          <div class="artist-actions">
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

      <!-- 专辑列表 -->
      <section class="section" v-if="artist.albums && artist.albums.length > 0">
        <div class="section-header">
          <h2 class="section-title">专辑</h2>
        </div>
        <AlbumList :albums="artist.albums" />
      </section>

      <!-- 歌曲列表 -->
      <section class="section" v-if="artist.tracks && artist.tracks.length > 0">
        <div class="section-header">
          <h2 class="section-title">热门歌曲</h2>
        </div>
        <TrackList 
          :tracks="artist.tracks.slice(0, 10)" 
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
      <h3 class="empty-state-title">歌手不存在</h3>
      <p class="empty-state-desc">找不到该歌手的信息</p>
      <router-link to="/artists" class="btn btn-primary" style="margin-top: 16px;">
        返回歌手列表
      </router-link>
    </div>
  </div>
</template>

<style scoped>
.artist-detail-page {
  max-width: 1200px;
  margin: 0 auto;
}

.artist-hero {
  display: flex;
  gap: 32px;
  margin-bottom: 40px;
}

.artist-cover-wrapper {
  flex-shrink: 0;
  width: 240px;
  height: 240px;
  border-radius: 50%;
  overflow: hidden;
  box-shadow: var(--shadow-md, 0 4px 6px rgba(0, 0, 0, 0.1));
}

.artist-cover {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.artist-cover-placeholder {
  width: 100%;
  height: 100%;
  background: var(--bg-tertiary, #e8e8ed);
  display: flex;
  align-items: center;
  justify-content: center;
}

.artist-cover-placeholder svg {
  width: 80px;
  height: 80px;
  color: var(--text-tertiary, #999);
}

.artist-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: center;
}

.artist-label {
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 1px;
  color: var(--text-secondary, #666);
  margin-bottom: 8px;
}

.artist-name {
  font-size: 32px;
  font-weight: 700;
  color: var(--text-primary, #333);
  margin: 0 0 16px 0;
  line-height: 1.2;
}

.artist-meta {
  font-size: 14px;
  color: var(--text-secondary, #666);
  margin-bottom: 16px;
}

.artist-meta .separator {
  margin: 0 8px;
}

.artist-bio {
  font-size: 14px;
  color: var(--text-secondary, #666);
  line-height: 1.6;
  margin-bottom: 24px;
  max-width: 600px;
}

.artist-actions {
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

.section-header {
  margin-bottom: 20px;
}

.section-title {
  font-size: 20px;
  font-weight: 700;
  color: var(--text-primary, #333);
  margin: 0;
}

@media (max-width: 767px) {
  .artist-hero {
    flex-direction: column;
    align-items: center;
    text-align: center;
  }

  .artist-cover-wrapper {
    width: 180px;
    height: 180px;
  }

  .artist-name {
    font-size: 24px;
  }

  .artist-actions {
    justify-content: center;
  }

  .section-title {
    font-size: 18px;
  }
}

@media (prefers-color-scheme: dark) {
  .artist-name,
  .section-title {
    color: var(--text-primary, #f6f6f6);
  }
}
</style>
