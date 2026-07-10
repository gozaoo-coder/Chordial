<script setup>
import { ref, onMounted } from 'vue';
import { open } from '@tauri-apps/plugin-dialog';
import {
  addLocalFolder,
  removeLocalFolder,
  getFolders,
  getLocalStats,
  rescanAll,
} from '../api/musicSource';
import { usePerf } from '@/utils/performanceMonitor.js';

const { start, end, log } = usePerf('MusicSourceManager');

// ── State ──────────────────────────────────────────────────────────────────
const folders = ref([]);
const stats = ref({ folder_count: 0, indexed_files: 0 });
const isLoading = ref(true);
const isScanning = ref(false);
const isAdding = ref(false);
const isRemoving = ref(false);

// ── Lifecycle ──────────────────────────────────────────────────────────────
onMounted(async () => {
  if (folders.value.length === 0) {
    await loadData();
  } else {
    isLoading.value = false;
  }
});

// ── Data loading ───────────────────────────────────────────────────────────
const loadData = async () => {
  start('loadData');
  try {
    const [paths, s] = await Promise.all([getFolders(), getLocalStats()]);
    folders.value = (paths || []).map((p) => ({
      path: p,
      // 从路径提取显示名（最后一个目录名）
      name: p.split(/[/\\]/).filter(Boolean).pop() || p,
    }));
    stats.value = s || { folder_count: 0, indexed_files: 0 };
    end('loadData', { folderCount: folders.value.length, indexedFiles: stats.value.indexed_files });
  } catch (error) {
    console.error('Failed to load sources:', error);
    end('loadData', { error: error.message });
  } finally {
    isLoading.value = false;
  }
};

// ── Actions ────────────────────────────────────────────────────────────────
const handleScanAll = async () => {
  isScanning.value = true;
  log('handleScanAll');
  start('scanAll');
  try {
    await rescanAll();
    await loadData();
    end('scanAll');
  } catch (error) {
    console.error('Failed to scan sources:', error);
    end('scanAll', { error: error.message });
  } finally {
    isScanning.value = false;
  }
};

const handleAddSource = async () => {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择音乐文件夹',
    });

    if (selected) {
      isAdding.value = true;
      log('handleAddSource', { path: selected });
      start('addSource');
      await addLocalFolder(selected);
      await loadData();
      end('addSource');
    }
  } catch (error) {
    console.error('Failed to add source:', error);
    alert('添加音乐源失败: ' + error.message);
  } finally {
    isAdding.value = false;
  }
};

const handleDeleteSource = async (folder) => {
  const confirmed = confirm(`确定要移除音乐源 "${folder.name}" 吗？\n\n该操作将从数据库中清理该文件夹下的所有索引数据，不会删除磁盘上的文件。`);
  if (!confirmed) return;

  isRemoving.value = true;
  log('handleDeleteSource', { path: folder.path });
  start('deleteSource');
  try {
    await removeLocalFolder(folder.path);
    await loadData();
    end('deleteSource');
  } catch (error) {
    console.error('Failed to remove source:', error);
    end('deleteSource', { error: error.message });
    alert('移除音乐源失败: ' + error.message);
  } finally {
    isRemoving.value = false;
  }
};

// ── Helpers ────────────────────────────────────────────────────────────────
const getSourceTypeLabel = () => '本地文件夹';

const formatFileCount = (n) => {
  if (n == null) return '—';
  return n.toLocaleString();
};
</script>

<template>
  <div class="music-source-page">
    <!-- Header -->
    <div class="page-header">
      <div>
        <h1 class="page-title">音乐源管理</h1>
        <p class="page-subtitle">
          管理本地音乐文件夹 —
          <template v-if="!isLoading">{{ stats.folder_count }} 个文件夹，共 {{ formatFileCount(stats.indexed_files) }} 个音频文件</template>
          <template v-else>加载中…</template>
        </p>
      </div>
      <div class="page-actions">
        <button class="btn btn-secondary" @click="handleScanAll" :disabled="isScanning">
          <svg v-if="isScanning" class="spin" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 12a9 9 0 11-6.219-8.56"/>
          </svg>
          <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21.5 2v6h-6M2.5 22v-6h6M2 11.5a10 10 0 0118.8-4.3M22 12.5a10 10 0 01-18.8 4.3"/>
          </svg>
          {{ isScanning ? '扫描中…' : '重新扫描' }}
        </button>
        <button class="btn btn-primary" @click="handleAddSource" :disabled="isAdding">
          <svg v-if="isAdding" class="spin" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 12a9 9 0 11-6.219-8.56"/>
          </svg>
          <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="12" y1="5" x2="12" y2="19"/>
            <line x1="5" y1="12" x2="19" y2="12"/>
          </svg>
          {{ isAdding ? '添加中…' : '添加本地文件夹' }}
        </button>
      </div>
    </div>

    <!-- Loading -->
    <div v-if="isLoading" class="loading-state">
      <div class="spinner"></div>
    </div>

    <!-- Folder list -->
    <template v-else>
      <div v-if="folders.length > 0" class="sources-list">
        <div v-for="folder in folders" :key="folder.path" class="source-card card">
          <div class="source-info">
            <div class="source-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
              </svg>
            </div>
            <div class="source-details">
              <h3 class="source-name">{{ folder.name }}</h3>
              <p class="source-path">{{ folder.path }}</p>
              <div class="source-meta">
                <span class="source-type">{{ getSourceTypeLabel() }}</span>
              </div>
            </div>
          </div>
          <div class="source-actions">
            <button
              class="btn-icon btn-delete"
              @click="handleDeleteSource(folder)"
              :disabled="isRemoving"
              title="移除文件夹（不删除磁盘文件）"
            >
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="3 6 5 6 21 6"/>
                <path d="M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2"/>
              </svg>
            </button>
          </div>
        </div>
      </div>

      <!-- Empty state -->
      <div v-else class="empty-state">
        <svg class="empty-state-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
        </svg>
        <h3 class="empty-state-title">暂无音乐源</h3>
        <p class="empty-state-desc">添加本地音乐文件夹开始管理你的音乐</p>
        <button class="btn btn-primary" style="margin-top: 16px;" @click="handleAddSource">
          添加文件夹
        </button>
      </div>
    </template>

    <!-- Unsupported source notice -->
    <div class="unsupported-note card" style="margin-top: 24px;">
      <div class="note-content">
        <p class="note-title">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="note-icon">
            <circle cx="12" cy="12" r="10"/>
            <line x1="12" y1="8" x2="12" y2="12"/>
            <line x1="12" y1="16" x2="12.01" y2="16"/>
          </svg>
          关于网盘和 WebDAV 源
        </p>
        <p class="note-text">
          网盘（WebDisk）和 WebDAV 来源目前后端尚未实现。
          当前仅支持本地文件夹作为音乐源。
        </p>
      </div>
    </div>
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
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
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
  min-width: 0;
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
  flex-shrink: 0;
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
  max-width: 500px;
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

.source-actions {
  display: flex;
  gap: 8px;
  flex-shrink: 0;
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

.btn-icon:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.btn-icon:disabled:hover {
  background: transparent;
  color: var(--text-secondary, #666);
}

/* ── Unsupported note ──────────────────────────────────────────────── */
.unsupported-note {
  background: var(--bg-secondary, #fafafa);
  border: 1px solid var(--border-color, #e8e8ed);
}

.note-content {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.note-title {
  display: flex;
  align-items: center;
  gap: 6px;
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary, #333);
}

.note-icon {
  width: 18px;
  height: 18px;
  color: var(--text-tertiary, #999);
  flex-shrink: 0;
}

.note-text {
  margin: 0;
  font-size: 13px;
  color: var(--text-secondary, #666);
  line-height: 1.5;
}

/* ── Responsive ────────────────────────────────────────────────────── */
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

  .unsupported-note {
    background: var(--bg-secondary, #2c2c2e);
  }
}
</style>
