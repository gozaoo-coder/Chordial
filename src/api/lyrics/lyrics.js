/**
 * 高级歌词组件 API 接口
 * 提供歌词解析、格式检测、文件读取等功能
 */

import { invoke } from '@tauri-apps/api/core'

/**
 * 解析歌词内容
 * @param {string} content - 歌词内容
 * @param {string} [format] - 歌词格式 (lrc, yrc, qrc, ttml)
 * @returns {Promise<ParsedLyric>} 解析后的歌词数据
 */
export async function parseLyricContent(content, format = null) {
  try {
    return await invoke('parse_lyric_content', { content, format })
  } catch (error) {
    console.error('解析歌词失败:', error)
    throw error
  }
}

/**
 * 检测歌词格式
 * @param {string} content - 歌词内容
 * @returns {Promise<string>} 检测到的格式 (lrc, yrc, qrc, ttml, unknown)
 */
export async function detectLyricFormat(content) {
  try {
    return await invoke('detect_lyric_format', { content })
  } catch (error) {
    console.error('检测歌词格式失败:', error)
    throw error
  }
}

/**
 * 从文件读取歌词内容
 * @param {string} filePath - 歌词文件路径
 * @returns {Promise<string>} 歌词内容
 */
export async function readLyricFile(filePath) {
  try {
    const response = await fetch(`file://${filePath}`)
    return await response.text()
  } catch (error) {
    console.error('读取歌词文件失败:', error)
    throw error
  }
}

/**
 * 获取歌词文件路径（与音频文件同目录）
 * @param {string} audioFilePath - 音频文件路径
 * @returns {string[]} 可能的歌词文件路径
 */
export function getPotentialLyricPaths(audioFilePath) {
  const path = audioFilePath.replace(/\\/g, '/')
  const lastDotIndex = path.lastIndexOf('.')
  const basePath = lastDotIndex > -1 ? path.substring(0, lastDotIndex) : path
  
  const extensions = ['lrc', 'yrc', 'qrc', 'txt']
  return extensions.map(ext => `${basePath}.${ext}`)
}

/**
 * 格式化时间戳（毫秒转换为 mm:ss.ms）
 * @param {number} milliseconds - 毫秒数
 * @returns {string} 格式化后的时间字符串
 */
export function formatTimestamp(milliseconds) {
  const seconds = Math.floor(milliseconds / 1000)
  const minutes = Math.floor(seconds / 60)
  const remainingSeconds = seconds % 60
  const remainingMilliseconds = milliseconds % 1000
  
  return `${minutes.toString().padStart(2, '0')}:${remainingSeconds.toString().padStart(2, '0')}.${Math.floor(remainingMilliseconds / 10).toString().padStart(2, '0')}`
}

/**
 * 解析 LRC 时间戳
 * @param {string} timestamp - LRC 时间戳格式 [mm:ss.xx]
 * @returns {number} 毫秒数
 */
export function parseLrcTimestamp(timestamp) {
  const cleanTimestamp = timestamp.replace(/[\[\]]/g, '')
  const parts = cleanTimestamp.split(/[:.]/)
  
  if (parts.length >= 2) {
    const minutes = parseInt(parts[0], 10)
    const seconds = parseFloat(`${parts[1]}.${parts[2] || '00'}`)
    return Math.floor(minutes * 60 * 1000 + seconds * 1000)
  }
  
  return 0
}

/**
 * 查找当前播放位置的歌词行
 * @param {Array} lines - 歌词行数组
 * @param {number} currentTime - 当前播放时间（秒）
 * @returns {Object} 当前歌词行信息和索引
 */
export function findCurrentLyricLine(lines, currentTime) {
  const timeMs = currentTime * 1000
  
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]
    const nextLine = lines[i + 1]
    
    if (timeMs >= line.start_time && (!nextLine || timeMs < nextLine.start_time)) {
      return {
        line,
        index: i,
        progress: nextLine ? (timeMs - line.start_time) / (nextLine.start_time - line.start_time) : 0
      }
    }
  }
  
  return { line: null, index: -1, progress: 0 }
}

/**
 * 计算歌词行在单词级别的进度
 * @param {Object} line - 歌词行对象
 * @param {number} currentTimeMs - 当前时间（毫秒）
 * @returns {Array} 每个单词的播放状态
 */
export function getWordProgress(line, currentTimeMs) {
  if (!line.words || line.words.length === 0) {
    return []
  }
  
  return line.words.map(word => {
    if (currentTimeMs < word.start_time) {
      return { word, status: 'pending', progress: 0 }
    } else if (currentTimeMs > word.end_time) {
      return { word, status: 'played', progress: 1 }
    } else {
      const progress = (currentTimeMs - word.start_time) / (word.end_time - word.start_time)
      return { word, status: 'playing', progress }
    }
  })
}

/**
 * 歌词数据结构
 * @typedef {Object} ParsedLyric
 * @property {Array<LyricLine>} lines - 歌词行数组
 * @property {Object} [metadata] - 歌词元数据
 */

/**
 * 歌词行数据结构
 * @typedef {Object} LyricLine
 * @property {Array<LyricWord>} words - 单词数组
 * @property {string} [translated_lyric] - 翻译歌词
 * @property {string} [roman_lyric] - 音译歌词
 * @property {boolean} is_bg - 是否为背景歌词
 * @property {boolean} is_duet - 是否为对唱歌词
 * @property {number} start_time - 开始时间（毫秒）
 * @property {number} end_time - 结束时间（毫秒）
 */

/**
 * 歌词单词数据结构
 * @typedef {Object} LyricWord
 * @property {number} start_time - 开始时间（毫秒）
 * @property {number} end_time - 结束时间（毫秒）
 * @property {string} word - 单词文本
 * @property {string} [roman_word] - 音译文本
 */