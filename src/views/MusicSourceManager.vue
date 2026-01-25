<template>
  <div class="music-source-manager">
    <h1>音乐源管理</h1>
    
    <!-- 添加音乐源区域 -->
    <div class="add-source-section">
      <h2>添加音乐源</h2>
      <div class="source-types">
        <button @click="showLocalFolderDialog = true" class="btn btn-primary">
          添加本地文件夹
        </button>
        <button @click="showWebDiskDialog = true" class="btn btn-secondary">
          添加网盘源
        </button>
      </div>
    </div>

    <!-- 本地文件夹添加对话框 -->
    <div v-if="showLocalFolderDialog" class="dialog-overlay" @click="showLocalFolderDialog = false">
      <div class="dialog" @click.stop>
        <h3>添加本地文件夹</h3>
        <input 
          v-model="localFolderPath" 
          placeholder="输入文件夹路径"
          class="input-field"
        />
        <label class="checkbox-label">
          <input type="checkbox" v-model="recursiveScan" />
          递归扫描子文件夹
        </label>
        <div class="dialog-buttons">
          <button @click="addLocalFolder" class="btn btn-primary">确定</button>
          <button @click="showLocalFolderDialog = false" class="btn btn-secondary">取消</button>
        </div>
      </div>
    </div>

    <!-- 网盘添加对话框 -->
    <div v-if="showWebDiskDialog" class="dialog-overlay" @click="showWebDiskDialog = false">
      <div class="dialog" @click.stop>
        <h3>添加网盘源</h3>
        <input 
          v-model="webDiskUrl" 
          placeholder="输入网盘URL (webdev://...)"
          class="input-field"
        />
        <input 
          v-model="webDiskUsername" 
          placeholder="用户名 (可选)"
          class="input-field"
        />
        <input 
          v-model="webDiskPassword" 
          type="password"
          placeholder="密码 (可选)"
          class="input-field"
        />
        <div class="dialog-buttons">
          <button @click="addWebDisk" class="btn btn-primary">确定</button>
          <button @click="showWebDiskDialog = false" class="btn btn-secondary">取消</button>
        </div>
      </div>
    </div>

    <!-- 音乐源列表 -->
    <div class="sources-section">
      <h2>音乐源列表</h2>
      <div class="sources-actions">
        <button @click="refreshAllSources" class="btn btn-info" :disabled="isScanning">
          {{ isScanning ? '扫描中...' : '扫描所有源' }}
        </button>
        <button @click="loadSources" class="btn btn-secondary">刷新列表</button>
      </div>
      
      <div v-if="sources.length === 0" class="no-sources">
        暂无音乐源，请添加音乐源
      </div>
      
      <div v-else class="sources-list">
        <div v-for="source in sources" :key="source.id" class="source-item">
          <div class="source-info">
            <h4>{{ source.name || source.path || source.url }}</h4>
            <p class="source-type">类型: {{ getSourceTypeName(source.type) }}</p>
            <p class="source-status">
              状态: 
              <span :class="['status', source.enabled ? 'enabled' : 'disabled']">
                {{ source.enabled ? '已启用' : '已禁用' }}
              </span>
            </p>
            <p v-if="source.last_scan" class="source-last-scan">
              最后扫描: {{ formatDate(source.last_scan) }}
            </p>
          </div>
          <div class="source-actions">
            <button 
              @click="toggleSource(source)" 
              :class="['btn', source.enabled ? 'btn-warning' : 'btn-success']"
            >
              {{ source.enabled ? '禁用' : '启用' }}
            </button>
            <button @click="refreshSource(source)" class="btn btn-info" :disabled="source.isRefreshing">
              {{ source.isRefreshing ? '刷新中...' : '刷新' }}
            </button>
            <button @click="removeSource(source)" class="btn btn-danger">删除</button>
          </div>
        </div>
      </div>
    </div>

    <!-- 扫描结果区域 -->
    <div v-if="scanResults" class="scan-results-section">
      <h2>扫描结果</h2>
      <div class="scan-summary">
        <div class="summary-item">
          <span class="label">总源数:</span>
          <span class="value">{{ scanResults.sources?.length || 0 }}</span>
        </div>
        <div class="summary-item">
          <span class="label">总曲目数:</span>
          <span class="value">{{ scanResults.tracks?.length || 0 }}</span>
        </div>
      </div>
      
      <div class="results-tabs">
        <button 
          @click="activeTab = 'sources'" 
          :class="['tab-btn', activeTab === 'sources' ? 'active' : '']"
        >
          源详情
        </button>
        <button 
          @click="activeTab = 'tracks'" 
          :class="['tab-btn', activeTab === 'tracks' ? 'active' : '']"
        >
          曲目列表
        </button>
      </div>

      <div v-if="activeTab === 'sources'" class="sources-details">
        <div v-for="source in scanResults.sources" :key="source.id" class="source-detail">
          <h4>{{ source.name }}</h4>
          <p>曲目数量: {{ source.track_count || 0 }}</p>
          <p>扫描状态: {{ source.scan_status || '未知' }}</p>
        </div>
      </div>

      <div v-if="activeTab === 'tracks'" class="tracks-list">
        <div class="tracks-header">
          <span>标题</span>
          <span>艺术家</span>
          <span>专辑</span>
          <span>时长</span>
        </div>
        <div v-for="track in scanResults.tracks" :key="track.id" class="track-item">
          <span class="track-title">{{ track.title }}</span>
          <span class="track-artist">{{ track.artist }}</span>
          <span class="track-album">{{ track.album }}</span>
          <span class="track-duration">{{ formatDuration(track.duration) }}</span>
        </div>
      </div>
    </div>

    <!-- 错误提示 -->
    <div v-if="error" class="error-message">
      {{ error }}
    </div>
  </div>
</template>

<script>
import { ref, onMounted } from 'vue'
import { 
  addLocalFolder as addLocalFolderApi, 
  addWebDisk as addWebDiskApi, 
  remove, 
  getAll, 
  setEnabled 
} from '@/api/musicSource/sources.js'
import { scanAll, getCached, refreshSource as refreshSourceApi } from '@/api/musicSource/library.js'

export default {
  name: 'MusicSourceManager',
  setup() {
    const sources = ref([])
    const scanResults = ref(null)
    const isScanning = ref(false)
    const error = ref('')
    const activeTab = ref('sources')

    // 对话框状态
    const showLocalFolderDialog = ref(false)
    const showWebDiskDialog = ref(false)
    
    // 表单数据
    const localFolderPath = ref('')
    const recursiveScan = ref(true)
    const webDiskUrl = ref('')
    const webDiskUsername = ref('')
    const webDiskPassword = ref('')

    // 加载音乐源列表
    const loadSources = async () => {
      try {
        error.value = ''
        sources.value = await getAll()
      } catch (err) {
        error.value = '加载音乐源失败: ' + err.message
      }
    }

    // 添加本地文件夹
    const addLocalFolder = async () => {
      if (!localFolderPath.value) {
        error.value = '请输入文件夹路径'
        return
      }
      
      try {
        console.log('开始添加本地文件夹:', localFolderPath.value)
        error.value = ''
        const result = await addLocalFolderApi(localFolderPath.value, recursiveScan.value)
        console.log('添加本地文件夹成功:', result)
        showLocalFolderDialog.value = false
        localFolderPath.value = ''
        await loadSources()
        console.log('音乐源列表已刷新')
      } catch (err) {
        console.error('添加本地文件夹失败:', err)
        error.value = '添加本地文件夹失败: ' + (err.message || err)
      }
    }

    // 添加网盘源
    const addWebDisk = async () => {
      if (!webDiskUrl.value) {
        error.value = '请输入网盘URL'
        return
      }
      
      try {
        console.log('开始添加网盘源:', webDiskUrl.value)
        error.value = ''
        const result = await addWebDiskApi(webDiskUrl.value, webDiskUsername.value || null, webDiskPassword.value || null)
        console.log('添加网盘源成功:', result)
        showWebDiskDialog.value = false
        webDiskUrl.value = ''
        webDiskUsername.value = ''
        webDiskPassword.value = ''
        await loadSources()
        console.log('音乐源列表已刷新')
      } catch (err) {
        console.error('添加网盘源失败:', err)
        error.value = '添加网盘源失败: ' + (err.message || err)
      }
    }

    // 切换源启用状态
    const toggleSource = async (source) => {
      try {
        error.value = ''
        await setEnabled(source.id, !source.enabled)
        await loadSources()
      } catch (err) {
        error.value = '切换源状态失败: ' + err.message
      }
    }

    // 刷新单个源
    const refreshSource = async (source) => {
      try {
        error.value = ''
        source.isRefreshing = true
        await refreshSourceApi(source.id)
        await loadSources()
        // 重新加载扫描结果
        await loadScanResults()
      } catch (err) {
        error.value = '刷新源失败: ' + err.message
      } finally {
        source.isRefreshing = false
      }
    }

    // 删除源
    const removeSource = async (source) => {
      if (!confirm(`确定要删除音乐源 "${source.name || source.path || source.url}" 吗？`)) {
        return
      }
      
      try {
        error.value = ''
        await remove(source.id)
        await loadSources()
      } catch (err) {
        error.value = '删除源失败: ' + err.message
      }
    }

    // 扫描所有源
    const refreshAllSources = async () => {
      try {
        error.value = ''
        isScanning.value = true
        scanResults.value = await scanAll()
      } catch (err) {
        error.value = '扫描失败: ' + err.message
      } finally {
        isScanning.value = false
      }
    }

    // 加载扫描结果
    const loadScanResults = async () => {
      try {
        const cached = await getCached()
        if (cached) {
          scanResults.value = cached
          console.log(cached);
          
        }
      } catch (err) {
        console.error('加载缓存失败:', err)
      }
    }

    // 工具函数
    const getSourceTypeName = (type) => {
      const typeMap = {
        'local': '本地文件夹',
        'webdisk': '网盘源'
      }
      return typeMap[type] || '未知类型'
    }

    const formatDate = (dateString) => {
      if (!dateString) return '从未'
      return new Date(dateString).toLocaleString('zh-CN')
    }

    const formatDuration = (seconds) => {
      if (!seconds) return '0:00'
      const minutes = Math.floor(seconds / 60)
      const remainingSeconds = Math.floor(seconds % 60)
      return `${minutes}:${remainingSeconds.toString().padStart(2, '0')}`
    }

    // 生命周期
    onMounted(async () => {
      await loadSources()
      await loadScanResults()
    })

    return {
      sources,
      scanResults,
      isScanning,
      error,
      activeTab,
      showLocalFolderDialog,
      showWebDiskDialog,
      localFolderPath,
      recursiveScan,
      webDiskUrl,
      webDiskUsername,
      webDiskPassword,
      loadSources,
      addLocalFolder,
      addWebDisk,
      toggleSource,
      refreshSource,
      removeSource,
      refreshAllSources,
      getSourceTypeName,
      formatDate,
      formatDuration
    }
  }
}
</script>

<style scoped>
.music-source-manager {
  padding: 20px;
  max-width: 1200px;
  margin: 0 auto;
}

.add-source-section {
  margin-bottom: 30px;
  padding: 20px;
  background: #f8f9fa;
  border-radius: 8px;
}

.source-types {
  display: flex;
  gap: 15px;
  margin-top: 15px;
}

.sources-section {
  margin-bottom: 30px;
}

.sources-actions {
  display: flex;
  gap: 15px;
  margin-bottom: 20px;
}

.no-sources {
  text-align: center;
  padding: 40px;
  color: #6c757d;
  font-size: 16px;
}

.sources-list {
  display: flex;
  flex-direction: column;
  gap: 15px;
}

.source-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px;
  background: white;
  border: 1px solid #dee2e6;
  border-radius: 8px;
  box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}

.source-info h4 {
  margin: 0 0 10px 0;
  color: #333;
}

.source-type, .source-status, .source-last-scan {
  margin: 5px 0;
  color: #6c757d;
  font-size: 14px;
}

.status.enabled {
  color: #28a745;
}

.status.disabled {
  color: #dc3545;
}

.source-actions {
  display: flex;
  gap: 10px;
}

.scan-results-section {
  margin-top: 30px;
  padding: 20px;
  background: white;
  border-radius: 8px;
  border: 1px solid #dee2e6;
}

.scan-summary {
  display: flex;
  gap: 30px;
  margin-bottom: 20px;
  padding: 15px;
  background: #f8f9fa;
  border-radius: 4px;
}

.summary-item {
  display: flex;
  align-items: center;
  gap: 10px;
}

.summary-item .label {
  font-weight: bold;
  color: #495057;
}

.summary-item .value {
  font-size: 18px;
  font-weight: bold;
  color: #007bff;
}

.results-tabs {
  display: flex;
  gap: 10px;
  margin-bottom: 20px;
  border-bottom: 1px solid #dee2e6;
}

.tab-btn {
  padding: 10px 20px;
  background: none;
  border: none;
  border-bottom: 2px solid transparent;
  cursor: pointer;
  transition: all 0.3s;
}

.tab-btn.active {
  border-bottom-color: #007bff;
  color: #007bff;
}

.sources-details {
  display: flex;
  flex-direction: column;
  gap: 15px;
}

.source-detail {
  padding: 15px;
  background: #f8f9fa;
  border-radius: 4px;
}

.tracks-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.tracks-header {
  display: grid;
  grid-template-columns: 2fr 1fr 1fr 1fr;
  gap: 15px;
  padding: 15px;
  background: #f8f9fa;
  font-weight: bold;
  border-radius: 4px;
}

.track-item {
  display: grid;
  grid-template-columns: 2fr 1fr 1fr 1fr;
  gap: 15px;
  padding: 15px;
  background: white;
  border: 1px solid #dee2e6;
  border-radius: 4px;
}

.track-title {
  font-weight: 500;
  color: #333;
}

.track-artist, .track-album, .track-duration {
  color: #6c757d;
}

.dialog-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0,0,0,0.5);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 1000;
}

.dialog {
  background: white;
  padding: 30px;
  border-radius: 8px;
  min-width: 400px;
  max-width: 500px;
}

.dialog h3 {
  margin-top: 0;
  margin-bottom: 20px;
}

.input-field {
  width: 100%;
  padding: 10px;
  margin-bottom: 15px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 14px;
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 20px;
}

.dialog-buttons {
  display: flex;
  gap: 15px;
  justify-content: flex-end;
}

.error-message {
  background: #f8d7da;
  color: #721c24;
  padding: 15px;
  border-radius: 4px;
  margin-top: 20px;
  border: 1px solid #f5c6cb;
}

.btn {
  padding: 8px 16px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  transition: all 0.3s;
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn-primary {
  background: #007bff;
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background: #0056b3;
}

.btn-secondary {
  background: #6c757d;
  color: white;
}

.btn-secondary:hover:not(:disabled) {
  background: #545b62;
}

.btn-success {
  background: #28a745;
  color: white;
}

.btn-success:hover:not(:disabled) {
  background: #1e7e34;
}

.btn-warning {
  background: #ffc107;
  color: #212529;
}

.btn-warning:hover:not(:disabled) {
  background: #e0a800;
}

.btn-danger {
  background: #dc3545;
  color: white;
}

.btn-danger:hover:not(:disabled) {
  background: #c82333;
}

.btn-info {
  background: #17a2b8;
  color: white;
}

.btn-info:hover:not(:disabled) {
  background: #138496;
}
</style>