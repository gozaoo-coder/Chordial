<script setup>
import { computed } from 'vue';
import { useRouter } from 'vue-router';
import { useCoverImages } from '@/composables/useCoverImage';
import CoverImage from './CoverImage.vue';
import PlayerStore from '@/stores/player.js';

const props = defineProps({
  tracks: {
    type: Array,
    default: () => []
  },
  showAlbum: {
    type: Boolean,
    default: true
  },
  showArtist: {
    type: Boolean,
    default: true
  },
  showDuration: {
    type: Boolean,
    default: true
  }
});

const router = useRouter();

const emit = defineEmits(['play', 'select']);

// 使用 composable 批量加载封面
const tracksRef = computed(() => props.tracks);
const { coverUrls } = useCoverImages(tracksRef, 'small');

const handleTrackClick = (track) => {
  emit('select', track);
  router.push(`/track/${track.id}`);
};

const handlePlayClick = (track, event) => {
  event.stopPropagation();
  emit('play', track);
  // 使用 PlayerStore 播放
  PlayerStore.play(track, props.tracks);
};

const handleTrackDoubleClick = (track) => {
  // 双击播放
  PlayerStore.play(track, props.tracks);
};

const formatDuration = (seconds) => {
  if (!seconds) return '0:00';
  const mins = Math.floor(seconds / 60);
  const secs = seconds % 60;
  return `${mins}:${secs.toString().padStart(2, '0')}`;
};

const getCoverUrl = (track) => {
  // 优先使用从 ResourceManager 加载的封面
  return coverUrls.value.get(track.id) || track.albumCoverData || track.album?.coverData || '';
};
</script>

<template>
  <div class="track-list">
    <div class="track-list-header">
      <div class="col-number">#</div>
      <div class="col-title">标题</div>
      <div v-if="showArtist" class="col-artist hide-mobile">歌手</div>
      <div v-if="showAlbum" class="col-album hide-mobile">专辑</div>
      <div v-if="showDuration" class="col-duration">时长</div>
    </div>
    
    <div class="track-list-body">
      <div
        v-for="(track, index) in tracks"
        :key="track.id"
        class="track-item"
        @click="handleTrackClick(track)"
        @dblclick="handleTrackDoubleClick(track)"
      >
        <div class="col-number">
          <span class="track-number">{{ index + 1 }}</span>
          <button class="play-btn" @click="handlePlayClick(track, $event)">
            <svg viewBox="0 0 24 24" fill="currentColor">
              <path d="M8 5v14l11-7z"/>
            </svg>
          </button>
        </div>
        
        <div class="col-title">
          <div class="track-cover-wrapper">
            <CoverImage
              :src="getCoverUrl(track)"
              :alt="track.title"
              type="track"
            />
          </div>
          <div class="track-info">
            <div class="track-name">{{ track.title || track.fileName || '未知歌曲' }}</div>
            <div v-if="!showArtist" class="track-artist-mobile">
              {{ track.getDisplayArtist ? track.getDisplayArtist() : (track.artist || '未知歌手') }}
            </div>
          </div>
        </div>
        
        <div v-if="showArtist" class="col-artist hide-mobile">
          <router-link 
            v-if="track.primaryArtist" 
            :to="`/artists/${track.primaryArtist.id}`"
            class="artist-link"
            @click.stop
          >
            {{ track.primaryArtist.name }}
          </router-link>
          <span v-else>{{ track.getDisplayArtist ? track.getDisplayArtist() : (track.artist || '未知歌手') }}</span>
        </div>
        
        <div v-if="showAlbum" class="col-album hide-mobile">
          <router-link 
            v-if="track.album" 
            :to="`/albums/${track.album.id}`"
            class="album-link"
            @click.stop
          >
            {{ track.album.title }}
          </router-link>
          <span v-else>未知专辑</span>
        </div>
        
        <div v-if="showDuration" class="col-duration">
          {{ track.getFormattedDuration ? track.getFormattedDuration() : formatDuration(track.duration) }}
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.track-list {
  background: var(--bg-secondary, #ffffff);
  border-radius: var(--radius-lg, 12px);
  overflow: hidden;
}

.track-list-header {
  display: grid;
  grid-template-columns: 50px 1fr 120px 120px 60px;
  gap: 12px;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border-color, rgba(0, 0, 0, 0.1));
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary, #666);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.track-list-header .col-artist,
.track-list-header .col-album {
  display: none;
}

.track-list-body {
  display: flex;
  flex-direction: column;
}

.track-item {
  display: grid;
  grid-template-columns: 50px 1fr 120px 120px 60px;
  gap: 12px;
  padding: 12px 16px;
  align-items: center;
  cursor: pointer;
  transition: background 0.2s ease;
}

.track-item:hover {
  background: var(--hover-bg, rgba(0, 0, 0, 0.02));
}

.track-item:hover .track-number {
  display: none;
}

.track-item:hover .play-btn {
  display: flex;
}

.col-number {
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
}

.track-number {
  font-size: 14px;
  color: var(--text-secondary, #666);
}

.play-btn {
  display: none;
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 50%;
  background: var(--primary-color, #667eea);
  color: white;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  position: absolute;
}

.play-btn svg {
  width: 16px;
  height: 16px;
}

.col-title {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
}

.track-cover-wrapper {
  width: 40px;
  height: 40px;
  flex-shrink: 0;
}

.track-info {
  min-width: 0;
}

.track-name {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary, #333);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.track-artist-mobile {
  font-size: 12px;
  color: var(--text-secondary, #666);
  margin-top: 2px;
}

.col-artist,
.col-album {
  font-size: 14px;
  color: var(--text-secondary, #666);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.artist-link,
.album-link {
  color: inherit;
}

.artist-link:hover,
.album-link:hover {
  color: var(--primary-color, #667eea);
  text-decoration: underline;
}

.col-duration {
  font-size: 14px;
  color: var(--text-secondary, #666);
  text-align: right;
}

@media (min-width: 768px) {
  .track-list-header .col-artist,
  .track-list-header .col-album {
    display: block;
  }
}

@media (max-width: 767px) {
  .track-list-header {
    grid-template-columns: 40px 1fr 60px;
  }
  
  .track-item {
    grid-template-columns: 40px 1fr 60px;
  }
  
  .track-cover-wrapper {
    width: 36px;
    height: 36px;
  }
}

@media (prefers-color-scheme: dark) {
  .track-list {
    background: var(--bg-secondary, #1a1a1a);
  }
  
  .track-name {
    color: var(--text-primary, #f6f6f6);
  }
}
</style>
