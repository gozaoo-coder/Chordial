<template>
  <Teleport to="body">
    <div ref="root" class="player-view" :class="{ 'is-desktop': isDesktop, 'is-immersive': isImmersive }"
      @keydown="onKeydown" @pointerdown="registerInteraction" tabindex="0">
      <!-- 模糊封面背景 -->
      <div class="player-bg">
        <transition :css="false" @enter="onBgEnter" @leave="onBgLeave">
          <img v-if="coverUrl" :key="coverBgKey" :src="coverUrl" class="bg-img" />
        </transition>
        <div class="bg-mask"></div>
      </div>

      <div class="player-body" ref="playerBody">
        <!-- 顶栏 -->
        <header class="top-bar immersive-hide">
          <button class="btn-icon" @click="handleClose" title="返回">
            <i class="bi bi-chevron-down"></i>
          </button>
          <span class="top-title">正在播放</span>
          <button v-if="currentTrack" class="btn-icon" @click="toggleQueueMode" title="播放列表">
            <i class="bi bi-music-note-list"></i>
          </button>
        </header>

        <!-- 空状态 -->
        <div v-if="!currentTrack" class="empty-state">
          <i class="bi bi-music-note-beamed empty-icon"></i>
          <p class="empty-text">选择一首歌曲开始播放</p>
        </div>

        <!-- ── 模式内容区 ── -->
        <template v-else>
          <!-- 移动端 -->
          <div v-show="!isDesktop" class="mode-content mobile-mode">
            <transition :css="false" mode="out-in" @enter="onModeEnter" @leave="onModeLeave">
              <!-- 详细信息模式 -->
              <div v-if="mode === 'details'" key="details" class="mode-details">
                <div class="details-cover" data-layout-id="cover">
                  <img v-if="coverUrl" :src="coverUrl" alt="" />
                  <i v-else class="bi bi-disc-fill cover-fallback"></i>
                </div>
                <div class="details-info" data-layout-id="meta">
                  <h2 class="details-title">{{ currentTrack.title }}</h2>
                  <p class="details-artist">{{ currentTrack.artist }}</p>
                  <p v-if="currentTrack.albumTitle" class="details-album">专辑：{{ currentTrack.albumTitle }}</p>
                </div>
                <div class="details-specs">
                  <div class="spec-item" v-if="currentTrack.sampleRate">
                    <span class="spec-label">采样率</span>
                    <span class="spec-value">{{ (currentTrack.sampleRate / 1000).toFixed(1) }} kHz</span>
                  </div>
                  <div class="spec-item" v-if="currentTrack.channels">
                    <span class="spec-label">声道</span>
                    <span class="spec-value">{{ currentTrack.channels === 2 ? '立体声' : currentTrack.channels + ' 声道' }}</span>
                  </div>
                  <div class="spec-item" v-if="currentTrack.formatName">
                    <span class="spec-label">格式</span>
                    <span class="spec-value">{{ currentTrack.formatName.toUpperCase() }}</span>
                  </div>
                  <div class="spec-item" v-if="duration">
                    <span class="spec-label">时长</span>
                    <span class="spec-value">{{ formattedDuration }}</span>
                  </div>
                  <div class="spec-item" v-if="currentTrack.sourceIds?.length">
                    <span class="spec-label">来源</span>
                    <span class="spec-value">{{ currentTrack.sourceIds.length }} 个音乐源</span>
                  </div>
                </div>
              </div>

              <!-- 常规模式 -->
              <div v-else-if="mode === 'regular'" key="regular" class="mode-regular">
                <!-- 滑动引导线 -->
                <div class="swipe-hint">
                  <i class="bi bi-chevron-left"></i>
                  <span>左右滑动切换模式</span>
                  <i class="bi bi-chevron-right"></i>
                </div>
                <!-- 专辑大图 -->
                <div class="cover-section">
                  <transition :css="false" mode="out-in" @enter="onCoverEnter" @leave="onCoverLeave">
                    <div class="cover-art" :key="currentTrack.id" data-layout-id="cover">
                      <img v-if="coverUrl" :src="coverUrl" alt="" />
                      <i v-else class="bi bi-disc-fill cover-fallback"></i>
                    </div>
                  </transition>
                </div>
                <!-- 音乐信息 bar -->
                <transition :css="false" mode="out-in" @enter="onMetaEnter" @leave="onMetaLeave">
                  <div class="track-meta" :key="currentTrack.id" data-layout-id="meta">
                    <h1 class="track-title">{{ currentTrack.title }}</h1>
                    <p class="track-artist">{{ currentTrack.artist }}</p>
                    <p v-if="currentTrack.albumTitle" class="track-album">{{ currentTrack.albumTitle }}</p>
                  </div>
                </transition>
                <!-- 进度条 + 控制按钮 + 功能控件 -->
                <div class="regular-controls" data-layout-id="controls">
                  <div class="progress-area">
                    <span class="time-label">{{ formattedCurrentTime }}</span>
                    <div class="progress-bar"
                      @mousedown="startSeek" @touchstart="startSeek">
                      <div class="progress-bg"></div>
                      <div class="progress-fill" :style="{ width: progressPercent + '%' }"></div>
                      <div class="progress-knob" :style="{ left: progressPercent + '%' }"></div>
                    </div>
                    <span class="time-label time-remaining">{{ formattedRemaining }}</span>
                  </div>
                  <div class="btn-row">
                    <button class="ctrl-btn" @click="PlayerStore.playPrevious" title="上一首">
                      <i class="bi bi-skip-start-fill"></i>
                    </button>
                    <button class="ctrl-btn ctrl-play" @click="PlayerStore.togglePlay" title="播放 / 暂停">
                      <i :class="isPlaying ? 'bi bi-pause-circle-fill' : 'bi bi-play-circle-fill'"></i>
                    </button>
                    <button class="ctrl-btn" @click="PlayerStore.playNext" title="下一首">
                      <i class="bi bi-skip-end-fill"></i>
                    </button>
                  </div>
                  <!-- 五个功能控件 -->
                  <div class="widget-row immersive-hide">
                    <button class="widget-btn" :class="{ on: playMode !== PlayMode.SEQUENCE }"
                      @click="PlayerStore.togglePlayMode" title="播放模式">
                      <i :class="playModeIcon"></i>
                      <span class="widget-label">模式</span>
                    </button>
                    <button class="widget-btn" :class="{ on: isMuted }"
                      @click="PlayerStore.toggleMute" title="静音">
                      <i :class="volumeIcon"></i>
                      <span class="widget-label">音量</span>
                    </button>
                    <button v-if="hasSyncedLyrics || hasPlainLyrics" class="widget-btn"
                      :class="{ on: mode === 'lyrics' }" @click="setMode('lyrics')" title="歌词">
                      <i class="bi bi-mic-fill"></i>
                      <span class="widget-label">歌词</span>
                    </button>
                    <button class="widget-btn" @click="setMode('details')" title="详情">
                      <i class="bi bi-info-circle"></i>
                      <span class="widget-label">详情</span>
                    </button>
                    <button class="widget-btn" @click="toggleQueueMode" title="歌单">
                      <i class="bi bi-list-ul"></i>
                      <span class="widget-label">歌单</span>
                    </button>
                  </div>
                </div>
              </div>

              <!-- 歌词模式 -->
              <div v-else-if="mode === 'lyrics'" key="lyrics" class="mode-lyrics">
                <!-- 顶部信息 bar（含小封面） -->
                <div class="lyrics-info-bar">
                  <div class="lyrics-cover">
                    <img v-if="coverUrl" :src="coverUrl" alt="" />
                    <i v-else class="bi bi-disc-fill"></i>
                  </div>
                  <div class="lyrics-info-text">
                    <span class="lyrics-track-title">{{ currentTrack.title }}</span>
                    <span class="lyrics-track-artist">{{ currentTrack.artist }}</span>
                  </div>
                </div>
                <!-- 歌词渲染 -->
                <div class="lyrics-render-area" data-layout-id="lyrics">
                  <transition :css="false" mode="out-in" @enter="onLyricsEnter" @leave="onLyricsLeave">
                    <div v-if="hasLyrics" class="lyrics-wrapper" :key="'sync-' + currentTrack.id">
                      <LyricPlayer
                        :lyric-lines="lyricLines"
                        :current-time="currentTimeMs"
                        :playing="isPlaying"
                        align-anchor="center"
                        :align-position="0.5"
                        :enable-spring="true"
                        :enable-blur="false"
                        :enable-scale="true"
                        :word-fade-width="0.5"
                        :hide-passed-lines="false"
                        @line-click="onLineClick"
                      />
                    </div>
                    <div v-else-if="plainLyrics.length > 0" class="plain-lyrics" :key="'plain-' + currentTrack.id">
                      <p v-for="(line, i) in plainLyrics" :key="i" class="plain-line">{{ line }}</p>
                    </div>
                    <div v-else class="no-lyrics" :key="'none-' + currentTrack.id">暂无歌词</div>
                  </transition>
                </div>
              </div>
            </transition>
          </div>

          <!-- 桌面端 -->
          <div v-show="isDesktop" class="mode-content desktop-mode">
            <!-- 左侧：常规控件（始终显示） -->
            <div class="desktop-left">
              <div class="cover-section">
                <transition :css="false" mode="out-in" @enter="onCoverEnter" @leave="onCoverLeave">
                  <div class="cover-art" :key="currentTrack.id" data-layout-id="cover">
                    <img v-if="coverUrl" :src="coverUrl" alt="" />
                    <i v-else class="bi bi-disc-fill cover-fallback"></i>
                  </div>
                </transition>
              </div>
              <transition :css="false" mode="out-in" @enter="onMetaEnter" @leave="onMetaLeave">
                <div class="track-meta" :key="currentTrack.id" data-layout-id="meta">
                  <h1 class="track-title">{{ currentTrack.title }}</h1>
                  <p class="track-artist">{{ currentTrack.artist }}</p>
                  <p v-if="currentTrack.albumTitle" class="track-album">{{ currentTrack.albumTitle }}</p>
                </div>
              </transition>
              <div class="regular-controls" :class="{ 'desktop-centered': mode === 'info' }" data-layout-id="controls">
                <div class="progress-area">
                  <span class="time-label">{{ formattedCurrentTime }}</span>
                  <div class="progress-bar"
                    @mousedown="startSeek" @touchstart="startSeek">
                    <div class="progress-bg"></div>
                    <div class="progress-fill" :style="{ width: progressPercent + '%' }"></div>
                    <div class="progress-knob" :style="{ left: progressPercent + '%' }"></div>
                  </div>
                  <span class="time-label time-remaining">{{ formattedRemaining }}</span>
                </div>
                <div class="btn-row">
                  <button class="ctrl-btn" @click="PlayerStore.playPrevious" title="上一首">
                    <i class="bi bi-skip-start-fill"></i>
                  </button>
                  <button class="ctrl-btn ctrl-play" @click="PlayerStore.togglePlay" title="播放 / 暂停">
                    <i :class="isPlaying ? 'bi bi-pause-circle-fill' : 'bi bi-play-circle-fill'"></i>
                  </button>
                  <button class="ctrl-btn" @click="PlayerStore.playNext" title="下一首">
                    <i class="bi bi-skip-end-fill"></i>
                  </button>
                </div>
                <div class="widget-row immersive-hide">
                  <button class="widget-btn" :class="{ on: playMode !== PlayMode.SEQUENCE }"
                    @click="PlayerStore.togglePlayMode" title="播放模式">
                    <i :class="playModeIcon"></i>
                    <span class="widget-label">模式</span>
                  </button>
                  <button class="widget-btn" :class="{ on: isMuted }"
                    @click="PlayerStore.toggleMute" title="静音">
                    <i :class="volumeIcon"></i>
                    <span class="widget-label">音量</span>
                  </button>
                  <button v-if="hasSyncedLyrics || hasPlainLyrics" class="widget-btn"
                    :class="{ on: mode === 'lyrics' }" @click="setMode('lyrics')" title="歌词">
                    <i class="bi bi-mic-fill"></i>
                    <span class="widget-label">歌词</span>
                  </button>
                  <button class="widget-btn" @click="setMode('info')" title="信息">
                    <i class="bi bi-info-circle"></i>
                    <span class="widget-label">信息</span>
                  </button>
                  <button class="widget-btn" :class="{ on: mode === 'playlist' }"
                    @click="setMode('playlist')" title="歌单">
                    <i class="bi bi-list-ul"></i>
                    <span class="widget-label">歌单</span>
                  </button>
                </div>
              </div>
            </div>

            <!-- 右侧：歌词 / 歌单（info 模式时隐藏） -->
            <transition :css="false" mode="out-in" @enter="onModeEnter" @leave="onModeLeave">
              <div v-if="mode === 'lyrics'" key="lyrics" class="desktop-right desktop-lyrics" data-layout-id="lyrics">
                <transition :css="false" mode="out-in" @enter="onLyricsEnter" @leave="onLyricsLeave">
                  <div v-if="hasLyrics" class="lyrics-wrapper" :key="'sync-' + currentTrack.id">
                    <LyricPlayer
                      :lyric-lines="lyricLines"
                      :current-time="currentTimeMs"
                      :playing="isPlaying"
                      align-anchor="center"
                      :align-position="0.5"
                      :enable-spring="true"
                      :enable-blur="false"
                      :enable-scale="true"
                      :word-fade-width="0.5"
                      :hide-passed-lines="false"
                      @line-click="onLineClick"
                    />
                  </div>
                  <div v-else-if="plainLyrics.length > 0" class="plain-lyrics" :key="'plain-' + currentTrack.id">
                    <p v-for="(line, i) in plainLyrics" :key="i" class="plain-line">{{ line }}</p>
                  </div>
                  <div v-else class="no-lyrics" :key="'none-' + currentTrack.id">暂无歌词</div>
                </transition>
              </div>
              <div v-else-if="mode === 'playlist'" key="playlist" class="desktop-right desktop-playlist">
                <h3 class="playlist-title">播放列表</h3>
                <div class="playlist-items">
                  <div v-for="(track, i) in playlist" :key="track.id"
                    class="playlist-item" :class="{ active: i === currentIndex }"
                    @click="PlayerStore.play(track)">
                    <span class="playlist-item-title">{{ track.title }}</span>
                    <span class="playlist-item-artist">{{ track.artist }}</span>
                  </div>
                  <div v-if="playlist.length === 0" class="playlist-empty">播放列表为空</div>
                </div>
              </div>
            </transition>
          </div>
        </template>

        <!-- 模式切换 bar（始终在最底端） -->
        <nav v-if="currentTrack" class="mode-switch-bar immersive-hide">
          <button v-for="m in modeOptions" :key="m.value"
            class="switch-btn" :class="{ active: mode === m.value }"
            @click="setMode(m.value)">
            <i :class="m.icon"></i>
            <span>{{ m.label }}</span>
          </button>
        </nav>
      </div>
    </div>
  </Teleport>
</template>

<script setup>
import { ref, computed, onMounted, onBeforeUnmount, watch, useTemplateRef, nextTick } from 'vue'
import { LyricPlayer } from '@applemusic-like-lyrics/vue'
import { PlayerStore, PlayMode } from '@/stores/player.js'
import { useCoverImage } from '@/composables/useCoverImage'
import { useLyricParser } from '@/composables/useLyricParser'
import { usePerf } from '@/utils/performanceMonitor.js'
import { useAnime } from '@/composables/useAnime.js'
import { ANIME_SPRINGS } from '@/utils/animePresets.js'

const { log } = usePerf('PlayerView')

const rootRef = useTemplateRef('root')
const playerBodyRef = useTemplateRef('playerBody')
const coverBgKey = ref(0)
let seekDragging = false

const { animate, enter, exit, spring, run, useLayout } = useAnime(() => rootRef.value)

// ── player state ──
const currentTrack = computed(() => PlayerStore.state.currentTrack)
const isPlaying = computed(() => PlayerStore.state.isPlaying)
const currentTime = computed(() => PlayerStore.state.currentTime)
const duration = computed(() => PlayerStore.state.duration)
const volume = computed(() => PlayerStore.state.volume)
const isMuted = computed(() => PlayerStore.state.muted)
const playMode = computed(() => PlayerStore.state.playMode)
const playlist = computed(() => PlayerStore.state.playlist)
const currentIndex = computed(() => PlayerStore.state.currentIndex)
const isDesktop = computed(() => PlayerStore.state.ui.currentDevice === 'desktop')
const isImmersive = computed(() => PlayerStore.state.ui.isImmersive)
const mode = computed(() => PlayerStore.playerViewMode.value)
const { coverUrl } = useCoverImage(currentTrack, 'large')

// ── mode options ──
const modeOptions = computed(() => {
  if (isDesktop.value) {
    return [
      { value: 'info', label: '信息', icon: 'bi bi-info-circle' },
      { value: 'lyrics', label: '歌词', icon: 'bi bi-mic-fill' },
      { value: 'playlist', label: '歌单', icon: 'bi bi-list-ul' },
    ]
  }
  return [
    { value: 'details', label: '详情', icon: 'bi bi-info-circle' },
    { value: 'regular', label: '常规', icon: 'bi bi-music-note' },
    { value: 'lyrics', label: '歌词', icon: 'bi bi-mic-fill' },
  ]
})

// ── time ──
const currentTimeMs = computed(() => Math.floor(currentTime.value * 1000))
const progressPercent = computed(() =>
  duration.value ? Math.min((currentTime.value / duration.value) * 100, 100) : 0
)
const formattedCurrentTime = computed(() => PlayerStore.formattedCurrentTime.value)
const formattedDuration = computed(() => PlayerStore.formattedDuration.value)
const formattedRemaining = computed(() => {
  const d = duration.value
  if (!d || isNaN(d)) return '0:00'
  const remain = Math.max(0, d - currentTime.value)
  const m = Math.floor(remain / 60)
  const s = Math.floor(remain % 60)
  return `-${m}:${s.toString().padStart(2, '0')}`
})

// ── lyrics ──
const syncedLyrics = computed(() => PlayerStore.state.lyricsData.syncedLyrics)
const hasSyncedLyrics = computed(() => PlayerStore.state.lyricsData.hasSyncedLyrics)
const hasPlainLyrics = computed(() => PlayerStore.state.lyricsData.hasPlainLyrics)
const lyricLines = useLyricParser(syncedLyrics)
const hasLyrics = computed(() => hasSyncedLyrics.value && lyricLines.value.length > 0)
const plainLyrics = computed(() => {
  if (!hasPlainLyrics.value) return []
  return PlayerStore.state.lyricsData.plainLyrics.split(/\r?\n/).filter(l => l.trim())
})

// ── icons ──
const playModeIcon = computed(() => {
  switch (playMode.value) {
    case PlayMode.RANDOM: return 'bi bi-shuffle'
    case PlayMode.LOOP: return 'bi bi-repeat'
    case PlayMode.LOOP_ONE: return 'bi bi-repeat-1'
    default: return 'bi bi-arrow-repeat'
  }
})
const volumeIcon = computed(() => {
  if (isMuted.value || volume.value === 0) return 'bi bi-volume-mute-fill'
  if (volume.value < 0.3) return 'bi bi-volume-off-fill'
  if (volume.value < 0.7) return 'bi bi-volume-down-fill'
  return 'bi bi-volume-up-fill'
})

// ── actions ──

function setMode(m) {
  PlayerStore.setPlayerViewMode(m)
}

function toggleQueueMode() {
  if (isDesktop.value) {
    setMode(mode.value === 'playlist' ? 'info' : 'playlist')
  } else {
    setMode('lyrics')
  }
}

function handleClose() {
  // 播放退场动画后关闭
  playExit().then(() => {
    PlayerStore.closePlayerView()
  })
}

function onLineClick(e) {
  const t = e?.line?.startTime
  if (t !== undefined) {
    PlayerStore.seek(t / 1000)
  }
}

function registerInteraction() {
  PlayerStore.registerInteraction()
}

// ── keyboard ──
function onKeydown(e) {
  if (e.target.tagName === 'INPUT') return
  switch (e.code) {
    case 'Space':
      e.preventDefault()
      PlayerStore.togglePlay()
      break
    case 'ArrowLeft':
      e.preventDefault()
      PlayerStore.seek(Math.max(0, currentTime.value - 5))
      break
    case 'ArrowRight':
      e.preventDefault()
      PlayerStore.seek(Math.min(duration.value || Infinity, currentTime.value + 5))
      break
    case 'ArrowUp':
      e.preventDefault()
      PlayerStore.setVolume(Math.min(1, volume.value + 0.05))
      break
    case 'ArrowDown':
      e.preventDefault()
      PlayerStore.setVolume(Math.max(0, volume.value - 0.05))
      break
    case 'Escape':
      e.preventDefault()
      handleClose()
      break
  }
  registerInteraction()
}

// ── seek ──
let seekTarget = null
function startSeek(e) {
  e.preventDefault?.()
  seekDragging = true
  seekTarget = e.currentTarget
  updateSeek(e)
  document.addEventListener('mousemove', updateSeek)
  document.addEventListener('mouseup', endSeek)
  document.addEventListener('touchmove', updateSeek, { passive: false })
  document.addEventListener('touchend', endSeek)
}
function updateSeek(e) {
  if (!seekDragging || !seekTarget) return
  const r = seekTarget.getBoundingClientRect()
  const x = e.touches ? e.touches[0].clientX : e.clientX
  const ratio = Math.max(0, Math.min(1, (x - r.left) / r.width))
  PlayerStore.seek(ratio * (duration.value || 0))
}
function endSeek() {
  seekDragging = false
  seekTarget = null
  document.removeEventListener('mousemove', updateSeek)
  document.removeEventListener('mouseup', endSeek)
  document.removeEventListener('touchmove', updateSeek)
  document.removeEventListener('touchend', endSeek)
}

// ── shared element FLIP (封面共享元素动画) ──
// 克隆源 img，从 PlayerControlBar 位置飞到 PlayerView cover-art 位置
function getSourceImg() {
  const src = PlayerStore.state.ui.sharedElementSource
  if (!src) return null
  // PlayerControlBar 的 .album-cover-thumb 内的 img
  if (src.tagName === 'IMG') return src
  return src.querySelector('img') || null
}

function getCoverTargetEl() {
  // 当前可见的 cover-art（移动端 regular / 桌面端 info）
  return rootRef.value?.querySelector('.cover-art') || null
}

function createGhost(img, fromRect) {
  const ghost = img.cloneNode(true)
  ghost.style.position = 'fixed'
  ghost.style.left = fromRect.left + 'px'
  ghost.style.top = fromRect.top + 'px'
  ghost.style.width = fromRect.width + 'px'
  ghost.style.height = fromRect.height + 'px'
  ghost.style.zIndex = '400'
  ghost.style.pointerEvents = 'none'
  ghost.style.margin = '0'
  ghost.style.objectFit = 'cover'
  ghost.style.borderRadius = '12px'
  ghost.style.boxShadow = '0 16px 60px rgba(0,0,0,0.55)'
  document.body.appendChild(ghost)
  return ghost
}

function animateGhost(ghost, fromRect, toRect, duration, ease) {
  const dx = toRect.left - fromRect.left
  const dy = toRect.top - fromRect.top
  const sx = toRect.width / fromRect.width
  const sy = toRect.height / fromRect.height
  return new Promise(resolve => {
    animate(ghost, {
      translateX: [0, dx],
      translateY: [0, dy],
      scaleX: [1, sx],
      scaleY: [1, sy],
      duration,
      ease,
      onComplete: resolve,
    })
  })
}

// ── enter / exit ──
async function playEnter() {
  if (!rootRef.value) return
  const sourceImg = getSourceImg()
  const coverTarget = getCoverTargetEl()

  // 共享元素 FLIP：封面从 PlayerControlBar 飞到 PlayerView
  if (sourceImg && coverTarget) {
    const fromRect = sourceImg.getBoundingClientRect()
    // 隐藏目标 cover，等克隆动画结束再显示
    coverTarget.style.opacity = '0'
    // 创建克隆
    const ghost = createGhost(sourceImg, fromRect)
    // 模态进入：从底部滑入 + 淡入
    animate(rootRef.value, {
      opacity: [0, 1],
      translateY: ['100%', 0],
      duration: 500,
      ease: ANIME_SPRINGS.powerful,
    })
    // 等待 DOM 渲染后获取目标位置
    await nextTick()
    const toRect = coverTarget.getBoundingClientRect()
    // 克隆飞行
    await animateGhost(ghost, fromRect, toRect, 500, ANIME_SPRINGS.powerful)
    // 清理：移除克隆，显示目标
    ghost.remove()
    coverTarget.style.opacity = '1'
    PlayerStore.setPlayerViewTransitioning(false)
  } else {
    // 无共享元素：普通进入
    animate(rootRef.value, {
      opacity: [0, 1],
      translateY: ['100%', 0],
      duration: 500,
      ease: ANIME_SPRINGS.powerful,
      onComplete: () => {
        PlayerStore.setPlayerViewTransitioning(false)
      },
    })
  }
}

async function playExit() {
  if (!rootRef.value) return
  PlayerStore.setPlayerViewTransitioning(true)
  const sourceImg = getSourceImg()
  const coverTarget = getCoverTargetEl()

  // 共享元素 FLIP：封面从 PlayerView 飞回 PlayerControlBar
  if (sourceImg && coverTarget) {
    const fromRect = coverTarget.getBoundingClientRect()
    const toRect = sourceImg.getBoundingClientRect()
    // 隐藏目标 cover
    coverTarget.style.opacity = '0'
    // 创建克隆
    const ghost = createGhost(sourceImg, fromRect)
    // 模态退出：向下滑出 + 淡出
    animate(rootRef.value, {
      opacity: [1, 0],
      translateY: [0, '100%'],
      duration: 350,
      ease: ANIME_SPRINGS.sensitive,
    })
    // 克隆飞回
    await animateGhost(ghost, fromRect, toRect, 350, ANIME_SPRINGS.sensitive)
    ghost.remove()
  } else {
    // 无共享元素：普通退出
    await new Promise((resolve) => {
      animate(rootRef.value, {
        opacity: [1, 0],
        translateY: [0, '100%'],
        duration: 350,
        ease: ANIME_SPRINGS.sensitive,
        onComplete: resolve,
      })
    })
  }
}

// ── transition hooks ──
function onBgEnter(el, done) {
  animate(el, { opacity: [0, 1], duration: 600, ease: ANIME_SPRINGS.default, onComplete: done })
}
function onBgLeave(el, done) {
  animate(el, { opacity: [1, 0], duration: 600, ease: ANIME_SPRINGS.default, onComplete: done })
}
function onCoverEnter(el, done) {
  animate(el, { opacity: [0, 1], scale: [0.95, 1], duration: 250, ease: ANIME_SPRINGS.bouncy, onComplete: done })
}
function onCoverLeave(el, done) {
  animate(el, { opacity: [1, 0], scale: [1, 1.02], duration: 150, ease: ANIME_SPRINGS.sensitive, onComplete: done })
}
function onMetaEnter(el, done) {
  animate(el, { opacity: [0, 1], translateY: [6, 0], duration: 250, delay: 50, ease: ANIME_SPRINGS.default, onComplete: done })
}
function onMetaLeave(el, done) {
  animate(el, { opacity: [1, 0], duration: 100, ease: ANIME_SPRINGS.sensitive, onComplete: done })
}
function onLyricsEnter(el, done) {
  animate(el, { opacity: [0, 1], duration: 300, ease: ANIME_SPRINGS.default, onComplete: done })
}
function onLyricsLeave(el, done) {
  animate(el, { opacity: [1, 0], duration: 150, ease: ANIME_SPRINGS.sensitive, onComplete: done })
}
function onModeEnter(el, done) {
  animate(el, { opacity: [0, 1], duration: 300, ease: ANIME_SPRINGS.default, onComplete: done })
}
function onModeLeave(el, done) {
  animate(el, { opacity: [1, 0], duration: 150, ease: ANIME_SPRINGS.sensitive, onComplete: done })
}

// ── immersive mode animation ──
// 用 anime.js 替代 CSS transition，实现 spring 物理的沉浸模式进出动画
watch(isImmersive, (immersive) => {
  if (!rootRef.value) return
  const targets = rootRef.value.querySelectorAll('.immersive-hide')
  if (targets.length === 0) return
  if (immersive) {
    // 进入沉浸：隐藏控件（灵敏弹簧，快速响应）
    animate(targets, {
      opacity: [1, 0],
      duration: 280,
      ease: ANIME_SPRINGS.sensitive,
      onComplete: () => {
        targets.forEach(t => { t.style.pointerEvents = 'none' })
      },
    })
  } else {
    // 退出沉浸：显示控件（弹跳弹簧，轻快回归）
    targets.forEach(t => { t.style.pointerEvents = '' })
    animate(targets, {
      opacity: [0, 1],
      duration: 350,
      ease: ANIME_SPRINGS.bouncy,
    })
  }
})

// ── draggable 手势 (左右滑模式切换 + 上滑歌单 + 下滑退出) ──
let dragInstance = null
let lastDragX = 0
let lastDragY = 0
const SWIPE_THRESHOLD = 60 // 最小滑动距离 (px)

function setupDraggable() {
  if (!playerBodyRef.value || !rootRef.value) return
  run(({ createDraggable }) => {
    dragInstance = createDraggable(playerBodyRef.value, {
      container: rootRef.value,   // 限制在 player-view 内，防止拖出边界
      x: true,
      y: true,
      snap: { x: [0], y: [0] },   // 总是回弹到原位
      releaseEase: ANIME_SPRINGS.bouncy,
      containerFriction: 0.6,     // 拖到边界时的阻力
      onDrag: (state) => {
        lastDragX = state.x
        lastDragY = state.y
      },
      onSettle: () => {
        const dx = lastDragX
        const dy = lastDragY
        const absDx = Math.abs(dx)
        const absDy = Math.abs(dy)

        // 主方向判断：水平优先（模式切换），其次垂直（歌单/退出）
        if (absDx > absDy && absDx > SWIPE_THRESHOLD) {
          swipeMode(dx > 0 ? -1 : 1)
        } else if (absDy > SWIPE_THRESHOLD) {
          if (dy > 0) {
            // 下滑：退出 player-view
            handleClose()
          } else {
            // 上滑：歌单（移动端切歌词，桌面端切歌单）
            swipeUpAction()
          }
        }
        lastDragX = 0
        lastDragY = 0
      },
    })
  })
}

function swipeMode(direction) {
  const modes = isDesktop.value
    ? ['info', 'lyrics', 'playlist']
    : ['details', 'regular', 'lyrics']
  const idx = modes.indexOf(mode.value)
  if (idx < 0) return
  const newIdx = Math.max(0, Math.min(modes.length - 1, idx + direction))
  if (newIdx !== idx) setMode(modes[newIdx])
}

function swipeUpAction() {
  if (isDesktop.value) setMode('playlist')
  else setMode('lyrics')
}

// ── cross-device createLayout FLIP ──
// 使用 anime.js v4 createLayout + data-layout-id 实现跨设备模式切换的父元素交换动画。
// 移动端和桌面端共享组件（封面/元信息/控件/歌词）标记 data-layout-id，
// v-show 切换 display 时 createLayout 自动检测配对元素并 FLIP 动画。
const deviceLayout = useLayout(
  () => playerBodyRef.value,
  {
    children: '[data-layout-id]',
    duration: 600,
    ease: ANIME_SPRINGS.powerful,
  }
)

// 设备切换：record() 记录旧布局 → nextTick 等 Vue 更新 v-show → animate() FLIP
watch(isDesktop, async () => {
  deviceLayout.record()
  await nextTick()
  deviceLayout.animate()
})

// ── lifecycle ──
onMounted(() => {
  // 曲目切换 → 背景 key 递增 → 触发 crossfade
  nextTick(() => {
    playEnter()
    rootRef.value?.focus()
    setupDraggable()
  })
})

onBeforeUnmount(() => {
  endSeek()
  // createDraggable 由 useAnime 的 onScopeDispose 自动 revert，无需手动清理
  dragInstance = null
})

// 曲目切换 → 背景 key 递增
watch(() => currentTrack.value?.id, (newId) => {
  log('trackSwitch', { id: newId })
  coverBgKey.value++
})
</script>

<style scoped>
/* ── container ── */
.player-view {
  position: fixed; inset: 0; z-index: 300;
  display: flex; flex-direction: column;
  background: #0b0b0e;
  outline: none;
  overflow: hidden;
  opacity: 0;
}

/* ── background ── */
.player-bg { position: absolute; inset: 0; overflow: hidden; contain: strict; }
.bg-img {
  position: absolute; inset: -60px;
  width: calc(100% + 120px); height: calc(100% + 120px);
  object-fit: cover;
  filter: blur(60px) brightness(0.28) saturate(1.8);
  transform: scale(1.05);
  pointer-events: none; user-select: none;
  will-change: transform, filter;
  backface-visibility: hidden;
}
.bg-mask {
  position: absolute; inset: 0;
  background:
    radial-gradient(ellipse at 50% 30%, transparent 0%, rgba(0,0,0,0.35) 70%),
    linear-gradient(to bottom, rgba(0,0,0,0.15), rgba(0,0,0,0.5));
}

/* ── body ── */
.player-body {
  position: relative; z-index: 1;
  display: flex; flex-direction: column;
  width: 100%; height: 100%;
  max-width: 1200px;
  margin: 0 auto;
  padding: 0 32px;
  padding-top: max(12px, var(--safe-area-top, 0px));
  padding-bottom: max(20px, var(--safe-area-bottom, 0px));
}

/* draggable 手势：内部滚动区域允许 pan-y，避免被 createDraggable 捕获垂直拖动 */
.mode-details,
.lyrics-wrapper,
.playlist-items,
.plain-lyrics {
  touch-action: pan-y;
}

/* ── top bar ── */
.top-bar {
  display: flex; align-items: center; justify-content: space-between;
  flex-shrink: 0; padding: 4px 0 12px;
}
.top-title { font-size: 0.8125rem; color: rgba(255,255,255,0.5); letter-spacing: 0.5px; }
.btn-icon {
  width: 36px; height: 36px; border: none; border-radius: 50%;
  background: rgba(255,255,255,0.08); color: rgba(255,255,255,0.65);
  display: flex; align-items: center; justify-content: center;
  cursor: pointer; font-size: 1.15rem;
  transition: background 0.15s, color 0.15s;
}
.btn-icon:hover { background: rgba(255,255,255,0.16); color: rgba(255,255,255,0.9); }

/* ── empty ── */
.empty-state { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; text-align: center; color: rgba(255,255,255,0.4); }
.empty-icon { font-size: 3rem; display: block; margin-bottom: 12px; }
.empty-text { font-size: 0.9375rem; margin: 0; }

/* ── mode content ── */
.mode-content {
  flex: 1; min-height: 0;
  display: flex; flex-direction: column;
}

/* ── mobile: details mode ── */
.mode-details {
  flex: 1; display: flex; flex-direction: column; align-items: center;
  gap: 20px; overflow-y: auto; padding: 8px 0;
}
.details-cover {
  width: min(50vw, 240px); aspect-ratio: 1; border-radius: 12px; overflow: hidden;
  box-shadow: 0 16px 60px rgba(0,0,0,0.55);
}
.details-cover img { width: 100%; height: 100%; object-fit: cover; }
.details-info { text-align: center; }
.details-title { font-size: 1.2rem; font-weight: 700; color: rgba(255,255,255,0.92); margin: 0 0 4px; }
.details-artist { font-size: 0.875rem; color: rgba(255,255,255,0.5); margin: 0 0 2px; }
.details-album { font-size: 0.75rem; color: rgba(255,255,255,0.32); margin: 0; }
.details-specs {
  display: grid; grid-template-columns: repeat(2, 1fr); gap: 12px;
  width: 100%; max-width: 400px;
}
.spec-item {
  background: rgba(255,255,255,0.06); border-radius: 12px; padding: 12px 16px;
  display: flex; flex-direction: column; gap: 4px;
}
.spec-label { font-size: 0.6875rem; color: rgba(255,255,255,0.4); text-transform: uppercase; letter-spacing: 0.5px; }
.spec-value { font-size: 0.9375rem; color: rgba(255,255,255,0.85); font-weight: 500; }

/* ── mobile: regular mode ── */
.mode-regular {
  flex: 1; display: flex; flex-direction: column; align-items: center;
  justify-content: center; gap: 12px; min-height: 0;
}
.swipe-hint {
  display: flex; align-items: center; gap: 6px;
  font-size: 0.6875rem; color: rgba(255,255,255,0.25);
  user-select: none;
}
.swipe-hint i { font-size: 0.625rem; }
.cover-section {
  flex-shrink: 0; display: flex; flex-direction: column; align-items: center;
}
.cover-art {
  width: min(48vw, 240px); aspect-ratio: 1;
  border-radius: 12px; overflow: hidden;
  box-shadow: 0 16px 60px rgba(0,0,0,0.55);
  background: rgba(255,255,255,0.03);
}
.cover-art img { width: 100%; height: 100%; object-fit: cover; display: block; }
.cover-fallback {
  width: 100%; height: 100%;
  display: flex; align-items: center; justify-content: center;
  font-size: 5rem; color: rgba(255,255,255,0.1);
}
.track-meta { text-align: center; margin-top: 4px; max-width: 100%; min-height: 3.5em; }
.track-title {
  font-size: 1.05rem; font-weight: 700; color: rgba(255,255,255,0.92);
  margin: 0 0 4px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
  max-width: min(70vw, 400px);
}
.track-artist {
  font-size: 0.8125rem; color: rgba(255,255,255,0.5); margin: 0 0 1px;
  white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
}
.track-album {
  font-size: 0.75rem; color: rgba(255,255,255,0.32); margin: 0;
  white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
}

/* ── controls ── */
.regular-controls {
  flex-shrink: 0; display: flex; flex-direction: column; align-items: center;
  gap: 14px; padding-top: 8px; width: 100%; max-width: 560px;
}
.progress-area {
  display: flex; align-items: center; gap: 12px; width: 100%;
}
.time-label {
  font-size: 0.6875rem; color: rgba(255,255,255,0.35);
  font-variant-numeric: tabular-nums; min-width: 34px; text-align: center;
  user-select: none; flex-shrink: 0;
}
.time-remaining { min-width: 40px; }
.progress-bar {
  flex: 1; height: 3px; position: relative; cursor: pointer;
  padding: 6px 0; margin: -6px 0;
}
.progress-bg {
  position: absolute; left: 0; right: 0; top: 50%; transform: translateY(-50%);
  height: 3px; background: rgba(255,255,255,0.15); border-radius: 2px;
  transition: height 0.15s;
}
.progress-fill {
  position: absolute; left: 0; top: 50%; transform: translateY(-50%);
  height: 3px; background: rgba(255,255,255,0.85); border-radius: 2px;
  pointer-events: none; transition: height 0.15s;
}
.progress-knob {
  position: absolute; top: 50%; width: 10px; height: 10px;
  background: #fff; border-radius: 50%; transform: translate(-50%, -50%);
  opacity: 0; pointer-events: none;
  box-shadow: 0 1px 4px rgba(0,0,0,0.4);
  transition: opacity 0.12s;
}
.progress-bar:hover .progress-bg,
.progress-bar:active .progress-bg { height: 5px; }
.progress-bar:hover .progress-fill,
.progress-bar:active .progress-fill { height: 5px; }
.progress-bar:hover .progress-knob,
.progress-bar:active .progress-knob { opacity: 1; }

.btn-row { display: flex; align-items: center; gap: 18px; }
.ctrl-btn {
  background: none; border: none;
  color: rgba(255,255,255,0.6); font-size: 1.15rem;
  width: 36px; height: 36px; display: flex; align-items: center; justify-content: center;
  cursor: pointer; border-radius: 50%;
  transition: color 0.15s, transform 0.1s, background 0.15s;
}
.ctrl-btn:hover { color: rgba(255,255,255,0.9); background: rgba(255,255,255,0.06); }
.ctrl-btn:active { transform: scale(0.9); }
.ctrl-btn.on { color: var(--primary-color, #0A84FF); }
.ctrl-play {
  font-size: 2.5rem; width: 50px; height: 50px; color: rgba(255,255,255,0.9);
}
.ctrl-play:hover { color: #fff; background: none; transform: scale(1.06); }
.ctrl-play:active { transform: scale(0.93); }

/* ── widget row (5 function widgets) ── */
.widget-row {
  display: flex; align-items: center; justify-content: center; gap: 8px;
  flex-wrap: wrap;
}
.widget-btn {
  background: none; border: none;
  color: rgba(255,255,255,0.45); font-size: 1.1rem;
  display: flex; flex-direction: column; align-items: center; gap: 2px;
  cursor: pointer; border-radius: 12px; padding: 8px 12px;
  min-width: 56px;
  transition: color 0.15s, background 0.15s;
}
.widget-btn:hover { color: rgba(255,255,255,0.8); background: rgba(255,255,255,0.06); }
.widget-btn.on { color: var(--primary-color, #0A84FF); }
.widget-label { font-size: 0.625rem; font-weight: 500; }

/* ── mobile: lyrics mode ── */
.mode-lyrics {
  flex: 1; display: flex; flex-direction: column; min-height: 0; gap: 8px;
}
.lyrics-info-bar {
  flex-shrink: 0; display: flex; align-items: center; gap: 10px;
  padding: 8px 12px; background: rgba(255,255,255,0.06); border-radius: 12px;
}
.lyrics-cover {
  width: 40px; height: 40px; border-radius: 8px; overflow: hidden; flex-shrink: 0;
}
.lyrics-cover img { width: 100%; height: 100%; object-fit: cover; }
.lyrics-cover i { width: 100%; height: 100%; display: flex; align-items: center; justify-content: center; font-size: 1.25rem; color: rgba(255,255,255,0.2); }
.lyrics-info-text { display: flex; flex-direction: column; min-width: 0; }
.lyrics-track-title {
  font-size: 0.875rem; font-weight: 600; color: rgba(255,255,255,0.9);
  white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
}
.lyrics-track-artist {
  font-size: 0.75rem; color: rgba(255,255,255,0.45);
  white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
}
.lyrics-render-area {
  flex: 1; min-height: 0; display: flex; align-items: center; justify-content: center;
}
.lyrics-wrapper {
  width: 100%; height: 100%;
  -webkit-mask-image: linear-gradient(to bottom, transparent 0%, black 6%, black 94%, transparent 100%);
  mask-image: linear-gradient(to bottom, transparent 0%, black 6%, black 94%, transparent 100%);
}
.lyrics-wrapper :deep(.amll-lyric-player) { width: 100%; height: 100%; }
.plain-lyrics { max-height: 100%; overflow-y: auto; text-align: center; padding: 8px 0; }
.plain-line {
  padding: 4px 0; font-size: 0.9375rem; color: rgba(255,255,255,0.5);
  line-height: 1.8; white-space: pre-wrap; word-break: break-word; margin: 0;
}
.no-lyrics { font-size: 0.9375rem; color: rgba(255,255,255,0.18); user-select: none; }

/* ── desktop mode ── */
.desktop-mode {
  flex-direction: row; gap: 32px;
}
.desktop-left {
  flex: 1; display: flex; flex-direction: column; align-items: center;
  justify-content: center; gap: 12px; min-width: 0;
}
.desktop-left .cover-section { width: min(30vw, 320px); }
.desktop-left .cover-art { width: 100%; }
.desktop-centered { max-width: 480px; }

.desktop-right {
  flex: 1; min-width: 0; height: 100%;
  display: flex; flex-direction: column;
  background: rgba(255,255,255,0.03); border-radius: 16px; padding: 16px;
}
.desktop-lyrics .lyrics-wrapper {
  -webkit-mask-image: none; mask-image: none;
}

/* ── playlist ── */
.playlist-title {
  font-size: 1rem; font-weight: 700; color: rgba(255,255,255,0.9);
  margin: 0 0 12px; flex-shrink: 0;
}
.playlist-items {
  flex: 1; overflow-y: auto; display: flex; flex-direction: column; gap: 4px;
}
.playlist-item {
  display: flex; flex-direction: column; gap: 2px;
  padding: 10px 12px; border-radius: 8px; cursor: pointer;
  transition: background 0.15s;
}
.playlist-item:hover { background: rgba(255,255,255,0.06); }
.playlist-item.active { background: rgba(255,255,255,0.12); }
.playlist-item-title {
  font-size: 0.875rem; font-weight: 500; color: rgba(255,255,255,0.85);
  white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
}
.playlist-item-artist {
  font-size: 0.75rem; color: rgba(255,255,255,0.4);
  white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
}
.playlist-empty {
  text-align: center; color: rgba(255,255,255,0.3); padding: 32px 0;
  font-size: 0.875rem;
}

/* ── mode switch bar ── */
.mode-switch-bar {
  flex-shrink: 0; display: flex; align-items: center; justify-content: center;
  gap: 4px; padding: 8px 0 4px;
  background: rgba(255,255,255,0.04); border-radius: 16px;
  margin: 8px 0 0;
}
.switch-btn {
  background: none; border: none;
  color: rgba(255,255,255,0.4); font-size: 0.875rem;
  display: flex; align-items: center; gap: 6px;
  cursor: pointer; border-radius: 12px; padding: 8px 16px;
  transition: color 0.15s, background 0.15s;
}
.switch-btn:hover { color: rgba(255,255,255,0.7); background: rgba(255,255,255,0.06); }
.switch-btn.active { color: rgba(255,255,255,0.95); background: rgba(255,255,255,0.12); }
.switch-btn i { font-size: 0.9375rem; }

/* ── immersive mode ── */
/* opacity/pointer-events 由 anime.js watch(isImmersive) 控制，避免 CSS transition 冲突 */
.is-immersive .immersive-hide {
  pointer-events: none;
}

/* ── responsive ── */
@media (max-width: 767px) {
  .player-body { padding: 0 20px; }
  .mode-regular { gap: 8px; }
  .cover-section { width: min(48vw, 220px); }
  .track-meta { margin-top: 4px; min-height: auto; }
  .track-title { font-size: 0.95rem; }
  .regular-controls { gap: 8px; padding-top: 4px; }
  .btn-row { gap: 12px; }
  .ctrl-btn { width: 32px; height: 32px; font-size: 1.05rem; }
  .ctrl-play { font-size: 2.1rem; width: 44px; height: 44px; }
  .progress-area { gap: 8px; }
  .widget-btn { padding: 6px 10px; min-width: 48px; }
  .widget-label { font-size: 0.5625rem; }
}

@media (min-width: 768px) {
  .player-body { padding: 0 40px; }
  .desktop-left .cover-section { width: min(28vw, 300px); }
}

@media (max-height: 640px) {
  .mode-regular { gap: 4px; }
  .cover-section { width: min(28vw, 160px); }
  .track-meta { margin-top: 2px; }
  .track-title { font-size: 0.85rem; }
  .track-artist { font-size: 0.7rem; }
  .regular-controls { gap: 4px; padding-top: 0; }
  .ctrl-play { font-size: 1.7rem; width: 36px; height: 36px; }
  .widget-row { gap: 4px; }
}
</style>
