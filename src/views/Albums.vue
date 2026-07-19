<script setup>
import { ref, shallowRef, onMounted, watch, nextTick, useTemplateRef } from 'vue';
import AlbumList from '../components/common/AlbumList.vue';
import { library } from '../api/musicSource';
import { usePerf } from '@/utils/performanceMonitor.js';
import { useAnime } from '@/composables/useAnime.js';
import { useLibraryEvents } from '@/composables/useLibraryEvents.js';

const { start, end } = usePerf('Albums');

const PAGE_SIZE = 50;
// shallowRef：列表数据为业务类实例，避免深代理开销
const albums = shallowRef([]);
const totalCount = ref(0);
const isLoading = ref(true);
const isLoadingMore = ref(false);
const hasMore = ref(true);

const rootRef = useTemplateRef('root');
const { run } = useAnime(() => rootRef.value);

// 订阅全局库变更事件 — 移除/添加音乐源后自动刷新列表
const { libraryVersion } = useLibraryEvents();

const loadAlbums = async () => {
  isLoading.value = true;
  start('loadAlbums');
  try {
    const data = await library.getAlbumsPage(0, PAGE_SIZE);
    if (data) {
      albums.value = data.albums;
      totalCount.value = data.total;
      hasMore.value = data.albums.length < data.total;
    }
    end('loadAlbums', { count: albums.value.length, total: totalCount.value });
  } catch (error) {
    console.error('Failed to load albums:', error);
    end('loadAlbums', { error: error.message });
  } finally {
    isLoading.value = false;
  }
};

const loadMore = async () => {
  if (isLoadingMore.value || !hasMore.value) return;
  isLoadingMore.value = true;
  try {
    const data = await library.getAlbumsPage(albums.value.length, PAGE_SIZE);
    if (data) {
      albums.value = [...albums.value, ...data.albums];
      hasMore.value = albums.value.length < data.total;
    }
  } catch (error) {
    console.error('Failed to load more albums:', error);
  } finally {
    isLoadingMore.value = false;
  }
};

// --- 动画（anime.js v4） ---
// loading spinner：用 ANIME_LOOP.spin 替代 CSS @keyframes spin
function playLoadingSpinner() {
  run(({ animate, loopPresets }) => {
    animate('.loading-state .spinner', { ...loopPresets.spin });
  });
}

// 加载更多按钮内的小 spinner
function playLoadMoreSpinner() {
  run(({ animate, loopPresets }) => {
    animate('.spinner-small', { ...loopPresets.spin });
  });
}

// 页面挂载时获取数据（已加载则跳过）
onMounted(() => {
  if (isLoading.value) {
    nextTick(playLoadingSpinner);
  }
  if (albums.value.length === 0) {
    loadAlbums();
  } else {
    isLoading.value = false;
  }
});

// 监听库变更事件：后端 emit "library-changed" 时 libraryVersion 自增，触发刷新。
// 跳过初始值 0，避免与 onMounted 的首次加载重复。
watch(libraryVersion, (v) => {
  if (v === 0) return;
  loadAlbums();
});

// loading 状态切换：启动 spinner（列表入场由 AlbumList 组件内部 playEnter 处理）
watch(
  isLoading,
  (val) => {
    if (val) {
      nextTick(playLoadingSpinner);
    }
  },
  { flush: 'post' }
);

// 加载更多 spinner
watch(
  isLoadingMore,
  (val) => {
    if (val) {
      nextTick(playLoadMoreSpinner);
    }
  },
  { flush: 'post' }
);
</script>

<template>
  <div class="albums-page" ref="root">
    <div class="page-header">
      <h1 class="page-title">专辑</h1>
      <p class="page-subtitle">共 {{ totalCount || albums.length }} 张专辑</p>
    </div>

    <div v-if="isLoading" class="loading-state">
      <div class="spinner"></div>
    </div>

    <template v-else>
      <AlbumList v-if="albums.length > 0" :albums="albums" />

      <div v-if="hasMore" class="load-more">
        <button class="btn btn-secondary" @click="loadMore" :disabled="isLoadingMore">
          <span v-if="isLoadingMore" class="spinner-small"></span>
          {{ isLoadingMore ? '加载中...' : `加载更多 (${albums.length}/${totalCount})` }}
        </button>
      </div>

      <div v-else-if="albums.length === 0" class="empty-state">
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

.load-more {
  display: flex;
  justify-content: center;
  padding: 24px 0;
}

/* 禁用全局 .spinner 的 CSS spin 动画，改由 anime.js ANIME_LOOP.spin 驱动 */
.loading-state .spinner {
  animation: none;
}

.spinner-small {
  display: inline-block;
  width: 16px;
  height: 16px;
  border: 2px solid var(--border-light, rgba(0,0,0,0.1));
  border-top-color: var(--primary-color, #0078d7);
  border-radius: 50%;
  margin-right: 8px;
  vertical-align: middle;
}

@media (prefers-color-scheme: dark) {
  .spinner-small {
    border-color: rgba(255,255,255,0.15);
    border-top-color: var(--primary-color, #0A84FF);
  }
}
</style>
