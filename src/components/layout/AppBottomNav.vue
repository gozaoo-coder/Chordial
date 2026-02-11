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
  background: var(--bg-glass);
  border-top: 1px solid var(--border-light);
  display: flex;
  justify-content: space-around;
  align-items: center;
  z-index: 100;
  padding: 0 16px;
  transition: bottom var(--transition-slow);
  backdrop-filter: saturate(180%) blur(20px);
  -webkit-backdrop-filter: saturate(180%) blur(20px);
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
  padding: 8px 16px;
  color: var(--text-secondary);
  text-decoration: none;
  transition: all var(--transition-fast);
  border-radius: var(--radius-md);
  flex: 1;
  max-width: 80px;
  font-weight: 500;
}

.bottom-nav-item:hover {
  color: var(--text-primary);
  background: var(--bg-hover);
}

.bottom-nav-item.active {
  color: var(--primary-color);
  background: var(--primary-light);
}

.bottom-nav-icon {
  font-size: 22px;
}

.bottom-nav-text {
  font-size: 11px;
  font-weight: 600;
}

@media (min-width: 1024px) {
  .app-bottom-nav {
    display: none;
  }
}

@media (prefers-color-scheme: dark) {
  .bottom-nav-item.active {
    color: var(--primary-color);
    background: var(--primary-light);
  }
}
</style>
