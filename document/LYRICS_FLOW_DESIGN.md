# 歌词处理完整流程设计

## 概述

本文档描述了 Chordial 应用中歌词获取、处理和显示的完整流程。

## 歌词数据来源

### 1. 嵌入在音频文件中的歌词
- **MP3**: ID3v2 标签中的 USLT 帧（同步歌词）和 SYLT 帧（同步歌词时间戳）
- **FLAC**: Vorbis 注释中的 LYRICS 字段
- **M4A**: iTunes 歌词标签
- **OGG**: Vorbis 注释中的 LYRICS 字段

### 2. 外部歌词文件
- `.lrc` 文件（标准带时间戳的歌词文件）
- `.txt` 文件（纯文本歌词）

## 数据结构

### AudioMetadata 中的歌词字段（现有）
```rust
pub struct AudioMetadata {
    // ... 其他字段
    pub lyrics: Option<String>,                    // 非同步歌词
    pub synced_lyrics: Option<Vec<LyricLine>>,    // 同步歌词
}

pub struct LyricLine {
    pub timestamp: Duration,  // 时间戳
    pub text: String,         // 歌词文本
}
```

### TrackMetadata 中的歌词字段（更新后）
```rust
pub struct TrackMetadata {
    // ... 其他字段
    pub lyrics: Option<String>,           // 非同步歌词文本
    pub synced_lyrics: Option<Vec<LyricLine>>, // 同步歌词（存储为JSON字符串）
}
```

## 歌词处理流程

### 阶段 1: 扫描阶段（提取歌词）

```
音频文件
    ↓
读取元数据（read_metadata）
    ↓
提取歌词信息
    ├─→ lyrics: 从 USLT/SYLT 帧提取
    └─→ synced_lyrics: 从 SYLT/USLT 帧提取
    ↓
转换为字符串格式
    ├─→ lyrics: 直接存储
    └─→ synced_lyrics: 序列化为 JSON
    ↓
存储到 TrackMetadata
    ↓
保存到缓存
```

### 阶段 2: 获取阶段（前端请求）

```
前端请求歌词
    ↓
检查本地缓存
    ├─→ 缓存存在 → 返回缓存的歌词
    └─→ 缓存不存在 → 从音频文件读取
    ↓
返回歌词数据
    ├─→ synced_lyrics: JSON 字符串
    └─→ lyrics: 纯文本
```

### 阶段 3: 显示阶段（歌词同步）

```
播放音乐
    ↓
获取当前播放时间
    ↓
匹配对应时间戳的歌词
    ├─→ 高亮当前歌词行
    ├─→ 自动滚动到当前行
    └─→ 预加载下一行
    ↓
显示歌词
```

## 歌词格式转换

### 1. 同步歌词（JSON → 前端格式）

```javascript
// 后端返回的 JSON 格式
const syncedLyricsJson = '[
  {"timestamp": 1250, "text": "第一句歌词"},
  {"timestamp": 3500, "text": "第二句歌词"}
]';

// 前端解析
const syncedLyrics = JSON.parse(syncedLyricsJson).map(line => ({
  time: line.timestamp / 1000,  // 转换为秒
  text: line.text
}));
```

### 2. 同步歌词（Duration → 毫秒）

```rust
// Duration 转换为毫秒
let milliseconds = line.timestamp.as_millis() as u64;
```

### 3. LRC 格式生成

```rust
fn to_lrc_format(lyrics: &[LyricLine]) -> String {
    lyrics.iter()
        .map(|line| {
            let total_secs = line.timestamp.as_secs();
            let minutes = total_secs / 60;
            let seconds = total_secs % 60;
            let millis = line.timestamp.subsec_millis();
            format!("[{:02}:{:02}.{:03}]{}", minutes, seconds, millis, line.text)
        })
        .collect::<Vec<_>>()
        .join("\n")
}
```

## 前端歌词组件设计

### LyricsView.vue 组件

```vue
<template>
  <div class="lyrics-container">
    <!-- 同步歌词模式 -->
    <div v-if="isSynced" class="synced-lyrics">
      <div 
        v-for="(line, index) in syncedLyrics" 
        :key="index"
        :class="['lyric-line', { active: currentIndex === index }]"
        :data-time="line.time"
      >
        {{ line.text }}
      </div>
    </div>
    
    <!-- 纯文本模式 -->
    <div v-else class="plain-lyrics">
      <pre>{{ plainLyrics }}</pre>
    </div>
  </div>
</template>

<script>
// 属性
props: {
  trackId: String,        // 曲目 ID
  syncedLyrics: Array,    // 同步歌词
  plainLyrics: String,    // 纯文本歌词
  isPlaying: Boolean,     // 是否正在播放
  currentTime: Number     // 当前播放时间（秒）
}

// 逻辑
watch: {
  currentTime(time) {
    this.updateCurrentLine(time);
  }
}
</script>
```

## 歌词播放器集成

### 播放流程

```
用户点击播放
    ↓
获取曲目信息和歌词
    ├─→ getTrackInfo(trackId)
    └─→ getLyrics(trackId)
    ↓
加载音频文件
    ├─→ getMusicFileResource(trackId)
    └─→ 创建 Audio 对象
    ↓
开始播放
    ↓
实时同步歌词
    ├─→ 更新 currentTime
    ├─→ 高亮当前歌词
    └─→ 自动滚动
    ↓
播放完成
    ↓
清理资源
```

### 歌词同步逻辑

```javascript
const updateCurrentLine = (currentTime) => {
  // 找到当前时间对应的歌词行
  const index = this.syncedLyrics.findIndex((line, i) => {
    const nextLine = this.syncedLyrics[i + 1];
    return nextLine ? 
      currentTime >= line.time && currentTime < nextLine.time :
      currentTime >= line.time;
  });
  
  if (index !== -1 && index !== this.currentIndex) {
    this.currentIndex = index;
    this.scrollToLine(index);
  }
};

const scrollToLine = (index) => {
  const element = this.$el.querySelector(`[data-time="${this.syncedLyrics[index].time}"]`);
  if (element) {
    element.scrollIntoView({ behavior: 'smooth', block: 'center' });
  }
};
```

## API 设计

### 后端 API

```rust
// 获取曲目信息（包含歌词）
#[tauri::command]
fn get_track_info(track_id: String) -> Result<TrackMetadata, String>

// 仅获取同步歌词
#[tauri::command]
fn get_synced_lyrics(track_id: String) -> Result<Option<String>, String>

// 仅获取纯文本歌词
#[tauri::command]
fn get_plain_lyrics(track_id: String) -> Result<Option<String>, String>

// 从外部文件加载歌词
#[tauri::command]
fn load_lyrics_from_file(path: String) -> Result<LyricsLoadResult, String>
```

### 前端 API

```javascript
// 获取曲目信息（包含歌词）
import { getTrackInfo } from '@/api/musicSource/musicResource.js'

// 专门获取歌词
import { getLyrics, getSyncedLyrics } from '@/api/musicSource/lyrics.js'

// 资源管理
import { getLyricsResource } from '@/api/musicSource/resourceLoader.js'
```

## 缓存策略

### 1. 扫描时缓存
- 在扫描音频文件时提取歌词
- 将歌词存储在 TrackMetadata 中
- 保存到整体缓存

### 2. 播放时缓存
- 首次请求歌词时提取并缓存
- 后续请求直接使用缓存

### 3. 内存管理
- 使用 ResourceManager 管理歌词资源
- 播放完成后清理歌词引用

## 错误处理

### 1. 无歌词文件
```
情况: 音频文件没有嵌入歌词
处理: 
  - 尝试查找同目录下的 .lrc 文件
  - 如果都不存在，返回空歌词
  - 前端显示"暂无歌词"
```

### 2. 歌词格式错误
```
情况: 歌词数据损坏或格式不正确
处理:
  - 返回空歌词
  - 记录错误日志
  - 前端显示"歌词加载失败"
```

### 3. 文件读取失败
```
情况: 音频文件不存在或无法读取
处理:
  - 返回错误信息
  - 标记曲目为损坏
  - 前端显示"无法加载歌词"
```

## 性能优化

### 1. 歌词提取优化
- 扫描时只提取必要信息
- 异步处理大文件
- 使用缓存避免重复读取

### 2. 歌词显示优化
- 使用虚拟滚动（大量歌词时）
- 预加载当前行附近的歌词
- 节流更新频率（每秒更新一次）

### 3. 内存优化
- 及时释放歌词资源
- 使用弱引用缓存
- 限制缓存大小

## 实现步骤

### 步骤 1: 更新数据结构
- [ ] 扩展 TrackMetadata 添加歌词字段
- [ ] 更新 scanner 提取歌词
- [ ] 添加歌词序列化/反序列化函数

### 步骤 2: 后端 API
- [ ] 实现 get_track_info（包含歌词）
- [ ] 实现专门的歌词获取命令
- [ ] 添加歌词格式转换函数

### 步骤 3: 前端 API
- [ ] 创建 lyrics.js API 文件
- [ ] 更新 musicResource.js
- [ ] 集成到 resourceLoader.js

### 步骤 4: 歌词组件
- [ ] 创建 LyricsView.vue 组件
- [ ] 实现歌词同步逻辑
- [ ] 添加动画效果

### 步骤 5: 播放器集成
- [ ] 在 TrackDetail 中集成歌词
- [ ] 实现播放时歌词同步
- [ ] 添加资源清理逻辑

## 总结

这个歌词处理流程提供了：
- ✅ 完整的歌词提取（从音频文件和外部文件）
- ✅ 同步歌词和纯文本歌词支持
- ✅ 高效的缓存和资源管理
- ✅ 流畅的歌词同步显示
- ✅ 完善的错误处理
- ✅ 良好的性能优化

您现在可以开始实现这个流程了！🎵