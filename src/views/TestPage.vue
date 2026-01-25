<template>
  <div class="test-page">
    <h1>音乐资源测试</h1>
    
    <div class="test-section">
      <h2>扫描并获取曲目</h2>
      <button @click="scanAndGetTracks" class="btn btn-primary" :disabled="isScanning">
        {{ isScanning ? '扫描中...' : '扫描所有源' }}
      </button>
    </div>

    <div class="test-section" v-if="tracks.length > 0">
      <h2>曲目列表</h2>
      <div class="tracks-grid">
        <div 
          v-for="track in tracks" 
          :key="track.id" 
          class="track-card"
          :class="{ 'selected': selectedTrackId === track.id }"
          @click="selectTrack(track)"
        >
          <div class="track-title">{{ track.title || track.file_name }}</div>
          <div class="track-artist">{{ track.artist || '未知艺术家' }}</div>
          <div class="track-album">{{ track.album || '未知专辑' }}</div>
          <div class="track-duration" v-if="track.duration">
            {{ formatDuration(track.duration) }}
          </div>
        </div>
      </div>
    </div>

    <div class="test-section" v-if="selectedTrackId">
      <h2>曲目详情</h2>
      <TrackDetail :trackId="selectedTrackId" />
    </div>

    <div class="test-section">
      <h2>资源管理测试</h2>
      <div class="resource-stats">
        <div class="stat-item">
          <span class="label">缓存资源数:</span>
          <span class="value">{{ resourceStats.cachedResources }}</span>
        </div>
        <button @click="clearResources" class="btn btn-warning">
          清理所有资源
        </button>
      </div>
    </div>

    <div v-if="error" class="test-section error-message">
      <h2>错误信息</h2>
      <div class="error-content">{{ error }}</div>
    </div>
  </div>
</template>

<script>
import { ref, onMounted } from 'vue'
import { scanAll, getCached } from '@/api/musicSource/library.js'
import { getResourceStats, clearAllResources } from '@/api/musicSource/resourceLoader.js'
import TrackDetail from './TrackDetail.vue'

export default {
  name: 'TestPage',
  components: {
    TrackDetail
  },
  setup() {
    const tracks = ref([])
    const selectedTrackId = ref(null)
    const isScanning = ref(false)
    const error = ref('')
    const resourceStats = ref({ cachedResources: 0 })

    const scanAndGetTracks = async () => {
      try {
        isScanning.value = true
        error.value = ''
        
        console.log('开始扫描...')
        const library = await scanAll()
        console.log('扫描完成:', library)
        
        tracks.value = library.tracks || []
        console.log('找到曲目数:', tracks.value.length)
      } catch (err) {
        error.value = '扫描失败: ' + err.message
        console.error('扫描失败:', err)
        
        // 尝试从缓存加载
        console.log('尝试从缓存加载...')
        try {
          const cached = await getCached()
          if (cached) {
            tracks.value = cached.tracks || []
            console.log('从缓存加载曲目数:', tracks.value.length)
          }
        } catch (cacheErr) {
          console.error('加载缓存失败:', cacheErr)
        }
      } finally {
        isScanning.value = false
      }
    }

    const selectTrack = (track) => {
      selectedTrackId.value = track.id
      console.log('选择曲目:', track.title)
    }

    const formatDuration = (seconds) => {
      if (!seconds) return '0:00'
      const minutes = Math.floor(seconds / 60)
      const remainingSeconds = Math.floor(seconds % 60)
      return `${minutes}:${remainingSeconds.toString().padStart(2, '0')}`
    }

    const clearResources = () => {
      clearAllResources()
      resourceStats.value = getResourceStats()
      console.log('资源已清理')
    }

    const updateStats = () => {
      resourceStats.value = getResourceStats()
    }

    // 定期更新资源统计
    setInterval(updateStats, 1000)

    return {
      tracks,
      selectedTrackId,
      isScanning,
      error,
      resourceStats,
      scanAndGetTracks,
      selectTrack,
      formatDuration,
      clearResources
    }
  }
}
</script>

<style scoped>
.test-page {
  padding: 20px;
  max-width: 1200px;
  margin: 0 auto;
}

.test-section {
  margin-bottom: 30px;
  padding: 20px;
  background: #f8f9fa;
  border-radius: 8px;
}

.test-section h2 {
  margin-top: 0;
  margin-bottom: 15px;
  color: #333;
  border-bottom: 1px solid #ddd;
  padding-bottom: 10px;
}

.tracks-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
  gap: 15px;
}

.track-card {
  padding: 15px;
  background: white;
  border: 1px solid #ddd;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.3s;
}

.track-card:hover {
  box-shadow: 0 4px 8px rgba(0,0,0,0.1);
}

.track-card.selected {
  border-color: #007bff;
  background: #f0f7ff;
}

.track-title {
  font-weight: bold;
  margin-bottom: 5px;
  color: #333;
}

.track-artist, .track-album {
  font-size: 14px;
  color: #666;
  margin-bottom: 3px;
}

.track-duration {
  font-size: 12px;
  color: #999;
  margin-top: 5px;
}

.resource-stats {
  display: flex;
  align-items: center;
  gap: 20px;
}

.stat-item {
  display: flex;
  align-items: center;
  gap: 10px;
}

.stat-item .label {
  font-weight: bold;
  color: #333;
}

.stat-item .value {
  font-size: 18px;
  color: #007bff;
}

.error-message {
  background: #f8d7da;
  color: #721c24;
  padding: 15px;
  border-radius: 4px;
  border: 1px solid #f5c6cb;
}

.btn {
  padding: 10px 20px;
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

.btn-warning {
  background: #ffc107;
  color: #212529;
}

.btn-warning:hover:not(:disabled) {
  background: #e0a800;
}
</style>