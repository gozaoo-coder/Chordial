<template>
  <div class="player-view">
    <!-- 背景 -->
    <PlayerBackground
      :cover-url="currentCoverUrl"
      :is-playing="isPlaying"
      :flow-speed="backgroundFlowSpeed"
      :fps="backgroundFps"
      :has-lyric="hasLyrics"
      :render-scale="backgroundRenderScale"
    />

    <!-- 内容区域 -->
    <div class="player-content">
      <!-- 顶部导航 -->
      <PlayerHeader
        @back="goBack"
        @settings="goToSettings"
      />

      <!-- 主内容区 -->
      <div class="player-main">
        <!-- 左侧：封面和歌曲信息 -->
        <div class="player-left">
          <div class="left-content">
            <!-- 专辑封面 -->
            <PlayerAlbumCard
              :cover-url="currentCoverUrl"
              :title="currentTrackTitle"
              :artist="currentTrackArtist"
              :album="currentTrackAlbum"
              :is-playing="isPlaying"
              @more="onMoreOptions"
            />

            <!-- 进度条区域 -->
            <div class="progress-section">
              <PlayerProgress
                :current-time="currentTime"
                :duration="duration"
                :progress-percent="progressPercent"
                quality="无损"
                @seek-to-percent="onSeekToPercent"
              />
            </div>

            <!-- 控制按钮区域 -->
            <div class="controls-section">
              <PlayerControls
                :is-playing="isPlaying"
                :play-mode="playMode"
                :can-play-previous="canPlayPrevious"
                :can-play-next="canPlayNext"
                :show-lyrics="true"
                @toggle-play="PlayerStore.togglePlay()"
                @previous="PlayerStore.playPrevious()"
                @next="PlayerStore.playNext()"
                @toggle-mode="PlayerStore.togglePlayMode()"
              />

              <PlayerVolume
                :volume="volume"
                :muted="muted"
                @set-volume="PlayerStore.setVolume"
                @toggle-mute="PlayerStore.toggleMute()"
              />
            </div>
          </div>
        </div>

        <!-- 右侧：歌词显示 -->
        <div class="player-right">
          <AMLLyrics
            :lyrics-data="lyricsData"
            :current-time="currentTime"
            :is-playing="isPlaying"
            :enable-spring="enableSpring"
            :enable-blur="enableBlur"
            :enable-scale="true"
            :align-position="0.5"
            align-anchor="center"
            :hide-passed-lines="false"
            :word-fade-width="0.5"
            @seek="onSeek"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import { ref, computed, onMounted, onUnmounted, watch } from 'vue';
import { useRouter } from 'vue-router';
import PlayerStore from '@/stores/player.js';
import {
  AMLLyrics,
  PlayerBackground,
  PlayerHeader,
  PlayerAlbumCard,
  PlayerProgress,
  PlayerControls,
  PlayerVolume
} from '@/components/player';
import { useCoverImage } from '@/composables/useCoverImage';

export default {
  name: 'PlayerView',
  components: {
    AMLLyrics,
    PlayerBackground,
    PlayerHeader,
    PlayerAlbumCard,
    PlayerProgress,
    PlayerControls,
    PlayerVolume
  },
  setup() {
    const router = useRouter();

    // AMLL 设置
    const enableSpring = ref(true);
    const enableBlur = ref(true);
    const backgroundFlowSpeed = ref(2);
    const backgroundFps = ref(30);
    const backgroundRenderScale = ref(0.5);

    // 从 PlayerStore 获取状态
    const isPlaying = computed(() => PlayerStore.state.isPlaying);
    const currentTrack = computed(() => PlayerStore.state.currentTrack);
    const currentTime = computed(() => PlayerStore.state.currentTime);
    const duration = computed(() => PlayerStore.state.duration);
    const volume = computed(() => PlayerStore.state.volume);
    const muted = computed(() => PlayerStore.state.muted);
    const playMode = computed(() => PlayerStore.state.playMode);
    const lyricsData = computed(() => PlayerStore.state.lyricsData);
    const canPlayPrevious = computed(() => PlayerStore.canPlayPrevious.value);
    const canPlayNext = computed(() => PlayerStore.canPlayNext.value);

    // 使用 useCoverImage 加载封面
    const { coverUrl: currentCoverUrl } = useCoverImage(currentTrack, 'large');

    // 是否有歌词
    const hasLyrics = computed(() => {
      return lyricsData.value?.hasSyncedLyrics || lyricsData.value?.hasPlainLyrics;
    });

    // 计算属性
    const currentTrackTitle = computed(() => {
      return currentTrack.value?.getDisplayTitle?.() || currentTrack.value?.title || '未知歌曲';
    });

    const currentTrackArtist = computed(() => {
      return currentTrack.value?.getDisplayArtist?.() || currentTrack.value?.artist || '未知歌手';
    });

    const currentTrackAlbum = computed(() => {
      return currentTrack.value?.getDisplayAlbum?.() || currentTrack.value?.album?.title || '';
    });

    const progressPercent = computed(() => {
      if (duration.value === 0) return 0;
      return (currentTime.value / duration.value) * 100;
    });

    // 方法
    const goBack = () => {
      PlayerStore.setShowLyrics(false);
      router.back();
    };

    const goToSettings = () => {
      router.push('/settings');
    };

    const onSeek = (time) => {
      PlayerStore.seek(time);
    };

    const onSeekToPercent = (percent) => {
      PlayerStore.seekToPercent(percent);
    };

    const onMoreOptions = () => {
      // TODO: 实现更多选项功能
      console.log('More options clicked');
    };

    onMounted(() => {
      console.log('PlayerView mounted');
      console.log('Current track:', currentTrack.value);
      console.log('Cover URL:', currentCoverUrl.value);
      console.log('Lyrics data:', lyricsData.value);
      console.log('Has lyrics:', hasLyrics.value);
    });

    // 监听当前歌曲变化
    watch(currentTrack, (newTrack) => {
      console.log('Current track changed:', newTrack);
      if (newTrack) {
        console.log('Track details:', {
          id: newTrack.id,
          title: newTrack.title,
          albumId: newTrack.albumId,
          album: newTrack.album,
          albumCoverData: newTrack.albumCoverData,
          lyrics: newTrack.lyrics?.substring(0, 100),
          syncedLyrics: newTrack.syncedLyrics?.substring(0, 100)
        });
      }
      console.log('Cover URL:', currentCoverUrl.value);
      console.log('Lyrics data:', lyricsData.value);
    }, { immediate: true });

    onUnmounted(() => {
      PlayerStore.setShowLyrics(false);
    });

    return {
      PlayerStore,
      enableSpring,
      enableBlur,
      backgroundFlowSpeed,
      backgroundFps,
      backgroundRenderScale,
      isPlaying,
      currentTrack,
      currentTime,
      duration,
      volume,
      muted,
      playMode,
      lyricsData,
      canPlayPrevious,
      canPlayNext,
      hasLyrics,
      currentTrackTitle,
      currentTrackArtist,
      currentTrackAlbum,
      currentCoverUrl,
      progressPercent,
      goBack,
      goToSettings,
      onSeek,
      onSeekToPercent,
      onMoreOptions
    };
  }
};
</script>

<style scoped>
.player-view {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 2000;
  overflow: hidden;
}

.player-content {
  position: relative;
  z-index: 1;
  height: 100%;
  display: flex;
  flex-direction: column;
  color: white;
}

/* 主内容区 */
.player-main {
  flex: 1;
  display: flex;
  padding: 3vh 4vw 4vh;
  gap: 5vw;
  min-height: 0;
  align-items: center;
}

/* 左侧：封面和歌曲信息 */
.player-left {
  flex: 0 0 min(28vw, 380px);
  display: flex;
  flex-direction: column;
  justify-content: center;
  padding: 0;
  min-height: 0;
}

.left-content {
  display: flex;
  flex-direction: column;
  gap: 3vh;
  width: 100%;
}

/* 进度条区域 */
.progress-section {
  width: 100%;
}

.progress-section :deep(.player-progress) {
  width: 100%;
}

/* 控制按钮区域 */
.controls-section {
  display: flex;
  flex-direction: column;
  align-items: stretch;
  gap: 1.5vh;
  width: 100%;
}

/* 右侧：歌词显示 */
.player-right {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  min-width: 0;
  min-height: 0;
  overflow: hidden;
  position: relative;
}

/* 响应式适配 */
@media (max-width: 1023px) {
  .player-main {
    flex-direction: column;
    padding: 2vh 3vw 4vh;
    gap: 2vh;
    overflow-y: auto;
  }

  .player-left {
    flex: 0 0 auto;
    justify-content: center;
    padding: 1vh 2vw;
  }

  .left-content {
    gap: 2vh;
  }

  .player-right {
    flex: 1;
    min-height: 30vh;
  }
}

@media (max-width: 767px) {
  .player-main {
    padding: 1.5vh 3vw 3vh;
    gap: 1.5vh;
  }

  .player-left {
    padding: 1vh 2vw;
  }

  .left-content {
    gap: 1.5vh;
  }

  .controls-section {
    gap: 1.5vh;
  }
}
</style>
