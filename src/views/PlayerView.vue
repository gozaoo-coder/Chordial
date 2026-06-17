<template>
  <div class="player-view">
    <!-- AMLL 播放器容器 -->
    <AMLLPlayerContainer
      ref="playerContainerRef"
      :album-cover="albumCoverUrl"
      :is-video="false"
      :track-name="currentTrack?.title || 'Unknown Track'"
      :artists="currentTrackArtists"
      :album-name="currentTrack?.album?.title || ''"
      :playing="isPlaying"
      :current-time="currentTimeMs"
      :low-freq-volume="lowFreqVolume"
      :lyric-lines="processedLyricLines"
      :render-scale="0.75"
      :fps="30"
      :flow-speed="4"
      :static-mode="false"
      :lyric-align-position="0.5"
      :lyric-align-anchor="'center'"
      :enable-blur="true"
      :enable-scale="true"
      :enable-spring="true"
      :word-fade-width="0.5"
      :hide-passed-lines="false"
      :lyric-font-size="32"
      :left-section-ratio="0.4"
      :vertical-layout="isMobile"
      @background-ready="onBackgroundReady"
      @lyric-ready="onLyricReady"
      @seek-to-line="onSeekToLine"
      @lyric-context-menu="onLyricContextMenu"
      @error="onPlayerError"
    >
      <!-- 自定义左侧内容插槽（可选） -->
      <template #left>
        <!-- 使用默认内容，或自定义 -->
      </template>
      
      <!-- 底部控制栏插槽 -->
      <template #controls>
        <PlayerControlBar
          :current-time="currentTime"
          :duration="duration"
          :is-playing="isPlaying"
          :volume="volume"
          :play-mode="playMode"
          @play="play"
          @pause="pause"
          @seek="seek"
          @volume-change="setVolume"
          @toggle-play-mode="togglePlayMode"
          @previous="playPrevious"
          @next="playNext"
        />
      </template>
    </AMLLPlayerContainer>
  </div>
</template>

<script setup>
/**
 * PlayerView - 播放器页面
 * 
 * 使用 AMLLPlayerContainer 组件实现完整的歌词播放界面
 * 包含背景渲染、歌词显示、播放控制等功能
 */
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { PlayerStore } from '@/stores/player.js'
import { AMLLPlayerContainer } from '@/components/player'
import PlayerControlBar from '@/components/player/PlayerControlBar.vue'

// 从 PlayerStore 获取状态
const { state, hasCurrentTrack, progress } = PlayerStore

// 组件引用
const playerContainerRef = ref(null)

// 响应式状态
const isMobile = ref(window.innerWidth < 768)

// 计算属性 - 歌曲信息
const currentTrack = computed(() => state.currentTrack)

const currentTrackArtists = computed(() => {
  if (!state.currentTrack?.artists) return []
  return state.currentTrack.artists.map(artist => ({
    name: artist.name || artist
  }))
})

const albumCoverUrl = computed(() => {
  if (!state.currentTrack?.album?.cover) return null
  return state.currentTrack.album.cover
})

// 计算属性 - 播放状态
const isPlaying = computed(() => state.isPlaying)
const currentTime = computed(() => state.currentTime)
const currentTimeMs = computed(() => state.currentTime * 1000)
const duration = computed(() => state.duration)
const volume = computed(() => state.volume)
const playMode = computed(() => state.playMode)

// 计算属性 - 歌词处理
const processedLyricLines = computed(() => {
  // 如果有同步歌词，解析并转换为 AMLL 格式
  if (state.lyricsData.hasSyncedLyrics && state.lyricsData.syncedLyrics) {
    return parseSyncedLyrics(state.lyricsData.syncedLyrics)
  }
  
  // 如果有普通歌词，转换为非逐词格式
  if (state.lyricsData.hasPlainLyrics && state.lyricsData.plainLyrics) {
    return parsePlainLyrics(state.lyricsData.plainLyrics)
  }
  
  return []
})

// 低频音量（用于背景脉动效果）
const lowFreqVolume = ref(0)

// 解析同步歌词（YRC/LRC 格式）
function parseSyncedLyrics(lyricsText) {
  const lines = []
  const lineRegex = /\[(\d{2}):(\d{2})\.(\d{2,3})\](.*)/g
  
  let match
  while ((match = lineRegex.exec(lyricsText)) !== null) {
    const minutes = parseInt(match[1])
    const seconds = parseInt(match[2])
    const milliseconds = parseInt(match[3].padEnd(3, '0'))
    const time = (minutes * 60 + seconds) * 1000 + milliseconds
    const text = match[4].trim()
    
    if (text) {
      // 简单处理：将整个行作为一个单词
      // 实际使用时可以进一步解析逐词时间戳
      lines.push({
        words: [{
          word: text,
          startTime: time,
          endTime: time + 2000, // 默认持续2秒
          romanWord: '',
          obscene: false
        }],
        startTime: time,
        endTime: time + 2000,
        translatedLyric: '',
        romanLyric: '',
        isBG: false,
        isDuet: false
      })
    }
  }
  
  return lines
}

// 解析普通歌词
function parsePlainLyrics(lyricsText) {
  const lines = lyricsText.split('\n').filter(line => line.trim())
  
  return lines.map((text, index) => {
    const startTime = index * 3000 // 每行默认3秒
    return {
      words: [{
        word: text.trim(),
        startTime: startTime,
        endTime: startTime + 3000,
        romanWord: '',
        obscene: false
      }],
      startTime: startTime,
      endTime: startTime + 3000,
      translatedLyric: '',
      romanLyric: '',
      isBG: false,
      isDuet: false
    }
  })
}

// 播放控制方法
const play = () => PlayerStore.play(state.currentTrack)
const pause = () => PlayerStore.pause()
const seek = (time) => PlayerStore.seek(time)
const setVolume = (vol) => PlayerStore.setVolume(vol)
const togglePlayMode = () => PlayerStore.togglePlayMode()
const playPrevious = () => PlayerStore.playPrevious()
const playNext = () => PlayerStore.playNext()

// 事件处理
const onBackgroundReady = (renderer) => {
  console.log('[PlayerView] 背景渲染器准备就绪')
}

const onLyricReady = (player) => {
  console.log('[PlayerView] 歌词播放器准备就绪')
}

const onSeekToLine = (event) => {
  // 点击歌词行跳转到对应时间
  if (event.line && event.line.startTime !== undefined) {
    const timeInSeconds = event.line.startTime / 1000
    PlayerStore.seek(timeInSeconds)
  }
}

const onLyricContextMenu = (event) => {
  console.log('[PlayerView] 歌词右键菜单:', event)
  // 可以在这里实现右键菜单功能
}

const onPlayerError = (error) => {
  console.error('[PlayerView] 播放器错误:', error)
}

// 窗口大小变化处理
const handleResize = () => {
  isMobile.value = window.innerWidth < 768
}

// 模拟低频音量更新（实际应该从音频分析器获取）
let volumeUpdateInterval = null
const startVolumeSimulation = () => {
  volumeUpdateInterval = setInterval(() => {
    if (isPlaying.value) {
      // 模拟低频音量波动
      lowFreqVolume.value = 0.3 + Math.random() * 0.4
    } else {
      lowFreqVolume.value = 0
    }
  }, 100)
}

const stopVolumeSimulation = () => {
  if (volumeUpdateInterval) {
    clearInterval(volumeUpdateInterval)
    volumeUpdateInterval = null
  }
}

// 生命周期
onMounted(() => {
  window.addEventListener('resize', handleResize)
  startVolumeSimulation()
})

onUnmounted(() => {
  window.removeEventListener('resize', handleResize)
  stopVolumeSimulation()
})
</script>

<style scoped>
.player-view {
  width: 100%;
  height: 100%;
  position: relative;
  overflow: hidden;
}
</style>
