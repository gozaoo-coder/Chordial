<script setup>
import { computed, ref } from 'vue';
import { useRouter } from 'vue-router';
import { useCoverImages } from '@/composables/useCoverImage';
import { useVirtualList } from '@/composables/useVirtualList';
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
  },
  // 是否启用虚拟列表
  virtualScroll: {
    type: Boolean,
    default: true
  },
  // 每项高度（用于虚拟列表）
  itemHeight: {
    type: Number,
    default: 64
  }
});

const router = useRouter();
const containerRef = ref(null);

const emit = defineEmits(['play', 'select']);

// 使用 composable 批量加载封面
const tracksRef = computed(() => props.tracks);
const { coverUrls } = useCoverImages(tracksRef, 'small');

// 虚拟列表
const {
  visibleItems,
  totalHeight,
  onScroll,
  poolSize
} = useVirtualList(tracksRef, {
  itemHeight: props.itemHeight,
  bufferSize: 5,
  containerRef
});

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

    <!-- 虚拟列表容器 -->
    <div
      v-if="virtualScroll && tracks.length > 50"
      ref="containerRef"
      class="track-list-body virtual-scroll"
      @scroll="onScroll"
    >
      <!-- 可视区域 -->
      <div class="viewport" :style="{ height: totalHeight + 'px' }">
        <div
          v-for="{ item: track, index, offsetY } in visibleItems"
          :key="track.id"
          class="track-item virtual-item"
          :style="{ transform: `translateY(${offsetY}px)` }"
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

      <!-- 性能指示器（开发环境显示） -->
      <div v-if="false" class="perf-indicator">
        DOM: {{ poolSize }} / {{ tracks.length }}
      </div>
    </div>

    <!-- 普通列表（数据量小或禁用虚拟列表时） -->
    <div v-else class="track-list-body">
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
  background: var(--bg-secondary);
  border-radius: var(--radius-lg);
  overflow: hidden;
  border: 1px solid var(--border-light);
  box-shadow: var(--shadow-card);
}

.track-list-header {
  display: grid;
  grid-template-columns: 50px 1fr 120px 120px 60px;
  gap: 12px;
  padding: 14px 20px;
  border-bottom: 1px solid var(--border-light);
  font-size: 11px;
  font-weight: 600;
  color: var(--text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.8px;
}

.track-list-header .col-artist,
.track-list-header .col-album {
  display: none;
}

.track-list-body {
  display: flex;
  flex-direction: column;
}

/* 虚拟列表样式 */
.track-list-body.virtual-scroll {
  position: relative;
  overflow-y: auto;
  overflow-x: hidden;
  max-height: 600px;
  -webkit-overflow-scrolling: touch;
}

.viewport {
  position: relative;
  width: 100%;
}

.track-item {
  display: grid;
  grid-template-columns: 50px 1fr 120px 120px 60px;
  gap: 12px;
  padding: 14px 20px;
  align-items: center;
  cursor: pointer;
  transition: background var(--transition-fast);
}

/* 虚拟列表项使用绝对定位 */
.track-item.virtual-item {
  position: absolute;
  left: 0;
  right: 0;
  height: v-bind(itemHeight + 'px');
  contain: layout style paint;
}

.track-item:hover {
  background: var(--bg-hover);
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
  font-weight: 500;
  color: var(--text-tertiary);
}

.play-btn {
  display: none;
  width: 32px;
  height: 32px;
  border: none;
  border-radius: 50%;
  background: var(--primary-color);
  color: white;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  position: absolute;
  transition: transform var(--transition-fast), box-shadow var(--transition-fast);
  box-shadow: var(--shadow-sm);
}

.play-btn:hover {
  transform: scale(1.1);
  box-shadow: var(--shadow-primary);
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
  width: 44px;
  height: 44px;
  flex-shrink: 0;
  border-radius: var(--radius-sm);
  overflow: hidden;
  box-shadow: var(--shadow-sm);
}

.track-info {
  min-width: 0;
}

.track-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  letter-spacing: -0.2px;
}

.track-artist-mobile {
  font-size: 12px;
  color: var(--text-secondary);
  margin-top: 3px;
  font-weight: 500;
}

.col-artist,
.col-album {
  font-size: 14px;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  font-weight: 500;
}

.artist-link,
.album-link {
  color: inherit;
  transition: color var(--transition-fast);
}

.artist-link:hover,
.album-link:hover {
  color: var(--primary-color);
  text-decoration: none;
}

.col-duration {
  font-size: 14px;
  color: var(--text-tertiary);
  text-align: right;
  font-weight: 500;
  font-variant-numeric: tabular-nums;
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
    background: var(--bg-secondary);
    border-color: var(--border-light);
  }

  .track-name {
    color: var(--text-primary);
  }
}
</style>
