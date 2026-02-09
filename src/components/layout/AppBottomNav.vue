<script setup>
import { useRoute } from 'vue-router';
import { computed } from 'vue';
import PlayerStore from '@/stores/player.js';

const route = useRoute();

const hasCurrentTrack = computed(() => PlayerStore.hasCurrentTrack.value);

const navItems = [
  {
    path: '/',
    name: '首页',
    icon: 'bi-house'
  },
  {
    path: '/tracks',
    name: '歌曲',
    icon: 'bi-music-note-beamed'
  },
  {
    path: '/artists',
    name: '歌手',
    icon: 'bi-people'
  },
  {
    path: '/albums',
    name: '专辑',
    icon: 'bi-disc'
  },
  {
    path: '/music-sources',
    name: '音乐源',
    icon: 'bi-folder'
  }
];

const isActive = (path) => {
  if (path === '/') {
    return route.path === '/';
  }
  return route.path.startsWith(path);
};
</script>

<template>
  <nav class="app-bottom-nav" :class="{ 'with-player': hasCurrentTrack }">
    <router-link
      v-for="item in navItems"
      :key="item.path"
      :to="item.path"
      class="bottom-nav-item"
      :class="{ active: isActive(item.path) }"
    >
      <i :class="item.icon" class="bottom-nav-icon"></i>
      <span class="bottom-nav-text">{{ item.name }}</span>
    </router-link>
  </nav>
</template>

<style scoped>
.app-bottom-nav {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  height: var(--bottom-nav-height);
  background: var(--bg-secondary);
  border-top: 1px solid var(--border-light);
  display: flex;
  justify-content: space-around;
  align-items: center;
  z-index: 100;
  padding: 0 8px;
  transition: bottom 0.3s ease;
}

/* 当播放器显示时，底部导航向上移动 */
.app-bottom-nav.with-player {
  bottom: var(--player-bar-height, 100px);
}

.bottom-nav-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 4px;
  padding: 8px 12px;
  color: var(--text-secondary);
  text-decoration: none;
  transition: all 0.2s ease;
  border-radius: 8px;
  flex: 1;
  max-width: 80px;
}

.bottom-nav-item:hover {
  color: var(--text-primary);
}

.bottom-nav-item.active {
  color: #0078d7;
}

.bottom-nav-icon {
  font-size: 20px;
}

.bottom-nav-text {
  font-size: 11px;
  font-weight: 500;
}

@media (min-width: 1024px) {
  .app-bottom-nav {
    display: none;
  }
}

@media (prefers-color-scheme: dark) {
  .bottom-nav-item.active {
    color: #4da3ff;
  }
}
</style>
