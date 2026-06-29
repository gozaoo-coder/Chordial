/**
 * useLyricParser — 将歌词字符串转为 AMLL LyricLine[] 格式
 *
 * 使用项目内置的 lyricConverter（纯 JS，无需 WASM 插件）
 * 自动检测 LRC/YRC/QRC/TTML 格式并转换为 LyricPlayer 所需的数据结构。
 */
import { computed } from 'vue'
import { parseLyrics } from '@/utils/lyricConverter.js'

/**
 * 将 lyricConverter 输出的歌词行转为 AMLL LyricLine 格式
 *
 * @param {object} line - 转换器输出的单行 { time, endTime, duration, text, words, translation }
 * @param {number} nextStartTime - 下一行的 startTime（用于填充 endTime）
 * @returns {import('@applemusic-like-lyrics/core').LyricLine}
 */
function toLyricLine(line, nextStartTime) {
  const startTime = line.time ?? 0
  const endTime = line.endTime
    ?? (nextStartTime > startTime ? nextStartTime : startTime + (line.duration || 5000))

  const words = (line.words && line.words.length > 0)
    ? line.words.map(w => ({
        startTime: w.time ?? startTime,
        endTime: w.time + (w.duration || 200),
        word: w.text || '',
        romanWord: '',
      }))
    : [{
        word: line.text || '',
        startTime,
        endTime,
        romanWord: '',
      }]

  return {
    words,
    startTime,
    endTime,
    translatedLyric: line.translation || '',
    romanLyric: '',
    isBG: false,
    isDuet: false,
  }
}

/**
 * 将歌词字符串转换为 AMLL LyricLine[]
 *
 * @param {import('vue').Ref<string>|string} lyricString - 歌词字符串（支持 LRC/YRC/QRC/TTML）
 * @returns {import('vue').ComputedRef<import('@applemusic-like-lyrics/core').LyricLine[]>}
 */
export function useLyricParser(lyricString) {
  return computed(() => {
    const src = typeof lyricString === 'string' ? lyricString : lyricString?.value
    if (!src || typeof src !== 'string') return []

    try {
      // 自动检测格式并解析
      const parsed = parseLyrics(src)
      if (!parsed || parsed.length === 0) return []

      // 按时间排序
      parsed.sort((a, b) => (a.time ?? 0) - (b.time ?? 0))

      // 转换为 LyricLine[]，endTime 指向下一行的 startTime
      return parsed.map((line, i) => {
        const nextLine = parsed[i + 1]
        const nextStartTime = nextLine ? (nextLine.time ?? 0) : undefined
        return toLyricLine(line, nextStartTime)
      })
    } catch (error) {
      console.warn('[useLyricParser] 解析歌词失败:', error)
      return []
    }
  })
}

export default useLyricParser
