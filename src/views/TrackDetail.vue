<script setup>
import { ref, onMounted, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { Track } from '../class';
import TrackList from '../components/common/TrackList.vue';
import { useCoverImage } from '@/composables/useCoverImage';
import PlayerStore from '@/stores/player.js';

const route = useRoute();
const router = useRouter();

const track = ref(null);
const albumTracks = ref([]);
const isLoading = ref(true);

// 使用 composable 加载封面
const { coverUrl, reload: reloadCover } = useCoverImage(track, 'large');

onMounted(async () => {
  const trackId = route.params.trackId;
  if (!trackId) {
    router.push('/tracks');
    return;
  }

  try {
    track.value = await Track.getById(trackId);
    // 歌曲数据加载后，封面会自动加载
  } catch (error) {
    console.error('Failed to load track:', error);
  } finally {
    isLoading.value = false;
  }
});

// 监听歌曲变化，重新加载封面
watch(() => track.value?.id, (newId) => {
  if (newId) {
    reloadCover();
  }
});

const handlePlay = () => {
  if (track.value) {
    PlayerStore.play(track.value);
  }
};

const getCoverUrl = () => {
  // 优先使用从 ResourceManager 加载的封面
  return coverUrl.value || track.value?.getCoverUrl?.() || track.value?.albumCoverData || '';
};

const formatDuration = (seconds) => {
  if (!seconds) return '0:00';
  const mins = Math.floor(seconds / 60);
  const secs = seconds % 60;
  return `${mins}:${secs.toString().padStart(2, '0')}`;
};
</script>

<template>
  <div class="track-detail-page">
    <div v-if="isLoading" class="loading-state">
      <div class="spinner"></div>
    </div>

    <template v-else-if="track">
      <div class="track-hero">
        <div class="track-cover-wrapper">
          <img 
            v-if="getCoverUrl()" 
            :src="getCoverUrl()" 
            class="track-cover"
            :alt="track.title"
          />
          <div v-else class="track-cover-placeholder">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M9 18V5l12-2v13"/>
              <circle cx="6" cy="18" r="3"/>
              <circle cx="18" cy="16" r="3"/>
            </svg>
          </div>
        </div>

        <div class="track-info">
          <span class="track-label">歌曲</span>
          <h1 class="track-title">{{ track.title || track.fileName || '未知歌曲' }}</h1>
          <div class="track-meta">
            <router-link 
              v-if="track.primaryArtist" 
              :to="`/artists/${track.primaryArtist.id}`"
              class="artist-link"
            >
              {{ track.primaryArtist.name }}
            </router-link>
            <span v-else>{{ track.getDisplayArtist ? track.getDisplayArtist() : (track.artist || '未知歌手') }}</span>
            <span class="separator">·</span>
            <router-link 
              v-if="track.album" 
              :to="`/albums/${track.album.id}`"
              class="album-link"
            >
              {{ track.album.title }}
            </router-link>
            <span v-else>未知专辑</span>
            <span class="separator">·</span>
            <span>{{ track.getFormattedDuration ? track.getFormattedDuration() : formatDuration(track.duration) }}</span>
          </div>

          <div class="track-actions">
            <button class="btn btn-primary btn-play" @click="handlePlay">
              <svg viewBox="0 0 24 24" fill="currentColor">
                <path d="M8 5v14l11-7z"/>
              </svg>
              播放
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

      <div class="track-details card">
        <h3 class="details-title">歌曲信息</h3>
        <div class="details-grid">
          <div class="detail-item">
            <span class="detail-label">格式</span>
            <span class="detail-value">{{ track.format?.toUpperCase() || '未知' }}</span>
          </div>
          <div class="detail-item">
            <span class="detail-label">比特率</span>
            <span class="detail-value">{{ track.getFormattedBitrate ? track.getFormattedBitrate() : (track.bitrate ? `${track.bitrate} kbps` : '未知') }}</span>
          </div>
          <div class="detail-item">
            <span class="detail-label">采样率</span>
            <span class="detail-value">{{ track.getFormattedSampleRate ? track.getFormattedSampleRate() : (track.sampleRate ? `${(track.sampleRate / 1000).toFixed(1)} kHz` : '未知') }}</span>
          </div>
          <div class="detail-item">
            <span class="detail-label">声道</span>
            <span class="detail-value">{{ track.channels ? `${track.channels} 声道` : '未知' }}</span>
          </div>
          <div class="detail-item">
            <span class="detail-label">文件大小</span>
            <span class="detail-value">{{ track.getFormattedFileSize ? track.getFormattedFileSize() : (track.fileSize ? `${(track.fileSize / 1024 / 1024).toFixed(2)} MB` : '未知') }}</span>
          </div>
          <div class="detail-item">
            <span class="detail-label">文件路径</span>
            <span class="detail-value text-truncate">{{ track.path }}</span>
          </div>
        </div>
      </div>
    </template>

    <div v-else class="empty-state">
      <svg class="empty-state-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <circle cx="12" cy="12" r="10"/>
        <line x1="12" y1="8" x2="12" y2="12"/>
        <line x1="12" y1="16" x2="12.01" y2="16"/>
      </svg>
      <h3 class="empty-state-title">歌曲不存在</h3>
      <p class="empty-state-desc">找不到该歌曲的信息</p>
      <router-link to="/tracks" class="btn btn-primary" style="margin-top: 16px;">
        返回歌曲列表
      </router-link>
    </div>
  </div>
</template>

<style scoped>
.track-detail-page {
  max-width: 1200px;
  margin: 0 auto;
}

.track-hero {
  display: flex;
  gap: 32px;
  margin-bottom: 32px;
}

.track-cover-wrapper {
  flex-shrink: 0;
  width: 240px;
  height: 240px;
  border-radius: var(--radius-lg, 12px);
  overflow: hidden;
  box-shadow: var(--shadow-md, 0 4px 6px rgba(0, 0, 0, 0.1));
}

.track-cover {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.track-cover-placeholder {
  width: 100%;
  height: 100%;
  background: var(--bg-tertiary, #e8e8ed);
  display: flex;
  align-items: center;
  justify-content: center;
}

.track-cover-placeholder svg {
  width: 80px;
  height: 80px;
  color: var(--text-tertiary, #999);
}

.track-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: center;
}

.track-label {
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 1px;
  color: var(--text-secondary, #666);
  margin-bottom: 8px;
}

.track-title {
  font-size: 32px;
  font-weight: 700;
  color: var(--text-primary, #333);
  margin: 0 0 16px 0;
  line-height: 1.2;
}

.track-meta {
  font-size: 14px;
  color: var(--text-secondary, #666);
  margin-bottom: 24px;
}

.track-meta .separator {
  margin: 0 8px;
}

.artist-link,
.album-link {
  color: var(--primary-color, #0078d7);
  font-weight: 500;
}

.artist-link:hover,
.album-link:hover {
  text-decoration: underline;
}

.track-actions {
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

.track-details {
  margin-top: 32px;
}

.details-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary, #333);
  margin: 0 0 20px 0;
}

.details-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 16px;
}

.detail-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.detail-label {
  font-size: 12px;
  color: var(--text-secondary, #666);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.detail-value {
  font-size: 14px;
  color: var(--text-primary, #333);
}

@media (max-width: 767px) {
  .track-hero {
    flex-direction: column;
    align-items: center;
    text-align: center;
  }

  .track-cover-wrapper {
    width: 200px;
    height: 200px;
  }

  .track-title {
    font-size: 24px;
  }

  .track-actions {
    justify-content: center;
  }

  .details-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}

@media (prefers-color-scheme: dark) {
  .track-title,
  .details-title,
  .detail-value {
    color: var(--text-primary, #f6f6f6);
  }
}
</style>
