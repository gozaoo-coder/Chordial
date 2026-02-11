<template>
  <div class="amll-lyrics-wrapper">
    <!-- AMLL 歌词容器 -->
    <div ref="lyricsContainerRef" class="amll-lyrics-container"></div>

    <!-- 无歌词状态 -->
    <div v-if="!hasLyrics" class="no-lyrics">
      <div class="no-lyrics-icon">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M9 18V5l12-2v13"/>
          <circle cx="6" cy="18" r="3"/>
          <circle cx="18" cy="16" r="3"/>
        </svg>
      </div>
      <p class="no-lyrics-text">{{ noLyricsMessage }}</p>
    </div>
  </div>
</template>

<script>
import { ref, computed, watch, onMounted, onUnmounted, nextTick, shallowRef } from 'vue';
import { LyricPlayer } from '@applemusic-like-lyrics/core';
import { parseLyrics } from '@/utils/lyricConverter.js';

export default {
  name: 'AMLLyrics',
  props: {
    // 歌词数据
    lyricsData: {
      type: Object,
      default: () => ({
        plainLyrics: '',
        syncedLyrics: '',
        hasSyncedLyrics: false,
        hasPlainLyrics: false
      })
    },
    // 当前播放时间（秒）
    currentTime: {
      type: Number,
      default: 0
    },
    // 是否正在播放
    isPlaying: {
      type: Boolean,
      default: false
    },
    // 是否启用弹簧动画
    enableSpring: {
      type: Boolean,
      default: true
    },
    // 是否启用模糊效果
    enableBlur: {
      type: Boolean,
      default: true
    },
    // 是否启用缩放效果
    enableScale: {
      type: Boolean,
      default: true
    },
    // 歌词对齐位置 (0-1)
    alignPosition: {
      type: Number,
      default: 0.5
    },
    // 歌词对齐锚点
    alignAnchor: {
      type: String,
      default: 'center'
    },
    // 是否隐藏已播放的歌词行
    hidePassedLines: {
      type: Boolean,
      default: false
    },
    // 文字渐隐宽度
    wordFadeWidth: {
      type: Number,
      default: 0.5
    }
  },
  emits: ['seek', 'line-click'],
  setup(props, { emit }) {
    const lyricsContainerRef = ref(null);
    const lyricPlayer = shallowRef(null);
    let animationFrameId = null;

    // 缓存解析后的歌词数据，避免重复解析
    const cachedLyricsKey = ref('');
    const cachedLyricsData = ref(null);

    // 将秒转换为毫秒
    const currentTimeMs = computed(() => {
      return Math.round(props.currentTime * 1000);
    });

    // 是否有歌词
    const hasLyrics = computed(() => {
      return props.lyricsData?.hasSyncedLyrics || props.lyricsData?.hasPlainLyrics;
    });

    // 无歌词提示信息
    const noLyricsMessage = computed(() => {
      if (props.lyricsData?.hasPlainLyrics && !props.lyricsData?.hasSyncedLyrics) {
        return '暂无同步歌词';
      }
      return '暂无歌词';
    });

    // 检测歌词格式
    const detectLyricFormat = (content) => {
      if (!content) return 'lrc';
      const trimmed = content.trim();

      // 检测 TTML/XML
      if (trimmed.includes('<?xml') || trimmed.includes('<tt')) {
        return 'ttml';
      }

      // 检测 JSON 格式 (YRC)
      if ((trimmed.startsWith('[') && trimmed.includes('"time"')) ||
          (trimmed.startsWith('{') && trimmed.includes('"lyrics"'))) {
        try {
          JSON.parse(trimmed);
          return 'json';
        } catch (e) {
          // 不是有效的 JSON，继续检测
        }
      }

      // 检测 QRC 格式: [1234,567]歌词
      if (/^\[\d+,\d+\]/.test(trimmed)) {
        return 'qrc';
      }

      // 检测 YRC 格式: [1234,567]逐词歌词(1234,567,0)内容
      if (/\[\d+,\d+\].*\(\d+,\d+,\d+\)/.test(trimmed)) {
        return 'yrc';
      }

      // 默认 LRC 格式: [mm:ss.xx]歌词
      return 'lrc';
    };

    // 默认歌词行持续时间（毫秒）
    const DEFAULT_LINE_DURATION = 5000;
    const DEFAULT_WORD_DURATION = 200;

    /**
     * 获取歌词内容
     * @returns {string|null}
     */
    const getLyricsContent = () => {
      if (props.lyricsData.syncedLyrics) {
        return props.lyricsData.syncedLyrics;
      }

      if (!props.lyricsData.plainLyrics) {
        return null;
      }

      // 检查是否包含 LRC 时间戳
      const lrcTimeRegex = /\[\d{1,2}:\d{2}\.\d{2,3}\]/;
      if (lrcTimeRegex.test(props.lyricsData.plainLyrics)) {
        return props.lyricsData.plainLyrics;
      }

      // 纯文本转换为 LRC 格式
      return convertPlainTextToLrc(props.lyricsData.plainLyrics);
    };

    /**
     * 将纯文本转换为 LRC 格式
     * @param {string} plainText - 纯文本歌词
     * @returns {string}
     */
    const convertPlainTextToLrc = (plainText) => {
      const lines = plainText.split('\n').filter(line => line.trim());
      return lines.map((line, index) => {
        const timeMs = index * DEFAULT_LINE_DURATION;
        const timeTag = formatTimeToLrcTag(timeMs);
        return `${timeTag}${line.trim()}`;
      }).join('\n');
    };

    /**
     * 将毫秒格式化为 LRC 时间标签
     * @param {number} timeMs - 时间（毫秒）
     * @returns {string}
     */
    const formatTimeToLrcTag = (timeMs) => {
      const mins = Math.floor(timeMs / 60000).toString().padStart(2, '0');
      const secs = Math.floor((timeMs % 60000) / 1000).toString().padStart(2, '0');
      return `[${mins}:${secs}.00]`;
    };

    /**
     * 计算歌词行结束时间
     * @param {Object} line - 当前行
     * @param {number} index - 当前索引
     * @param {Array} array - 歌词数组
     * @returns {number}
     */
    const calculateEndTime = (line, index, array) => {
      const startTime = line.time || 0;
      if (line.duration) {
        return startTime + line.duration;
      }
      return array[index + 1]?.time || startTime + DEFAULT_LINE_DURATION;
    };

    /**
     * 转换逐字歌词
     * @param {Array} wordList - 逐字列表
     * @param {number} lineStartTime - 行开始时间
     * @returns {Array}
     */
    const convertWordList = (wordList, lineStartTime) => {
      return wordList.map(word => ({
        startTime: word.time || lineStartTime,
        endTime: (word.time || lineStartTime) + (word.duration || DEFAULT_WORD_DURATION),
        word: word.text || '',
        romanWord: '',
        obscene: false
      }));
    };

    /**
     * 创建单行歌词的 word 列表
     * @param {number} startTime - 开始时间
     * @param {number} endTime - 结束时间
     * @param {string} text - 歌词文本
     * @returns {Array}
     */
    const createWordList = (startTime, endTime, text) => [{
      startTime,
      endTime,
      word: text || '',
      romanWord: '',
      obscene: false
    }];

    /**
     * 将解析后的歌词转换为 AMLL 格式
     * @param {Array} parsedLyrics - 解析后的歌词
     * @returns {Array}
     */
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

    // 转换歌词数据为 AMLL 格式（带缓存）
    // 使用歌曲 ID 作为缓存键，避免哈希冲突
    const lyricLinesData = computed(() => {
      if (!hasLyrics.value) return null;

      try {
        const lyricsContent = getLyricsContent();
        if (!lyricsContent) return null;

        // 使用歌词内容前 50 个字符的哈希 + 歌词长度作为缓存键
        const lyricsPrefix = lyricsContent.substring(0, 50);
        const hash = lyricsPrefix.split('').reduce((acc, char) => acc + char.charCodeAt(0), 0);
        const cacheKey = `${hash}_${lyricsContent.length}`;
        
        if (cacheKey === cachedLyricsKey.value && cachedLyricsData.value) {
          return cachedLyricsData.value;
        }

        // 解析并转换
        const format = detectLyricFormat(lyricsContent);
        const parsedLyrics = parseLyrics(lyricsContent, format);

        if (!parsedLyrics?.length) return null;

        const result = convertToAmllFormat(parsedLyrics);

        // 更新缓存
        cachedLyricsKey.value = cacheKey;
        cachedLyricsData.value = result;

        return result;
      } catch (error) {
        console.error('AMLLyrics: 解析歌词失败:', error);
        return null;
      }
    });

    // 防抖定时器
    let initDebounceTimer = null;
    let isInitializing = false;

    // 初始化 LyricPlayer
    const initLyricPlayer = async () => {
      // 防止重复初始化
      if (isInitializing) return;
      isInitializing = true;

      try {
        if (!lyricsContainerRef.value || !lyricLinesData.value) return;

        // 清理旧的实例
        if (lyricPlayer.value) {
          lyricPlayer.value.dispose();
          lyricPlayer.value = null;
        }

        // 等待容器有正确的尺寸
        await nextTick();
        const container = lyricsContainerRef.value;

        // 检查容器尺寸，最多重试 10 次（1秒）- 减少重试次数
        let retryCount = 0;
        const maxRetries = 10;

        while ((container.clientHeight === 0 || container.clientWidth === 0) && retryCount < maxRetries) {
          await new Promise(resolve => setTimeout(resolve, 100));
          retryCount++;
        }

        if (container.clientHeight === 0 || container.clientWidth === 0) {
          return;
        }

        // 创建新的 LyricPlayer 实例
        const player = new LyricPlayer();
        lyricPlayer.value = player;

        // 将 LyricPlayer 的元素添加到容器中
        container.appendChild(player.getElement());

        // 设置歌词
        player.setLyricLines(lyricLinesData.value);

        // 设置配置
        player.setEnableSpring(props.enableSpring);
        player.setEnableBlur(props.enableBlur);
        player.setEnableScale(props.enableScale);
        player.setAlignPosition(props.alignPosition);
        player.setAlignAnchor(props.alignAnchor);
        player.setHidePassedLines(props.hidePassedLines);
        player.setWordFadeWidth(props.wordFadeWidth);

        // 添加点击事件监听
        player.addEventListener('line-click', (line) => {
          if (line && line.startTime !== undefined) {
            const timeInSeconds = line.startTime / 1000;
            emit('seek', timeInSeconds);
            emit('line-click', { line, time: timeInSeconds });
          }
        });

        // 计算布局并设置初始时间 - 使用 requestAnimationFrame 避免强制同步布局
        await new Promise(resolve => requestAnimationFrame(resolve));
        await player.calcLayout();
        player.setCurrentTime(currentTimeMs.value, true);

        // 开始动画循环
        startAnimationLoop();
      } finally {
        isInitializing = false;
      }
    };

    // 防抖版本的初始化
    const debouncedInitLyricPlayer = () => {
      if (initDebounceTimer) {
        clearTimeout(initDebounceTimer);
      }
      initDebounceTimer = setTimeout(() => {
        initLyricPlayer();
      }, 100);
    };

    // 动画循环
    let lastTime = -1;
    let isRunning = false;
    const startAnimationLoop = () => {
      if (isRunning) return;
      isRunning = true;

      const animate = (timestamp) => {
        if (!lyricPlayer.value) {
          isRunning = false;
          return;
        }

        if (lastTime === -1) {
          lastTime = timestamp;
        }
        const deltaTime = timestamp - lastTime;
        lastTime = timestamp;

        lyricPlayer.value.update(deltaTime);
        animationFrameId = requestAnimationFrame(animate);
      };
      animationFrameId = requestAnimationFrame(animate);
    };

    // 监听歌词数据变化
    watch(() => lyricLinesData.value, async (newLines, oldLines) => {
      // 只在数据真正变化时处理
      if (newLines === oldLines) return;

      if (newLines && newLines.length > 0) {
        if (!lyricPlayer.value) {
          await initLyricPlayer();
        } else {
          lyricPlayer.value.setLyricLines(newLines);
          // 重新计算布局 - 不使用强制同步布局
          await lyricPlayer.value.calcLayout();
          lyricPlayer.value.setCurrentTime(currentTimeMs.value, true);
        }
      }
    }, { immediate: true });

    // 监听播放状态变化
    watch(() => props.isPlaying, (isPlaying) => {
      if (!lyricPlayer.value) return;
      if (isPlaying) {
        lyricPlayer.value.resume();
      } else {
        lyricPlayer.value.pause();
      }
    });

    // 监听当前时间变化 - 使用防抖避免频繁更新
    let timeUpdateTimer = null;
    watch(() => currentTimeMs.value, (timeMs) => {
      if (!lyricPlayer.value) return;
      // 取消之前的定时器
      if (timeUpdateTimer) {
        clearTimeout(timeUpdateTimer);
      }
      // 延迟更新，避免频繁调用
      timeUpdateTimer = setTimeout(() => {
        if (lyricPlayer.value) {
          lyricPlayer.value.setCurrentTime(timeMs);
        }
      }, 16); // 约 60fps
    });

    // 监听其他配置变化 - 批量处理
    let configUpdateTimer = null;
    const updateConfig = () => {
      if (configUpdateTimer) {
        clearTimeout(configUpdateTimer);
      }
      configUpdateTimer = setTimeout(() => {
        if (!lyricPlayer.value) return;
        lyricPlayer.value.setEnableSpring(props.enableSpring);
        lyricPlayer.value.setEnableBlur(props.enableBlur);
        lyricPlayer.value.setEnableScale(props.enableScale);
        lyricPlayer.value.setAlignPosition(props.alignPosition);
        lyricPlayer.value.setWordFadeWidth(props.wordFadeWidth);
      }, 50);
    };

    watch(() => props.enableSpring, updateConfig);
    watch(() => props.enableBlur, updateConfig);
    watch(() => props.enableScale, updateConfig);
    watch(() => props.alignPosition, updateConfig);
    watch(() => props.wordFadeWidth, updateConfig);

    // ResizeObserver 用于监听容器尺寸变化 - 带防抖
    let resizeObserver = null;
    let resizeDebounceTimer = null;
    let hasInitialized = false;

    onMounted(() => {
      if (lyricLinesData.value && lyricLinesData.value.length > 0) {
        initLyricPlayer();
      }

      // 使用 ResizeObserver 监听容器尺寸变化 - 带防抖处理
      if (lyricsContainerRef.value && window.ResizeObserver) {
        resizeObserver = new ResizeObserver((entries) => {
          // 清除之前的定时器
          if (resizeDebounceTimer) {
            clearTimeout(resizeDebounceTimer);
          }

          resizeDebounceTimer = setTimeout(() => {
            for (const entry of entries) {
              const { width, height } = entry.contentRect;
              // 只在首次获得有效尺寸且未初始化时触发
              if (width > 0 && height > 0 && !hasInitialized && !lyricPlayer.value && lyricLinesData.value) {
                hasInitialized = true;
                initLyricPlayer();
              }
            }
          }, 200); // 200ms 防抖
        });
        resizeObserver.observe(lyricsContainerRef.value);
      }
    });

    onUnmounted(() => {
      // 清理定时器
      if (initDebounceTimer) {
        clearTimeout(initDebounceTimer);
      }
      if (resizeDebounceTimer) {
        clearTimeout(resizeDebounceTimer);
      }
      if (timeUpdateTimer) {
        clearTimeout(timeUpdateTimer);
      }
      if (configUpdateTimer) {
        clearTimeout(configUpdateTimer);
      }

      // 清理 ResizeObserver
      if (resizeObserver) {
        resizeObserver.disconnect();
        resizeObserver = null;
      }

      // 清理动画循环
      if (animationFrameId) {
        cancelAnimationFrame(animationFrameId);
        isRunning = false;
      }

      // 清理 LyricPlayer
      if (lyricPlayer.value) {
        lyricPlayer.value.dispose();
        lyricPlayer.value = null;
      }
    });

    return {
      lyricsContainerRef,
      hasLyrics,
      noLyricsMessage
    };
  }
};
</script>

<style scoped>
.amll-lyrics-wrapper {
  width: 100%;
  height: 100%;
  position: relative;
}

.amll-lyrics-container {
  width: 100%;
  height: 100%;
  position: relative;
  overflow: hidden;
  /* AMLL 需要的 CSS 变量 - 增大字体 */
  --amll-lp-color: white;
  --amll-lp-font-size: max(max(6vh, 3.5vw), 28px);
  --amll-lp-line-width-aspect: 0.85;
  --amll-lp-line-padding-x: 1.2em;
}

/* 无歌词状态 */
.no-lyrics {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: rgba(255, 255, 255, 0.5);
  text-align: center;
  padding: 40px;
}

.no-lyrics-icon {
  width: 80px;
  height: 80px;
  margin-bottom: 16px;
  opacity: 0.5;
}

.no-lyrics-icon svg {
  width: 100%;
  height: 100%;
}

.no-lyrics-text {
  font-size: 16px;
  margin: 0;
}

/* AMLL 样式覆盖 */
:deep(.amll-lyric-player) {
  width: 100%;
  height: 100%;
}

/* 确保歌词文字左对齐 - dom 版本 */
:deep([class*="_lyricLine_"]) {
  text-align: left !important;
}

/* 确保 AMLL 内部容器宽度正确 */
:deep(.amll-lyric-player > div) {
  width: 100% !important;
}
</style>
