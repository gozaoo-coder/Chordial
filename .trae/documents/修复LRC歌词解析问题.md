## 问题分析

从截图可以看到，LRC歌词 `[00:05.19]詞は有り余るばかり` 没有正确解析，时间戳直接显示在歌词文本中。

## 根本原因

`parseSyncedLyrics` 函数（`src/api/musicSource/musicResource.js` 第150-166行）只处理 JSON 格式的歌词数据，但 LRC 格式是原始文本格式 `[mm:ss.xx]歌词内容`。

## 修复方案

修改 `parseSyncedLyrics` 函数，使其同时支持两种格式：
1. JSON 格式（现有的）
2. LRC 文本格式 `[mm:ss.xx]歌词内容`

### 具体修改

**文件: `src/api/musicSource/musicResource.js`**

将 `parseSyncedLyrics` 函数修改为：
1. 首先尝试解析为 JSON
2. 如果 JSON 解析失败，尝试按 LRC 格式解析
3. LRC 解析使用正则表达式 `\[(\d{1,2}):(\d{2})\.(\d{2,3})\]` 匹配时间戳
4. 提取时间（转换为秒）和歌词文本

### 代码变更示例

```javascript
export function parseSyncedLyrics(content) {
  if (!content) return [];
  
  // 首先尝试解析为 JSON
  try {
    const data = JSON.parse(content);
    if (Array.isArray(data)) {
      return data.map(line => ({
        time: line.timestamp / 1000,
        text: line.text
      }));
    }
  } catch (e) {
    // 不是 JSON，继续尝试 LRC 格式
  }
  
  // 解析 LRC 格式: [mm:ss.xx]歌词内容
  const lyrics = [];
  const lines = content.split('\n');
  // 支持 [mm:ss.xx] 或 [mm:ss.xxx] 格式
  const timeRegex = /\[(\d{1,2}):(\d{2})\.(\d{2,3})\](.*)/;
  
  for (const line of lines) {
    const match = line.trim().match(timeRegex);
    if (match) {
      const minutes = parseInt(match[1], 10);
      const seconds = parseInt(match[2], 10);
      const msPart = match[3];
      // 处理毫秒：2位是百分之一秒，3位是毫秒
      const milliseconds = msPart.length === 2 
        ? parseInt(msPart, 10) * 10 
        : parseInt(msPart, 10);
      const time = (minutes * 60 + seconds) + milliseconds / 1000;
      const text = match[4].trim();
      
      if (text) {
        lyrics.push({ time, text });
      }
    }
  }
  
  return lyrics.sort((a, b) => a.time - b.time);
}
```

这个修复将使 `LyricsView.vue` 组件能够正确显示 LRC 格式的歌词，时间戳会被解析并用于同步高亮，而不会显示在歌词文本中。