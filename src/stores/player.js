/**
 * PlayerStore - 全局播放器状态管理
 * 使用 Vue 3 Composition API 实现单例模式
 * 
 * 功能：
 * - 管理播放器核心状态（当前歌曲、播放状态、进度等）
 * - 提供播放控制方法（播放、暂停、跳转等）
 * - 管理播放列表和播放模式
 * - 自动处理音频资源生命周期
 * - 支持 Automix 智能混音功能
 */

import { reactive, readonly, computed } from 'vue';
import {
  analyzeAudioBeat,
  getMixPoints,
  setCrossfadeEnabled,
  setCrossfadeConfig,
  preloadNextAudio,
  setCurrentTrackBpm,
  setNextTrackBpm,
  setBpmSyncEnabled,
  setPlaybackSpeed,
  listenAnalysisProgress,
  AutomixConfig
} from '@/api/automix.js';

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

  // ========== Automix 相关状态 ==========
  automixEnabled: false,     // Automix 开关
  crossfadeEnabled: false,   // 交叉淡化开关
  crossfadeDuration: 10,     // 交叉淡化时长（秒）
  crossfadeCurve: 's_curve', // 交叉淡化曲线类型
  bpmSyncEnabled: false,     // BPM 同步开关
  playbackSpeed: 1.0,        // 播放速度 (0.5 - 2.0)
  
  // BPM 信息
  currentBpm: null,          // 当前歌曲 BPM
  currentBeatPositions: [],  // 当前歌曲 Beat 位置
  nextTrackBpm: null,        // 下一首歌曲 BPM
  nextTrackBeatPositions: [],// 下一首歌曲 Beat 位置
  
  // 分析状态
  isAnalyzing: false,        // 是否正在分析
  analysisProgress: 0,       // 分析进度 (0-100)
  analysisCurrentFile: '',   // 当前分析的文件
  
  // 预加载状态
  isPreloading: false,       // 是否正在预加载
  preloadedTrack: null       // 已预加载的曲目
});

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

  // ========== Automix 计算属性 ==========
  // 是否显示 Automix 控制
  showAutomixControls: computed(() => state.automixEnabled),
  
  // 格式化 BPM 显示
  formattedBpm: computed(() => {
    if (state.currentBpm) {
      return Math.round(state.currentBpm).toString();
    }
    return '--';
  }),
  
  // 格式化播放速度
  formattedSpeed: computed(() => {
    return `${Math.round(state.playbackSpeed * 100)}%`;
  })
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

  // 时间更新
  audioEventHandlers.timeupdate = () => {
    state.currentTime = audio.currentTime || 0;

    // Automix: 检查是否需要预加载下一首
    if (state.automixEnabled && state.crossfadeEnabled) {
      checkPreloadTrigger();
    }
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

// 检查是否需要触发预加载
function checkPreloadTrigger() {
  if (state.isPreloading || state.preloadedTrack) return;
  
  const remainingTime = state.duration - state.currentTime;
  const preloadThreshold = state.crossfadeDuration + 5; // 提前 crossfade + 5 秒
  
  if (remainingTime <= preloadThreshold && remainingTime > 0) {
    const nextTrack = getNextTrack();
    if (nextTrack) {
      actions.preloadNextTrack(nextTrack);
    }
  }
}

// 获取下一首歌曲
function getNextTrack() {
  if (state.playlist.length === 0) return null;
  
  let nextIndex;
  
  if (state.playMode === PlayMode.RANDOM) {
    nextIndex = Math.floor(Math.random() * state.playlist.length);
  } else {
    nextIndex = state.currentIndex + 1;
    if (nextIndex >= state.playlist.length) {
      nextIndex = 0;
    }
  }
  
  return state.playlist[nextIndex];
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
      // Automix 模式下使用交叉淡化
      if (state.automixEnabled && state.crossfadeEnabled && state.preloadedTrack) {
        actions.playPreloadedTrack();
      } else {
        actions.playNext();
      }
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
    if (!track || !track.id) {
      console.warn('play: 无效的 track 参数');
      return;
    }

    try {
      state.isLoading = true;
      state.error = null;

      // 如果提供了播放列表，更新播放列表
      if (playlist && Array.isArray(playlist)) {
        state.playlist = playlist;
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
      
      // 停止当前播放
      if (state.audioElement) {
        state.audioElement.pause();
        state.audioElement.currentTime = 0;
      }
      
      // 更新当前歌曲
      state.currentTrack = track;
      state.currentIndex = getTrackIndex(track);
      state.currentTime = 0;
      state.duration = track.duration || 0;
      
      // 重置预加载状态
      state.preloadedTrack = null;
      state.isPreloading = false;
      
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
      
      // 加载歌词
      await actions.loadLyrics(track);
      
      // 播放
      await state.audioElement.play();
      state.isPlaying = true;
      
      // Automix: 分析当前歌曲 BPM
      if (state.automixEnabled) {
        actions.analyzeCurrentTrack();
      }
      
    } catch (error) {
      console.error('播放失败:', error);
      state.error = error.message || '播放失败';
      state.isPlaying = false;
    } finally {
      state.isLoading = false;
    }
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
    if (state.audioElement) {
      const clampedTime = Math.max(0, Math.min(time, state.duration));
      state.audioElement.currentTime = clampedTime;
      state.currentTime = clampedTime;
    }
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
    if (state.playlist.length === 0) return;
    
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
  },
  
  /**
   * 播放下一首
   */
  playNext() {
    if (state.playlist.length === 0) return;
    
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
  },
  
  /**
   * 播放预加载的歌曲（Automix 模式）
   */
  async playPreloadedTrack() {
    if (!state.preloadedTrack) return;
    
    try {
      const track = state.preloadedTrack;
      state.preloadedTrack = null;
      
      // 更新索引
      state.currentIndex = getTrackIndex(track);
      state.currentTrack = track;
      state.currentTime = 0;
      state.duration = track.duration || 0;
      
      // 获取音频 URL
      const audioUrl = await track.getAudioBlobUrl();
      if (!audioUrl) {
        throw new Error('无法获取音频文件');
      }
      
      // 切换音频源
      state.audioElement.src = audioUrl;
      
      // 加载歌词
      await actions.loadLyrics(track);
      
      // 播放
      await state.audioElement.play();
      state.isPlaying = true;
      
      // 分析 BPM
      if (state.automixEnabled) {
        actions.analyzeCurrentTrack();
      }
      
    } catch (error) {
      console.error('播放预加载歌曲失败:', error);
      // 失败时播放下一首
      actions.playNext();
    }
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
    state.playlist = playlist || [];
    // 更新当前歌曲索引
    if (state.currentTrack) {
      state.currentIndex = getTrackIndex(state.currentTrack);
    }
  },
  
  /**
   * 添加歌曲到播放列表
   * @param {Track} track - 歌曲
   */
  addToPlaylist(track) {
    if (!track) return;
    
    // 检查是否已存在
    const exists = state.playlist.some(t => t.id === track.id);
    if (!exists) {
      state.playlist.push(track);
    }
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
    try {
      const lyricsInfo = track.getLyricsInfo();
      state.lyricsData = lyricsInfo;
    } catch (error) {
      console.warn('加载歌词失败:', error);
      state.lyricsData = {
        plainLyrics: '',
        syncedLyrics: '',
        hasSyncedLyrics: false,
        hasPlainLyrics: false
      };
    }
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
    
    state.currentTrack = null;
    state.isPlaying = false;
    state.playlist = [];
    state.currentIndex = -1;
  },

  // ========== Automix Actions ==========
  
  /**
   * 启用 Automix
   */
  async enableAutomix() {
    state.automixEnabled = true;
    
    // 启用交叉淡化
    await setCrossfadeEnabled(true);
    state.crossfadeEnabled = true;
    
    // 分析当前歌曲
    if (state.currentTrack) {
      actions.analyzeCurrentTrack();
    }
    
    console.log('Automix 已启用');
  },
  
  /**
   * 禁用 Automix
   */
  async disableAutomix() {
    state.automixEnabled = false;
    
    // 禁用交叉淡化
    await setCrossfadeEnabled(false);
    state.crossfadeEnabled = false;
    
    // 禁用 BPM 同步
    await setBpmSyncEnabled(false);
    state.bpmSyncEnabled = false;
    
    // 重置速度
    await setPlaybackSpeed(1.0);
    state.playbackSpeed = 1.0;
    
    console.log('Automix 已禁用');
  },
  
  /**
   * 切换 Automix 开关
   */
  async toggleAutomix() {
    if (state.automixEnabled) {
      await actions.disableAutomix();
    } else {
      await actions.enableAutomix();
    }
  },
  
  /**
   * 设置交叉淡化
   * @param {boolean} enabled - 是否启用
   * @param {number} duration - 时长（秒）
   * @param {string} curve - 曲线类型
   */
  async setCrossfade(enabled, duration = 10, curve = 's_curve') {
    state.crossfadeEnabled = enabled;
    state.crossfadeDuration = duration;
    state.crossfadeCurve = curve;
    
    await setCrossfadeEnabled(enabled);
    
    if (enabled) {
      await setCrossfadeConfig(duration, curve);
    }
  },
  
  /**
   * 设置 BPM 同步
   * @param {boolean} enabled - 是否启用
   */
  async setBpmSync(enabled) {
    state.bpmSyncEnabled = enabled;
    await setBpmSyncEnabled(enabled);
    
    if (enabled && state.currentBpm && state.nextTrackBpm) {
      // 计算速度比率
      const speedRatio = state.currentBpm / state.nextTrackBpm;
      await setPlaybackSpeed(speedRatio);
      state.playbackSpeed = speedRatio;
    }
  },
  
  /**
   * 设置播放速度
   * @param {number} speed - 速度比率 (0.5 - 2.0)
   */
  async setSpeed(speed) {
    const clampedSpeed = Math.max(0.5, Math.min(2.0, speed));
    state.playbackSpeed = clampedSpeed;
    await setPlaybackSpeed(clampedSpeed);
  },
  
  /**
   * 分析当前歌曲 BPM
   */
  async analyzeCurrentTrack() {
    if (!state.currentTrack || !state.currentTrack.path) return;
    
    try {
      state.isAnalyzing = true;
      
      const result = await analyzeAudioBeat(state.currentTrack.path);
      
      state.currentBpm = result.bpm;
      state.currentBeatPositions = result.beat_positions;
      
      // 发送到后端
      await setCurrentTrackBpm(result.bpm, result.beat_positions);
      
      console.log('当前歌曲 BPM 分析完成:', result.bpm);
    } catch (error) {
      console.error('分析当前歌曲 BPM 失败:', error);
    } finally {
      state.isAnalyzing = false;
    }
  },
  
  /**
   * 预加载下一首歌曲
   * @param {Track} track - 下一首歌曲
   */
  async preloadNextTrack(track) {
    if (!track || !track.path) return;
    
    try {
      state.isPreloading = true;
      
      // 分析下一首歌曲 BPM
      const result = await analyzeAudioBeat(track.path);
      
      state.nextTrackBpm = result.bpm;
      state.nextTrackBeatPositions = result.beat_positions;
      
      // 发送到后端
      await setNextTrackBpm(result.bpm, result.beat_positions);
      
      // 预加载音频
      await preloadNextAudio(track.path);
      
      state.preloadedTrack = track;
      
      console.log('下一首歌曲预加载完成:', track.title, 'BPM:', result.bpm);
    } catch (error) {
      console.error('预加载下一首歌曲失败:', error);
    } finally {
      state.isPreloading = false;
    }
  },
  
  /**
   * 初始化分析进度监听
   */
  async initAnalysisProgressListener() {
    const unlisten = await listenAnalysisProgress((progress) => {
      state.analysisProgress = progress.percent;
      state.analysisCurrentFile = progress.file;
    });
    
    return unlisten;
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
  PlayMode,
  
  // Automix 配置
  AutomixConfig
};

// 默认导出
export default PlayerStore;
