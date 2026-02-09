<script setup>
import { ref, onMounted, onActivated } from 'vue';
import AlbumList from '../components/common/AlbumList.vue';
import { library } from '../api/musicSource';

const albums = ref([]);
const isLoading = ref(true);

const loadAlbums = async () => {
  isLoading.value = true;
  try {
    const data = await library.getCached();
    if (data) {
      albums.value = data.albums;
    }
  } catch (error) {
    console.error('Failed to load albums:', error);
  } finally {
    isLoading.value = false;
  }
};

// 页面挂载时获取数据
onMounted(() => {
  loadAlbums();
});

// 页面重新激活时刷新数据（从其他页面返回）
onActivated(() => {
  loadAlbums();
});
</script>

<template>
  <div class="albums-page">
    <div class="page-header">
      <h1 class="page-title">专辑</h1>
      <p class="page-subtitle">共 {{ albums.length }} 张专辑</p>
    </div>

    <div v-if="isLoading" class="loading-state">
      <div class="spinner"></div>
    </div>

    <template v-else>
      <AlbumList v-if="albums.length > 0" :albums="albums" />

      <div v-else class="empty-state">
        <svg class="empty-state-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <circle cx="12" cy="12" r="10"/>
          <circle cx="12" cy="12" r="3"/>
        </svg>
        <h3 class="empty-state-title">暂无专辑</h3>
        <p class="empty-state-desc">添加音乐源开始扫描专辑信息</p>
        <router-link to="/music-sources" class="btn btn-primary" style="margin-top: 16px;">
          添加音乐源
        </router-link>
      </div>
    </template>
  </div>
</template>

<style scoped>
.albums-page {
  max-width: 1200px;
  margin: 0 auto;
}
</style>
