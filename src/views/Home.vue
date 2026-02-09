<script setup>
import { ref, onMounted, onActivated } from 'vue';
import { useRouter } from 'vue-router';
import TrackList from '../components/common/TrackList.vue';
import ArtistList from '../components/common/ArtistList.vue';
import AlbumList from '../components/common/AlbumList.vue';
import { library } from '../api/musicSource';

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

const loadHomeData = async () => {
  isLoading.value = true;
  try {
    const data = await library.getCached();
    if (data) {
      recentTracks.value = data.tracks.slice(0, 10);
      featuredArtists.value = data.artists.slice(0, 6);
      recentAlbums.value = data.albums.slice(0, 6);
      // 更新统计数据
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

// 页面挂载时获取数据
onMounted(() => {
  loadHomeData();
});

// 页面重新激活时刷新数据（从其他页面返回）
onActivated(() => {
  loadHomeData();
});

const handleTrackSelect = (track) => {
  router.push(`/track/${track.id}`);
};

const handleTrackPlay = (track) => {
  console.log('Play track:', track);
};
</script>

<template>
  <div class="home-page">
    <div class="page-header">
      <h1 class="page-title">欢迎回来</h1>
      <p class="page-subtitle">继续你的音乐之旅</p>
    </div>

    <!-- 统计卡片 -->
    <div v-if="!isLoading && stats.tracks > 0" class="stats-grid">
      <div class="stat-card">
        <span class="stat-number">{{ stats.artists }}</span>
        <span class="stat-label">歌手</span>
      </div>
      <div class="stat-card">
        <span class="stat-number">{{ stats.albums }}</span>
        <span class="stat-label">专辑</span>
      </div>
      <div class="stat-card">
        <span class="stat-number">{{ stats.tracks }}</span>
        <span class="stat-label">歌曲</span>
      </div>
    </div>

    <div v-if="isLoading" class="loading-state">
      <div class="spinner"></div>
    </div>

    <template v-else>
      <!-- 最近播放 -->
      <section class="section" v-if="recentTracks.length > 0">
        <div class="section-header">
          <h2 class="section-title">最近添加</h2>
          <router-link to="/tracks" class="section-link">查看全部</router-link>
        </div>
        <TrackList 
          :tracks="recentTracks" 
          @select="handleTrackSelect"
          @play="handleTrackPlay"
        />
      </section>

      <!-- 推荐歌手 -->
      <section class="section" v-if="featuredArtists.length > 0">
        <div class="section-header">
          <h2 class="section-title">推荐歌手</h2>
          <router-link to="/artists" class="section-link">查看全部</router-link>
        </div>
        <ArtistList :artists="featuredArtists" />
      </section>

      <!-- 推荐专辑 -->
      <section class="section" v-if="recentAlbums.length > 0">
        <div class="section-header">
          <h2 class="section-title">推荐专辑</h2>
          <router-link to="/albums" class="section-link">查看全部</router-link>
        </div>
        <AlbumList :albums="recentAlbums" />
      </section>

      <!-- 空状态 -->
      <div v-if="recentTracks.length === 0 && featuredArtists.length === 0 && recentAlbums.length === 0" class="empty-state">
        <svg class="empty-state-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M9 18V5l12-2v13"/>
          <circle cx="6" cy="18" r="3"/>
          <circle cx="18" cy="16" r="3"/>
        </svg>
        <h3 class="empty-state-title">音乐库为空</h3>
        <p class="empty-state-desc">添加音乐源开始管理你的音乐</p>
        <router-link to="/music-sources" class="btn btn-primary" style="margin-top: 16px;">
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

.section {
  margin-bottom: 40px;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 20px;
}

.section-title {
  font-size: 20px;
  font-weight: 700;
  color: var(--text-primary, #333);
  margin: 0;
}

.section-link {
  font-size: 14px;
  color: var(--primary-color, #0078d7);
  text-decoration: none;
}

.section-link:hover {
  text-decoration: underline;
}

@media (max-width: 767px) {
  .section {
    margin-bottom: 32px;
  }
  
  .section-title {
    font-size: 18px;
  }
}

@media (prefers-color-scheme: dark) {
  .section-title {
    color: var(--text-primary, #f6f6f6);
  }
}

/* 统计卡片样式 */
.stats-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
  margin-bottom: 40px;
}

.stat-card {
  background: var(--bg-secondary, #FBFBFB);
  border-radius: var(--radius-lg, 18px);
  padding: 24px;
  text-align: center;
  box-shadow: var(--shadow-sm, 0 1px 2px rgba(0, 0, 0, 0.05));
  transition: transform 0.2s ease, box-shadow 0.2s ease;
}

.stat-card:hover {
  transform: translateY(-2px);
  box-shadow: var(--shadow-md, 0 8px 32px rgba(0, 0, 0, 0.08));
}

.stat-number {
  display: block;
  font-size: 32px;
  font-weight: 700;
  color: var(--primary-color, #0078d7);
  margin-bottom: 4px;
}

.stat-label {
  font-size: 14px;
  color: var(--text-secondary, #666);
}

@media (max-width: 767px) {
  .stats-grid {
    grid-template-columns: repeat(3, 1fr);
    gap: 12px;
  }

  .stat-card {
    padding: 16px;
  }

  .stat-number {
    font-size: 24px;
  }

  .stat-label {
    font-size: 12px;
  }
}

@media (prefers-color-scheme: dark) {
  .stat-card {
    background: var(--bg-secondary, #1a1a1a);
  }

  .stat-label {
    color: var(--text-secondary, #999);
  }
}
</style>
