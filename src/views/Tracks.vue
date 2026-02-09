<script setup>
import { ref, onMounted, onActivated } from 'vue';
import { useRouter } from 'vue-router';
import TrackList from '../components/common/TrackList.vue';
import { library } from '../api/musicSource';

const router = useRouter();

const tracks = ref([]);
const isLoading = ref(true);

const loadTracks = async () => {
  isLoading.value = true;
  try {
    const data = await library.getCached();
    if (data) {
      tracks.value = data.tracks;
    }
  } catch (error) {
    console.error('Failed to load tracks:', error);
  } finally {
    isLoading.value = false;
  }
};

// 页面挂载时获取数据
onMounted(() => {
  loadTracks();
});

// 页面重新激活时刷新数据（从其他页面返回）
onActivated(() => {
  loadTracks();
});

const handleTrackSelect = (track) => {
  router.push(`/track/${track.id}`);
};

const handleTrackPlay = (track) => {
  console.log('Play track:', track);
};
</script>

<template>
  <div class="tracks-page">
    <div class="page-header">
      <h1 class="page-title">歌曲</h1>
      <p class="page-subtitle">共 {{ tracks.length }} 首歌曲</p>
    </div>

    <div v-if="isLoading" class="loading-state">
      <div class="spinner"></div>
    </div>

    <template v-else>
      <TrackList 
        v-if="tracks.length > 0"
        :tracks="tracks" 
        @select="handleTrackSelect"
        @play="handleTrackPlay"
      />

      <div v-else class="empty-state">
        <svg class="empty-state-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M9 18V5l12-2v13"/>
          <circle cx="6" cy="18" r="3"/>
          <circle cx="18" cy="16" r="3"/>
        </svg>
        <h3 class="empty-state-title">暂无歌曲</h3>
        <p class="empty-state-desc">添加音乐源开始扫描歌曲</p>
        <router-link to="/music-sources" class="btn btn-primary" style="margin-top: 16px;">
          添加音乐源
        </router-link>
      </div>
    </template>
  </div>
</template>

<style scoped>
.tracks-page {
  max-width: 1200px;
  margin: 0 auto;
}
</style>
