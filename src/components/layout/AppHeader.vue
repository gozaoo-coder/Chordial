<script setup>
import { ref, computed } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import WindowControls from '@/components/WindowControls.vue';

const route = useRoute();
const router = useRouter();

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
  <header class="app-header">
    <!-- 窗口拖动区域 -->
    <div class="drag-region"></div>

    <div class="header-left">
      <h1 class="app-title">Chordial</h1>
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
      <!-- 窗口控制按钮 -->
      <WindowControls />
    </div>
  </header>
</template>

<style scoped>
.app-header {
  height: var(--header-height);
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-light);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px 0 24px;
  position: sticky;
  top: 0;
  z-index: 100;
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
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
  font-size: 20px;
  font-weight: 700;
  color: #0078d7;
  margin: 0;
  -webkit-app-region: no-drag;
}

.page-title {
  font-size: 14px;
  color: var(--text-secondary);
  padding-left: 16px;
  border-left: 1px solid var(--border-light);
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
  height: 36px;
  padding: 0 12px 0 36px;
  border: 1px solid var(--border-light);
  border-radius: 18px;
  background: rgba(15, 15, 15, 0.05);
  color: var(--text-primary);
  font-size: 13px;
  transition: all 0.2s ease;
  -webkit-app-region: no-drag;
}

.search-box input:focus {
  outline: none;
  border-color: #0078d7;
  background: var(--bg-secondary);
  box-shadow: 0 0 0 3px rgba(0, 120, 215, 0.1);
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
  background: rgba(15, 15, 15, 0.05);
  color: var(--text-secondary);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.2s ease;
  -webkit-app-region: no-drag;
}

.icon-btn:hover {
  background: rgba(15, 15, 15, 0.08);
  color: var(--text-primary);
}

.icon-btn:active {
  transform: scale(0.95);
}

.icon-btn i {
  font-size: 16px;
}

@media (max-width: 768px) {
  .app-header {
    padding: 0 12px;
  }

  .drag-region {
    right: 100px;
  }

  .app-title {
    font-size: 18px;
  }

  .page-title {
    display: none;
  }

  .header-center {
    margin: 0 12px;
    max-width: 200px;
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
    padding: 0 8px;
  }

  .drag-region {
    right: 90px;
  }

  .header-center {
    margin: 0 8px;
    max-width: 120px;
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
    color: #4da3ff;
  }

  .search-box input {
    background: rgba(255, 255, 255, 0.1);
    border-color: var(--border-light);
  }

  .search-box input:focus {
    background: var(--bg-secondary);
    border-color: #4da3ff;
    box-shadow: 0 0 0 3px rgba(77, 163, 255, 0.15);
  }

  .icon-btn {
    background: rgba(255, 255, 255, 0.1);
    color: var(--text-secondary);
  }

  .icon-btn:hover {
    background: rgba(255, 255, 255, 0.15);
    color: var(--text-primary);
  }
}
</style>
