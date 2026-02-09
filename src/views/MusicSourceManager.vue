<script setup>
import { ref, onMounted } from 'vue';
import { open } from '@tauri-apps/plugin-dialog';
import { library, addLocalFolder } from '../api/musicSource';

const sources = ref([]);
const isLoading = ref(true);
const isScanning = ref(false);
const isAdding = ref(false);

onMounted(async () => {
  await loadSources();
});

const loadSources = async () => {
  try {
    const data = await library.getCached();
    if (data) {
      sources.value = data.sources || [];
    }
  } catch (error) {
    console.error('Failed to load sources:', error);
  } finally {
    isLoading.value = false;
  }
};

const handleScanAll = async () => {
  isScanning.value = true;
  try {
    // 扫描所有源，后端会自动更新 library 缓存
    await library.scanAll();
    // 重新加载数据，获取最新的 artists、albums 统计
    await loadSources();
  } catch (error) {
    console.error('Failed to scan sources:', error);
  } finally {
    isScanning.value = false;
  }
};

const handleAddSource = async () => {
  console.log('handleAddSource clicked');
  try {
    // 打开文件夹选择对话框
    console.log('Opening dialog...');
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择音乐文件夹'
    });
    console.log('Dialog result:', selected);

    if (selected) {
      isAdding.value = true;
      console.log('Adding local folder:', selected);
      // 添加本地文件夹作为音乐源
      const result = await addLocalFolder(selected, true);
      console.log('Add source result:', result);
      // 刷新列表
      await loadSources();
    }
  } catch (error) {
    console.error('Failed to add source:', error);
    alert('添加音乐源失败: ' + error.message);
  } finally {
    isAdding.value = false;
  }
};

const handleEditSource = (source) => {
  console.log('Edit source:', source);
};

const handleDeleteSource = (source) => {
  console.log('Delete source:', source);
};

const formatDate = (dateString) => {
  if (!dateString) return '从未';
  const date = new Date(dateString);
  return date.toLocaleDateString('zh-CN');
};
</script>

<template>
  <div class="music-source-page">
    <div class="page-header">
      <div>
        <h1 class="page-title">音乐源管理</h1>
        <p class="page-subtitle">管理你的音乐文件夹和网盘</p>
      </div>
      <div class="page-actions">
        <button class="btn btn-secondary" @click="handleScanAll" :disabled="isScanning">
          <svg v-if="isScanning" class="spin" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 12a9 9 0 11-6.219-8.56"/>
          </svg>
          <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21.5 2v6h-6M2.5 22v-6h6M2 11.5a10 10 0 0118.8-4.3M22 12.5a10 10 0 01-18.8 4.3"/>
          </svg>
          {{ isScanning ? '扫描中...' : '扫描全部' }}
        </button>
        <button class="btn btn-primary" @click="handleAddSource" :disabled="isAdding">
          <svg v-if="isAdding" class="spin" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 12a9 9 0 11-6.219-8.56"/>
          </svg>
          <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="12" y1="5" x2="12" y2="19"/>
            <line x1="5" y1="12" x2="19" y2="12"/>
          </svg>
          {{ isAdding ? '添加中...' : '添加音乐源' }}
        </button>
      </div>
    </div>

    <div v-if="isLoading" class="loading-state">
      <div class="spinner"></div>
    </div>

    <template v-else>
      <div v-if="sources.length > 0" class="sources-list">
        <div v-for="source in sources" :key="source.id" class="source-card card">
          <div class="source-info">
            <div class="source-icon">
              <svg v-if="source.source_type === 'LocalFolder'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
              </svg>
              <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M18 10h-1.26A8 8 0 109 20h9a5 5 0 000-10z"/>
              </svg>
            </div>
            <div class="source-details">
              <h3 class="source-name">{{ source.name }}</h3>
              <p class="source-path">{{ source.path }}</p>
              <div class="source-meta">
                <span class="source-type">{{ source.source_type === 'LocalFolder' ? '本地文件夹' : '网盘' }}</span>
                <span class="separator">·</span>
                <span>上次扫描: {{ formatDate(source.last_scanned_at) }}</span>
              </div>
            </div>
          </div>
          <div class="source-actions">
            <button class="btn-icon" @click="handleEditSource(source)" title="编辑">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M11 4H4a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2v-7"/>
                <path d="M18.5 2.5a2.121 2.121 0 013 3L12 15l-4 1 1-4 9.5-9.5z"/>
              </svg>
            </button>
            <button class="btn-icon btn-delete" @click="handleDeleteSource(source)" title="删除">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="3 6 5 6 21 6"/>
                <path d="M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2"/>
              </svg>
            </button>
          </div>
        </div>
      </div>

      <div v-else class="empty-state">
        <svg class="empty-state-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
        </svg>
        <h3 class="empty-state-title">暂无音乐源</h3>
        <p class="empty-state-desc">添加音乐文件夹开始管理你的音乐</p>
        <button class="btn btn-primary" style="margin-top: 16px;" @click="handleAddSource">
          添加音乐源
        </button>
      </div>
    </template>
  </div>
</template>

<style scoped>
.music-source-page {
  max-width: 1200px;
  margin: 0 auto;
}

.page-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  margin-bottom: 24px;
}

.page-actions {
  display: flex;
  gap: 12px;
}

.page-actions .btn {
  display: flex;
  align-items: center;
  gap: 8px;
}

.page-actions .btn svg {
  width: 18px;
  height: 18px;
}

.spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.sources-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.source-card {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 20px;
}

.source-info {
  display: flex;
  align-items: center;
  gap: 16px;
}

.source-icon {
  width: 48px;
  height: 48px;
  border-radius: var(--radius-md, 8px);
  background: var(--bg-tertiary, rgba(15, 15, 15, 0.05));
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--primary-color, #0078d7);
}

.source-icon svg {
  width: 24px;
  height: 24px;
}

.source-details {
  min-width: 0;
}

.source-name {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary, #333);
  margin: 0 0 4px 0;
}

.source-path {
  font-size: 13px;
  color: var(--text-secondary, #666);
  margin: 0 0 8px 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 400px;
}

.source-meta {
  font-size: 12px;
  color: var(--text-tertiary, #999);
}

.source-type {
  display: inline-block;
  padding: 2px 8px;
  background: var(--bg-tertiary, #e8e8ed);
  border-radius: 4px;
  font-weight: 500;
}

.separator {
  margin: 0 6px;
}

.source-actions {
  display: flex;
  gap: 8px;
}

.btn-icon {
  width: 36px;
  height: 36px;
  border: none;
  border-radius: var(--radius-md, 8px);
  background: transparent;
  color: var(--text-secondary, #666);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-icon:hover {
  background: var(--hover-bg, rgba(0, 0, 0, 0.05));
  color: var(--text-primary, #333);
}

.btn-icon svg {
  width: 18px;
  height: 18px;
}

.btn-delete:hover {
  background: rgba(239, 68, 68, 0.1);
  color: #ef4444;
}

@media (max-width: 767px) {
  .page-header {
    flex-direction: column;
    gap: 16px;
  }

  .page-actions {
    width: 100%;
  }

  .page-actions .btn {
    flex: 1;
    justify-content: center;
  }

  .source-card {
    flex-direction: column;
    align-items: flex-start;
    gap: 16px;
  }

  .source-path {
    max-width: 280px;
  }

  .source-actions {
    width: 100%;
    justify-content: flex-end;
  }
}

@media (prefers-color-scheme: dark) {
  .source-name {
    color: var(--text-primary, #f6f6f6);
  }
}
</style>
