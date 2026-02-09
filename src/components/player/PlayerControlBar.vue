<template>
  <div class="player-control-bar" v-if="PlayerStore.hasCurrentTrack.value" :style="{ '--player-bar-height': barHeight + 'px' }" ref="controlBarRef">
    <!-- 进度条 -->
    <div class="progress-section">
      <div 
        class="progress-bar" 
        @click="onProgressClick"
        @mousedown="onProgressMouseDown"
        ref="progressBarRef"
      >
        <div class="progress-track">
          <div 
            class="progress-fill" 
            :style="{ width: progressPercent + '%' }"
          ></div>
          <div 
            class="progress-handle" 
            :style="{ left: progressPercent + '%' }"
            :class="{ dragging: isDragging }"
          ></div>
        </div>
      </div>
      <div class="time-display">
        <span class="current-time">{{ PlayerStore.formattedCurrentTime.value }}</span>
        <span class="time-separator">/</span>
        <span class="total-time">{{ PlayerStore.formattedDuration.value }}</span>
      </div>
    </div>
    
    <!-- 控制区域 -->
    <div class="control-section">
      <!-- 歌曲信息 -->
      <div class="track-info" @click="goToPlayerView">
        <div class="track-cover">
          <img 
            v-if="currentCoverUrl" 
            :src="currentCoverUrl" 
            :alt="currentTrackTitle"
          />
          <div v-else class="cover-placeholder">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M9 18V5l12-2v13"/>
              <circle cx="6" cy="18" r="3"/>
              <circle cx="18" cy="16" r="3"/>
            </svg>
          </div>
        </div>
        <div class="track-meta">
          <div class="track-title" :title="currentTrackTitle">{{ currentTrackTitle }}</div>
          <div class="track-artist" :title="currentTrackArtist">{{ currentTrackArtist }}</div>
        </div>
      </div>
      
      <!-- 播放控制 -->
      <div class="playback-controls">
        <!-- 播放模式 -->
        <button 
          class="control-btn mode-btn" 
          @click="PlayerStore.togglePlayMode()"
          :title="playModeText"
        >
          <svg v-if="playMode === 'sequence'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M4 4h16v16H4z" style="display:none"/>
            <path d="M3 6h18M3 12h18M3 18h18"/>
          </svg>
          <svg v-else-if="playMode === 'random'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M16 3h5v5M4 20L21 3M21 16v5h-5M15 15l6 6M4 4l5 5"/>
          </svg>
          <svg v-else-if="playMode === 'loop'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M17 2l4 4-4 4M3 12V9a4 4 0 014-4h13M7 22l-4-4 4-4M21 12v3a4 4 0 01-4 4H4"/>
          </svg>
          <svg v-else-if="playMode === 'loop_one'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M17 2l4 4-4 4M3 12V9a4 4 0 014-4h13M7 22l-4-4 4-4M21 12v3a4 4 0 01-4 4H4"/>
            <text x="12" y="15" text-anchor="middle" font-size="8" fill="currentColor">1</text>
          </svg>
        </button>
        
        <!-- 上一首 -->
        <button 
          class="control-btn previous-btn" 
          @click="PlayerStore.playPrevious()"
          :disabled="!PlayerStore.canPlayPrevious.value"
          title="上一首"
        >
          <svg viewBox="0 0 24 24" fill="currentColor">
            <path d="M6 6h2v12H6zm3.5 6l8.5 6V6z"/>
          </svg>
        </button>
        
        <!-- 播放/暂停 -->
        <button 
          class="control-btn play-btn" 
          @click="PlayerStore.togglePlay()"
          :title="isPlaying ? '暂停' : '播放'"
        >
          <svg v-if="!isPlaying" viewBox="0 0 24 24" fill="currentColor">
            <path d="M8 5v14l11-7z"/>
          </svg>
          <svg v-else viewBox="0 0 24 24" fill="currentColor">
            <path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z"/>
          </svg>
        </button>
        
        <!-- 下一首 -->
        <button 
          class="control-btn next-btn" 
          @click="PlayerStore.playNext()"
          :disabled="!PlayerStore.canPlayNext.value"
          title="下一首"
        >
          <svg viewBox="0 0 24 24" fill="currentColor">
            <path d="M6 18l8.5-6L6 6v12zM16 6v12h2V6h-2z"/>
          </svg>
        </button>
        
        <!-- 歌词按钮 -->
        <button 
          class="control-btn lyrics-btn" 
          @click="toggleLyrics"
          :class="{ active: showLyrics }"
          title="歌词"
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
          </svg>
        </button>
      </div>
      
      <!-- 音量控制 -->
      <div class="volume-section">
        <button 
          class="control-btn volume-btn" 
          @click="PlayerStore.toggleMute()"
          :title="muted ? '取消静音' : '静音'"
        >
          <svg v-if="muted || volume === 0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z"/>
            <path d="M17 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2"/>
          </svg>
          <svg v-else-if="volume < 0.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z"/>
            <path d="M15.536 8.464a5 5 0 010 7.072"/>
          </svg>
          <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z"/>
            <path d="M15.536 8.464a5 5 0 010 7.072m2.828-9.9a9 9 0 010 12.728"/>
          </svg>
        </button>
        
        <div 
          class="volume-slider" 
          @click="onVolumeClick"
          @mousedown="onVolumeMouseDown"
          ref="volumeSliderRef"
        >
          <div class="volume-track">
            <div 
              class="volume-fill" 
              :style="{ width: (muted ? 0 : volume * 100) + '%' }"
            ></div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import { ref, computed, onMounted, onUnmounted, watch } from 'vue';
import { useRouter } from 'vue-router';
import PlayerStore from '@/stores/player.js';
import { useCoverImage } from '@/composables/useCoverImage';

export default {
  name: 'PlayerControlBar',
  setup() {
    const router = useRouter();
    const progressBarRef = ref(null);
    const volumeSliderRef = ref(null);
    const isDragging = ref(false);
    const isVolumeDragging = ref(false);
    const barHeight = ref(100);
    const controlBarRef = ref(null);

    // 从 PlayerStore 获取状态
    const isPlaying = computed(() => PlayerStore.state.isPlaying);
    const currentTrack = computed(() => PlayerStore.state.currentTrack);
    const volume = computed(() => PlayerStore.state.volume);
    const muted = computed(() => PlayerStore.state.muted);
    const playMode = computed(() => PlayerStore.state.playMode);
    const showLyrics = computed(() => PlayerStore.state.showLyrics);
    const progressPercent = computed(() => PlayerStore.progress.value);

    // 当前歌曲信息
    const currentTrackTitle = computed(() => {
      return currentTrack.value?.getDisplayTitle?.() || currentTrack.value?.title || '未知歌曲';
    });

    const currentTrackArtist = computed(() => {
      return currentTrack.value?.getDisplayArtist?.() || currentTrack.value?.artist || '未知歌手';
    });

    // 使用 useCoverImage 加载封面
    const { coverUrl: currentCoverUrlFromResource } = useCoverImage(currentTrack, 'small');

    const currentCoverUrl = computed(() => {
      // 优先使用从 ResourceManager 加载的封面
      return currentCoverUrlFromResource.value || currentTrack.value?.getCoverUrl?.() || currentTrack.value?.albumCoverData || '';
    });
    
    // 播放模式文本
    const playModeText = computed(() => {
      const modeTexts = {
        'sequence': '顺序播放',
        'random': '随机播放',
        'loop': '列表循环',
        'loop_one': '单曲循环'
      };
      return modeTexts[playMode.value] || '顺序播放';
    });
    
    // 进度条点击
    const onProgressClick = (e) => {
      if (!progressBarRef.value) return;
      const rect = progressBarRef.value.getBoundingClientRect();
      const percent = (e.clientX - rect.left) / rect.width;
      PlayerStore.seekToPercent(percent * 100);
    };
    
    // 进度条拖拽
    const onProgressMouseDown = (e) => {
      isDragging.value = true;
      document.addEventListener('mousemove', onProgressMouseMove);
      document.addEventListener('mouseup', onProgressMouseUp);
    };
    
    const onProgressMouseMove = (e) => {
      if (!isDragging.value || !progressBarRef.value) return;
      const rect = progressBarRef.value.getBoundingClientRect();
      const percent = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
      PlayerStore.seekToPercent(percent * 100);
    };
    
    const onProgressMouseUp = () => {
      isDragging.value = false;
      document.removeEventListener('mousemove', onProgressMouseMove);
      document.removeEventListener('mouseup', onProgressMouseUp);
    };
    
    // 音量控制
    const onVolumeClick = (e) => {
      if (!volumeSliderRef.value) return;
      const rect = volumeSliderRef.value.getBoundingClientRect();
      const percent = (e.clientX - rect.left) / rect.width;
      PlayerStore.setVolume(percent);
    };
    
    const onVolumeMouseDown = (e) => {
      isVolumeDragging.value = true;
      document.addEventListener('mousemove', onVolumeMouseMove);
      document.addEventListener('mouseup', onVolumeMouseUp);
    };
    
    const onVolumeMouseMove = (e) => {
      if (!isVolumeDragging.value || !volumeSliderRef.value) return;
      const rect = volumeSliderRef.value.getBoundingClientRect();
      const percent = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
      PlayerStore.setVolume(percent);
    };
    
    const onVolumeMouseUp = () => {
      isVolumeDragging.value = false;
      document.removeEventListener('mousemove', onVolumeMouseMove);
      document.removeEventListener('mouseup', onVolumeMouseUp);
    };
    
    // 切换歌词显示
    const toggleLyrics = () => {
      PlayerStore.toggleLyrics();
      if (PlayerStore.state.showLyrics) {
        router.push('/player');
      }
    };
    
    // 跳转到播放器页面
    const goToPlayerView = () => {
      router.push('/player');
    };
    
    onMounted(() => {
      // 组件挂载后计算实际高度
      if (controlBarRef.value) {
        barHeight.value = controlBarRef.value.offsetHeight;
      }
    });
    
    onUnmounted(() => {
      // 清理事件监听
      document.removeEventListener('mousemove', onProgressMouseMove);
      document.removeEventListener('mouseup', onProgressMouseUp);
      document.removeEventListener('mousemove', onVolumeMouseMove);
      document.removeEventListener('mouseup', onVolumeMouseUp);
    });
    
    return {
      PlayerStore,
      progressBarRef,
      volumeSliderRef,
      controlBarRef,
      isDragging,
      isPlaying,
      currentTrack,
      volume,
      muted,
      playMode,
      playModeText,
      showLyrics,
      progressPercent,
      currentTrackTitle,
      currentTrackArtist,
      currentCoverUrl,
      barHeight,
      onProgressClick,
      onProgressMouseDown,
      onVolumeClick,
      onVolumeMouseDown,
      toggleLyrics,
      goToPlayerView
    };
  }
};
</script>

<style scoped>
.player-control-bar {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  background: rgba(255, 255, 255, 0.95);
  backdrop-filter: blur(20px);
  border-top: 1px solid rgba(0, 0, 0, 0.1);
  padding: 8px 16px;
  z-index: 1000;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

/* 进度条区域 */
.progress-section {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 0 8px;
}

.progress-bar {
  flex: 1;
  height: 20px;
  display: flex;
  align-items: center;
  cursor: pointer;
  position: relative;
}

.progress-track {
  width: 100%;
  height: 4px;
  background: rgba(0, 0, 0, 0.1);
  border-radius: 2px;
  position: relative;
  overflow: visible;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, #0078d7, #00b4d8);
  border-radius: 2px;
  transition: width 0.1s linear;
}

.progress-handle {
  position: absolute;
  top: 50%;
  width: 12px;
  height: 12px;
  background: #0078d7;
  border-radius: 50%;
  transform: translate(-50%, -50%);
  opacity: 0;
  transition: opacity 0.2s, transform 0.2s;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
}

.progress-bar:hover .progress-handle,
.progress-handle.dragging {
  opacity: 1;
}

.time-display {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: #666;
  min-width: 90px;
  justify-content: flex-end;
}

.time-separator {
  opacity: 0.5;
}

/* 控制区域 */
.control-section {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

/* 歌曲信息 */
.track-info {
  display: flex;
  align-items: center;
  gap: 12px;
  flex: 1;
  min-width: 0;
  cursor: pointer;
  padding: 4px;
  border-radius: 8px;
  transition: background 0.2s;
}

.track-info:hover {
  background: rgba(0, 0, 0, 0.05);
}

.track-cover {
  width: 48px;
  height: 48px;
  border-radius: 6px;
  overflow: hidden;
  flex-shrink: 0;
  background: #f0f0f0;
}

.track-cover img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.cover-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #999;
}

.cover-placeholder svg {
  width: 24px;
  height: 24px;
}

.track-meta {
  min-width: 0;
  flex: 1;
}

.track-title {
  font-size: 14px;
  font-weight: 600;
  color: #333;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  line-height: 1.4;
}

.track-artist {
  font-size: 12px;
  color: #666;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  line-height: 1.4;
}

/* 播放控制按钮 */
.playback-controls {
  display: flex;
  align-items: center;
  gap: 8px;
}

.control-btn {
  width: 36px;
  height: 36px;
  border: none;
  background: transparent;
  border-radius: 50%;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #333;
  transition: all 0.2s;
}

.control-btn:hover:not(:disabled) {
  background: rgba(0, 0, 0, 0.1);
}

.control-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.control-btn svg {
  width: 20px;
  height: 20px;
}

.play-btn {
  width: 44px;
  height: 44px;
  background: #0078d7;
  color: white;
}

.play-btn:hover:not(:disabled) {
  background: #006cbd;
  transform: scale(1.05);
}

.play-btn svg {
  width: 24px;
  height: 24px;
}

.mode-btn svg,
.lyrics-btn svg {
  width: 18px;
  height: 18px;
}

.lyrics-btn.active {
  color: #0078d7;
  background: rgba(0, 120, 215, 0.1);
}

/* 音量控制 */
.volume-section {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  justify-content: flex-end;
}

.volume-btn svg {
  width: 20px;
  height: 20px;
}

.volume-slider {
  width: 80px;
  height: 20px;
  display: flex;
  align-items: center;
  cursor: pointer;
}

.volume-track {
  width: 100%;
  height: 4px;
  background: rgba(0, 0, 0, 0.1);
  border-radius: 2px;
  overflow: hidden;
}

.volume-fill {
  height: 100%;
  background: #0078d7;
  border-radius: 2px;
  transition: width 0.1s;
}

/* 移动端适配 */
@media (max-width: 767px) {
  .player-control-bar {
    padding: 6px 12px;
    gap: 4px;
  }

  .progress-section {
    padding: 0 4px;
    gap: 8px;
  }

  .time-display {
    min-width: auto;
    font-size: 10px;
  }

  .control-section {
    gap: 8px;
  }

  .track-info {
    flex: 1;
    min-width: 0;
    gap: 8px;
  }

  .track-cover {
    width: 36px;
    height: 36px;
  }

  .track-meta {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .track-title {
    font-size: 12px;
  }

  .track-artist {
    font-size: 10px;
  }

  .playback-controls {
    gap: 4px;
    flex-shrink: 0;
  }

  .control-btn {
    width: 32px;
    height: 32px;
  }

  .control-btn svg {
    width: 16px;
    height: 16px;
  }

  .play-btn {
    width: 40px;
    height: 40px;
  }

  .play-btn svg {
    width: 20px;
    height: 20px;
  }

  .mode-btn,
  .lyrics-btn {
    display: none;
  }

  .volume-section {
    display: none;
  }
}

@media (max-width: 480px) {
  .player-control-bar {
    padding: 4px 8px;
  }

  .track-cover {
    width: 32px;
    height: 32px;
  }

  .track-title {
    font-size: 11px;
  }

  .track-artist {
    font-size: 9px;
  }

  .control-btn {
    width: 28px;
    height: 28px;
  }

  .play-btn {
    width: 36px;
    height: 36px;
  }
}

/* 深色模式 */
@media (prefers-color-scheme: dark) {
  .player-control-bar {
    background: rgba(40, 40, 40, 0.95);
    border-top-color: rgba(255, 255, 255, 0.1);
  }
  
  .progress-track,
  .volume-track {
    background: rgba(255, 255, 255, 0.1);
  }
  
  .time-display {
    color: #aaa;
  }
  
  .track-title {
    color: #f0f0f0;
  }
  
  .track-artist {
    color: #aaa;
  }
  
  .control-btn {
    color: #f0f0f0;
  }
  
  .control-btn:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.1);
  }
  
  .track-info:hover {
    background: rgba(255, 255, 255, 0.05);
  }
  
  .track-cover {
    background: #3a3a3a;
  }
  
  .cover-placeholder {
    color: #666;
  }
}
</style>
