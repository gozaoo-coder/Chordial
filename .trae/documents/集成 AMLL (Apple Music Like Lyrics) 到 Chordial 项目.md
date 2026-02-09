## 项目概述

将 AMLL (Apple Music Like Lyrics) 组件库集成到 Chordial 音乐播放器中，实现类 Apple Music 的歌词显示效果，同时完成完整的音乐播放流程。

---

## 技术栈分析

**Chordial 现有技术栈:**
- Vue 3 + Composition API
- Vite 构建工具
- Tauri 2 (Rust 后端)
- 无全局状态管理 (Props/Events + Composables)

**AMLL 组件库:**
- `@applemusic-like-lyrics/vue` - Vue 3 绑定
- `@applemusic-like-lyrics/core` - 核心组件
- `@applemusic-like-lyrics/lyric` - 歌词解析

---

## 实施计划

### Phase 1: 安装依赖 (优先级: 高)

1. **安装 AMLL 相关包**
   ```bash
   npm install @applemusic-like-lyrics/vue @applemusic-like-lyrics/core @applemusic-like-lyrics/lyric
   ```

2. **检查依赖兼容性**
   - 确认 Vue 3 版本兼容
   - 检查 Vite 配置是否需要调整

### Phase 2: 核心状态管理 - PlayerStore (优先级: 高)

**创建 `src/stores/player.js`**
- 使用 Vue 3 Composition API 创建全局播放器状态
- 实现单例模式，不引入 Pinia/Vuex
- 状态包括:
  - `currentTrack`: 当前播放歌曲
  - `isPlaying`: 播放状态
  - `currentTime`: 当前播放时间
  - `duration`: 总时长
  - `volume`: 音量
  - `playlist`: 播放列表
  - `playMode`: 播放模式 (顺序/随机/循环)
  - `audioElement`: HTMLAudioElement 实例

**关键方法:**
- `play(track)` - 播放指定歌曲
- `pause()` - 暂停
- `resume()` - 恢复播放
- `seek(time)` - 跳转
- `next()` / `previous()` - 下一首/上一首
- `setVolume(vol)` - 设置音量

### Phase 3: AMLL 歌词组件封装 (优先级: 高)

**创建 `src/components/player/AMLLyrics.vue`**
- 封装 AMLL Vue 组件
- 接收 props: `lyricsData`, `currentTime`, `isPlaying`
- 将现有歌词格式转换为 AMLL 支持的 TTML 格式
- 支持逐字歌词高亮效果
- 支持点击歌词跳转

**创建 `src/components/player/PlayerBackground.vue`**
- 使用 AMLL 的流体背景组件
- 根据当前歌曲封面动态生成背景效果

### Phase 4: 播放器控制栏 (优先级: 高)

**创建 `src/components/player/PlayerControlBar.vue`**
- 固定在底部的音乐播放控制栏
- 包含:
  - 歌曲信息 (封面、标题、歌手)
  - 播放/暂停按钮
  - 上一首/下一首按钮
  - 进度条 (可拖动)
  - 音量控制
  - 播放模式切换
  - 歌词显示切换按钮

**修改 `src/components/layout/AppLayout.vue`**
- 引入 PlayerControlBar 组件
- 调整布局为: Header + Sidebar + MainContent + PlayerBar

### Phase 5: 歌词显示页面 (优先级: 中)

**创建 `src/views/PlayerView.vue`**
- 全屏歌词显示页面
- 使用 AMLL 组件展示歌词
- 集成流体背景效果
- 支持从底部控制栏点击进入

**修改路由 `src/routers/index.js`**
- 添加 `/player` 路由

### Phase 6: 设置页面 (优先级: 中)

**创建 `src/views/SettingsView.vue`**
- AMLL 相关设置:
  - 歌词字体大小
  - 歌词动画效果开关
  - 背景效果强度
  - 歌词对齐方式
- 播放器设置:
  - 默认音量
  - 自动播放
  - 音频输出设备

**修改 `src/components/layout/AppHeader.vue`**
- 连接设置按钮到设置页面

### Phase 7: 现有组件适配 (优先级: 中)

**修改 `src/views/TrackDetail.vue`**
- 播放按钮连接 PlayerStore
- 添加"添加到播放列表"功能

**修改 `src/components/common/TrackList.vue`**
- 双击播放
- 右键菜单: 播放、添加到队列

**修改 `src/components/LyricsView.vue`**
- 保留作为降级方案
- 当 AMLL 不可用时使用

### Phase 8: 工具函数和类型定义 (优先级: 中)

**创建 `src/utils/lyricConverter.js`**
- LRC/YRC/QRC/TTML 格式转换
- 为 AMLL 提供统一的歌词数据格式

**创建 `src/types/player.ts`**
- PlayerState 类型定义
- PlayMode 枚举
- Playlist 类型

### Phase 9: 后端支持 (优先级: 低)

**修改 `src-tauri/src/lib.rs`**
- 添加音频播放相关命令 (如需要)
- 当前前端 Audio API 已足够，此阶段可选

---

## 组件去耦设计

### 1. PlayerStore (状态层)
```
职责: 管理播放器核心状态
依赖: 无
被依赖: PlayerControlBar, PlayerView, TrackDetail, TrackList
```

### 2. AMLLyrics (展示层)
```
职责: 歌词显示
依赖: AMLL 库
被依赖: PlayerView
Props: lyricsData, currentTime, isPlaying
Emits: seek(time)
```

### 3. PlayerControlBar (控制层)
```
职责: 播放控制 UI
依赖: PlayerStore
被依赖: AppLayout
```

### 4. PlayerView (页面层)
```
职责: 全屏播放页面
依赖: PlayerStore, AMLLyrics, PlayerBackground
被依赖: 路由
```

---

## 文件结构规划

```
src/
├── stores/
│   └── player.js              # 播放器状态管理
├── components/
│   ├── player/                # 播放器相关组件
│   │   ├── PlayerControlBar.vue
│   │   ├── AMLLyrics.vue
│   │   ├── PlayerBackground.vue
│   │   ├── ProgressBar.vue
│   │   ├── VolumeControl.vue
│   │   └── index.js
│   └── ...
├── views/
│   ├── PlayerView.vue         # 全屏播放页面
│   ├── SettingsView.vue       # 设置页面
│   └── ...
├── utils/
│   └── lyricConverter.js      # 歌词格式转换
├── types/
│   └── player.ts              # 播放器类型定义
└── ...
```

---

## 验收标准

1. **AMLL 集成**
   - [ ] 成功安装并导入 AMLL 组件
   - [ ] 歌词显示效果与 Apple Music 类似
   - [ ] 支持逐字高亮
   - [ ] 支持点击跳转

2. **播放功能**
   - [ ] 播放/暂停/停止
   - [ ] 上一首/下一首
   - [ ] 进度条拖动
   - [ ] 音量控制
   - [ ] 播放模式切换

3. **UI 组件**
   - [ ] 底部播放器控制栏
   - [ ] 全屏歌词页面
   - [ ] 设置页面
   - [ ] 响应式布局

4. **去耦检查**
   - [ ] 组件间通过 Props/Events 通信
   - [ ] 状态集中管理在 PlayerStore
   - [ ] 无循环依赖
   - [ ] 各组件可独立测试

---

## 风险与注意事项

1. **AMLL 浏览器兼容性**: 需要 Chromium 91+ / Firefox 100+ / Safari 9.1+
2. **性能**: 流体背景效果对 GPU 有要求
3. **歌词格式**: 需要将现有歌词转换为 AMLL 支持的格式
4. **内存管理**: 音频资源需要正确释放

---

请确认此计划后，我将开始实施。