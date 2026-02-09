<script setup>
import { ref, onMounted, onActivated } from 'vue';
import ArtistList from '../components/common/ArtistList.vue';
import { library } from '../api/musicSource';

const artists = ref([]);
const isLoading = ref(true);

const loadArtists = async () => {
  isLoading.value = true;
  try {
    const data = await library.getCached();
    console.log('Cached data:', data);
    
    if (data) {
      artists.value = data.artists;
    }
  } catch (error) {
    console.error('Failed to load artists:', error);
  } finally {
    isLoading.value = false;
  }
};

// 页面挂载时获取数据
onMounted(() => {
  loadArtists();
});

// 页面重新激活时刷新数据（从其他页面返回）
onActivated(() => {
  loadArtists();
});
</script>

<template>
  <div class="artists-page">
    <div class="page-header">
      <h1 class="page-title">歌手</h1>
      <p class="page-subtitle">共 {{ artists.length }} 位歌手</p>
    </div>

    <div v-if="isLoading" class="loading-state">
      <div class="spinner"></div>
    </div>

    <template v-else>
      <ArtistList v-if="artists.length > 0" :artists="artists" />

      <div v-else class="empty-state">
        <svg class="empty-state-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/>
          <circle cx="12" cy="7" r="4"/>
        </svg>
        <h3 class="empty-state-title">暂无歌手</h3>
        <p class="empty-state-desc">添加音乐源开始扫描歌手信息</p>
        <router-link to="/music-sources" class="btn btn-primary" style="margin-top: 16px;">
          添加音乐源
        </router-link>
      </div>
    </template>
  </div>
</template>

<style scoped>
.artists-page {
  max-width: 1200px;
  margin: 0 auto;
}
</style>
