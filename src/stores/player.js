/**
 * PlayerStore - 全局播放器状态管理
 * 使用 Vue 3 Composition API 实现单例模式
 *
 * 功能：
 * - 管理播放器核心状态（当前歌曲、播放状态、进度等）
 * - 提供播放控制方法（播放、暂停、跳转等）
 * - 管理播放列表和播放模式
 * - 自动处理音频资源生命周期
 */

import { reactive, readonly, computed, markRaw } from 'vue';
import { perf } from '@/utils/performanceMonitor.js';

// 播放模式枚举
export const PlayMode = {
  SEQUENCE: 'sequence',  // 顺序播放
  RANDOM: 'random',      // 随机播放
  LOOP: 'loop',          // 循环播放
  LOOP_ONE: 'loop_one'   // 单曲循环
};

// 创建音频元素
const createAudioElement = () => {
  const audio = new Audio();
  audio.preload = 'metadata';
  return audio;
};

// PlayerStore 状态
const state = reactive({
  // 当前播放状态
  currentTrack: null,        // 当前播放的歌曲 (Track 实例)
  isPlaying: false,          // 是否正在播放
  currentTime: 0,            // 当前播放时间 (秒)
  duration: 0,               // 总时长 (秒)
  volume: 0.8,               // 音量 (0-1)
  muted: false,              // 是否静音

  // 播放列表
  playlist: [],              // 当前播放列表
  currentIndex: -1,          // 当前歌曲在播放列表中的索引
  playMode: PlayMode.SEQUENCE, // 播放模式

  // 音频元素
  audioElement: null,        // HTMLAudioElement

  // 加载状态
  isLoading: false,          // 是否正在加载
  error: null,               // 错误信息

  // 歌词显示
  showLyrics: false,         // 是否显示歌词页面
  lyricsData: {              // 歌词数据
    plainLyrics: '',
    syncedLyrics: '',
    hasSyncedLyrics: false,
    hasPlainLyrics: false
  },

  // ── PlayerView UI 状态（模态化，不再通过 router）──────────────
  ui: {
    isPlayerViewOpen: false,          // PlayerView 模态是否打开
    playerViewModeMobile: 'regular',  // 移动端模式: 'details' | 'regular' | 'lyrics'
    playerViewModeDesktop: 'info',    // 桌面端模式: 'info' | 'lyrics' | 'playlist'
    playerViewExpandProgress: 0,      // 0-1, 拖动展开进度（player-control-bar → player-view）
    isImmersive: false,               // 自动观赏模式（隐藏控件）
    currentDevice: 'mobile',          // 'mobile' | 'desktop'（768px 断点）
    sharedElementSource: null,        // markRaw DOM 元素引用，用于 FLIP 共享元素动画
    playerViewTransitioning: false,   // 过渡中，阻止交互
  }
});

// 沉浸模式定时器（模块级，不放入 reactive state）
let _immersiveTimer = null;
const IMMERSIVE_DELAY = 5000; // 5s 无操作进入观赏模式

// 计算属性
const getters = {
  // 是否有正在播放的歌曲
  hasCurrentTrack: computed(() => state.currentTrack !== null),

  // 播放进度百分比
  progress: computed(() => {
    if (state.duration === 0) return 0;
    return (state.currentTime / state.duration) * 100;
  }),

  // 格式化后的当前时间
  formattedCurrentTime: computed(() => formatTime(state.currentTime)),

  // 格式化后的总时长
  formattedDuration: computed(() => formatTime(state.duration)),

  // 是否可以播放上一首
  canPlayPrevious: computed(() => {
    if (state.playlist.length === 0) return false;
    if (state.playMode === PlayMode.RANDOM) return true;
    return state.currentIndex > 0;
  }),

  // 是否可以播放下一首
  canPlayNext: computed(() => {
    if (state.playlist.length === 0) return false;
    if (state.playMode === PlayMode.RANDOM) return true;
    return state.currentIndex < state.playlist.length - 1;
  }),

  // 播放列表是否为空
  isPlaylistEmpty: computed(() => state.playlist.length === 0),

  // 播放列表长度
  playlistLength: computed(() => state.playlist.length),

  // ── PlayerView UI getters ────────────────────────────────────
  isPlayerViewOpen: computed(() => state.ui.isPlayerViewOpen),
  playerViewMode: computed(() =>
    state.ui.currentDevice === 'desktop'
      ? state.ui.playerViewModeDesktop
      : state.ui.playerViewModeMobile
  ),
  isImmersive: computed(() => state.ui.isImmersive),
  currentDevice: computed(() => state.ui.currentDevice),
  playerViewTransitioning: computed(() => state.ui.playerViewTransitioning),
};

// 格式化时间
function formatTime(seconds) {
  if (!seconds || isNaN(seconds)) return '0:00';

  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins}:${secs.toString().padStart(2, '0')}`;
}

// 初始化音频元素
function initAudioElement() {
  if (state.audioElement) {
    // 清理旧的事件监听
    cleanupAudioEvents();
  }

  state.audioElement = createAudioElement();
  state.audioElement.volume = state.volume;
  state.audioElement.muted = state.muted;

  // 绑定事件
  setupAudioEvents();
}

// 音频事件处理函数（用于正确移除监听）
const audioEventHandlers = {
  timeupdate: null,
  loadedmetadata: null,
  canplay: null,
  waiting: null,
  ended: null,
  error: null,
  volumechange: null
};

// 设置音频事件监听
function setupAudioEvents() {
  const audio = state.audioElement;
  if (!audio) return;

  // 时间更新（节流处理，每 100ms 更新一次）
  let lastTimeUpdate = 0;
  const TIME_UPDATE_THROTTLE = 100; // 100ms

  audioEventHandlers.timeupdate = () => {
    const now = Date.now();
    if (now - lastTimeUpdate < TIME_UPDATE_THROTTLE) return;
    lastTimeUpdate = now;

    state.currentTime = audio.currentTime || 0;
  };
  audio.addEventListener('timeupdate', audioEventHandlers.timeupdate);

  // 加载完成
  audioEventHandlers.loadedmetadata = () => {
    state.duration = audio.duration || 0;
    state.isLoading = false;
  };
  audio.addEventListener('loadedmetadata', audioEventHandlers.loadedmetadata);

  // 可以播放
  audioEventHandlers.canplay = () => {
    state.isLoading = false;
  };
  audio.addEventListener('canplay', audioEventHandlers.canplay);

  // 等待加载
  audioEventHandlers.waiting = () => {
    state.isLoading = true;
  };
  audio.addEventListener('waiting', audioEventHandlers.waiting);

  // 播放结束
  audioEventHandlers.ended = () => {
    handleTrackEnded();
  };
  audio.addEventListener('ended', audioEventHandlers.ended);

  // 错误处理
  audioEventHandlers.error = (e) => {
    console.error('Audio playback error:', e);
    state.error = '播放出错';
    state.isLoading = false;
    state.isPlaying = false;
  };
  audio.addEventListener('error', audioEventHandlers.error);

  // 音量变化
  audioEventHandlers.volumechange = () => {
    state.volume = audio.volume;
    state.muted = audio.muted;
  };
  audio.addEventListener('volumechange', audioEventHandlers.volumechange);
}

// 清理音频事件监听
function cleanupAudioEvents() {
  const audio = state.audioElement;
  if (!audio) return;

  // 移除所有事件监听
  if (audioEventHandlers.timeupdate) {
    audio.removeEventListener('timeupdate', audioEventHandlers.timeupdate);
  }
  if (audioEventHandlers.loadedmetadata) {
    audio.removeEventListener('loadedmetadata', audioEventHandlers.loadedmetadata);
  }
  if (audioEventHandlers.canplay) {
    audio.removeEventListener('canplay', audioEventHandlers.canplay);
  }
  if (audioEventHandlers.waiting) {
    audio.removeEventListener('waiting', audioEventHandlers.waiting);
  }
  if (audioEventHandlers.ended) {
    audio.removeEventListener('ended', audioEventHandlers.ended);
  }
  if (audioEventHandlers.error) {
    audio.removeEventListener('error', audioEventHandlers.error);
  }
  if (audioEventHandlers.volumechange) {
    audio.removeEventListener('volumechange', audioEventHandlers.volumechange);
  }
}

// 处理歌曲播放结束
function handleTrackEnded() {
  switch (state.playMode) {
    case PlayMode.LOOP_ONE:
      // 单曲循环，重新播放
      state.audioElement.currentTime = 0;
      state.audioElement.play();
      break;
    case PlayMode.LOOP:
    case PlayMode.RANDOM:
    case PlayMode.SEQUENCE:
    default:
      actions.playNext();
      break;
  }
}

// 获取歌曲在播放列表中的索引
function getTrackIndex(track) {
  if (!track) return -1;
  return state.playlist.findIndex(t => t.id === track.id);
}

// Actions
const actions = {
  /**
   * 播放指定歌曲
   * @param {Track} track - 要播放的歌曲
   * @param {Array} playlist - 可选的播放列表
   */
  async play(track, playlist = null) {
    return perf.measureAsync('PlayerStore.play', (async () => {
    if (!track || !track.id) {
      console.warn('play: 无效的 track 参数');
      return;
    }

    try {
      state.isLoading = true;
      state.error = null;

      // 如果提供了播放列表，更新播放列表（markRaw 避免深代理 Track 实例）
      if (playlist && Array.isArray(playlist)) {
        state.playlist = playlist.map(t => markRaw(t));
      }

      // 如果当前已经在播放这首歌，继续播放
      if (state.currentTrack?.id === track.id) {
        if (state.audioElement?.paused) {
          await state.audioElement.play();
          state.isPlaying = true;
        }
        state.isLoading = false;
        return;
      }

      // 停止当前播放并释放旧的音频资源
      if (state.audioElement) {
        // chordial:// 协议下无需 revokeObjectURL；保留 blob: 兼容
        const oldSrc = state.audioElement.src;
        if (oldSrc && oldSrc.startsWith('blob:')) {
          URL.revokeObjectURL(oldSrc);
        }
        state.audioElement.pause();
        state.audioElement.currentTime = 0;
      }

      // 更新当前歌曲（markRaw 避免 Vue 对 Track 业务类实例创建深代理）
      state.currentTrack = markRaw(track);
      state.currentIndex = getTrackIndex(track);
      state.currentTime = 0;
      state.duration = track.duration || 0;

      // 获取音频 URL
      const audioUrl = await track.getAudioBlobUrl();
      if (!audioUrl) {
        throw new Error('无法获取音频文件');
      }

      // 初始化音频元素
      if (!state.audioElement) {
        initAudioElement();
      }

      // 设置音频源
      state.audioElement.src = audioUrl;

      // 播放（先启动播放，歌词后台加载，不阻塞）
      await state.audioElement.play();
      state.isPlaying = true;

      // 后台加载歌词，不阻塞播放启动
      actions.loadLyrics(track);

    } catch (error) {
      console.error('播放失败:', error);
      state.error = error.message || '播放失败';
      state.isPlaying = false;
    } finally {
      state.isLoading = false;
    }
    })());
  },

  /**
   * 暂停播放
   */
  pause() {
    if (state.audioElement && !state.audioElement.paused) {
      state.audioElement.pause();
      state.isPlaying = false;
    }
  },

  /**
   * 恢复播放
   */
  async resume() {
    if (state.audioElement && state.audioElement.paused) {
      try {
        await state.audioElement.play();
        state.isPlaying = true;
      } catch (error) {
        console.error('恢复播放失败:', error);
      }
    }
  },

  /**
   * 切换播放/暂停
   */
  togglePlay() {
    if (state.isPlaying) {
      actions.pause();
    } else {
      actions.resume();
    }
  },

  /**
   * 跳转到指定时间
   * @param {number} time - 时间（秒）
   */
  seek(time) {
    perf.start('PlayerStore.seek');
    if (state.audioElement) {
      const clampedTime = Math.max(0, Math.min(time, state.duration));
      state.audioElement.currentTime = clampedTime;
      state.currentTime = clampedTime;
    }
    perf.end('PlayerStore.seek');
  },

  /**
   * 跳转到进度百分比
   * @param {number} percent - 进度百分比 (0-100)
   */
  seekToPercent(percent) {
    const time = (percent / 100) * state.duration;
    actions.seek(time);
  },

  /**
   * 播放上一首
   */
  playPrevious() {
    perf.start('PlayerStore.playPrevious');
    if (state.playlist.length === 0) { perf.end('PlayerStore.playPrevious'); return; }

    let previousIndex;

    if (state.playMode === PlayMode.RANDOM) {
      // 随机模式，随机选择一首
      previousIndex = Math.floor(Math.random() * state.playlist.length);
    } else {
      // 其他模式，播放上一首
      previousIndex = state.currentIndex - 1;
      if (previousIndex < 0) {
        previousIndex = state.playlist.length - 1; // 循环到最后一首
      }
    }

    const previousTrack = state.playlist[previousIndex];
    if (previousTrack) {
      actions.play(previousTrack);
    }
    perf.end('PlayerStore.playPrevious');
  },

  /**
   * 播放下一首
   */
  playNext() {
    perf.start('PlayerStore.playNext');
    if (state.playlist.length === 0) { perf.end('PlayerStore.playNext'); return; }

    let nextIndex;

    if (state.playMode === PlayMode.RANDOM) {
      // 随机模式，随机选择一首
      nextIndex = Math.floor(Math.random() * state.playlist.length);
    } else {
      // 其他模式，播放下一首
      nextIndex = state.currentIndex + 1;
      if (nextIndex >= state.playlist.length) {
        nextIndex = 0; // 循环到第一首
      }
    }

    const nextTrack = state.playlist[nextIndex];
    if (nextTrack) {
      actions.play(nextTrack);
    }
    perf.end('PlayerStore.playNext');
  },

  /**
   * 随机播放
   */
  playRandom() {
    if (state.playlist.length === 0) return;

    const randomIndex = Math.floor(Math.random() * state.playlist.length);
    const randomTrack = state.playlist[randomIndex];
    if (randomTrack) {
      actions.play(randomTrack);
    }
  },

  /**
   * 设置音量
   * @param {number} volume - 音量 (0-1)
   */
  setVolume(volume) {
    const clampedVolume = Math.max(0, Math.min(1, volume));
    state.volume = clampedVolume;
    if (state.audioElement) {
      state.audioElement.volume = clampedVolume;
    }
  },

  /**
   * 切换静音
   */
  toggleMute() {
    state.muted = !state.muted;
    if (state.audioElement) {
      state.audioElement.muted = state.muted;
    }
  },

  /**
   * 设置播放模式
   * @param {string} mode - 播放模式
   */
  setPlayMode(mode) {
    if (Object.values(PlayMode).includes(mode)) {
      state.playMode = mode;
    }
  },

  /**
   * 切换播放模式
   */
  togglePlayMode() {
    const modes = Object.values(PlayMode);
    const currentIndex = modes.indexOf(state.playMode);
    const nextIndex = (currentIndex + 1) % modes.length;
    state.playMode = modes[nextIndex];
  },

  /**
   * 设置播放列表
   * @param {Array} playlist - 播放列表
   */
  setPlaylist(playlist) {
    perf.start('PlayerStore.setPlaylist');
    // markRaw 每首 Track 实例，避免 Vue 对业务类（含方法）创建深代理
    state.playlist = (playlist || []).map(t => markRaw(t));
    // 更新当前歌曲索引
    if (state.currentTrack) {
      state.currentIndex = getTrackIndex(state.currentTrack);
    }
    perf.end('PlayerStore.setPlaylist');
  },

  /**
   * 添加歌曲到播放列表
   * @param {Track} track - 歌曲
   */
  addToPlaylist(track) {
    perf.start('PlayerStore.addToPlaylist');
    if (!track) { perf.end('PlayerStore.addToPlaylist'); return; }

    // 检查是否已存在
    const exists = state.playlist.some(t => t.id === track.id);
    if (!exists) {
      state.playlist.push(markRaw(track));
    }
    perf.end('PlayerStore.addToPlaylist');
  },

  /**
   * 从播放列表移除歌曲
   * @param {string} trackId - 歌曲 ID
   */
  removeFromPlaylist(trackId) {
    const index = state.playlist.findIndex(t => t.id === trackId);
    if (index !== -1) {
      state.playlist.splice(index, 1);

      // 更新当前索引
      if (state.currentTrack) {
        state.currentIndex = getTrackIndex(state.currentTrack);
      }
    }
  },

  /**
   * 清空播放列表
   */
  clearPlaylist() {
    state.playlist = [];
    state.currentIndex = -1;
    if (state.isPlaying) {
      actions.stop();
    }
  },

  /**
   * 停止播放
   */
  stop() {
    if (state.audioElement) {
      state.audioElement.pause();
      state.audioElement.currentTime = 0;
    }
    state.isPlaying = false;
    state.currentTime = 0;
  },

  /**
   * 加载歌词
   * @param {Track} track - 歌曲
   */
  async loadLyrics(track) {
    return perf.measureAsync('PlayerStore.loadLyrics', (async () => {
    try {
      const lyricsInfo = await track.getLyricsInfo();

      // 比较是否有变化，避免不必要的更新
      const hasChanged =
        state.lyricsData.plainLyrics !== lyricsInfo.plainLyrics ||
        state.lyricsData.syncedLyrics !== lyricsInfo.syncedLyrics ||
        state.lyricsData.hasSyncedLyrics !== lyricsInfo.hasSyncedLyrics ||
        state.lyricsData.hasPlainLyrics !== lyricsInfo.hasPlainLyrics;

      if (hasChanged) {
        console.log('[PlayerStore] lyricsData changed, updating');
        // 逐个更新字段，而不是替换整个对象
        state.lyricsData.plainLyrics = lyricsInfo.plainLyrics;
        state.lyricsData.syncedLyrics = lyricsInfo.syncedLyrics;
        state.lyricsData.hasSyncedLyrics = lyricsInfo.hasSyncedLyrics;
        state.lyricsData.hasPlainLyrics = lyricsInfo.hasPlainLyrics;
      } else {
        console.log('[PlayerStore] lyricsData unchanged, skipping update');
      }
    } catch (error) {
      console.warn('加载歌词失败:', error);
      // 只有当当前有内容时才清空
      if (state.lyricsData.plainLyrics || state.lyricsData.syncedLyrics) {
        state.lyricsData.plainLyrics = '';
        state.lyricsData.syncedLyrics = '';
        state.lyricsData.hasSyncedLyrics = false;
        state.lyricsData.hasPlainLyrics = false;
      }
    }
    })());
  },

  /**
   * 切换歌词显示
   */
  toggleLyrics() {
    state.showLyrics = !state.showLyrics;
  },

  /**
   * 设置歌词显示状态
   * @param {boolean} show - 是否显示
   */
  setShowLyrics(show) {
    state.showLyrics = show;
  },

  // ── PlayerView UI Actions ────────────────────────────────────

  /**
   * 打开 PlayerView 模态
   * @param {HTMLElement|null} sourceEl - 触发元素（album-cover-thumb），用于 FLIP 共享元素动画
   */
  openPlayerView(sourceEl = null) {
    state.ui.sharedElementSource = sourceEl ? markRaw(sourceEl) : null;
    state.ui.isPlayerViewOpen = true;
    state.ui.playerViewTransitioning = true;
    actions.resetImmersiveTimer();
  },

  /**
   * 关闭 PlayerView 模态
   */
  closePlayerView() {
    state.ui.isPlayerViewOpen = false;
    state.ui.sharedElementSource = null;
    state.ui.isImmersive = false;
    state.ui.playerViewExpandProgress = 0;
    state.ui.playerViewTransitioning = false;
    if (_immersiveTimer) {
      clearTimeout(_immersiveTimer);
      _immersiveTimer = null;
    }
  },

  /**
   * 标记过渡结束
   */
  setPlayerViewTransitioning(v) {
    state.ui.playerViewTransitioning = v;
  },

  /**
   * 设置展开进度（拖动时 0→1）
   */
  setExpandProgress(p) {
    state.ui.playerViewExpandProgress = Math.max(0, Math.min(1, p));
  },

  /**
   * 切换 PlayerView 模式
   * @param {string} mode - 移动端: 'details'|'regular'|'lyrics'  桌面端: 'info'|'lyrics'|'playlist'
   */
  setPlayerViewMode(mode) {
    if (state.ui.currentDevice === 'desktop') {
      state.ui.playerViewModeDesktop = mode;
    } else {
      state.ui.playerViewModeMobile = mode;
    }
    actions.registerInteraction();
  },

  /**
   * 进入观赏模式（隐藏控件）
   */
  enterImmersive() {
    state.ui.isImmersive = true;
  },

  /**
   * 退出观赏模式（显示控件）
   */
  exitImmersive() {
    state.ui.isImmersive = false;
    actions.resetImmersiveTimer();
  },

  /**
   * 注册用户交互（重置沉浸定时器）
   */
  registerInteraction() {
    if (state.ui.isImmersive) {
      actions.exitImmersive();
    } else {
      actions.resetImmersiveTimer();
    }
  },

  /**
   * 重置沉浸模式定时器
   */
  resetImmersiveTimer() {
    if (_immersiveTimer) {
      clearTimeout(_immersiveTimer);
    }
    if (state.ui.isPlayerViewOpen) {
      _immersiveTimer = setTimeout(() => {
        actions.enterImmersive();
      }, IMMERSIVE_DELAY);
    }
  },

  /**
   * 设备检测（768px 断点）
   * @param {number} width - 视口宽度
   */
  detectDevice(width) {
    const newDevice = width >= 768 ? 'desktop' : 'mobile';
    if (state.ui.currentDevice !== newDevice) {
      state.ui.currentDevice = newDevice;
    }
  },

  /**
   * 清理资源
   */
  dispose() {
    if (state.audioElement) {
      state.audioElement.pause();
      state.audioElement.src = '';
      cleanupAudioEvents();
      state.audioElement = null;
    }

    // 释放当前歌曲资源
    if (state.currentTrack) {
      state.currentTrack.releaseAudio();
    }

    // 清理沉浸模式定时器
    if (_immersiveTimer) {
      clearTimeout(_immersiveTimer);
      _immersiveTimer = null;
    }

    state.currentTrack = null;
    state.isPlaying = false;
    state.playlist = [];
    state.currentIndex = -1;
    state.ui.isPlayerViewOpen = false;
    state.ui.isImmersive = false;
  }
};

// 初始化
initAudioElement();

// 导出 PlayerStore
export const PlayerStore = {
  // 只读状态
  state: readonly(state),

  // Getters
  ...getters,

  // Actions
  ...actions,

  // 播放模式枚举
  PlayMode
};

// 默认导出
export default PlayerStore;
