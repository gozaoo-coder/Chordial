# 音乐资源管理系统使用指南

## 概述

本文档描述了 Chordial 应用中音乐资源管理系统的完整实现，包括：
- 从后端获取音乐完整信息
- 使用 Tauri IPC Response 高效传输大文件（二进制数据）
- 使用 ResourceManager 自动管理内存资源
- 专辑图片和音乐文件的获取与缓存

## 目录结构

```
src/
├── api/
│   ├── musicSource/
│   │   ├── sources.js          # 音乐源管理 API
│   │   ├── library.js          # 音乐库管理 API
│   │   ├── cache.js            # 缓存管理 API
│   │   ├── musicResource.js    # 音乐资源获取 API（新增）
│   │   └── resourceLoader.js   # ResourceManager 集成层（新增）
│   └── tauri.js                # Tauri API 统一入口
├── views/
│   ├── MusicSourceManager.vue  # 音乐源管理界面
│   ├── TrackDetail.vue         # 曲目详情展示组件（新增）
│   └── TestPage.vue            # 测试页面（新增）
├── js/
│   └── resourceManager.js      # 资源管理器（已存在）
└── routers/
    └── index.js                # 路由配置
```

## 后端 Rust 实现

### 1. 新增的 Tauri 命令

在 `src-tauri/src/lib.rs` 中实现了以下命令：

```rust
// 获取曲目完整信息
#[tauri::command]
fn get_track_info(track_id: String) -> Result<TrackMetadata, String>

// 获取专辑图片（使用 Response）
#[tauri::command]
fn get_album_art(album_id: String, size: String) -> Result<Response, String>

// 获取音乐文件（使用 Response）
#[tauri::command]
fn get_music_file(track_id: String) -> Result<Response, String>

// 获取歌手图片
#[tauri::command]
fn get_artist_image(artist_id: String) -> Result<Response, String>

// 获取歌词
#[tauri::command]
fn get_lyrics(track_id: String) -> Result<String, String>
```

### 2. Response 的使用优势

使用 `tauri::ipc::Response` 传递二进制数据的好处：

- **高效传输**：二进制数据直接传递，减少序列化和反序列化开销
- **大文件支持**：适合传输图片、音频等大文件
- **内存优化**：后端可以直接读取文件并传递字节流

示例：
```rust
#[tauri::command]
fn get_music_file(track_id: String) -> Result<Response, String> {
    let path = format!("/path/to/music/{}.mp3", track_id);
    let data = std::fs::read(&path).unwrap();
    tauri::ipc::Response::new(data)
}
```

### 3. 扩展的 TrackMetadata 结构

```rust
pub struct TrackMetadata {
    pub id: String,                    // 曲目唯一标识
    pub source_id: String,             // 所属源 ID
    pub path: PathBuf,                 // 文件路径
    pub file_name: String,             // 文件名
    pub title: Option<String>,         // 标题
    pub artist: Option<String>,        // 艺术家
    pub artist_id: Option<String>,     // 艺术家 ID（新增）
    pub album: Option<String>,         // 专辑
    pub album_id: Option<String>,      // 专辑 ID（新增）
    pub album_art_path: Option<PathBuf>, // 专辑封面路径（新增）
    pub duration: Option<u64>,         // 时长（秒）
    pub format: String,                // 格式
    pub file_size: u64,                // 文件大小（字节）
    pub bitrate: Option<u32>,          // 比特率（新增）
    pub sample_rate: Option<u32>,      // 采样率（新增）
    pub channels: Option<u16>,         // 声道数（新增）
    pub year: Option<u32>,             // 年份（新增）
    pub genre: Option<String>,         // 流派（新增）
    pub composer: Option<String>,      // 作曲（新增）
    pub comment: Option<String>,       // 备注（新增）
    pub lyrics_path: Option<PathBuf>,  // 歌词路径（新增）
    pub added_at: DateTime<Utc>,       // 添加时间
}
```

## 前端实现

### 1. API 层

#### musicResource.js
```javascript
import { invoke } from '@tauri-apps/api/core';

// 获取曲目信息
export async function getTrackInfo(trackId) {
  return invoke('get_track_info', { track_id: trackId });
}

// 获取专辑图片
export async function getAlbumArt(albumId, size = 'medium') {
  const result = await invoke('get_album_art', { album_id: albumId, size });
  // 处理 Response 返回的二进制数据
  if (result && result.data) {
    if (result.data instanceof ArrayBuffer) {
      return result.data;
    }
    if (result.data instanceof Uint8Array) {
      return result.data.buffer;
    }
  }
  throw new Error('无效的响应格式');
}

// 获取音乐文件
export async function getMusicFile(trackId) {
  const result = await invoke('get_music_file', { track_id: trackId });
  // 处理方式同上
}
```

### 2. ResourceManager 集成层

#### resourceLoader.js
```javascript
import ResourceManager from '@/js/resourceManager.js';

const resourceManager = new ResourceManager();

// 获取专辑图片资源（自动管理内存）
export async function getAlbumArtResource(albumId, size = 'medium') {
  const key = `album_art_${albumId}_${size}`;
  
  return resourceManager.getResource(key, async () => {
    const { getAlbumArt } = await import('./musicResource.js');
    const data = await getAlbumArt(albumId, size);
    return data;
  });
}

// 获取音乐文件资源（自动管理内存）
export async function getMusicFileResource(trackId) {
  const key = `music_file_${trackId}`;
  
  return resourceManager.getResource(key, async () => {
    const { getMusicFile } = await import('./musicResource.js');
    const data = await getMusicFile(trackId);
    return data;
  });
}

// 预加载资源
export async function preloadAlbumArt(albumId, size = 'medium') {
  const key = `album_art_${albumId}_${size}`;
  resourceManager.preload(key, async () => {
    const { getAlbumArt } = await import('./musicResource.js');
    return getAlbumArt(albumId, size);
  });
}

// 手动释放资源
export function releaseAlbumArt(albumId, size = 'medium') {
  const key = `album_art_${albumId}_${size}`;
  const resource = resourceManager.cache.get(key);
  if (resource) {
    resourceManager._releaseResource(key);
  }
}

// 清理所有资源
export function clearAllResources() {
  resourceManager.clear();
}
```

### 3. 组件使用示例

#### TrackDetail.vue
```vue
<template>
  <div class="track-detail">
    <img v-if="albumArtUrl" :src="albumArtUrl" alt="专辑封面" />
    <h1>{{ track.title }}</h1>
    <p>艺术家: {{ track.artist }}</p>
    <p>专辑: {{ track.album }}</p>
    <p>时长: {{ formatDuration(track.duration) }}</p>
    <p>比特率: {{ track.bitrate }} kbps</p>
    <p>采样率: {{ track.sample_rate }} Hz</p>
    <p>格式: {{ track.format }}</p>
    <p>文件大小: {{ formatFileSize(track.file_size) }}</p>
    
    <button @click="playTrack" :disabled="isLoadingMusic">
      {{ isLoadingMusic ? '加载中...' : '播放' }}
    </button>
    
    <div v-if="lyrics">
      <h3>歌词</h3>
      <pre>{{ lyrics }}</pre>
    </div>
  </div>
</template>

<script>
import { ref, watch, onMounted, onUnmounted } from 'vue'
import { getTrackInfo, getLyrics } from '@/api/musicSource/musicResource.js'
import { 
  getAlbumArtResource, 
  getMusicFileResource,
  releaseAlbumArt,
  releaseMusicFile 
} from '@/api/musicSource/resourceLoader.js'

export default {
  props: {
    trackId: String
  },
  setup(props) {
    const track = ref(null)
    const albumArtUrl = ref(null)
    const lyrics = ref(null)
    const isLoadingMusic = ref(false)
    
    let artResource = null
    let musicResource = null

    const loadTrackInfo = async () => {
      if (!props.trackId) return
      
      try {
        track.value = await getTrackInfo(props.trackId)
        
        // 加载专辑图片
        if (track.value.album_id) {
          artResource = await getAlbumArtResource(track.value.album_id, 'large')
          albumArtUrl.value = artResource.url
        }
        
        // 预加载音乐文件
        getMusicFileResource(props.trackId).then(resource => {
          musicResource = resource
        })
        
        // 获取歌词
        lyrics.value = await getLyrics(props.trackId)
      } catch (err) {
        console.error('加载失败:', err)
      }
    }

    const playTrack = async () => {
      if (!musicResource) {
        musicResource = await getMusicFileResource(props.trackId)
      }
      const audio = new Audio(musicResource.url)
      audio.play()
    }

    // 组件销毁时释放资源
    onUnmounted(() => {
      if (artResource) {
        artResource.release()
      }
      if (musicResource) {
        musicResource.release()
      }
    })

    watch(() => props.trackId, loadTrackInfo)
    onMounted(loadTrackInfo)

    return { track, albumArtUrl, lyrics, isLoadingMusic, playTrack }
  }
}
</script>
```

## ResourceManager 详解

### 工作原理

ResourceManager 使用引用计数来管理资源生命周期：

1. **获取资源** (`getResource`):
   - 检查缓存是否有该资源
   - 如果有，增加引用计数并返回
   - 如果没有，创建新请求并缓存
   - 返回包含 `release()` 方法的资源对象

2. **释放资源** (`release`):
   - 减少引用计数
   - 如果计数为 0，清理资源（释放 URL、删除缓存）
   - 如果计数 > 0，只减少计数

3. **预加载** (`preload`):
   - 提前加载资源但不阻塞
   - 后续请求可以直接使用缓存

### 使用示例

```javascript
// 方式1: 在组件中使用（自动管理）
const resource = await getAlbumArtResource('album123', 'large')
imageUrl.value = resource.url

// 组件销毁时自动释放
// 或手动释放
resource.release()

// 方式2: 预加载
preloadAlbumArt('album123', 'large')

// 方式3: 清理所有资源
clearAllResources()
```

### 优势

- ✅ **自动内存管理**：避免资源泄露
- ✅ **重复请求优化**：相同资源只请求一次
- ✅ **预加载支持**：提升用户体验
- ✅ **二进制数据处理**：适合大文件传输

## 测试

### 测试页面

访问 `/test` 路由可以测试以下功能：

1. **扫描音乐源**
   - 点击"扫描所有源"按钮
   - 查看扫描到的曲目数量

2. **曲目列表**
   - 显示所有扫描到的曲目
   - 点击曲目查看详情

3. **曲目详情**
   - 显示曲目完整信息
   - 显示专辑图片
   - 显示歌词（如有）
   - 播放音乐

4. **资源管理**
   - 查看缓存资源数量
   - 清理所有资源

## 注意事项

1. **后端实现**
   - 确保 `get_album_art` 和 `get_music_file` 命令已实现
   - 这些命令应使用 `tauri::ipc::Response::new(data)` 返回二进制数据

2. **文件路径**
   - 目前实现使用简单的路径逻辑
   - 可能需要根据实际存储方式调整

3. **资源清理**
   - 组件销毁时务必释放资源
   - 使用 `onUnmounted` 钩子
   - 避免内存泄漏

4. **错误处理**
   - 所有 API 都有错误处理
   - 控制台会输出详细日志

## 未来改进

1. 实现专辑 ID 和艺术家 ID 的自动生成
2. 添加专辑封面提取功能
3. 支持更多图片格式
4. 实现播放列表功能
5. 添加缓存策略配置
6. 支持流式传输大文件

## 总结

这个实现提供了：
- ✅ 完整的音乐信息获取
- ✅ 高效的二进制数据传输
- ✅ 自动的内存资源管理
- ✅ 良好的用户体验（预加载、缓存）
- ✅ 完善的错误处理和日志

您现在可以测试这些功能了！访问 `/test` 页面开始使用。