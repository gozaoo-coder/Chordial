<script setup>
import { ref, onMounted, onActivated, computed } from 'vue';
import { useRouter } from 'vue-router';
import TrackList from '../components/common/TrackList.vue';
import ArtistList from '../components/common/ArtistList.vue';
import AlbumList from '../components/common/AlbumList.vue';
import CoverImage from '../components/common/CoverImage.vue';
import AlbumCollageBackground from '../components/common/AlbumCollageBackground.vue';
import { library } from '../api/musicSource';
import PlayerStore from '@/stores/player.js';

const router = useRouter();

const recentTracks = ref([]);
const featuredArtists = ref([]);
const recentAlbums = ref([]);
const stats = ref({
  artists: 0,
  albums: 0,
  tracks: 0
});
const isLoading = ref(true);

// 获取当前时间问候语
const greeting = computed(() => {
  const hour = new Date().getHours();
  if (hour < 6) return '夜深了';
  if (hour < 12) return '早上好';
  if (hour < 18) return '下午好';
  return '晚上好';
});

// 获取今日推荐歌曲（随机选择）
const todayPicks = computed(() => {
  if (recentTracks.value.length === 0) return [];
  const shuffled = [...recentTracks.value].sort(() => 0.5 - Math.random());
  return shuffled.slice(0, 5);
});

// 获取热门专辑（随机选择）
const featuredAlbums = computed(() => {
  if (recentAlbums.value.length === 0) return [];
  const shuffled = [...recentAlbums.value].sort(() => 0.5 - Math.random());
  return shuffled.slice(0, 4);
});

const loadHomeData = async () => {
  isLoading.value = true;
  try {
    const data = await library.getCached();
    if (data) {
      recentTracks.value = data.tracks.slice(0, 10);
      featuredArtists.value = data.artists.slice(0, 6);
      recentAlbums.value = data.albums.slice(0, 8);
      stats.value = {
        artists: data.artists.length,
        albums: data.albums.length,
        tracks: data.tracks.length
      };
    }
  } catch (error) {
    console.error('Failed to load home data:', error);
  } finally {
    isLoading.value = false;
  }
};

onMounted(() => {
  loadHomeData();
});

onActivated(() => {
  loadHomeData();
});

const handleTrackSelect = (track) => {
  router.push(`/track/${track.id}`);
};

const handleTrackPlay = (track) => {
  PlayerStore.play(track, recentTracks.value);
};

const playAllTracks = () => {
  if (recentTracks.value.length > 0) {
    PlayerStore.play(recentTracks.value[0], recentTracks.value);
  }
};

const shufflePlay = () => {
  if (recentTracks.value.length > 0) {
    const shuffled = [...recentTracks.value].sort(() => 0.5 - Math.random());
    PlayerStore.play(shuffled[0], shuffled);
  }
};

const getCoverUrl = (track) => {
  if (track.coverData) return track.coverData;
  if (track.albumCoverData) return track.albumCoverData;
  if (track.album?.coverData) return track.album.coverData;
  return '';
};

const getAlbumCoverUrl = (album) => {
  return album.coverData || '';
};
</script>

<template>
  <div class="home-page">
    <!-- Hero Banner -->
    <section class="hero-banner" v-if="todayPicks.length > 0">
      <!-- 多专辑封面拼接背景 -->
      <AlbumCollageBackground 
        :albums="recentAlbums" 
        :max-display="9"
        :animation-enabled="true"
        class="hero-collage-bg"
      />
      <div class="hero-content">
        <div class="hero-badge">
          <i class="bi bi-stars"></i>
          <span>今日推荐</span>
        </div>
        <h1 class="hero-title">{{ greeting }}，音乐爱好者</h1>
        <p class="hero-subtitle">探索你的音乐世界，发现无限可能</p>
        <div class="hero-actions">
          <button class="hero-btn primary" @click="playAllTracks">
            <i class="bi bi-play-fill"></i>
            <span>播放全部</span>
          </button>
          <button class="hero-btn secondary" @click="shufflePlay">
            <i class="bi bi-shuffle"></i>
            <span>随机播放</span>
          </button>
        </div>
      </div>
      <div class="hero-stats">
        <div class="hero-stat">
          <span class="hero-stat-value">{{ stats.tracks }}</span>
          <span class="hero-stat-label">首歌曲</span>
        </div>
        <div class="hero-stat-divider"></div>
        <div class="hero-stat">
          <span class="hero-stat-value">{{ stats.albums }}</span>
          <span class="hero-stat-label">张专辑</span>
        </div>
        <div class="hero-stat-divider"></div>
        <div class="hero-stat">
          <span class="hero-stat-value">{{ stats.artists }}</span>
          <span class="hero-stat-label">位歌手</span>
        </div>
      </div>
    </section>

    <div v-if="isLoading" class="loading-state">
      <div class="spinner"></div>
      <span>加载中...</span>
    </div>

    <template v-else>
      <!-- 为你推荐 - 横向滚动 -->
      <section class="section" v-if="todayPicks.length > 0">
        <div class="section-header">
          <h2 class="section-title">为你推荐</h2>
          <router-link to="/tracks" class="section-link">
            查看全部
            <i class="bi bi-arrow-right"></i>
          </router-link>
        </div>
        <div class="horizontal-scroll">
          <div 
            v-for="track in todayPicks" 
            :key="track.id"
            class="track-card"
            @click="handleTrackSelect(track)"
          >
            <div class="track-card-cover">
              <CoverImage
                :src="getCoverUrl(track)"
                :alt="track.title"
                type="track"
              />
              <button class="track-card-play" @click.stop="handleTrackPlay(track)">
                <i class="bi bi-play-fill"></i>
              </button>
            </div>
            <div class="track-card-info">
              <h3 class="track-card-title">{{ track.title || track.fileName || '未知歌曲' }}</h3>
              <p class="track-card-artist">{{ track.getDisplayArtist ? track.getDisplayArtist() : (track.artist || '未知歌手') }}</p>
            </div>
          </div>
        </div>
      </section>

      <!-- 热门专辑 - 网格布局 -->
      <section class="section" v-if="featuredAlbums.length > 0">
        <div class="section-header">
          <h2 class="section-title">热门专辑</h2>
          <router-link to="/albums" class="section-link">
            查看全部
            <i class="bi bi-arrow-right"></i>
          </router-link>
        </div>
        <div class="albums-grid">
          <div 
            v-for="album in featuredAlbums" 
            :key="album.id"
            class="album-card"
            @click="router.push(`/album/${album.id}`)"
          >
            <div class="album-cover-wrapper">
              <CoverImage
                :src="getAlbumCoverUrl(album)"
                :alt="album.title"
                type="album"
              />
              <div class="album-overlay">
                <button class="album-play-btn">
                  <i class="bi bi-play-fill"></i>
                </button>
              </div>
            </div>
            <div class="album-info">
              <h3 class="album-title">{{ album.title || '未知专辑' }}</h3>
              <p class="album-artist">{{ album.artist || '未知歌手' }}</p>
            </div>
          </div>
        </div>
      </section>

      <!-- 最近添加 - 列表 -->
      <section class="section section-compact" v-if="recentTracks.length > 0">
        <div class="section-header">
          <h2 class="section-title">最近添加</h2>
          <router-link to="/tracks" class="section-link">
            查看全部
            <i class="bi bi-arrow-right"></i>
          </router-link>
        </div>
        <TrackList 
          :tracks="recentTracks.slice(0, 5)" 
          @select="handleTrackSelect"
          @play="handleTrackPlay"
        />
      </section>

      <!-- 推荐歌手 -->
      <section class="section" v-if="featuredArtists.length > 0">
        <div class="section-header">
          <h2 class="section-title">推荐歌手</h2>
          <router-link to="/artists" class="section-link">
            查看全部
            <i class="bi bi-arrow-right"></i>
          </router-link>
        </div>
        <ArtistList :artists="featuredArtists.slice(0, 5)" />
      </section>

      <!-- 空状态 -->
      <div v-if="recentTracks.length === 0 && featuredArtists.length === 0 && recentAlbums.length === 0" class="empty-state">
        <div class="empty-icon">
          <i class="bi bi-music-note-beamed"></i>
        </div>
        <h3 class="empty-title">开始你的音乐之旅</h3>
        <p class="empty-desc">添加音乐源，发现属于你的音乐世界</p>
        <router-link to="/music-sources" class="empty-action">
          <i class="bi bi-plus-lg"></i>
          添加音乐源
        </router-link>
      </div>
    </template>
  </div>
</template>

<style scoped>
.home-page {
  max-width: 1200px;
  margin: 0 auto;
}

/* Hero Banner - 沉浸式大背景 */
.hero-banner {
  position: relative;
  margin-bottom: 48px;
  padding: 48px;
  border-radius: var(--radius-xl);
  overflow: hidden;
  min-height: 320px;
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  background: linear-gradient(135deg, #1a1a2e 0%, #16213e 50%, #0f0f23 100%);
}

/* 多专辑封面背景 */
.hero-collage-bg {
  position: absolute;
  inset: 0;
  z-index: 0;
}

.hero-content {
  position: relative;
  z-index: 1;
  max-width: 500px;
}

.hero-badge {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  background: rgba(255, 255, 255, 0.15);
  backdrop-filter: blur(10px);
  border-radius: 20px;
  color: white;
  font-size: 13px;
  font-weight: 600;
  margin-bottom: 16px;
}

.hero-badge i {
  color: #ffd700;
}

.hero-title {
  font-size: 42px;
  font-weight: 800;
  color: white;
  margin: 0 0 12px 0;
  letter-spacing: -1px;
  line-height: 1.1;
  text-shadow: 0 2px 20px rgba(0, 0, 0, 0.3);
}

.hero-subtitle {
  font-size: 16px;
  color: rgba(255, 255, 255, 0.8);
  margin: 0 0 24px 0;
  font-weight: 400;
}

.hero-actions {
  display: flex;
  gap: 12px;
}

.hero-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 14px 28px;
  border-radius: 100px;
  font-size: 15px;
  font-weight: 600;
  cursor: pointer;
  transition: all var(--transition-fast);
  border: none;
}

.hero-btn.primary {
  background: white;
  color: var(--text-primary);
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.2);
}

.hero-btn.primary:hover {
  transform: scale(1.05);
  box-shadow: 0 6px 30px rgba(0, 0, 0, 0.3);
}

.hero-btn.secondary {
  background: rgba(255, 255, 255, 0.15);
  backdrop-filter: blur(10px);
  color: white;
}

.hero-btn.secondary:hover {
  background: rgba(255, 255, 255, 0.25);
}

.hero-btn i {
  font-size: 18px;
}

.hero-stats {
  position: relative;
  z-index: 1;
  display: flex;
  align-items: center;
  gap: 20px;
  padding: 20px 24px;
  background: rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(20px);
  border-radius: var(--radius-lg);
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.hero-stat {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}

.hero-stat-value {
  font-size: 28px;
  font-weight: 800;
  color: white;
  line-height: 1;
  letter-spacing: -1px;
}

.hero-stat-label {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.7);
  font-weight: 500;
}

.hero-stat-divider {
  width: 1px;
  height: 40px;
  background: rgba(255, 255, 255, 0.2);
}

/* 区块样式 */
.section {
  margin-bottom: 48px;
}

.section-compact {
  margin-bottom: 40px;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 20px;
}

.section-title {
  font-size: 24px;
  font-weight: 700;
  color: var(--text-primary);
  margin: 0;
  letter-spacing: -0.5px;
}

.section-link {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 14px;
  font-weight: 600;
  color: var(--primary-color);
  text-decoration: none;
  padding: 8px 16px;
  border-radius: 100px;
  transition: all var(--transition-fast);
  background: transparent;
}

.section-link:hover {
  background: var(--primary-light);
}

.section-link i {
  font-size: 14px;
}

/* 横向滚动 - 为你推荐 */
.horizontal-scroll {
  display: flex;
  gap: 20px;
  overflow-x: auto;
  overflow-y: visible;
  padding-bottom: 16px;
  margin-bottom: -8px;
  scrollbar-width: none;
  -ms-overflow-style: none;
}

.horizontal-scroll::-webkit-scrollbar {
  display: none;
}

.track-card {
  flex: 0 0 180px;
  cursor: pointer;
  transition: transform var(--transition-spring);
}

.track-card:hover {
  transform: translateY(-8px);
}

.track-card:hover .track-card-play {
  opacity: 1;
  transform: scale(1);
}

.track-card-cover {
  position: relative;
  aspect-ratio: 1;
  border-radius: var(--radius-lg);
  overflow: hidden;
  margin-bottom: 12px;
  box-shadow: var(--shadow-sm);
  transition: box-shadow var(--transition-normal);
}

.track-card:hover .track-card-cover {
  box-shadow: var(--shadow-md);
}

.track-card-cover :deep(.cover-wrapper) {
  width: 100%;
  height: 100%;
}

.track-card-play {
  position: absolute;
  bottom: 12px;
  right: 12px;
  width: 44px;
  height: 44px;
  border-radius: 50%;
  background: var(--primary-color);
  color: white;
  border: none;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  opacity: 0;
  transform: scale(0.8);
  transition: all var(--transition-fast);
  box-shadow: var(--shadow-primary);
}

.track-card-play:hover {
  transform: scale(1.1) !important;
}

.track-card-play i {
  font-size: 20px;
  margin-left: 2px;
}

.track-card-info {
  padding: 0 4px;
}

.track-card-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0 0 4px 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  letter-spacing: -0.2px;
}

.track-card-artist {
  font-size: 13px;
  color: var(--text-secondary);
  margin: 0;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* 专辑网格 */
.albums-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 24px;
}

.album-card {
  cursor: pointer;
  transition: transform var(--transition-spring);
}

.album-card:hover {
  transform: translateY(-8px);
}

.album-card:hover .album-overlay {
  opacity: 1;
}

.album-cover-wrapper {
  position: relative;
  aspect-ratio: 1;
  border-radius: var(--radius-lg);
  overflow: hidden;
  margin-bottom: 14px;
  box-shadow: var(--shadow-sm);
  transition: box-shadow var(--transition-normal);
}

.album-card:hover .album-cover-wrapper {
  box-shadow: var(--shadow-md);
}

.album-cover-wrapper :deep(.cover-wrapper) {
  width: 100%;
  height: 100%;
}

.album-overlay {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transition: opacity var(--transition-fast);
}

.album-play-btn {
  width: 56px;
  height: 56px;
  border-radius: 50%;
  background: var(--primary-color);
  color: white;
  border: none;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: transform var(--transition-fast);
  box-shadow: var(--shadow-primary);
}

.album-play-btn:hover {
  transform: scale(1.1);
}

.album-play-btn i {
  font-size: 24px;
  margin-left: 2px;
}

.album-info {
  padding: 0 4px;
}

.album-title {
  font-size: 15px;
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
  margin: 0;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* 加载状态 */
.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 100px 0;
  gap: 16px;
  color: var(--text-secondary);
}

.spinner {
  width: 44px;
  height: 44px;
  border: 3px solid var(--border-light);
  border-top-color: var(--primary-color);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* 空状态 */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 100px 40px;
  text-align: center;
}

.empty-icon {
  width: 80px;
  height: 80px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--primary-light);
  color: var(--primary-color);
  border-radius: 50%;
  font-size: 36px;
  margin-bottom: 24px;
}

.empty-title {
  font-size: 24px;
  font-weight: 700;
  color: var(--text-primary);
  margin: 0 0 8px 0;
}

.empty-desc {
  font-size: 15px;
  color: var(--text-secondary);
  margin: 0 0 28px 0;
}

.empty-action {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 14px 28px;
  background: var(--primary-color);
  color: white;
  border-radius: 100px;
  font-size: 15px;
  font-weight: 600;
  text-decoration: none;
  transition: all var(--transition-fast);
  box-shadow: var(--shadow-primary);
}

.empty-action:hover {
  background: var(--primary-hover);
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(0, 122, 255, 0.35);
}

/* 响应式 */
@media (max-width: 1024px) {
  .hero-banner {
    flex-direction: column;
    align-items: flex-start;
    gap: 32px;
    padding: 32px;
  }

  .hero-stats {
    width: 100%;
    justify-content: space-around;
  }

  .albums-grid {
    grid-template-columns: repeat(3, 1fr);
  }
}

@media (max-width: 767px) {
  .hero-banner {
    padding: 28px;
    min-height: 280px;
  }

  .hero-title {
    font-size: 32px;
  }

  .hero-subtitle {
    font-size: 14px;
  }

  .hero-btn {
    padding: 12px 20px;
    font-size: 14px;
  }

  .hero-stats {
    padding: 16px 20px;
    gap: 16px;
  }

  .hero-stat-value {
    font-size: 22px;
  }

  .section-title {
    font-size: 20px;
  }

  .track-card {
    flex: 0 0 150px;
  }

  .track-card-play {
    width: 36px;
    height: 36px;
    opacity: 1;
    transform: scale(1);
    bottom: 8px;
    right: 8px;
  }

  .track-card-play i {
    font-size: 16px;
  }

  .albums-grid {
    grid-template-columns: repeat(2, 1fr);
    gap: 16px;
  }

  .album-play-btn {
    width: 44px;
    height: 44px;
  }

  .album-play-btn i {
    font-size: 20px;
  }
}

@media (max-width: 480px) {
  .hero-actions {
    flex-wrap: wrap;
  }

  .hero-btn span {
    display: none;
  }

  .hero-btn i {
    margin: 0;
    font-size: 20px;
  }

  .albums-grid {
    grid-template-columns: repeat(2, 1fr);
    gap: 12px;
  }
}

@media (prefers-color-scheme: dark) {
  .hero-badge {
    background: rgba(255, 255, 255, 0.1);
  }

  .hero-btn.secondary {
    background: rgba(255, 255, 255, 0.1);
  }

  .hero-stats {
    background: rgba(255, 255, 255, 0.08);
  }
}
</style>
