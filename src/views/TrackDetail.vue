<template>
  <div class="track-detail" v-if="track">
    <div class="track-main-content">
      <div class="left-panel">
        <div class="album-art-container">
          <img 
            v-if="albumArtUrl" 
            :src="albumArtUrl" 
            alt="专辑封面" 
            class="album-art"
          />
          <div v-else class="album-art-placeholder">
            <span>🎵</span>
          </div>
          <div v-if="isLoadingArt" class="loading-overlay">
            <div class="spinner"></div>
          </div>
        </div>
        
        <div class="track-actions">
          <button @click="playTrack" class="btn btn-primary btn-large" :disabled="isLoadingMusic">
            {{ isLoadingMusic ? '加载中...' : (isPlaying ? '暂停' : '播放') }}
          </button>
          <button @click="addToPlaylist" class="btn btn-secondary">
            添加到播放列表
          </button>
        </div>
      </div>
      
      <div class="right-panel">
        <div class="track-info">
          <h1 class="track-title">{{ track.title }}</h1>
          <h2 class="track-artist">{{ track.artist }}</h2>
          <h3 class="track-album">{{ track.album }}</h3>
          
          <div class="track-meta">
            <span class="meta-item">
              <strong>时长:</strong> {{ formatDuration(track.duration) }}
            </span>
            <span class="meta-item" v-if="track.year">
              <strong>年份:</strong> {{ track.year }}
            </span>
            <span class="meta-item">
              <strong>格式:</strong> {{ track.format || '未知' }}
            </span>
            <span class="meta-item" v-if="track.bitrate">
              <strong>比特率:</strong> {{ track.bitrate }} kbps
            </span>
          </div>

          <div class="progress-bar" v-if="isPlaying || currentTime > 0">
            <div class="progress" :style="{ width: progressPercentage + '%' }"></div>
            <span class="time">{{ formatTime(currentTime) }} / {{ formatTime(duration) }}</span>
          </div>
        </div>

        <div class="track-details-section">
          <h3>详细信息</h3>
          <div class="info-grid">
            <div class="info-item">
              <span class="label">文件路径:</span>
              <span class="value">{{ track.file_path }}</span>
            </div>
            <div class="info-item">
              <span class="label">文件大小:</span>
              <span class="value">{{ formatFileSize(track.file_size) }}</span>
            </div>
            <div class="info-item" v-if="track.sample_rate">
              <span class="label">采样率:</span>
              <span class="value">{{ track.sample_rate }} Hz</span>
            </div>
            <div class="info-item" v-if="track.channels">
              <span class="label">声道:</span>
              <span class="value">{{ track.channels }}</span>
            </div>
            <div class="info-item" v-if="track.genre">
              <span class="label">流派:</span>
              <span class="value">{{ track.genre }}</span>
            </div>
            <div class="info-item" v-if="track.composer">
              <span class="label">作曲:</span>
              <span class="value">{{ track.composer }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 歌词显示区域 -->
    <div class="lyrics-section" v-if="trackId">
      <h3>🎵 歌词</h3>
      <LyricsView 
        :trackId="trackId"
        :lyricsData="lyricsData"
        :isPlaying="isPlaying"
        :currentTime="currentTime"
        :showTimeTags="true"
        @seek="onSeek"
      />
    </div>

    <div class="error-message" v-if="error">
      {{ error }}
    </div>
  </div>
  
  <div v-else class="no-track">
    <span class="icon">🎵</span>
    <p>请选择一个曲目查看详情</p>
  </div>
</template>

<script>
import { ref, watch, onMounted, onUnmounted, computed } from 'vue'
import { 
  getTrackInfo, 
  getLyrics 
} from '@/api/musicSource/musicResource.js'
import { 
  getMusicFileResource,
  releaseMusicFile
} from '@/api/musicSource/resourceLoader.js'
import LyricsView from '@/components/LyricsView.vue'

export default {
  name: 'TrackDetail',
  components: {
    LyricsView
  },
  props: {
    trackId: {
      type: String,
      default: null
    }
  },
  setup(props) {
    const track = ref(null)
    const albumArtUrl = ref(null)
    const lyricsData = ref({
      plainLyrics: '',
      syncedLyrics: '',
      hasSyncedLyrics: false,
      hasPlainLyrics: false
    })
    const error = ref('')
    const isLoadingArt = ref(false)
    const isLoadingMusic = ref(false)
    const isPlaying = ref(false)
    const currentTime = ref(0)
    const duration = ref(0)
    
    // 资源引用
    let musicResource = null
    let audioElement = null
    let timeUpdateInterval = null
    
    // 计算属性
    const progressPercentage = computed(() => {
      if (duration.value === 0) return 0
      return (currentTime.value / duration.value) * 100
    })
    
    // 加载曲目信息
    const loadTrackInfo = async () => {
      if (!props.trackId) return
      
      try {
        error.value = ''
        console.log('加载曲目信息:', props.trackId)
        track.value = await getTrackInfo(props.trackId)
        
        // 设置时长
        if (track.value.duration) {
          duration.value = track.value.duration
        }
        
        // 使用返回的专辑封面数据
        if (track.value.album_cover_data) {
          albumArtUrl.value = track.value.album_cover_data
          console.log('专辑封面数据已加载')
        }
        
        // 获取歌词
        await loadLyrics()
        
        // 预加载音乐文件
        preloadMusic(props.trackId)
        
      } catch (err) {
        error.value = '加载曲目信息失败: ' + err.message
        console.error('加载曲目信息失败:', err)
      }
    }
    
    // 加载专辑图片（保留向后兼容）
    const loadAlbumArt = async (albumId) => {
      if (!albumId) return
      
      try {
        isLoadingArt.value = true
        console.log('加载专辑图片:', albumId)
        
        artResource = await getAlbumArtResource(albumId, 'large')
        albumArtUrl.value = artResource.url
        
        console.log('专辑图片加载成功')
      } catch (err) {
        console.error('加载专辑图片失败:', err)
        albumArtUrl.value = null
      } finally {
        isLoadingArt.value = false
      }
    }
    
    // 加载歌词
    const loadLyrics = async () => {
      try {
        console.log('加载歌词:', props.trackId)
        const result = await getLyrics(props.trackId)
        
        lyricsData.value = {
          plainLyrics: result.plainLyrics || '',
          syncedLyrics: result.syncedLyrics || '',
          hasSyncedLyrics: result.hasSyncedLyrics || false,
          hasPlainLyrics: result.hasPlainLyrics || false
        }
        
        console.log('歌词加载完成:', {
          hasSynced: result.hasSyncedLyrics,
          hasPlain: result.hasPlainLyrics
        })
      } catch (err) {
        console.error('加载歌词失败:', err)
        lyricsData.value = {
          plainLyrics: '',
          syncedLyrics: '',
          hasSyncedLyrics: false,
          hasPlainLyrics: false
        }
      }
    }
    
    // 预加载音乐
    const preloadMusic = async (trackId) => {
      try {
        console.log('预加载音乐文件:', trackId)
        musicResource = await getMusicFileResource(trackId)
        console.log('音乐文件预加载成功')
      } catch (err) {
        console.error('预加载音乐文件失败:', err)
      }
    }
    
    // 播放音乐
    const playTrack = async () => {
      if (!props.trackId) return
      
      try {
        // 如果已经在播放，则暂停
        if (isPlaying.value) {
          pauseTrack()
          return
        }
        
        isLoadingMusic.value = true
        console.log('开始播放:', props.trackId)
        
        // 如果没有预加载的资源，则加载
        if (!musicResource) {
          musicResource = await getMusicFileResource(props.trackId)
        }
        
        // 创建音频元素
        audioElement = new Audio(musicResource.url)
        
        // 设置事件监听
        audioElement.addEventListener('loadedmetadata', () => {
          duration.value = audioElement.duration
        })
        
        audioElement.addEventListener('timeupdate', () => {
          currentTime.value = audioElement.currentTime
        })
        
        audioElement.addEventListener('ended', () => {
          isPlaying.value = false
          currentTime.value = 0
          stopTimeUpdate()
        })
        
        audioElement.addEventListener('error', (err) => {
          console.error('播放错误:', err)
          error.value = '播放失败: ' + (audioElement.error?.message || '未知错误')
          isPlaying.value = false
          stopTimeUpdate()
        })
        
        // 开始播放
        await audioElement.play()
        isPlaying.value = true
        startTimeUpdate()
        
        console.log('开始播放')
      } catch (err) {
        error.value = '播放失败: ' + err.message
        console.error('播放失败:', err)
      } finally {
        isLoadingMusic.value = false
      }
    }
    
    // 暂停播放
    const pauseTrack = () => {
      if (audioElement) {
        audioElement.pause()
      }
      isPlaying.value = false
      stopTimeUpdate()
      console.log('暂停播放')
    }
    
    // 开始时间更新
    const startTimeUpdate = () => {
      if (timeUpdateInterval) {
        clearInterval(timeUpdateInterval)
      }
      timeUpdateInterval = setInterval(() => {
        if (audioElement && isPlaying.value) {
          currentTime.value = audioElement.currentTime
        }
      }, 100)
    }
    
    // 停止时间更新
    const stopTimeUpdate = () => {
      if (timeUpdateInterval) {
        clearInterval(timeUpdateInterval)
        timeUpdateInterval = null
      }
    }
    
    // 跳转到指定时间
    const onSeek = (time) => {
      console.log('跳转到:', time)
      if (audioElement) {
        audioElement.currentTime = time
        currentTime.value = time
      }
    }
    
    // 格式化时长
    const formatDuration = (seconds) => {
      if (!seconds) return '0:00'
      const minutes = Math.floor(seconds / 60)
      const remainingSeconds = Math.floor(seconds % 60)
      return `${minutes}:${remainingSeconds.toString().padStart(2, '0')}`
    }
    
    // 格式化时间
    const formatTime = (seconds) => {
      if (!seconds || isNaN(seconds)) return '0:00'
      const mins = Math.floor(seconds / 60)
      const secs = Math.floor(seconds % 60)
      return `${mins}:${secs.toString().padStart(2, '0')}`
    }
    
    // 格式化文件大小
    const formatFileSize = (bytes) => {
      if (!bytes) return '未知'
      const units = ['B', 'KB', 'MB', 'GB']
      let unitIndex = 0
      let size = bytes
      
      while (size >= 1024 && unitIndex < units.length - 1) {
        size /= 1024
        unitIndex++
      }
      
      return `${size.toFixed(2)} ${units[unitIndex]}`
    }
    
    // 添加到播放列表
    const addToPlaylist = () => {
      console.log('添加到播放列表:', track.value?.title)
    }
    
    // 释放资源
    const releaseResources = () => {
      console.log('释放资源')
      
      if (audioElement) {
        audioElement.pause()
        audioElement = null
      }
      
      stopTimeUpdate()
      
      if (musicResource) {
        musicResource.release()
        musicResource = null
      }
    }
    
    // 监听 trackId 变化
    watch(() => props.trackId, (newId) => {
      if (newId) {
        releaseResources()
        loadTrackInfo()
      } else {
        track.value = null
        albumArtUrl.value = null
        lyricsData.value = {
          plainLyrics: '',
          syncedLyrics: '',
          hasSyncedLyrics: false,
          hasPlainLyrics: false
        }
        releaseResources()
      }
    })
    
    // 组件挂载时加载
    onMounted(() => {
      if (props.trackId) {
        loadTrackInfo()
      }
    })
    
    // 组件销毁时释放资源
    onUnmounted(() => {
      releaseResources()
    })
    
    return {
      track,
      albumArtUrl,
      lyricsData,
      error,
      isLoadingArt,
      isLoadingMusic,
      isPlaying,
      currentTime,
      duration,
      progressPercentage,
      playTrack,
      addToPlaylist,
      onSeek,
      formatDuration,
      formatTime,
      formatFileSize
    }
  }
}
</script>

<style scoped>
.track-detail {
  padding: 20px;
  max-width: 1200px;
  margin: 0 auto;
}

.track-main-content {
  display: flex;
  gap: 40px;
  margin-bottom: 30px;
}

.left-panel {
  flex-shrink: 0;
  width: 300px;
}

.right-panel {
  flex: 1;
}

.album-art-container {
  position: relative;
  width: 100%;
  aspect-ratio: 1;
  margin-bottom: 20px;
}

.album-art {
  width: 100%;
  height: 100%;
  object-fit: cover;
  border-radius: 12px;
  box-shadow: 0 8px 24px rgba(0,0,0,0.15);
}

.album-art-placeholder {
  width: 100%;
  height: 100%;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 12px;
  font-size: 80px;
}

.loading-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(255,255,255,0.9);
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 12px;
}

.spinner {
  width: 50px;
  height: 50px;
  border: 4px solid #f3f3f3;
  border-top: 4px solid #667eea;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.track-actions {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.btn-large {
  padding: 15px 30px;
  font-size: 18px;
}

.track-info {
  margin-bottom: 30px;
}

.track-title {
  margin: 0 0 8px 0;
  font-size: 32px;
  color: #1a1a2e;
  font-weight: bold;
}

.track-artist {
  margin: 0 0 4px 0;
  font-size: 20px;
  color: #666;
}

.track-album {
  margin: 0 0 20px 0;
  font-size: 16px;
  color: #999;
}

.track-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 15px;
  margin-bottom: 20px;
}

.meta-item {
  color: #555;
  font-size: 14px;
}

.meta-item strong {
  color: #333;
}

.progress-bar {
  position: relative;
  height: 8px;
  background: #e0e0e0;
  border-radius: 4px;
  overflow: hidden;
  margin-top: 15px;
}

.progress {
  height: 100%;
  background: linear-gradient(90deg, #667eea 0%, #764ba2 100%);
  border-radius: 4px;
  transition: width 0.1s linear;
}

.progress-bar .time {
  display: block;
  margin-top: 8px;
  font-size: 12px;
  color: #999;
  text-align: right;
}

.track-details-section {
  margin-bottom: 30px;
}

.track-details-section h3 {
  margin: 0 0 15px 0;
  color: #333;
  border-bottom: 2px solid #667eea;
  padding-bottom: 8px;
}

.info-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
  gap: 15px;
}

.info-item {
  display: flex;
  flex-direction: column;
}

.info-item .label {
  font-size: 12px;
  color: #999;
  margin-bottom: 4px;
}

.info-item .value {
  font-size: 14px;
  color: #333;
  word-break: break-all;
}

.lyrics-section {
  margin-top: 30px;
  border-radius: 12px;
  overflow: hidden;
  box-shadow: 0 4px 15px rgba(0,0,0,0.1);
}

.lyrics-section h3 {
  margin: 0;
  padding: 15px 20px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  font-size: 18px;
}

.lyrics-section > div {
  max-height: 400px;
  overflow: hidden;
}

.no-track {
  text-align: center;
  padding: 80px 20px;
  color: #999;
}

.no-track .icon {
  font-size: 64px;
  margin-bottom: 15px;
  display: block;
}

.no-track p {
  font-size: 18px;
}

.error-message {
  background: #f8d7da;
  color: #721c24;
  padding: 15px;
  border-radius: 8px;
  margin-top: 20px;
  border: 1px solid #f5c6cb;
}

.btn {
  padding: 10px 20px;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  font-size: 14px;
  transition: all 0.3s;
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn-primary {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
}

.btn-primary:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4);
}

.btn-secondary {
  background: #f0f0f0;
  color: #333;
}

.btn-secondary:hover:not(:disabled) {
  background: #e0e0e0;
}

@media (max-width: 768px) {
  .track-main-content {
    flex-direction: column;
  }
  
  .left-panel {
    width: 100%;
    max-width: 300px;
    margin: 0 auto;
  }
}
</style>