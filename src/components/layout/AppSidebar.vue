<script setup>
import { computed, onMounted, useTemplateRef, watch, nextTick } from 'vue';
import { useRoute } from 'vue-router';
import { useAnime } from '@/composables/useAnime.js';
import { ANIME_SPRINGS } from '@/utils/animePresets.js';

const route = useRoute();

const rootRef = useTemplateRef('root');
const indicatorRef = useTemplateRef('indicator');
const { run, animate } = useAnime(() => rootRef.value);

// 菜单项错峰入场（slideInLeft + stagger）
onMounted(() => {
  run(({ animate, stagger, presets }) => {
    animate('.nav-item', {
      ...presets.slideInLeft,
      delay: stagger(50),
      onComplete: () => {
        // 清除内联 transform，恢复 CSS :hover / .active 过渡
        rootRef.value?.querySelectorAll('.nav-item').forEach((el) => {
          el.style.transform = '';
        });
      },
    });
  });
  // 初始化 active 指示器
  nextTick(updateIndicator);
});

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

// active 指示器滑动动画（anime.js spring 物理，垂直滑动）
function updateIndicator() {
  const root = rootRef.value;
  const indicator = indicatorRef.value;
  if (!root || !indicator) return;
  const activeEl = root.querySelector('.nav-item.active');
  if (!activeEl) {
    animate(indicator, { opacity: 0, duration: 200, ease: ANIME_SPRINGS.sensitive });
    return;
  }
  const rootRect = root.getBoundingClientRect();
  const itemRect = activeEl.getBoundingClientRect();
  const y = itemRect.top - rootRect.top;
  const h = itemRect.height;
  animate(indicator, {
    translateY: y,
    height: h,
    opacity: 1,
    duration: 420,
    ease: ANIME_SPRINGS.bouncy,
  });
}

// 路由切换 → FLIP 指示器滑动
watch(() => route.path, () => {
  nextTick(updateIndicator);
});
</script>

<template>
  <aside ref="root" class="app-sidebar">
    <div ref="indicator" class="nav-indicator"></div>
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
  position: relative;
}

/* active 指示器（绝对定位背景，由 anime.js animate translateY/height 垂直滑动） */
.nav-indicator {
  position: absolute;
  top: 0;
  left: 16px;
  right: 16px;
  height: 0;
  background: var(--primary-light);
  border-radius: var(--radius-md);
  opacity: 0;
  z-index: 0;
  pointer-events: none;
  will-change: transform, height, opacity;
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
  transition: color var(--transition-fast);
  cursor: pointer;
  font-weight: 500;
  position: relative;
  z-index: 1;
}

.nav-item:hover {
  color: var(--text-primary);
}

.nav-item.active {
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
    color: var(--primary-color);
  }
}
</style>
