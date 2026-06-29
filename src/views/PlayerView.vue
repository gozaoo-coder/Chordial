<template>
  <div class="player-view" :class="{ 'is-active': isActive }" @keydown="onKeydown">
    <!-- 背景 -->
    <div class="player-bg">
      <Transition name="bg-fade">
        <img v-if="coverUrl" :key="coverBgKey" :src="coverUrl" class="bg-img" />
      </Transition>
      <div class="bg-mask"></div>
    </div>

    <!-- 内容层 -->
    <div class="player-body">
      <!-- 顶栏：始终可见 -->
      <header class="top-bar">
        <button class="btn-icon" @click="goBack" title="返回">
          <i class="bi bi-chevron-down"></i>
        </button>
        <span class="top-title">正在播放</span>
        <button v-if="currentTrack" class="btn-icon" @click="PlayerStore.playNext" title="播放列表">
          <i class="bi bi-music-note-list"></i>
        </button>
        <span v-else class="top-title"></span>
      </header>

      <!-- 空状态 -->
      <div v-if="!currentTrack" class="empty-state">
        <i class="bi bi-music-note-beamed empty-icon"></i>
        <p class="empty-text">选择一首歌曲开始播放</p>
      </div>

      <!-- 播放中 -->
      <template v-else>
      <!-- 核心区：无歌词时居中、有歌词时分栏 -->
      <div class="core" :class="{ 'core--centered': !showLyrics || !hasLyrics }">
        <!-- 封面 -->
        <div class="cover-section">
          <Transition name="cover-swap" mode="out-in">
            <div class="cover-art" :key="currentTrack.id">
              <img v-if="coverUrl" :src="coverUrl" alt="" />
              <i v-else class="bi bi-disc-fill cover-fallback"></i>
            </div>
          </Transition>

          <Transition name="meta-swap" mode="out-in">
            <div class="track-meta" :key="currentTrack.id">
              <h1 class="track-title">{{ currentTrack.title }}</h1>
              <p class="track-artist">{{ currentTrack.artist }}</p>
              <p v-if="currentTrack.albumTitle" class="track-album">
                {{ currentTrack.albumTitle }}
              </p>
            </div>
          </Transition>
        </div>

        <!-- 歌词 -->
        <div v-if="showLyrics" class="lyrics-section">
          <Transition name="lyrics-fade" mode="out-in">
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
          </Transition>
        </div>
      </div>

      <!-- 底部控制 -->
      <div class="controls">
        <!-- 进度条 -->
        <div class="progress-area">
          <span class="time-label">{{ formattedCurrentTime }}</span>
          <div
            class="progress-bar"
            ref="progressTrack"
            @mousedown="startSeek"
            @touchstart="startSeek"
          >
            <div class="progress-bg"></div>
            <div class="progress-fill" :style="{ width: progressPercent + '%' }"></div>
            <div class="progress-knob" :style="{ left: progressPercent + '%' }"></div>
          </div>
          <span class="time-label time-remaining">{{ formattedRemaining }}</span>
        </div>

        <!-- 按钮 -->
        <div class="btn-row">
          <button class="ctrl-btn" :class="{ on: playMode !== PlayMode.SEQUENCE }"
            @click="PlayerStore.togglePlayMode" title="播放模式">
            <i :class="playModeIcon"></i>
          </button>
          <button class="ctrl-btn" @click="PlayerStore.playPrevious" title="上一首">
            <i class="bi bi-skip-start-fill"></i>
          </button>
          <button class="ctrl-btn ctrl-play" @click="PlayerStore.togglePlay" title="播放 / 暂停">
            <i :class="isPlaying ? 'bi bi-pause-circle-fill' : 'bi bi-play-circle-fill'"></i>
          </button>
          <button class="ctrl-btn" @click="PlayerStore.playNext" title="下一首">
            <i class="bi bi-skip-end-fill"></i>
          </button>
          <button class="ctrl-btn" :class="{ on: isMuted }"
            @click="PlayerStore.toggleMute" title="静音">
            <i :class="volumeIcon"></i>
          </button>
          <button v-if="hasSyncedLyrics || hasPlainLyrics" class="ctrl-btn"
            :class="{ on: showLyrics }" @click="showLyrics = !showLyrics" title="歌词">
            <i class="bi bi-mic-fill"></i>
          </button>
        </div>
      </div>
      </template>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { LyricPlayer } from '@applemusic-like-lyrics/vue'
import { PlayerStore, PlayMode } from '@/stores/player.js'
import { useCoverImage } from '@/composables/useCoverImage'
import { useLyricParser } from '@/composables/useLyricParser'
import { usePerf } from '@/utils/performanceMonitor.js'

const { start, end, log } = usePerf('PlayerView')

const router = useRouter()
const isActive = ref(false)
const showLyrics = ref(true)
const progressTrack = ref(null)
const coverBgKey = ref(0)
let seekDragging = false

// ── player ──

const currentTrack = computed(() => PlayerStore.state.currentTrack)
const isPlaying = computed(() => PlayerStore.state.isPlaying)
const currentTime = computed(() => PlayerStore.state.currentTime)
const duration = computed(() => PlayerStore.state.duration)
const volume = computed(() => PlayerStore.state.volume)
const isMuted = computed(() => PlayerStore.state.muted)
const playMode = computed(() => PlayerStore.state.playMode)
const { coverUrl } = useCoverImage(currentTrack, 'large')

// 曲目切换 → 背景 key 递增 → 触发 crossfade
watch(() => currentTrack.value?.id, () => { coverBgKey.value++ })

// ── time ──

const currentTimeMs = computed(() => Math.floor(currentTime.value * 1000))
const progressPercent = computed(() =>
  duration.value ? Math.min((currentTime.value / duration.value) * 100, 100) : 0
)
const formattedCurrentTime = computed(() => PlayerStore.formattedCurrentTime)
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

function goBack() {
  log('goBack');
  if (window.history.length > 1) router.back()
  else router.push('/')
}

function onLineClick(e) {
  const t = e?.line?.startTime
  if (t !== undefined) {
    log('seekByLyric', { time: t });
    PlayerStore.seek(t / 1000)
  }
}

// ── keyboard ──

function onKeydown(e) {
  if (e.target.tagName === 'INPUT') return
  switch (e.code) {
    case 'Space':
      e.preventDefault()
      log('keyboard:togglePlay')
      PlayerStore.togglePlay()
      break
    case 'ArrowLeft':
      e.preventDefault()
      log('keyboard:seekBack', { delta: -5 })
      PlayerStore.seek(Math.max(0, currentTime.value - 5))
      break
    case 'ArrowRight':
      e.preventDefault()
      log('keyboard:seekForward', { delta: 5 })
      PlayerStore.seek(Math.min(duration.value || Infinity, currentTime.value + 5))
      break
    case 'ArrowUp':
      e.preventDefault()
      log('keyboard:volumeUp')
      PlayerStore.setVolume(Math.min(1, volume.value + 0.05))
      break
    case 'ArrowDown':
      e.preventDefault()
      log('keyboard:volumeDown')
      PlayerStore.setVolume(Math.max(0, volume.value - 0.05))
      break
  }
}

// ── seek ──

function startSeek(e) {
  e.preventDefault?.()
  seekDragging = true
  log('seekStart')
  updateSeek(e)
  document.addEventListener('mousemove', updateSeek)
  document.addEventListener('mouseup', endSeek)
  document.addEventListener('touchmove', updateSeek, { passive: false })
  document.addEventListener('touchend', endSeek)
}
function updateSeek(e) {
  if (!seekDragging || !progressTrack.value) return
  const r = progressTrack.value.getBoundingClientRect()
  const x = e.touches ? e.touches[0].clientX : e.clientX
  const ratio = Math.max(0, Math.min(1, (x - r.left) / r.width))
  PlayerStore.seek(ratio * (duration.value || 0))
}
function endSeek() {
  seekDragging = false
  log('seekEnd')
  document.removeEventListener('mousemove', updateSeek)
  document.removeEventListener('mouseup', endSeek)
  document.removeEventListener('touchmove', updateSeek)
  document.removeEventListener('touchend', endSeek)
}

// ── lifecycle ──

onMounted(() => {
  requestAnimationFrame(() => {
    start('pageActive')
    isActive.value = true
    end('pageActive')
  })
})
</script>

<style scoped>
/* ── container ── */

.player-view {
  position: fixed; inset: 0; z-index: 200;
  display: flex; align-items: center; justify-content: center;
  background: #0b0b0e;
  opacity: 0;
  transition: opacity 0.4s ease;
  outline: none;
}
.player-view.is-active { opacity: 1; }

/* ── background ── */

.player-bg { position: absolute; inset: 0; overflow: hidden; }
.bg-img {
  position: absolute; inset: -60px;
  width: calc(100% + 120px); height: calc(100% + 120px);
  object-fit: cover;
  filter: blur(80px) brightness(0.28) saturate(1.8);
  transform: scale(1.05);
  pointer-events: none; user-select: none;
}
.bg-mask {
  position: absolute; inset: 0;
  background:
    radial-gradient(ellipse at 50% 30%, transparent 0%, rgba(0,0,0,0.35) 70%),
    linear-gradient(to bottom, rgba(0,0,0,0.15), rgba(0,0,0,0.5));
}

.bg-fade-enter-active,
.bg-fade-leave-active { transition: opacity 0.6s ease; }
.bg-fade-enter-from,
.bg-fade-leave-to { opacity: 0; }

/* ── empty ── */

.empty-state { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; text-align: center; color: rgba(255,255,255,0.4); }
.empty-icon { font-size: 3rem; display: block; margin-bottom: 12px; }
.empty-text { font-size: 0.9375rem; margin: 0; }

/* ── body ── */

.player-body {
  position: relative; z-index: 1;
  display: flex; flex-direction: column;
  width: 100%; height: 100%;
  max-width: 1100px;
  padding: 0 32px;
  padding-top: max(12px, var(--safe-area-top, 0px));
  padding-bottom: max(20px, var(--safe-area-bottom, 0px));
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

/* ── core ── */

.core {
  flex: 1; min-height: 0;
  display: flex; align-items: center; gap: 48px;
}

/* centered mode (no lyrics) */
.core--centered {
  flex-direction: column; justify-content: center; gap: 0;
}
.core--centered .cover-section {
  width: min(45vw, 360px);
}
.core--centered .cover-art {
  box-shadow: 0 24px 80px rgba(0,0,0,0.6);
}

/* ── cover ── */

.cover-section {
  flex-shrink: 0; display: flex; flex-direction: column; align-items: center;
  width: min(38vw, 300px);
}
.cover-art {
  width: 100%; aspect-ratio: 1;
  border-radius: 12px; overflow: hidden;
  box-shadow: 0 16px 60px rgba(0,0,0,0.55);
  background: rgba(255,255,255,0.03);
}
.cover-art img {
  width: 100%; height: 100%; object-fit: cover; display: block;
}
.cover-fallback {
  width: 100%; height: 100%;
  display: flex; align-items: center; justify-content: center;
  font-size: 5rem; color: rgba(255,255,255,0.1);
}

.track-meta { text-align: center; margin-top: 18px; max-width: 100%; min-height: 3.5em; }
.track-title {
  font-size: 1.05rem; font-weight: 700; color: rgba(255,255,255,0.92);
  margin: 0 0 4px;
  white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
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

/* transition: cover swap */
.cover-swap-enter-active { transition: opacity 0.25s ease, transform 0.25s ease; }
.cover-swap-leave-active { transition: opacity 0.15s ease, transform 0.15s ease; }
.cover-swap-enter-from { opacity: 0; transform: scale(0.95); }
.cover-swap-leave-to { opacity: 0; transform: scale(1.02); }

.meta-swap-enter-active { transition: opacity 0.25s ease 0.05s, transform 0.25s ease 0.05s; }
.meta-swap-leave-active { transition: opacity 0.1s ease; }
.meta-swap-enter-from { opacity: 0; transform: translateY(6px); }
.meta-swap-leave-to { opacity: 0; }

/* ── lyrics ── */

.lyrics-section {
  flex: 1; min-width: 0; height: 100%;
  display: flex; align-items: center; justify-content: center;
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

.lyrics-fade-enter-active { transition: opacity 0.3s ease; }
.lyrics-fade-leave-active { transition: opacity 0.15s ease; }
.lyrics-fade-enter-from,
.lyrics-fade-leave-to { opacity: 0; }

/* ── controls ── */

.controls {
  flex-shrink: 0; display: flex; flex-direction: column; align-items: center;
  gap: 14px; padding-top: 16px;
}

/* progress */

.progress-area {
  display: flex; align-items: center; gap: 12px; width: 100%; max-width: 560px;
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
  pointer-events: none;
  transition: height 0.15s;
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

/* buttons */

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

/* ── responsive ── */

@media (max-width: 767px) {
  .player-body { padding: 0 20px; }
  .core { gap: 12px; }
  .core:not(.core--centered) { flex-direction: column; justify-content: center; }
  .core--centered .cover-section { width: min(60vw, 300px); }
  .cover-section { width: min(48vw, 220px); }
  .track-meta { margin-top: 10px; min-height: auto; }
  .track-title { font-size: 0.95rem; }
  .lyrics-section { width: 100%; flex: 1; }
  .controls { gap: 8px; padding-top: 8px; }
  .btn-row { gap: 12px; }
  .ctrl-btn { width: 32px; height: 32px; font-size: 1.05rem; }
  .ctrl-play { font-size: 2.1rem; width: 44px; height: 44px; }
  .progress-area { gap: 8px; }
}

@media (max-height: 640px) {
  .core { gap: 8px; }
  .cover-section { width: min(24vw, 130px); }
  .core--centered .cover-section { width: min(35vw, 220px); }
  .cover-art { border-radius: 8px; }
  .track-meta { margin-top: 4px; }
  .track-title { font-size: 0.85rem; }
  .track-artist { font-size: 0.7rem; }
  .controls { gap: 4px; padding-top: 0; }
  .ctrl-play { font-size: 1.7rem; width: 36px; height: 36px; }
}
</style>
