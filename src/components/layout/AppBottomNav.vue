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
    <router-link v-for="item in navItems" :key="item.path" :to="item.path" class="bottom-nav-item"
      :class="{ active: isActive(item.path) }">
      <i :class="item.icon" class="bottom-nav-icon"></i>
      <span class="bottom-nav-text">{{ item.name }}</span>
    </router-link>
  </nav>
</template>

<style scoped>
.app-bottom-nav {
  position: fixed;
  bottom: 1.5rem;
  left: 2rem;
  border-radius: calc(var(--bottom-nav-height) * 0.5);
  background: var(--bg-glass);
  border: 1px solid var(--border-light);
  /* right: 0; */
  height: var(--bottom-nav-height);

  display: flex;
  justify-content: space-around;
  align-items: center;
  z-index: 100;
  padding: 0 16px;
  padding-bottom: var(--safe-area-bottom);
  transition: bottom var(--transition-slow);
  backdrop-filter: saturate(180%) blur(20px);
  -webkit-backdrop-filter: saturate(180%) blur(20px);
}


@media (max-width: 767px) {
  .app-bottom-nav {
    /* 当播放器显示时，底部导航向上移动 */
    right: 1rem;
    left: 1rem;
    /* bottom: calc(var(--bottom-nav-height) + 0.5rem); */
    /* border-radius: 1rem; */
    /* padding: 0.4rem 0.75rem; */

    bottom: var(--player-bar-height, 100px);
  }
}

.bottom-nav-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 0 18px;
  color: var(--text-secondary);
  text-decoration: none;
  transition: all var(--transition-fast);
  border-radius: var(--radius-md);
  flex: 1;
  font-weight: 500;
  min-width: 0;
}

.bottom-nav-item:hover {
  color: var(--text-primary);
}

.bottom-nav-item.active {
  color: var(--primary-color);
  text-shadow: var(--primary-color) 0 0 2em;
  /* background: var(--primary-light); */
}

.bottom-nav-icon {
  font-size: 22px;
}

.bottom-nav-text {
  font-size: 11px;
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
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
