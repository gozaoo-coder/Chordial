<template>
  <div class="player-view">
    <!-- AMLL 流体背景容器 -->
    <div ref="bgContainerRef" class="amll-background-container"></div>

    <!-- 遮罩层 -->
    <div class="background-overlay"></div>

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
            <div class="cover-container">
              <div class="cover-wrapper" :class="{ playing: isPlaying }">
                <img
                  v-if="stableCoverUrl"
                  :src="stableCoverUrl"
                  :alt="currentTrackTitle"
                  class="album-cover"
                />
                <div v-else class="cover-placeholder">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                    <path d="M9 18V5l12-2v13"/>
                    <circle cx="6" cy="18" r="3"/>
                    <circle cx="18" cy="16" r="3"/>
                  </svg>
                </div>
              </div>
            </div>

            <!-- 歌曲信息 -->
            <div class="track-info">
              <h1 class="track-title" :title="currentTrackTitle">{{ currentTrackTitle }}</h1>
              <p class="track-artist" :title="currentTrackArtist">{{ currentTrackArtist }}</p>
              <p class="track-album" v-if="currentTrackAlbum" :title="currentTrackAlbum">{{ currentTrackAlbum }}</p>
            </div>

            <!-- 进度条区域 -->
            <div class="progress-section">
              <div class="quality-wrapper">
                <div class="quality-badge">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3"/>
                  </svg>
                  <span>无损</span>
                </div>
              </div>
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
              <div class="time-row">
                <span class="time current">{{ formattedCurrentTime }}</span>
                <span class="time duration">{{ formattedDuration }}</span>
              </div>
            </div>

            <!-- 控制按钮区域 -->
            <div class="controls-section">
              <!-- 播放控制 -->
              <div class="player-controls">
                <!-- 播放模式 -->
                <button
                  class="control-btn mode-btn"
                  @click="onToggleMode"
                  :title="playModeText"
                >
                  <svg v-if="playMode === 'sequence'" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M4 4h16v16H4z" style="display:none"/>
                    <path d="M3 6h18M3 12h18M3 18h18"/>
                  </svg>
                  <svg v-else-if="playMode === 'random'" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M16 3h5v5M4 20L21 3M21 16v5h-5M15 15l6 6M4 4l5 5"/>
                  </svg>
                  <svg v-else-if="playMode === 'loop'" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M17 2l4 4-4 4M3 12V9a4 4 0 014-4h13M7 22l-4-4 4-4M21 12v3a4 4 0 01-4 4H4"/>
                  </svg>
                  <svg v-else-if="playMode === 'loop_one'" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M17 2l4 4-4 4M3 12V9a4 4 0 014-4h13M7 22l-4-4 4-4M21 12v3a4 4 0 01-4 4H4"/>
                    <text x="12" y="15" text-anchor="middle" font-size="8" fill="currentColor">1</text>
                  </svg>
                </button>

                <!-- 上一首 -->
                <button
                  class="control-btn previous-btn"
                  @click="PlayerStore.playPrevious()"
                  :disabled="!canPlayPrevious"
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
                  :disabled="!canPlayNext"
                  title="下一首"
                >
                  <svg viewBox="0 0 24 24" fill="currentColor">
                    <path d="M6 18l8.5-6L6 6v12zM16 6v12h2V6h-2z"/>
                  </svg>
                </button>

                <!-- Automix 按钮 -->
                <button
                  class="control-btn automix-btn"
                  @click="PlayerStore.toggleAutomix()"
                  :class="{ active: automixEnabled }"
                  title="智能混音"
                >
                  <svg viewBox="0 0 24 24" fill="currentColor">
                    <path d="M12 3v10.55c-.59-.34-1.27-.55-2-.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z"/>
                  </svg>
                </button>
              </div>

              <!-- 音量控制 -->
              <div class="player-volume">
                <button
                  class="volume-btn"
                  @click="PlayerStore.toggleMute()"
                  :title="muted ? '取消静音' : '静音'"
                >
                  <svg v-if="muted || volume === 0" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z"/>
                    <path d="M17 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2"/>
                  </svg>
                  <svg v-else-if="volume < 0.5" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z"/>
                    <path d="M15.536 8.464a5 5 0 010 7.072"/>
                  </svg>
                  <svg v-else viewBox="0 0 24 24" fill="currentColor">
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
                    <div
                      class="volume-handle"
                      :style="{ left: (muted ? 0 : volume * 100) + '%' }"
                    ></div>
                  </div>
                </div>
              </div>
            </div>

            <!-- Automix 控制面板 -->
            <div v-if="showAutomixPanel" class="automix-panel">
              <AutomixControls />
            </div>
          </div>
        </div>

        <!-- 右侧：AMLL 歌词显示 -->
        <div class="player-right">
          <div ref="lyricContainerRef" class="amll-lyric-container"></div>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import { ref, computed, onMounted, onUnmounted, watch, shallowRef } from 'vue';
import { useRouter } from 'vue-router';
import { BackgroundRender, PixiRenderer, DomLyricPlayer } from '@applemusic-like-lyrics/core';
import PlayerStore from '@/stores/player.js';
import { PlayerHeader, AutomixControls } from '@/components/player';
import { useCoverImage } from '@/composables/useCoverImage';
import { parseLyrics } from '@/utils/lyricConverter.js';

// 导入 AMLL player 的样式
import '@/styles/amll-player.css';

export default {
  name: 'PlayerView',
  components: {
    PlayerHeader,
    AutomixControls
  },
  setup() {
    const router = useRouter();
    const lyricContainerRef = ref(null);
    const progressBarRef = ref(null);
    const volumeSliderRef = ref(null);
    const isDragging = ref(false);
    const bgContainerRef = ref(null);
    let bgRender = null;
    let lyricPlayer = null;
    let animationFrameId = null;
    
    // AMLL 设置
    const enableSpring = ref(true);
    const enableBlur = ref(true);
    const enableScale = ref(true);
    const backgroundFlowSpeed = ref(2);
    const backgroundFps = ref(30);
    const backgroundRenderScale = ref(0.5);
    const backgroundStaticMode = ref(false);
    
    // AMLL 弹簧参数（按照官方 test.ts 的默认值）
    const linePosYSpringParams = { mass: 1, damping: 15, stiffness: 100 };
    const linePosXSpringParams = { mass: 1, damping: 10, stiffness: 100 };
    const lineScaleSpringParams = { mass: 1, damping: 20, stiffness: 100 };
    
    // 歌词缓存
    let lastLyricsHash = '';

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
    const automixEnabled = computed(() => PlayerStore.state.automixEnabled);
    const showAutomixPanel = computed(() => PlayerStore.state.automixEnabled);

    // 是否有歌词（提前定义，因为后面的 watch 要用到）
    const hasLyrics = computed(() => {
      return lyricsData.value?.hasSyncedLyrics || lyricsData.value?.hasPlainLyrics;
    });

    // 使用 useCoverImage 加载封面
    const { coverUrl } = useCoverImage(currentTrack, 'large');

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

    const currentTimeMs = computed(() => {
      return Math.round(currentTime.value * 1000);
    });

    const formattedCurrentTime = computed(() => formatTime(currentTime.value));
    const formattedDuration = computed(() => formatTime(duration.value));

    const playModeText = computed(() => {
      const modeTexts = {
        'sequence': '顺序播放',
        'random': '随机播放',
        'loop': '列表循环',
        'loop_one': '单曲循环'
      };
      return modeTexts[playMode.value] || '顺序播放';
    });

    // 格式化时间
    function formatTime(seconds) {
      if (!seconds || isNaN(seconds)) return '0:00';
      const mins = Math.floor(seconds / 60);
      const secs = Math.floor(seconds % 60);
      return `${mins}:${secs.toString().padStart(2, '0')}`;
    }

    // 检测歌词格式
    const detectLyricFormat = (content) => {
      if (!content) return 'lrc';
      const trimmed = content.trim();

      if (trimmed.includes('<?xml') || trimmed.includes('<tt')) {
        return 'ttml';
      }

      if ((trimmed.startsWith('[') && trimmed.includes('"time"')) ||
          (trimmed.startsWith('{') && trimmed.includes('"lyrics"'))) {
        try {
          JSON.parse(trimmed);
          return 'json';
        } catch (e) {
          // 不是有效的 JSON
        }
      }

      if (/^\[\d+,\d+\]/.test(trimmed)) {
        return 'qrc';
      }

      if (/\[\d+,\d+\].*\(\d+,\d+,\d+\)/.test(trimmed)) {
        return 'yrc';
      }

      return 'lrc';
    };

    // 默认歌词行持续时间
    const DEFAULT_LINE_DURATION = 5000;
    const DEFAULT_WORD_DURATION = 200;

    // 获取歌词内容
    const getLyricsContent = () => {
      if (lyricsData.value?.syncedLyrics) {
        return lyricsData.value.syncedLyrics;
      }

      if (!lyricsData.value?.plainLyrics) {
        return null;
      }

      const lrcTimeRegex = /\[\d{1,2}:\d{2}\.\d{2,3}\]/;
      if (lrcTimeRegex.test(lyricsData.value.plainLyrics)) {
        return lyricsData.value.plainLyrics;
      }

      return convertPlainTextToLrc(lyricsData.value.plainLyrics);
    };

    // 将纯文本转换为 LRC 格式
    const convertPlainTextToLrc = (plainText) => {
      const lines = plainText.split('\n').filter(line => line.trim());
      return lines.map((line, index) => {
        const timeMs = index * DEFAULT_LINE_DURATION;
        const timeTag = formatTimeToLrcTag(timeMs);
        return `${timeTag}${line.trim()}`;
      }).join('\n');
    };

    // 将毫秒格式化为 LRC 时间标签
    const formatTimeToLrcTag = (timeMs) => {
      const mins = Math.floor(timeMs / 60000).toString().padStart(2, '0');
      const secs = Math.floor((timeMs % 60000) / 1000).toString().padStart(2, '0');
      return `[${mins}:${secs}.00]`;
    };

    // 计算歌词行结束时间
    const calculateEndTime = (line, index, array) => {
      const startTime = line.time || 0;
      if (line.duration) {
        return startTime + line.duration;
      }
      return array[index + 1]?.time || startTime + DEFAULT_LINE_DURATION;
    };

    // 转换逐字歌词
    const convertWordList = (wordList, lineStartTime) => {
      return wordList.map(word => ({
        startTime: word.time || lineStartTime,
        endTime: (word.time || lineStartTime) + (word.duration || DEFAULT_WORD_DURATION),
        word: word.text || '',
        romanWord: '',
        obscene: false
      }));
    };

    // 创建单行歌词的 word 列表
    const createWordList = (startTime, endTime, text) => [{
      startTime,
      endTime,
      word: text || '',
      romanWord: '',
      obscene: false
    }];

    // 将解析后的歌词转换为 AMLL 格式
    const convertToAmllFormat = (parsedLyrics) => {
      return parsedLyrics.map((line, index, array) => {
        const startTime = line.time || 0;
        const endTime = calculateEndTime(line, index, array);

        const words = line.words?.length > 0
          ? convertWordList(line.words, startTime)
          : createWordList(startTime, endTime, line.text);

        return {
          startTime,
          endTime,
          words,
          translatedLyric: line.translation || '',
          romanLyric: '',
          isBG: false,
          isDuet: false
        };
      });
    };

    // 简单的字符串哈希函数
    const hashString = (str) => {
      let hash = 0;
      if (str.length === 0) return hash;
      for (let i = 0; i < str.length; i++) {
        const char = str.charCodeAt(i);
        hash = ((hash << 5) - hash) + char;
        hash = hash & hash;
      }
      return hash.toString();
    };

    // 解析歌词并更新（只在内容真正变化时更新）
    const updateLyricLines = () => {
      console.log('[Lyric] updateLyricLines called');
      
      if (!hasLyrics.value) {
        if (amllLyricLines.value.length > 0) {
          console.log('[Lyric] No lyrics, clearing');
          amllLyricLines.value = [];
          lastLyricsHash = '';
        }
        return;
      }

      try {
        const lyricsContent = getLyricsContent();
        if (!lyricsContent) {
          if (lyricPlayer) {
            console.log('[Lyric] Empty lyrics content, clearing');
            lyricPlayer.setLyricLines([]);
          }
          lastLyricsHash = '';
          return;
        }

        // 计算哈希，避免重复解析相同内容
        const currentHash = hashString(lyricsContent);
        if (currentHash === lastLyricsHash) {
          console.log('[Lyric] Lyrics content unchanged, skipping update');
          return;
        }

        console.log('[Lyric] Lyrics content changed, parsing...');
        lastLyricsHash = currentHash;

        const format = detectLyricFormat(lyricsContent);
        const parsedLyrics = parseLyrics(lyricsContent, format);

        if (!parsedLyrics?.length) {
          console.log('[Lyric] No parsed lyrics, clearing');
          if (lyricPlayer) {
            lyricPlayer.setLyricLines([]);
          }
          return;
        }

        const result = convertToAmllFormat(parsedLyrics);
        console.log('[Lyric] Successfully parsed', result.length, 'lines');
        
        // 将结果转换为普通对象，避免 Proxy 问题
        const plainResult = JSON.parse(JSON.stringify(result));
        console.log('[Lyric] Plain result sample:', plainResult[0]);
        
        // 直接设置歌词到 DomLyricPlayer（内部会自动调用 calcLayout）
        if (lyricPlayer) {
          lyricPlayer.setLyricLines(plainResult);
        }
      } catch (error) {
        console.error('[Lyric] 解析歌词失败:', error);
        if (lyricPlayer) {
          lyricPlayer.setLyricLines([]);
        }
      }
    };

    // 监听歌词数据变化，更新歌词（只监听具体字段）
    watch([
      () => lyricsData.value?.plainLyrics,
      () => lyricsData.value?.syncedLyrics
    ], () => {
      console.log('[Lyric] lyrics content changed');
      updateLyricLines();
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
      document.addEventListener('mousemove', onVolumeMouseMove);
      document.addEventListener('mouseup', onVolumeMouseUp);
    };

    const onVolumeMouseMove = (e) => {
      if (!volumeSliderRef.value) return;
      const rect = volumeSliderRef.value.getBoundingClientRect();
      const percent = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
      PlayerStore.setVolume(percent);
    };

    const onVolumeMouseUp = () => {
      document.removeEventListener('mousemove', onVolumeMouseMove);
      document.removeEventListener('mouseup', onVolumeMouseUp);
    };

    // 播放模式切换
    const onToggleMode = () => {
      PlayerStore.togglePlayMode();
    };

    // 导航方法
    const goBack = () => {
      PlayerStore.setShowLyrics(false);
      router.back();
    };

    const goToSettings = () => {
      router.push('/settings');
    };

    onMounted(() => {
      console.log('[PlayerView] mounted');
      
      // 初始化 DomLyricPlayer
      if (lyricContainerRef.value) {
        console.log('[Lyric] Initializing DomLyricPlayer...');
        lyricPlayer = new DomLyricPlayer();
        
        // 设置弹簧参数
        lyricPlayer.setLinePosYSpringParams(linePosYSpringParams);
        lyricPlayer.setLinePosXSpringParams(linePosXSpringParams);
        lyricPlayer.setLineScaleSpringParams(lineScaleSpringParams);
        
        // 设置对齐方式
        lyricPlayer.setAlignPosition(0.5);
        lyricPlayer.setAlignAnchor('center');
        
        // 设置其他参数
        lyricPlayer.setEnableSpring(enableSpring.value);
        lyricPlayer.setEnableBlur(enableBlur.value);
        lyricPlayer.setEnableScale(enableScale.value);
        lyricPlayer.setHidePassedLines(false);
        
        // 监听歌词行点击事件
        lyricPlayer.addEventListener('line-click', (e) => {
          e.preventDefault();
          e.stopImmediatePropagation();
          e.stopPropagation();
          if (e.line) {
            const timeInSeconds = e.line.getLine().startTime / 1000;
            PlayerStore.seek(timeInSeconds);
          }
        });
        
        // 将歌词播放器元素添加到容器
        lyricContainerRef.value.appendChild(lyricPlayer.getElement());
        console.log('[Lyric] DomLyricPlayer initialized');
        
        // 启动动画帧（按照官方 test.ts 的方式）
        let lastTime = -1;
        const frame = (time) => {
          if (lastTime === -1) {
            lastTime = time;
          }
          // 更新歌词时间（只有在播放时才更新）
          if (isPlaying.value) {
            lyricPlayer.setCurrentTime(Math.round(currentTime.value * 1000));
          }
          // 始终更新动画（按照官方 test.ts）
          lyricPlayer.update(time - lastTime);
          lastTime = time;
          animationFrameId = requestAnimationFrame(frame);
        };
        animationFrameId = requestAnimationFrame(frame);
        
        // 初始化歌词
        console.log('[Lyric] Component mounted, initial update');
        updateLyricLines();
      }
      
      // 按 test.ts 的方式初始化 AMLL BackgroundRender
      if (bgContainerRef.value) {
        console.log('[AMLL] Initializing BackgroundRender...');
        bgRender = BackgroundRender.new(PixiRenderer);
        const canvas = bgRender.getElement();
        
        // 设置 canvas 样式（完全按照 test.ts）
        canvas.style.position = 'absolute';
        canvas.style.top = '0';
        canvas.style.left = '0';
        canvas.style.width = '100%';
        canvas.style.height = '100%';
        
        bgContainerRef.value.appendChild(canvas);
        console.log('[AMLL] Canvas appended to container');
        
        // 设置初始参数
        bgRender.setRenderScale(backgroundRenderScale.value);
        bgRender.setFlowSpeed(backgroundFlowSpeed.value);
        bgRender.setFPS(backgroundFps.value);
        bgRender.setStaticMode(backgroundStaticMode.value);
        bgRender.setLowFreqVolume(1.0);
        console.log('[AMLL] Initial parameters set');
        
        // 如果已有封面，立即设置
        if (coverUrl.value) {
          console.log('[AMLL] Setting initial album:', coverUrl.value);
          bgRender.setAlbum(coverUrl.value);
        }
        
        // 如果正在播放，立即 resume
        if (isPlaying.value) {
          console.log('[AMLL] Resuming playback');
          bgRender.resume();
        }
        
        // 设置 hasLyric
        bgRender.setHasLyric(hasLyrics.value);
        console.log('[AMLL] Initialization complete');
      }
    });

    // 分别监听各个状态变化，避免不必要的更新
    watch(coverUrl, (newUrl) => {
      if (!bgRender || !newUrl) return;
      console.log('[AMLL] coverUrl changed:', newUrl);
      bgRender.setAlbum(newUrl);
    });

    watch(isPlaying, (newIsPlaying) => {
      if (!bgRender) return;
      console.log('[AMLL] isPlaying changed:', newIsPlaying);
      if (newIsPlaying) {
        bgRender.resume();
      } else {
        bgRender.pause();
      }
    });

    watch(hasLyrics, (newHasLyrics) => {
      if (!bgRender) return;
      console.log('[AMLL] hasLyrics changed:', newHasLyrics);
      bgRender.setHasLyric(newHasLyrics);
    });

    watch(backgroundFlowSpeed, (newSpeed) => {
      if (!bgRender) return;
      console.log('[AMLL] backgroundFlowSpeed changed:', newSpeed);
      bgRender.setFlowSpeed(newSpeed);
    });

    watch(backgroundFps, (newFps) => {
      if (!bgRender) return;
      console.log('[AMLL] backgroundFps changed:', newFps);
      bgRender.setFPS(newFps);
    });

    watch(backgroundRenderScale, (newScale) => {
      if (!bgRender) return;
      console.log('[AMLL] backgroundRenderScale changed:', newScale);
      bgRender.setRenderScale(newScale);
    });

    watch(backgroundStaticMode, (newStaticMode) => {
      if (!bgRender) return;
      console.log('[AMLL] backgroundStaticMode changed:', newStaticMode);
      bgRender.setStaticMode(newStaticMode);
    });

    onUnmounted(() => {
      console.log('[PlayerView] unmounting');
      PlayerStore.setShowLyrics(false);
      document.removeEventListener('mousemove', onProgressMouseMove);
      document.removeEventListener('mouseup', onProgressMouseUp);
      document.removeEventListener('mousemove', onVolumeMouseMove);
      document.removeEventListener('mouseup', onVolumeMouseUp);
      
      // 停止动画帧
      if (animationFrameId) {
        cancelAnimationFrame(animationFrameId);
        animationFrameId = null;
      }
      
      // 销毁 DomLyricPlayer
      if (lyricPlayer) {
        console.log('[Lyric] Disposing DomLyricPlayer');
        lyricPlayer.dispose();
        lyricPlayer = null;
      }
      
      // 销毁 AMLL 实例
      if (bgRender) {
        console.log('[AMLL] Disposing BackgroundRender');
        bgRender.dispose();
        bgRender = null;
      }
    });

    return {
      PlayerStore,
      lyricContainerRef,
      progressBarRef,
      volumeSliderRef,
      isDragging,
      enableSpring,
      enableBlur,
      enableScale,
      backgroundFlowSpeed,
      backgroundFps,
      backgroundRenderScale,
      backgroundStaticMode,
      isPlaying,
      currentTrack,
      currentTime,
      currentTimeMs,
      duration,
      volume,
      muted,
      playMode,
      playModeText,
      lyricsData,
      canPlayPrevious,
      canPlayNext,
      hasLyrics,
      automixEnabled,
      showAutomixPanel,
      currentTrackTitle,
      currentTrackArtist,
      currentTrackAlbum,
      stableCoverUrl: coverUrl,
      bgContainerRef,
      progressPercent,
      formattedCurrentTime,
      formattedDuration,
      goBack,
      goToSettings,
      onProgressClick,
      onProgressMouseDown,
      onVolumeClick,
      onVolumeMouseDown,
      onToggleMode
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

/* AMLL 流体背景容器 */
.amll-background-container {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  z-index: 0;
}

/* AMLL 歌词容器 */
.amll-lyric-container {
  width: 100%;
  height: 100%;
  overflow: hidden;
}

/* 遮罩层 */
.background-overlay {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: rgba(0, 0, 0, 0.3);
  z-index: 1;
}

.player-content {
  position: relative;
  z-index: 2;
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

/* 封面容器 */
.cover-container {
  position: relative;
}

.cover-wrapper {
  width: 280px;
  height: 280px;
  border-radius: 12px;
  overflow: hidden;
  box-shadow: 0 24px 80px rgba(0, 0, 0, 0.6);
  transition: transform 0.3s ease;
  will-change: transform;
}

.cover-wrapper.playing {
  animation: pulse 4s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% {
    transform: scale3d(1, 1, 1);
  }
  50% {
    transform: scale3d(1.02, 1.02, 1);
  }
}

.album-cover {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.cover-placeholder {
  width: 100%;
  height: 100%;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  display: flex;
  align-items: center;
  justify-content: center;
}

.cover-placeholder svg {
  width: 64px;
  height: 64px;
  opacity: 0.5;
  color: white;
}

/* 歌曲信息 */
.track-info {
  text-align: left;
  width: 100%;
  max-width: min(30vmin, 320px);
}

.track-title {
  font-size: 22px;
  font-weight: 700;
  margin: 0 0 8px 0;
  color: white;
  text-shadow: 0 2px 10px rgba(0, 0, 0, 0.3);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  letter-spacing: -0.3px;
}

.track-artist {
  font-size: 15px;
  opacity: 0.85;
  margin: 0 0 4px 0;
  color: white;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  font-weight: 500;
}

.track-album {
  font-size: 13px;
  opacity: 0.6;
  margin: 0;
  color: white;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* 进度条区域 */
.progress-section {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: clamp(4px, 0.8vh, 8px);
}

.quality-wrapper {
  display: flex;
  justify-content: center;
  width: 100%;
}

.quality-badge {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: clamp(2px, 0.4vh, 4px) clamp(6px, 0.8vw, 10px);
  background: rgba(255, 255, 255, 0.12);
  border-radius: clamp(4px, 0.6vmin, 6px);
  font-size: clamp(10px, 1.2vh, 12px);
  color: rgba(255, 255, 255, 0.85);
  backdrop-filter: blur(10px);
  font-weight: 500;
}

.quality-badge svg {
  width: clamp(10px, 1.2vh, 12px);
  height: clamp(10px, 1.2vh, 12px);
}

.progress-bar {
  width: 100%;
  height: clamp(16px, 2vh, 20px);
  display: flex;
  align-items: center;
  cursor: pointer;
  position: relative;
}

.progress-track {
  width: 100%;
  height: clamp(4px, 0.6vh, 5px);
  background: rgba(255, 255, 255, 0.2);
  border-radius: clamp(2px, 0.4vmin, 3px);
  position: relative;
  overflow: visible;
}

.progress-fill {
  height: 100%;
  background: white;
  border-radius: clamp(2px, 0.4vmin, 3px);
  transition: width 0.1s linear;
}

.progress-handle {
  position: absolute;
  top: 50%;
  width: clamp(10px, 1.2vh, 12px);
  height: clamp(10px, 1.2vh, 12px);
  background: white;
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

.time-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
}

.time {
  font-size: clamp(10px, 1.2vh, 12px);
  color: rgba(255, 255, 255, 0.6);
  font-variant-numeric: tabular-nums;
  min-width: clamp(30px, 3vw, 36px);
}

/* 控制按钮区域 */
.controls-section {
  display: flex;
  flex-direction: column;
  align-items: stretch;
  gap: 1.5vh;
  width: 100%;
}

/* 播放控制 */
.player-controls {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: clamp(12px, 1.5vw, 24px);
  flex-wrap: nowrap;
  flex-shrink: 0;
}

.control-btn {
  width: clamp(40px, 5vmin, 52px);
  height: clamp(40px, 5vmin, 52px);
  border: none;
  background: rgba(255, 255, 255, 0.08);
  border-radius: 50%;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  transition: all 0.2s ease;
  backdrop-filter: blur(10px);
}

.control-btn:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.2);
  transform: scale(1.05);
}

.control-btn:active:not(:disabled) {
  transform: scale(0.95);
}

.control-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.control-btn svg {
  width: clamp(18px, 2.2vmin, 24px);
  height: clamp(18px, 2.2vmin, 24px);
}

.play-btn {
  width: clamp(56px, 7vmin, 72px);
  height: clamp(56px, 7vmin, 72px);
  background: white;
  color: #1a1a1a;
}

.play-btn:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.95);
  transform: scale(1.08);
}

.play-btn svg {
  width: clamp(24px, 3vmin, 32px);
  height: clamp(24px, 3vmin, 32px);
}

.mode-btn,
.automix-btn {
  width: clamp(32px, 4vmin, 40px);
  height: clamp(32px, 4vmin, 40px);
}

.mode-btn svg,
.automix-btn svg {
  width: clamp(16px, 2vmin, 20px);
  height: clamp(16px, 2vmin, 20px);
}

.automix-btn.active {
  color: #667eea;
  background: rgba(102, 126, 234, 0.15);
}

/* 音量控制 */
.player-volume {
  display: flex;
  align-items: center;
  gap: clamp(6px, 0.8vw, 10px);
  width: 100%;
}

.volume-btn {
  width: clamp(32px, 4vmin, 40px);
  height: clamp(32px, 4vmin, 40px);
  border: none;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 50%;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  transition: all 0.2s ease;
  backdrop-filter: blur(10px);
  flex-shrink: 0;
}

.volume-btn:hover {
  background: rgba(255, 255, 255, 0.2);
}

.volume-btn svg {
  width: clamp(16px, 2vmin, 20px);
  height: clamp(16px, 2vmin, 20px);
}

.volume-slider {
  flex: 1;
  height: clamp(16px, 2vh, 20px);
  display: flex;
  align-items: center;
  cursor: pointer;
}

.volume-track {
  width: 100%;
  height: clamp(3px, 0.5vh, 4px);
  background: rgba(255, 255, 255, 0.2);
  border-radius: clamp(1px, 0.3vmin, 2px);
  position: relative;
  overflow: visible;
}

.volume-fill {
  height: 100%;
  background: white;
  border-radius: 2px;
  transition: width 0.1s;
}

.volume-handle {
  position: absolute;
  top: 50%;
  width: clamp(8px, 1vh, 10px);
  height: clamp(8px, 1vh, 10px);
  background: white;
  border-radius: 50%;
  transform: translate(-50%, -50%);
  opacity: 0;
  transition: opacity 0.2s;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
}

.volume-slider:hover .volume-handle {
  opacity: 1;
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

/* AMLL 歌词样式覆盖 */
.player-right :deep(.amll-lyric-player) {
  width: 100%;
  height: 100%;
}

.player-right :deep([class*="_lyricLine_"]) {
  text-align: left !important;
}

.player-right :deep(.amll-lyric-player > div) {
  width: 100% !important;
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

  .cover-wrapper {
    width: 220px;
    height: 220px;
  }

  .track-title {
    font-size: 18px;
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

  .cover-wrapper {
    width: 180px;
    height: 180px;
  }

  .track-title {
    font-size: 16px;
  }

  .track-artist {
    font-size: 13px;
  }

  .controls-section {
    gap: 1.5vh;
  }

  .player-controls {
    gap: clamp(10px, 3vw, 16px);
  }

  .control-btn {
    width: clamp(40px, 12vw, 52px);
    height: clamp(40px, 12vw, 52px);
  }

  .control-btn svg {
    width: clamp(18px, 5vw, 24px);
    height: clamp(18px, 5vw, 24px);
  }

  .play-btn {
    width: clamp(56px, 16vw, 72px);
    height: clamp(56px, 16vw, 72px);
  }

  .play-btn svg {
    width: clamp(24px, 7vw, 32px);
    height: clamp(24px, 7vw, 32px);
  }

  .mode-btn,
  .automix-btn {
    display: none;
  }

  .player-volume {
    display: none;
  }
}
</style>
