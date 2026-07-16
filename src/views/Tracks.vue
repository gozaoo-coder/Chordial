<script setup>
import { ref, shallowRef, onMounted, watch, nextTick, useTemplateRef } from 'vue';
import { useRouter } from 'vue-router';
import TrackList from '../components/common/TrackList.vue';
import { library } from '../api/musicSource';
import { usePerf } from '@/utils/performanceMonitor.js';
import { useAnime } from '@/composables/useAnime.js';

const { start, end } = usePerf('Tracks');

const router = useRouter();

const PAGE_SIZE = 100;
// shallowRef：列表数据为业务类实例，避免深代理开销
const tracks = shallowRef([]);
const totalCount = ref(0);
const isLoading = ref(true);
const isLoadingMore = ref(false);
const hasMore = ref(true);

const rootRef = useTemplateRef('root');
const { run } = useAnime(() => rootRef.value);

const loadTracks = async () => {
  isLoading.value = true;
  start('loadTracks');
  try {
    const data = await library.getSongsPage(0, PAGE_SIZE);
    if (data) {
      tracks.value = data.songs;
      totalCount.value = data.total;
      hasMore.value = data.songs.length < data.total;
    }
    end('loadTracks', { count: tracks.value.length, total: totalCount.value });
  } catch (error) {
    console.error('Failed to load tracks:', error);
    end('loadTracks', { error: error.message });
  } finally {
    isLoading.value = false;
  }
};

const loadMore = async () => {
  if (isLoadingMore.value || !hasMore.value) return;
  isLoadingMore.value = true;
  try {
    const data = await library.getSongsPage(tracks.value.length, PAGE_SIZE);
    if (data) {
      tracks.value = [...tracks.value, ...data.songs];
      hasMore.value = tracks.value.length < data.total;
    }
  } catch (error) {
    console.error('Failed to load more tracks:', error);
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
  if (tracks.value.length === 0) {
    loadTracks();
  } else {
    isLoading.value = false;
  }
});

// loading 状态切换：启动 spinner（列表入场由 TrackList 组件内部 playEnter 处理）
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

const handleTrackSelect = (track) => {
  router.push(`/track/${track.id}`);
};

const handleTrackPlay = (track) => {
  console.log('Play track:', track);
};
</script>

<template>
  <div class="tracks-page" ref="root">
    <div class="page-header">
      <!-- <h1 class="page-title">歌曲</h1> -->
      <p class="page-subtitle">共 {{ totalCount || tracks.length }} 首歌曲</p>
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

      <div v-if="hasMore" class="load-more">
        <button class="btn btn-secondary" @click="loadMore" :disabled="isLoadingMore">
          <span v-if="isLoadingMore" class="spinner-small"></span>
          {{ isLoadingMore ? '加载中...' : `加载更多 (${tracks.length}/${totalCount})` }}
        </button>
      </div>

      <div v-else-if="tracks.length === 0" class="empty-state">
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
