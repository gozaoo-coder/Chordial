<template>
  <div class="player-album-card">
    <!-- 专辑封面 -->
    <div class="cover-container">
      <div class="cover-wrapper" :class="{ playing: isPlaying }">
        <img
          v-if="coverUrl"
          :src="coverUrl"
          :alt="title"
          class="album-cover"
        />
        <div v-else class="cover-placeholder">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M9 18V5l12-2v13"/>
            <circle cx="6" cy="18" r="3"/>
            <circle cx="18" cy="16" r="3"/>
          </svg>
        </div>
      </div>
      <!-- 更多选项按钮 -->
      <button class="more-btn" @click="onMore" v-if="showMore">
        <svg viewBox="0 0 24 24" fill="currentColor">
          <circle cx="12" cy="6" r="2"/>
          <circle cx="12" cy="12" r="2"/>
          <circle cx="12" cy="18" r="2"/>
        </svg>
      </button>
    </div>

    <!-- 歌曲信息 -->
    <div class="track-info">
      <h1 class="track-title" :title="title">{{ title }}</h1>
      <p class="track-artist" :title="artist">{{ artist }}</p>
      <p class="track-album" v-if="album" :title="album">{{ album }}</p>
    </div>
  </div>
</template>

<script>
export default {
  name: 'PlayerAlbumCard',
  props: {
    coverUrl: {
      type: String,
      default: ''
    },
    title: {
      type: String,
      default: '未知歌曲'
    },
    artist: {
      type: String,
      default: '未知歌手'
    },
    album: {
      type: String,
      default: ''
    },
    isPlaying: {
      type: Boolean,
      default: false
    },
    showMore: {
      type: Boolean,
      default: true
    }
  },
  emits: ['more'],
  setup(props, { emit }) {
    const onMore = () => emit('more');

    return {
      onMore
    };
  }
};
</script>

<style scoped>
.player-album-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2vh;
  position: relative;
}

.cover-container {
  position: relative;
}

.cover-wrapper {
  width: 280px;
  height: 280px;
  border-radius: 12px;
  overflow: hidden;
  box-shadow: 0 24px 80px rgba(0, 0, 0, 0.6);
  transition: transform 0.3s ease;
  will-change: transform;
}

.cover-wrapper.playing {
  animation: pulse 4s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% {
    transform: scale3d(1, 1, 1);
  }
  50% {
    transform: scale3d(1.02, 1.02, 1);
  }
}

.album-cover {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.cover-placeholder {
  width: 100%;
  height: 100%;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  display: flex;
  align-items: center;
  justify-content: center;
}

.cover-placeholder svg {
  width: 64px;
  height: 64px;
  opacity: 0.5;
  color: white;
}

.track-info {
  text-align: left;
  width: 100%;
  max-width: min(30vmin, 320px);
}

.track-title {
  font-size: 22px;
  font-weight: 700;
  margin: 0 0 8px 0;
  color: white;
  text-shadow: 0 2px 10px rgba(0, 0, 0, 0.3);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  letter-spacing: -0.3px;
}

.track-artist {
  font-size: 15px;
  opacity: 0.85;
  margin: 0 0 4px 0;
  color: white;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  font-weight: 500;
}

.track-album {
  font-size: 13px;
  opacity: 0.6;
  margin: 0;
  color: white;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.more-btn {
  position: absolute;
  top: 12px;
  right: 12px;
  width: 32px;
  height: 32px;
  border: none;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 50%;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  transition: background 0.2s;
  backdrop-filter: blur(10px);
}

.more-btn:hover {
  background: rgba(255, 255, 255, 0.2);
}

.more-btn svg {
  width: 16px;
  height: 16px;
}

@media (max-width: 1023px) {
  .cover-wrapper {
    width: 220px;
    height: 220px;
  }

  .track-title {
    font-size: 18px;
  }
}

@media (max-width: 767px) {
  .cover-wrapper {
    width: 180px;
    height: 180px;
  }

  .track-title {
    font-size: 16px;
  }

  .track-artist {
    font-size: 13px;
  }
}
</style>
