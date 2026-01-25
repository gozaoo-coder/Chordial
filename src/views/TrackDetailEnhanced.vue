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

    <!-- 高级歌词显示区域 -->
    <div class="lyrics-section" v-if="trackId && advancedLyricsData.lines.length > 0">
      <h3>🎵 高级歌词</h3>
      <div class="lyrics-controls">
        <button @click="toggleLyricsView" class="btn btn-small">
          {{ showAdvancedLyrics ? '切换到普通歌词' : '切换到高级歌词' }}
        </button>
        <button @click="toggleTheme" class="btn btn-small">
          主题: {{ currentTheme }}
        </button>
        <button @click="toggleSettings" class="btn btn-small">
          设置
        </button>
      </div>
      
      <AdvancedLyricsView 
        v-if="showAdvancedLyrics"
        :lyricsData="advancedLyricsData"
        :currentTime="currentTime"
        :duration="duration"
        :isPlaying="isPlaying"
        :theme="currentTheme"
        :fontSize="fontSize"
        :showTranslation="showTranslation"
        :showRoman="showRoman"
        :enableAnimation="enableAnimation"
        @seek="onSeek"
        @line-change="onLineChange"
      />
      
      <LyricsView 
        v-else
        :trackId="trackId"
        :lyricsData="lyricsData"
        :isPlaying="isPlaying"
        :currentTime="currentTime"
        :showTimeTags="true"
        @seek="onSeek"
      />
    </div>

    <!-- 普通歌词显示区域（兼容原有功能） -->
    <div class="lyrics-section" v-else-if="trackId">
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
import AdvancedLyricsView from '@/components/AdvancedLyricsView.vue'
import { parseLyricContent, detectLyricFormat } from '@/api/lyrics/index.js'

export default {
  name: 'TrackDetail',
  components: {
    LyricsView,
    AdvancedLyricsView
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
    const advancedLyricsData = ref({
      lines: [],
      metadata: null
    })
    const error = ref('')
    const isLoadingArt = ref(false)
    const isLoadingMusic = ref(false)
    const isPlaying = ref(false)
    const currentTime = ref(0)
    const duration = ref(0)
    
    // 高级歌词设置
    const showAdvancedLyrics = ref(true)
    const currentTheme = ref('default')
    const fontSize = ref(18)
    const showTranslation = ref(true)
    const showRoman = ref(true)
    const enableAnimation = ref(true)
    
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
        
        // 尝试解析高级歌词
        await loadAdvancedLyrics()
        
        // 预加载音乐文件
        preloadMusic(props.trackId)
        
      } catch (err) {
        error.value = '加载曲目信息失败: ' + err.message
        console.error('加载曲目信息失败:', err)
      }
    }
    
    // 加载高级歌词
    const loadAdvancedLyrics = async () => {
      try {
        console.log('尝试加载高级歌词...')
        
        // 如果有同步歌词，尝试解析
        if (lyricsData.value.syncedLyrics) {
          console.log('检测到同步歌词，尝试解析为高级格式')
          const parsed = await parseLyricContent(lyricsData.value.syncedLyrics)
          console.log('高级歌词解析结果:', parsed)
          
          if (parsed && parsed.lines && parsed.lines.length > 0) {
            advancedLyricsData.value = parsed
            console.log('高级歌词加载成功，共', parsed.lines.length, '行')
            return
          }
        }
        
        // 如果有普通歌词，尝试解析
        if (lyricsData.value.plainLyrics) {
          console.log('检测到普通歌词，尝试解析为高级格式')
          const format = await detectLyricFormat(lyricsData.value.plainLyrics)
          console.log('检测到的歌词格式:', format)
          
          if (format !== 'unknown') {
            const parsed = await parseLyricContent(lyricsData.value.plainLyrics, format)
            console.log('高级歌词解析结果:', parsed)
            
            if (parsed && parsed.lines && parsed.lines.length > 0) {
              advancedLyricsData.value = parsed
              console.log('高级歌词加载成功，共', parsed.lines.length, '行')
              return
            }
          }
        }
        
        console.log('没有可用的歌词数据')
        advancedLyricsData.value = { lines: [], metadata: null }
        
      } catch (err) {
        console.error('加载高级歌词失败:', err)
        advancedLyricsData.value = { lines: [], metadata: null }
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
    
    // 歌词控制函数
    const toggleLyricsView = () => {
      showAdvancedLyrics.value = !showAdvancedLyrics.value
    }
    
    const toggleTheme = () => {
      const themes = ['default', 'dark', 'light']
      const currentIndex = themes.indexOf(currentTheme.value)
      currentTheme.value = themes[(currentIndex + 1) % themes.length]
    }
    
    const toggleSettings = () => {
      // TODO: 打开设置面板
      console.log('打开歌词设置面板')
    }
    
    const onLineChange = (event) => {
      console.log('歌词行切换:', event)
    }
    
    // 预加载音乐
    const preloadMusic = async (trackId) => {
      try {
        console.log('预加载音乐文件:', trackId)
        musicResource = await getMusicFileResource(trackId)
        console.log('音乐文件预加载成功')
      } catch (err) {
        console.error('音乐文件预加载失败:', err)
      }
    }
    
    // 播放控制
    const playTrack = async () => {
      if (!musicResource) {
        console.error('音乐资源未加载')
        return
      }
      
      try {
        isLoadingMusic.value = true
        
        if (!audioElement) {
          audioElement = new Audio()
          audioElement.src = musicResource.url
          
          audioElement.addEventListener('loadedmetadata', () => {
            duration.value = audioElement.duration
            isLoadingMusic.value = false
          })
          
          audioElement.addEventListener('play', () => {
            isPlaying.value = true
            startTimeUpdate()
          })
          
          audioElement.addEventListener('pause', () => {
            isPlaying.value = false
            stopTimeUpdate()
          })
          
          audioElement.addEventListener('ended', () => {
            isPlaying.value = false
            stopTimeUpdate()
            currentTime.value = 0
          })
          
          audioElement.addEventListener('error', (e) => {
            console.error('音频播放错误:', e)
            isLoadingMusic.value = false
            error.value = '音频播放失败'
          })
        }
        
        if (isPlaying.value) {
          audioElement.pause()
        } else {
          await audioElement.play()
        }
        
      } catch (err) {
        console.error('播放失败:', err)
        error.value = '播放失败: ' + err.message
        isLoadingMusic.value = false
      }
    }
    
    // 时间更新
    const startTimeUpdate = () => {
      stopTimeUpdate()
      timeUpdateInterval = setInterval(() => {
        if (audioElement) {
          currentTime.value = audioElement.currentTime
        }
      }, 100)
    }
    
    const stopTimeUpdate = () => {
      if (timeUpdateInterval) {
        clearInterval(timeUpdateInterval)
        timeUpdateInterval = null
      }
    }
    
    // 跳转
    const onSeek = (time) => {
      if (audioElement) {
        audioElement.currentTime = time
        currentTime.value = time
      }
    }
    
    // 添加到播放列表
    const addToPlaylist = () => {
      console.log('添加到播放列表:', track.value?.title)
      // TODO: 实现播放列表功能
    }
    
    // 工具函数
    const formatDuration = (seconds) => {
      if (!seconds) return '0:00'
      const mins = Math.floor(seconds / 60)
      const secs = Math.floor(seconds % 60)
      return `${mins}:${secs.toString().padStart(2, '0')}`
    }
    
    const formatTime = (seconds) => {
      if (!seconds) return '0:00'
      const mins = Math.floor(seconds / 60)
      const secs = Math.floor(seconds % 60)
      return `${mins}:${secs.toString().padStart(2, '0')}`
    }
    
    const formatFileSize = (bytes) => {
      if (!bytes) return '0 B'
      const k = 1024
      const sizes = ['B', 'KB', 'MB', 'GB']
      const i = Math.floor(Math.log(bytes) / Math.log(k))
      return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
    }
    
    // 生命周期
    onMounted(() => {
      if (props.trackId) {
        loadTrackInfo()
      }
    })
    
    onUnmounted(() => {
      if (audioElement) {
        audioElement.pause()
        audioElement = null
      }
      stopTimeUpdate()
      if (musicResource) {
        releaseMusicFile(musicResource)
      }
    })
    
    watch(() => props.trackId, (newTrackId) => {
      if (newTrackId) {
        loadTrackInfo()
      }
    })
    
    return {
      track,
      albumArtUrl,
      lyricsData,
      advancedLyricsData,
      error,
      isLoadingArt,
      isLoadingMusic,
      isPlaying,
      currentTime,
      duration,
      progressPercentage,
      showAdvancedLyrics,
      currentTheme,
      fontSize,
      showTranslation,
      showRoman,
      enableAnimation,
      loadTrackInfo,
      loadAlbumArt,
      loadLyrics,
      loadAdvancedLyrics,
      toggleLyricsView,
      toggleTheme,
      toggleSettings,
      onLineChange,
      preloadMusic,
      playTrack,
      onSeek,
      addToPlaylist,
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
  display: grid;
  grid-template-columns: 300px 1fr;
  gap: 40px;
  margin-bottom: 40px;
}

.left-panel {
  display: flex;
  flex-direction: column;
  align-items: center;
}

.album-art-container {
  position: relative;
  width: 250px;
  height: 250px;
  margin-bottom: 20px;
  border-radius: 12px;
  overflow: hidden;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
}

.album-art {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.album-art-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  font-size: 48px;
}

.loading-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
}

.spinner {
  width: 40px;
  height: 40px;
  border: 4px solid rgba(255, 255, 255, 0.3);
  border-top: 4px solid white;
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
  gap: 12px;
  width: 100%;
}

.btn {
  padding: 12px 24px;
  border: none;
  border-radius: 8px;
  font-size: 16px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.3s ease;
}

.btn-primary {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
}

.btn-primary:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 4px 16px rgba(102, 126, 234, 0.4);
}

.btn-secondary {
  background: rgba(255, 255, 255, 0.1);
  color: white;
  border: 1px solid rgba(255, 255, 255, 0.2);
}

.btn-secondary:hover {
  background: rgba(255, 255, 255, 0.2);
}

.btn-large {
  padding: 16px 32px;
  font-size: 18px;
}

.btn-small {
  padding: 8px 16px;
  font-size: 14px;
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.right-panel {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.track-info {
  background: rgba(255, 255, 255, 0.05);
  padding: 24px;
  border-radius: 12px;
  backdrop-filter: blur(10px);
}

.track-title {
  font-size: 32px;
  font-weight: 700;
  margin: 0 0 8px 0;
  color: white;
}

.track-artist {
  font-size: 24px;
  font-weight: 500;
  margin: 0 0 8px 0;
  color: rgba(255, 255, 255, 0.8);
}

.track-album {
  font-size: 18px;
  font-weight: 400;
  margin: 0 0 16px 0;
  color: rgba(255, 255, 255, 0.6);
}

.track-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
  margin-bottom: 20px;
}

.meta-item {
  color: rgba(255, 255, 255, 0.7);
  font-size: 14px;
}

.meta-item strong {
  color: rgba(255, 255, 255, 0.9);
}

.progress-bar {
  position: relative;
  height: 4px;
  background: rgba(255, 255, 255, 0.2);
  border-radius: 2px;
  overflow: hidden;
  margin-bottom: 8px;
}

.progress {
  height: 100%;
  background: linear-gradient(90deg, #667eea 0%, #764ba2 100%);
  border-radius: 2px;
  transition: width 0.1s ease;
}

.time {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.6);
  text-align: center;
}

.track-details-section {
  background: rgba(255, 255, 255, 0.05);
  padding: 24px;
  border-radius: 12px;
  backdrop-filter: blur(10px);
}

.track-details-section h3 {
  margin: 0 0 16px 0;
  color: white;
  font-size: 18px;
  font-weight: 600;
}

.info-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 12px;
}

.info-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.label {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.6);
  font-weight: 500;
}

.value {
  font-size: 14px;
  color: rgba(255, 255, 255, 0.9);
  word-break: break-word;
}

.lyrics-section {
  background: rgba(255, 255, 255, 0.05);
  padding: 24px;
  border-radius: 12px;
  backdrop-filter: blur(10px);
  margin-top: 20px;
}

.lyrics-section h3 {
  margin: 0 0 16px 0;
  color: white;
  font-size: 18px;
  font-weight: 600;
}

.lyrics-controls {
  display: flex;
  gap: 12px;
  margin-bottom: 20px;
  flex-wrap: wrap;
}

.no-track {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 400px;
  color: rgba(255, 255, 255, 0.6);
}

.no-track .icon {
  font-size: 64px;
  margin-bottom: 16px;
}

.error-message {
  background: rgba(255, 0, 0, 0.1);
  color: #ff6b6b;
  padding: 12px;
  border-radius: 8px;
  margin: 20px 0;
  border: 1px solid rgba(255, 107, 107, 0.3);
}

/* 响应式设计 */
@media (max-width: 768px) {
  .track-main-content {
    grid-template-columns: 1fr;
    gap: 20px;
  }
  
  .left-panel {
    align-items: center;
  }
  
  .album-art-container {
    width: 200px;
    height: 200px;
  }
  
  .track-title {
    font-size: 24px;
  }
  
  .track-artist {
    font-size: 18px;
  }
  
  .info-grid {
    grid-template-columns: 1fr;
  }
  
  .lyrics-controls {
    justify-content: center;
  }
}

@media (max-width: 480px) {
  .track-detail {
    padding: 16px;
  }
  
  .album-art-container {
    width: 150px;
    height: 150px;
  }
  
  .track-title {
    font-size: 20px;
  }
  
  .track-artist {
    font-size: 16px;
  }
}
</style>