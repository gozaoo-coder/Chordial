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

import { reactive, readonly, computed } from 'vue';

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
  }
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
  playlistLength: computed(() => state.playlist.length)
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

// 设置音频事件监听
function setupAudioEvents() {
  const audio = state.audioElement;
  if (!audio) return;
  
  // 时间更新
  audio.addEventListener('timeupdate', () => {
    state.currentTime = audio.currentTime || 0;
  });
  
  // 加载完成
  audio.addEventListener('loadedmetadata', () => {
    state.duration = audio.duration || 0;
    state.isLoading = false;
  });
  
  // 可以播放
  audio.addEventListener('canplay', () => {
    state.isLoading = false;
  });
  
  // 等待加载
  audio.addEventListener('waiting', () => {
    state.isLoading = true;
  });
  
  // 播放结束
  audio.addEventListener('ended', () => {
    handleTrackEnded();
  });
  
  // 错误处理
  audio.addEventListener('error', (e) => {
    console.error('Audio playback error:', e);
    state.error = '播放出错';
    state.isLoading = false;
    state.isPlaying = false;
  });
  
  // 音量变化
  audio.addEventListener('volumechange', () => {
    state.volume = audio.volume;
    state.muted = audio.muted;
  });
}

// 清理音频事件监听
function cleanupAudioEvents() {
  const audio = state.audioElement;
  if (!audio) return;
  
  // 克隆音频元素以移除所有事件监听
  const newAudio = audio.cloneNode(true);
  audio.parentNode?.replaceChild(newAudio, audio);
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
      // 列表循环，播放下一首
      actions.playNext();
      break;
    case PlayMode.RANDOM:
      // 随机播放
      actions.playRandom();
      break;
    case PlayMode.SEQUENCE:
    default:
      // 顺序播放
      if (state.currentIndex < state.playlist.length - 1) {
        actions.playNext();
      } else {
        // 播放列表结束
        state.isPlaying = false;
        state.currentTime = 0;
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
