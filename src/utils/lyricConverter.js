/**
 * 歌词格式转换工具
 * 支持 LRC、YRC、QRC、TTML 格式及翻译歌词
 * 整合优化版本
 * 
 * # 性能优化
 * - 预编译正则表达式，避免重复编译
 * - 使用 Map 存储时间戳映射，提高查找效率
 */

// 预编译正则表达式，避免每次调用时重新编译
const LRC_TIME_REGEX = /\[(\d{1,2}):(\d{2})\.(\d{2,3})\]/g;
const LRC_METADATA_REGEX = /^\[[a-zA-Z]+:[^\]]+\]$/;
const LINE_SPLIT_REGEX = /\r?\n/;

/**
 * 解析LRC格式歌词
 * @param {string} lrcContent - LRC格式歌词内容
 * @returns {Array} 解析后的歌词数组 [{time, text, translation, words}]
 */
export function parseLRC(lrcContent) {
  if (!lrcContent || typeof lrcContent !== 'string') {
    return [];
  }

  const lines = lrcContent.split(LINE_SPLIT_REGEX);
  const lyrics = [];
  const timeToLyricMap = new Map();

  for (const line of lines) {
    const trimmed = line.trim();
    if (!trimmed) continue;

    // 跳过元数据标签 [ti:], [ar:], [al:], [offset:]等（键名必须以字母开头）
    if (LRC_METADATA_REGEX.test(trimmed)) {
      continue;
    }

    // 使用新的正则实例，避免 lastIndex 污染
    const timeRegex = new RegExp(LRC_TIME_REGEX.source, 'g');

    // 查找所有时间标签
    const timeMatches = [...trimmed.matchAll(timeRegex)];

    if (timeMatches.length > 0) {
      // 获取最后一个时间标签后的文本
      const lastMatch = timeMatches[timeMatches.length - 1];
      const textStartIndex = lastMatch.index + lastMatch[0].length;
      const fullText = trimmed.substring(textStartIndex).trim();

      if (!fullText) continue;

      // 尝试分离原文和翻译
      let text = fullText;
      let translation = '';

      // 检测常见的分隔符模式
      const separators = [' | ', ' / ', ' // ', '  ', '\t'];
      for (const sep of separators) {
        const sepIndex = fullText.indexOf(sep);
        if (sepIndex > 0) {
          const afterSep = fullText.substring(sepIndex + sep.length).trim();
          if (afterSep && /[^\x00-\x7F]/.test(afterSep)) {
            text = fullText.substring(0, sepIndex).trim();
            translation = afterSep;
            break;
          }
        }
      }

      // 为每个时间标签创建歌词条目
      for (const match of timeMatches) {
        const minutes = parseInt(match[1], 10);
        const seconds = parseInt(match[2], 10);
        const msPart = match[3];

        // 处理毫秒：2位乘以10，3位直接使用
        const milliseconds = msPart.length === 2
          ? parseInt(msPart, 10) * 10
          : parseInt(msPart, 10);

        // 转换为毫秒
        const time = (minutes * 60 + seconds) * 1000 + milliseconds;
        const timeKey = time.toString();

        if (text) {
          if (timeToLyricMap.has(timeKey)) {
            // 已存在相同时间的歌词，处理翻译
            const existing = timeToLyricMap.get(timeKey);
            if (/[^\x00-\x7F]/.test(text) && /^[\x00-\x7F]*$/.test(existing.text)) {
              existing.translation = text;
            } else if (/[^\x00-\x7F]/.test(existing.text) && /^[\x00-\x7F]*$/.test(text)) {
              existing.translation = existing.text;
              existing.text = text;
            }
          } else {
            lyrics.push({
              time: time,
              text: text,
              translation: translation,
              words: []
            });
            timeToLyricMap.set(timeKey, lyrics[lyrics.length - 1]);
          }
        }
      }
    }
  }

  // 按时间排序并去重
  return lyrics.sort((a, b) => a.time - b.time).filter((item, index, arr) => {
    return index === 0 || item.time !== arr[index - 1].time;
  });
}

/**
 * 解析逐字格式歌词（YRC/QRC 通用）
 * @param {string} content - 歌词内容
 * @param {Object} options - 解析选项
 * @param {RegExp} options.lineRegex - 行匹配正则
 * @param {RegExp} options.wordRegex - 逐字匹配正则
 * @param {Function} options.parseWordMatch - 解析单词匹配的函数
 * @returns {Array} 解析后的歌词数组
 */
function parseWordLevelLyrics(content, options) {
  if (!content || typeof content !== 'string') {
    return [];
  }

  const { lineRegex, wordRegex, parseWordMatch } = options;
  const lines = content.split(/\r?\n/);
  const lyrics = [];

  for (const line of lines) {
    const trimmed = line.trim();
    if (!trimmed) continue;

    const lineMatch = trimmed.match(lineRegex);
    if (!lineMatch) continue;

    const startTime = parseInt(lineMatch[1], 10);
    const duration = parseInt(lineMatch[2], 10);
    const lineContent = lineMatch[3];

    if (isNaN(startTime) || isNaN(duration)) continue;

    const words = [];
    let fullText = '';
    let wordMatch;

    wordRegex.lastIndex = 0;

    while ((wordMatch = wordRegex.exec(lineContent)) !== null) {
      const parsed = parseWordMatch(wordMatch);
      if (parsed && parsed.duration > 0) {
        fullText += parsed.text;
        words.push(parsed);
      }
    }

    if (fullText || words.length === 0) {
      lyrics.push({
        time: startTime,
        duration: duration,
        endTime: startTime + duration,
        text: fullText || lineContent,
        words: words,
        translation: ''
      });
    }
  }

  return lyrics.sort((a, b) => a.time - b.time);
}

/**
 * 解析YRC格式歌词（网易云音乐逐字歌词）
 * @param {string} yrcContent - YRC格式歌词内容
 * @returns {Array} 解析后的歌词数组
 */
export function parseYRC(yrcContent) {
  return parseWordLevelLyrics(yrcContent, {
    // YRC格式: [开始时间,持续时间]逐字内容
    lineRegex: /\[(\d+),(\d+)\](.*)/,
    // 逐字格式: (开始时间,持续时间,音高)文字
    wordRegex: /\((\d+),(\d+),(\d+)\)([^\(]*)/g,
    parseWordMatch: (match) => ({
      time: parseInt(match[1], 10),
      duration: parseInt(match[2], 10),
      text: match[4] || ''
    })
  });
}

/**
 * 解析QRC格式歌词（QQ音乐）
 * @param {string} qrcContent - QRC格式歌词内容
 * @returns {Array} 解析后的歌词数组
 */
export function parseQRC(qrcContent) {
  return parseWordLevelLyrics(qrcContent, {
    // QRC格式: [时间,持续时间]歌词内容
    lineRegex: /\[(\d+),(\d+)\](.*)/,
    // 逐字格式: (开始时间,持续时间,0)文字
    wordRegex: /\((\d+),(\d+),\d+\)([^()]+)/g,
    parseWordMatch: (match) => ({
      time: parseInt(match[1], 10),
      duration: parseInt(match[2], 10),
      text: match[3] || ''
    })
  });
}

/**
 * 解析TTML格式歌词
 * @param {string} ttmlContent - TTML格式歌词内容
 * @returns {Array} 解析后的歌词数组
 */
export function parseTTML(ttmlContent) {
  if (!ttmlContent || typeof ttmlContent !== 'string') {
    return [];
  }

  const lyrics = [];

  // 解析body内容
  const bodyMatch = ttmlContent.match(/<body[^>]*>([\s\S]*?)<\/body>/i);
  if (!bodyMatch) return [];

  const body = bodyMatch[1];

  // 解析p标签（歌词行）
  const pRegex = /<p[^>]*begin="([^"]*)"[^>]*end="([^"]*)"[^>]*>([\s\S]*?)<\/p>/gi;
  let match;

  while ((match = pRegex.exec(body)) !== null) {
    const begin = parseTimeString(match[1]);
    const end = parseTimeString(match[2]);
    const content = match[3];

    // 解析span标签（逐字）
    const words = [];
    const spanRegex = /<span[^>]*begin="([^"]*)"[^>]*end="([^"]*)"[^>]*>([^<]*)<\/span>/gi;
    let spanMatch;

    while ((spanMatch = spanRegex.exec(content)) !== null) {
      const wordBegin = parseTimeString(spanMatch[1]);
      const wordEnd = parseTimeString(spanMatch[2]);
      words.push({
        time: wordBegin,
        duration: wordEnd - wordBegin,
        text: spanMatch[3]
      });
    }

    // 提取纯文本
    const text = content.replace(/<[^>]+>/g, '').trim();

    lyrics.push({
      time: begin,
      duration: end - begin,
      endTime: end,
      text: text,
      words: words,
      translation: ''
    });
  }

  return lyrics;
}

/**
 * 解析时间字符串为毫秒
 * @param {string} timeStr - 时间字符串
 * @returns {number} 毫秒数
 */
function parseTimeString(timeStr) {
  if (!timeStr) return 0;

  // 毫秒格式: 1500ms
  if (timeStr.endsWith('ms')) {
    return parseInt(timeStr.slice(0, -2), 10);
  }

  // 秒格式: 1.5s
  if (timeStr.endsWith('s')) {
    return Math.round(parseFloat(timeStr.slice(0, -1)) * 1000);
  }

  // 标准格式: mm:ss.xx 或 mm:ss.xxx
  const parts = timeStr.split(':');
  if (parts.length === 2) {
    const minutes = parseInt(parts[0], 10) || 0;
    const seconds = parseFloat(parts[1]) || 0;
    return Math.round((minutes * 60 + seconds) * 1000);
  }

  // 纯毫秒数
  const num = parseInt(timeStr, 10);
  return isNaN(num) ? 0 : num;
}

/**
 * 合并翻译歌词
 * @param {Array} mainLyrics - 主歌词数组
 * @param {Array} tranLyrics - 翻译歌词数组
 * @param {string} type - 翻译类型
 */
function mergeTranslation(mainLyrics, tranLyrics, type = 'translation') {
  if (!mainLyrics || !tranLyrics || mainLyrics.length === 0 || tranLyrics.length === 0) {
    return;
  }

  tranLyrics.forEach(tranLine => {
    const mainIndex = mainLyrics.findIndex(mainLine =>
      Math.abs(mainLine.time - tranLine.time) < 100 // 时间差小于100毫秒
    );

    if (mainIndex !== -1) {
      mainLyrics[mainIndex][type] = tranLine.text;
    }
  });
}

/**
 * 自动检测并解析歌词格式
 * @param {string} content - 歌词内容
 * @param {string} format - 可选的格式提示 ('lrc', 'yrc', 'qrc', 'ttml')
 * @returns {Array} 解析后的歌词数组
 */
export function parseLyrics(content, format = null) {
  if (!content || typeof content !== 'string') {
    return [];
  }

  // 如果指定了格式
  if (format) {
    switch (format.toLowerCase()) {
      case 'lrc':
        return parseLRC(content);
      case 'yrc':
        return parseYRC(content);
      case 'qrc':
        return parseQRC(content);
      case 'ttml':
        return parseTTML(content);
      default:
        break;
    }
  }

  // 自动检测格式
  const trimmed = content.trim();

  // 检测TTML
  if (trimmed.includes('<?xml') || trimmed.includes('<tt')) {
    return parseTTML(content);
  }

  // 检测YRC（包含逐字标记）
  if (/\[\d+,\d+\]\(\d+,\d+,\d+\)/.test(trimmed)) {
    return parseYRC(content);
  }

  // 检测QRC
  if (/\[\d+,\d+\]/.test(trimmed) && !trimmed.includes('[') && trimmed.includes('(')) {
    return parseQRC(content);
  }

  // 默认按LRC解析
  return parseLRC(content);
}

/**
 * 将歌词数组转换为TTML格式
 * @param {Array} lyrics - 歌词数组
 * @returns {string} TTML格式字符串
 */
export function convertToTTML(lyrics) {
  if (!lyrics || lyrics.length === 0) {
    return '<?xml version="1.0" encoding="UTF-8"?><tt xmlns="http://www.w3.org/ns/ttml"><body></body></tt>';
  }

  const lines = lyrics.map((line) => {
    const begin = formatTimeForTTML(line.time);
    const end = formatTimeForTTML(line.endTime || (line.time + (line.duration || 5000)));

    // 如果有逐字信息
    if (line.words && line.words.length > 0) {
      const wordSpans = line.words.map(word => {
        const wordBegin = formatTimeForTTML(word.time);
        const wordEnd = formatTimeForTTML(word.time + (word.duration || 200));
        return `<span begin="${wordBegin}" end="${wordEnd}">${escapeXml(word.text)}</span>`;
      }).join('');

      return `    <p begin="${begin}" end="${end}">${wordSpans}</p>`;
    }

    // 只有整行歌词
    return `    <p begin="${begin}" end="${end}">${escapeXml(line.text)}</p>`;
  }).join('\n');

  return `<?xml version="1.0" encoding="UTF-8"?>
<tt xmlns="http://www.w3.org/ns/ttml">
  <body>
    <div>
${lines}
    </div>
  </body>
</tt>`;
}

/**
 * 格式化时间为TTML格式
 * @param {number} ms - 毫秒数
 * @returns {string} 格式化后的时间字符串
 */
function formatTimeForTTML(ms) {
  const totalSeconds = Math.floor(ms / 1000);
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  const milliseconds = Math.floor(ms % 1000);

  return `${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}.${milliseconds.toString().padStart(3, '0')}`;
}

/**
 * 转义XML特殊字符
 * @param {string} text - 原始文本
 * @returns {string} 转义后的文本
 */
function escapeXml(text) {
  if (!text) return '';
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&apos;');
}

/**
 * 检查歌词是否有逐字信息
 * @param {Array} lyrics - 歌词数组
 * @returns {boolean} 是否有逐字信息
 */
export function hasWordLevelLyrics(lyrics) {
  if (!lyrics || lyrics.length === 0) return false;
  return lyrics.some(line => line.words && line.words.length > 0);
}

/**
 * 获取歌词的时间范围
 * @param {Array} lyrics - 歌词数组
 * @returns {Object} { startTime, endTime }
 */
export function getLyricsTimeRange(lyrics) {
  if (!lyrics || lyrics.length === 0) {
    return { startTime: 0, endTime: 0 };
  }

  const times = lyrics.map(line => line.time).filter(t => t !== undefined);
  const endTimes = lyrics.map(line => line.endTime || (line.time + (line.duration || 0))).filter(t => t !== undefined);

  return {
    startTime: Math.min(...times),
    endTime: Math.max(...endTimes)
  };
}

/**
 * 解析完整的歌词数据对象（包含主歌词和翻译）
 * @param {Object} lyricsData - 歌词数据对象
 * @returns {Array} 解析后的歌词数组
 */
export function parseLyricsData(lyricsData) {
  if (!lyricsData) return [];

  let mainLyrics = [];
  let tranLyrics = [];

  // 解析主歌词
  if (lyricsData.lrc || lyricsData.lrcx) {
    mainLyrics = parseLRC(lyricsData.lrc || lyricsData.lrcx);
  } else if (lyricsData.yrc) {
    mainLyrics = parseYRC(lyricsData.yrc);
  }

  // 解析翻译歌词
  if (lyricsData.tlrc || lyricsData.tran) {
    tranLyrics = parseLRC(lyricsData.tlrc || lyricsData.tran);
    mergeTranslation(mainLyrics, tranLyrics, 'translation');
  }

  // 解析YRC翻译
  if (lyricsData.ytlrc) {
    tranLyrics = parseLRC(lyricsData.ytlrc);
    mergeTranslation(mainLyrics, tranLyrics, 'yrcTranslation');
  }

  return mainLyrics;
}

export default {
  parseLRC,
  parseYRC,
  parseQRC,
  parseTTML,
  parseLyrics,
  parseLyricsData,
  convertToTTML,
  hasWordLevelLyrics,
  getLyricsTimeRange
};
