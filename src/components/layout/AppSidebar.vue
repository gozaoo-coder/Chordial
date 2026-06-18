<script setup>
import { computed } from 'vue';
import { useRoute } from 'vue-router';

const route = useRoute();

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
  <aside class="app-sidebar">
    <nav class="sidebar-nav">
      <router-link
        v-for="item in navItems"
        :key="item.path"
        :to="item.path"
        class="nav-item"
        :class="{ active: isActive(item.path) }"
      >
        <i :class="item.icon" class="nav-icon"></i>
        <span class="nav-text">{{ item.name }}</span>
      </router-link>
    </nav>
    
    <div class="sidebar-footer">
      <router-link to="/about" class="nav-item" :class="{ active: isActive('/about') }">
        <i class="bi bi-info-circle nav-icon"></i>
        <span class="nav-text">关于</span>
      </router-link>
    </div>
  </aside>
</template>

<style scoped>
.app-sidebar {
  width: var(--sidebar-width);
  height: 100%;
  display: flex;
  flex-direction: column;
  margin: 28px 16px;
  left: 2rem;  border-radius: calc(var(--bottom-nav-height) * 0.5);
  background: var(--bg-glass);
  border: 1px solid var(--border-light);

  height: fit-content;
}

.sidebar-nav {
  padding: 20px 16px;
  flex: 1;
  display: flex;
  flex-direction: column;

}

.nav-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 16px;
  color: var(--text-secondary);
  text-decoration: none;
  transition: all var(--transition-fast);
  cursor: pointer;
  font-weight: 500;
}

.nav-item:hover {
  /* background: var(--bg-hover); */
  color: var(--text-primary);
}

.nav-item.active {
  /* background: var(--primary-color); */
  color: var(--primary-color);
  text-shadow: var(--primary-color)  0 0 3em;
  font-weight: 600;
}

.nav-icon {
  font-size: 18px;
  flex-shrink: 0;
}

.nav-text {
  font-size: 14px;
}

.sidebar-footer {
  margin-top: auto;
  padding-top: 16px;
  border-top: 1px solid var(--border-light);
}

@media (prefers-color-scheme: dark) {
  .nav-item:hover {
    background: var(--bg-hover);
  }

  .nav-item.active {
    background: var(--primary-color);
  }
}
</style>
