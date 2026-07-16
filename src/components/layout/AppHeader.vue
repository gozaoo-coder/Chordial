<script setup>
import { ref, computed, onMounted, useTemplateRef } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import WindowControls from '@/components/WindowControls.vue';
import { usePlatform } from '@/composables/usePlatform.js';
import { useAnime } from '@/composables/useAnime.js';

const route = useRoute();
const router = useRouter();

const { isDesktop } = usePlatform();

const rootRef = useTemplateRef('root');
const { run } = useAnime(() => rootRef.value);

// header 三段区块错峰入场（fadeInDown + stagger）
onMounted(() => {
  run(({ animate, stagger, presets }) => {
    animate('.header-left, .header-center, .header-right', {
      ...presets.fadeInDown,
      delay: stagger(60),
      onComplete: () => {
        // 清除内联 transform，恢复 CSS 过渡
        rootRef.value?.querySelectorAll('.header-left, .header-center, .header-right').forEach((el) => {
          el.style.transform = '';
        });
      },
    });
  });
});

const pageTitle = computed(() => {
  return route.meta?.title || 'Chordial';
});

const searchQuery = ref('');

const emit = defineEmits(['search']);

const handleSearch = () => {
  emit('search', searchQuery.value);
};

const goToSettings = () => {
  router.push('/settings');
};
</script>

<template>
  <header ref="root" class="app-header">
    <!-- 窗口拖动区域 - 仅桌面端 -->
    <div v-if="isDesktop" class="drag-region"></div>

    <div class="header-left">
      <!-- <h1 class="app-title">Chordial</h1> -->
      <span class="page-title">{{ pageTitle }}</span>
    </div>

    <div class="header-center">
      <div class="search-box">
        <i class="bi bi-search search-icon"></i>
        <input
          v-model="searchQuery"
          type="text"
          placeholder="搜索歌曲、歌手、专辑..."
          @keyup.enter="handleSearch"
        />
      </div>
    </div>

    <div class="header-right">
      <button class="icon-btn settings-btn" title="设置" @click="goToSettings">
        <i class="bi bi-gear"></i>
      </button>
      <button class="icon-btn notification-btn" title="通知">
        <i class="bi bi-bell"></i>
      </button>
      <!-- 窗口控制按钮 - 仅桌面端 -->
      <WindowControls v-if="isDesktop" />
    </div>
  </header>
</template>

<style scoped>
.app-header {
  height: var(--header-height);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 20px 0 24px;
  padding-top: var(--safe-area-top);
  position: sticky;
  top: 0;
  z-index: 100;
  backdrop-filter: saturate(180%) blur(16px);
  -webkit-backdrop-filter: saturate(180%) blur(16px);
  background: var(--bg-glass);
  border-bottom: 1px solid var(--border-light);
  /* GPU: 限制重绘范围，减少合成器开销 */
  contain: layout paint;
  will-change: backdrop-filter;
}

/* 窗口拖动区域 */
.drag-region {
  position: absolute;
  top: 0;
  left: 0;
  right: 220px;
  height: var(--header-height);
  -webkit-app-region: drag;
  z-index: 0;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 16px;
  position: relative;
  z-index: 1;
}

.app-title {
  font-size: 22px;
  font-weight: 700;
  background: var(--primary-gradient);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  margin: 0;
  -webkit-app-region: no-drag;
  letter-spacing: -0.5px;
}

.page-title {
  font-size: 26px;
  font-weight: 900;
}

.header-center {
  flex: 1;
  max-width: 400px;
  margin: 0 24px;
  position: relative;
  z-index: 1;
}

.search-box {
  position: relative;
  display: flex;
  align-items: center;

}

.search-icon {
  position: absolute;
  left: 12px;
  font-size: 14px;
  color: var(--text-tertiary);
  pointer-events: none;
}

.search-box input {
  width: 100%;
  height: 38px;
  padding: 0 16px 0 40px;
  border: 1px solid var(--border-light);
  border-radius: 19px;
  background: var(--bg-glass);
  color: var(--text-primary);
  font-size: 14px;
  font-weight: 400;
  transition: all var(--transition-normal);
  -webkit-app-region: no-drag;
  backdrop-filter: saturate(180%) blur(12px);
  -webkit-backdrop-filter: saturate(180%) blur(12px);
  /* GPU: 输入框 backdrop 独立合成层 */
  will-change: backdrop-filter;
  contain: paint;
}

.search-box input:focus {
  outline: none;
  border-color: var(--primary-color);
  background: var(--bg-secondary);
  box-shadow: 0 0 0 4px var(--primary-light);
}

.search-box input::placeholder {
  color: var(--text-tertiary);
}

.header-right {
  display: flex;
  align-items: center;
  gap: 8px;
  position: relative;
  z-index: 1;
}

.icon-btn {
  width: 36px;
  height: 36px;
  border: none;
  border-radius: 50%;
  background: var(--bg-glass);
  color: var(--text-secondary);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  backdrop-filter: saturate(180%) blur(12px);
  -webkit-backdrop-filter: saturate(180%) blur(12px);
  border: 1px solid var(--border-light);
  transition: all var(--transition-fast);
  -webkit-app-region: no-drag;
  /* GPU: 小图标 backdrop 最小化模糊半径 */
  will-change: backdrop-filter;
  contain: paint;
}

.icon-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.icon-btn:active {
  transform: scale(0.92);
  background: var(--bg-active);
}

.icon-btn i {
  font-size: 16px;
}

@media (max-width: 768px) {
  .app-header {
    padding: 0 0;
    padding-top: var(--safe-area-top);
  }

  .app-title {
    font-size: 18px;
  }

  .page-title {
    display: none;
  }

  .header-center {
  margin: 0 20px;
    max-width: none;
  }

  .search-box input {
    font-size: 13px;
    padding: 0 10px 0 32px;
  }

  .search-icon {
    left: 10px;
  }

  .icon-btn {
    width: 32px;
    height: 32px;
  }

  .icon-btn i {
    font-size: 14px;
  }

  /* 隐藏通知按钮，保留设置按钮 */
  .notification-btn {
    display: none;
  }
}

@media (max-width: 480px) {
  .app-header {
    padding: 0 0;
    padding-top: var(--safe-area-top);
  }

  .header-center {
    /* margin: 0 8px; */
  margin: 0 20px;
    max-width: none;
    /* max-width: 120px; */
  }

  .search-box input::placeholder {
    font-size: 11px;
  }

  .search-box input {
    height: 32px;
    font-size: 12px;
  }

  .app-title {
    font-size: 16px;
  }
}

@media (prefers-color-scheme: dark) {
  .app-title {
    background: var(--primary-gradient);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }

  .search-box input {
    background: var(--bg-glass);
    border-color: var(--border-light);
  }

  .search-box input:focus {
    background: var(--bg-secondary);
    border-color: var(--primary-color);
    box-shadow: 0 0 0 4px var(--primary-light);
  }

  .icon-btn {
    background: var(--bg-glass);
    color: var(--text-secondary);
  }

  .icon-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }
}
</style>
