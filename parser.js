/**
 * 歌词解析器
 * 支持 LRC、YRC 格式及翻译歌词
 */

/**
 * 解析LRC格式歌词
 * @param {string} lrc - LRC歌词文本
 * @returns {Array} 解析后的歌词数组 [{t: 时间, c: 内容}]
 */
function parseLrc(lrc) {
  if (!lrc || typeof lrc !== 'string') {
    return []
  }

  const lines = lrc.split('\n')
  const lyrics = []

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i].replace(/(^\s*)|(\s*$)/g, '') // 去除前后空格
    
    // 提取时间标签 [mm:ss.xx]
    const timeMatches = line.match(/\[(\d+):([\d.]+)\]/g)
    if (!timeMatches) continue

    // 提取歌词内容（去除所有时间标签）
    let content = line
    timeMatches.forEach(match => {
      content = content.replace(match, '')
    })
    content = content.trim()

    if (!content) continue

    // 解析每个时间标签
    timeMatches.forEach(timeTag => {
      const timeMatch = timeTag.match(/\[(\d+):([\d.]+)\]/)
      if (timeMatch) {
        const minutes = parseInt(timeMatch[1])
        const seconds = parseFloat(timeMatch[2])
        const time = (minutes * 60 + seconds).toFixed(3)
        
        if (parseFloat(time) > 0) {
          lyrics.push({
            t: parseFloat(time),
            c: content
          })
        }
      }
    })
  }

  // 按时间排序
  lyrics.sort((a, b) => a.t - b.t)
  
  return lyrics
}

/**
 * 解析YRC格式歌词（逐字歌词）
 * @param {string} content - YRC歌词文本
 * @returns {Array} 解析后的逐字歌词数组
 */
function parseYrc(content) {
  if (!content || typeof content !== 'string') {
    return []
  }

  const lines = content.split('\n')
  const yrcLyrics = []
  let mergeCount = 0 // 合并次数统计

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i].trim()
    if (!line) continue

    // 解析时间信息 [开始时间,持续时间]
    const timeInfo = line.substring(line.indexOf('[') + 1, line.indexOf(']')).split(',')
    const startTime = Number(timeInfo[0]) / 1000
    const duration = Number(timeInfo[1]) / 1000

    if (isNaN(startTime) || isNaN(duration)) continue

    // 创建歌词行对象
    const lyricLine = {
      t: startTime,
      edt: startTime + duration,
      c: [], // 逐字数组
      strC: '', // 完整字符串
      playing: false,
      index: 0,
      width: undefined,
      fontSize: undefined,
      progressleft: '0px',
      lastResizeTime: undefined
    }

    // 提取歌词内容部分
    const contentStart = line.indexOf(']') + 1
    const content = line.substring(contentStart).trim()
    
    // 解析逐字信息 (开始时间,持续时间,音高)文字
    const wordMatches = content.match(/\((\d+),(\d+),(\d+)\)([^\(]*)/g)
    if (!wordMatches) continue

    let fullText = ''

    wordMatches.forEach(wordMatch => {
      const wordInfo = wordMatch.match(/\((\d+),(\d+),(\d+)\)(.*)/)
      if (!wordInfo) return

      const wordStart = Number(wordInfo[1]) / 1000
      const wordDuration = Number(wordInfo[2]) / 1000
      const wordContent = wordInfo[4] || ''

      if (wordDuration <= 0) return

      fullText += wordContent

      const wordObj = {
        t: wordStart,
        dur: wordDuration,
        originDur: wordDuration,
        str: wordContent,
        shine: wordDuration >= 2 ? 'long' : '', // 长歌词加特效
        progress: 0,
        width: undefined,
        left: undefined
      }

      lyricLine.c.push(wordObj)
    })

    lyricLine.strC = fullText
    yrcLyrics.push(lyricLine)
  }

  console.log(`YRC解析完成，合并${mergeCount}次`)
  return yrcLyrics
}

/**
 * 合并翻译歌词
 * @param {Array} mainLyrics - 主歌词数组
 * @param {Array} tranLyrics - 翻译歌词数组
 * @param {string} type - 翻译类型 (tran, ytlrc)
 */
function mergeTranslation(mainLyrics, tranLyrics, type = 'tran') {
  if (!mainLyrics || !tranLyrics || mainLyrics.length === 0 || tranLyrics.length === 0) {
    return
  }

  tranLyrics.forEach(tranLine => {
    const mainIndex = mainLyrics.findIndex(mainLine => 
      Math.abs(mainLine.t - tranLine.t) < 0.1 // 时间差小于0.1秒认为是同一句
    )
    
    if (mainIndex !== -1) {
      mainLyrics[mainIndex][type + 'C'] = tranLine.c
    }
  })
}

/**
 * 主解析函数
 * @param {string|object} lyrics - 歌词数据，可以是字符串或包含多种歌词的对象
 * @returns {object} 解析后的歌词对象
 */
export function parseLyrics(lyrics) {
  if (!lyrics) {
    return {
      offset: 0,
      ms: [],
      tran: false,
      yrc: false,
      ytlrc: false
    }
  }

  const result = {
    offset: 0, // 时间偏移（毫秒）
    ms: [],    // 普通LRC歌词
    tran: false, // 是否有翻译
    yrc: false,  // 是否有YRC逐字歌词
    ytlrc: false // 是否有YRC翻译
  }

  try {
    if (typeof lyrics === 'string') {
      // 纯字符串，按LRC格式解析
      result.ms = parseLrc(lyrics)
      
    } else if (typeof lyrics === 'object') {
      // 对象格式，可能包含多种歌词
      
      // 解析主歌词
      if (lyrics.lrc || lyrics.lrcx) {
        const mainLyrics = parseLrc(lyrics.lrc || lyrics.lrcx)
        result.ms = mainLyrics
      }

      // 解析翻译歌词
      if (lyrics.tlrc || lyrics.tran) {
        const tranLyrics = parseLrc(lyrics.tlrc || lyrics.tran)
        mergeTranslation(result.ms, tranLyrics, 'tran')
        result.tran = true
      }

      // 解析YRC逐字歌词
      if (lyrics.yrc) {
        const yrcLyrics = parseYrc(lyrics.yrc)
        result.yrc = yrcLyrics
        
        // 解析YRC翻译
        if (lyrics.ytlrc) {
          const ytlrcLyrics = parseLrc(lyrics.ytlrc)
          mergeTranslation(result.yrc, ytlrcLyrics, 'ytlrc')
          result.ytlrc = true
        }
      }
    }

    // 如果没有主歌词但有YRC，用YRC生成普通歌词
    if (result.ms.length === 0 && result.yrc && result.yrc.length > 0) {
      result.ms = result.yrc.map(yrcLine => ({
        t: yrcLine.t,
        c: yrcLine.strC
      }))
    }

  } catch (error) {
    console.error('歌词解析错误:', error)
    return {
      offset: 0,
      ms: [],
      tran: false,
      yrc: false,
      ytlrc: false
    }
  }

  return result
}

/**
 * 格式化时间显示
 * @param {number} seconds - 秒数
 * @returns {string} 格式化后的时间字符串 (mm:ss)
 */
export function formatTime(seconds) {
  if (isNaN(seconds) || seconds < 0) {
    return '00:00'
  }

  const totalSeconds = Math.floor(seconds)
  const minutes = Math.floor(totalSeconds / 60)
  const secs = totalSeconds % 60
  
  const paddedMinutes = minutes < 10 ? '0' + minutes : minutes
  const paddedSeconds = secs < 10 ? '0' + secs : secs
  
  return `${paddedMinutes}:${paddedSeconds}`
}

/**
 * 查找当前时间对应的歌词行
 * @param {Array} lyrics - 歌词数组
 * @param {number} currentTime - 当前时间（秒）
 * @returns {number} 当前歌词行的索引，-1表示未找到
 */
export function findCurrentLine(lyrics, currentTime) {
  if (!lyrics || lyrics.length === 0) {
    return -1
  }

  // 普通LRC歌词
  if (Array.isArray(lyrics)) {
    let index = lyrics.findIndex(item => item.t >= (currentTime + 0.6)) - 1
    return index === -2 ? lyrics.length - 1 : index
  }

  // YRC歌词
  if (lyrics.yrc) {
    let index = -1
    for (let i = 0; i < lyrics.yrc.length; i++) {
      const line = lyrics.yrc[i]
      if (!line.c || line.c.length === 0) continue
      
      const lastWord = line.c[line.c.length - 1]
      const lineEndTime = lastWord.t + lastWord.dur
      
      if (lineEndTime >= currentTime + 0.1 && line.t >= currentTime + 0.2) {
        index = i - 1
        break
      }
    }
    return index === -1 ? lyrics.yrc.length - 1 : index
  }

  return -1
}