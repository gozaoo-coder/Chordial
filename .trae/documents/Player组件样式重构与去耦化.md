## 目标
将PlayerView改造成类似Apple Music风格的播放器界面，左侧显示专辑封面和歌曲信息，右侧显示歌词，同时实现组件去耦化。

## 当前结构分析
- PlayerView.vue: 主播放器视图，包含背景、头部、主内容区（左侧封面+右侧歌词）、底部控制区
- PlayerBackground.vue: 背景组件（已解耦）
- AMLLyrics.vue: 歌词显示组件（已解耦）
- PlayerControlBar.vue: 底部控制栏组件（独立使用）

## 改造计划

### 1. 创建新的独立组件（去耦化）

#### 1.1 PlayerAlbumCard.vue - 专辑卡片组件
- 职责：显示专辑封面、歌曲标题、艺术家、专辑名
- Props: coverUrl, title, artist, album, isPlaying
- 包含播放动画效果

#### 1.2 PlayerControls.vue - 播放控制组件
- 职责：播放/暂停、上一首/下一首、播放模式切换
- Props: isPlaying, playMode, canPlayPrevious, canPlayNext
- Emits: toggle-play, previous, next, toggle-mode

#### 1.3 PlayerProgress.vue - 进度条组件
- 职责：显示播放进度、当前时间/总时长、支持点击跳转
- Props: currentTime, duration, progressPercent
- Emits: seek, seek-to-percent

#### 1.4 PlayerVolume.vue - 音量控制组件
- 职责：音量调节、静音切换
- Props: volume, muted
- Emits: set-volume, toggle-mute

#### 1.5 PlayerHeader.vue - 播放器头部组件
- 职责：返回按钮、标题、设置按钮
- Emits: back, settings

### 2. 重构PlayerView.vue
- 使用新创建的组件替换原有内联代码
- 保持整体布局结构
- 简化script部分，通过组件通信管理状态

### 3. 样式调整
- 左侧区域：专辑封面+歌曲信息（参考截图布局）
- 右侧区域：歌词显示
- 底部：进度条+播放控制
- 整体采用深色主题，与AMLL歌词组件协调

### 4. 文件变更清单
1. 新建: src/components/player/PlayerAlbumCard.vue
2. 新建: src/components/player/PlayerControls.vue
3. 新建: src/components/player/PlayerProgress.vue
4. 新建: src/components/player/PlayerVolume.vue
5. 新建: src/components/player/PlayerHeader.vue
6. 修改: src/components/player/index.js（导出新组件）
7. 修改: src/views/PlayerView.vue（使用新组件）

## 界面布局（参考截图）
```
+------------------+---------------------------+
|  ←  正在播放  ⚙️ |                           |
+------------------+---------------------------+
|                  |   無垢の音が流れてく      |
|   [专辑封面]     |   无垢之声流淌着          |
|                  |                           |
|   歌曲标题       |   あなたが愛に塗れる      |
|   艺术家         |   まで 直至你被爱遍染     |
|   专辑名      ...|   全身                    |
|                  |                           |
|   0:11  无损 4:39|   その色は幻だ 那景象     |
|   |=======|      |   也不过是幻想            |
|                  |                           |
|   ⤮  ⏮  ⏸  ⏭  🔁|   ひとりぼっち音に 就     |
|                  |   此一人孤单地            |
+------------------+---------------------------+
```

## 组件关系图
```
PlayerView
├── PlayerBackground（已有）
├── PlayerHeader（新建）
├── PlayerAlbumCard（新建）
├── AMLLyrics（已有）
├── PlayerProgress（新建）
├── PlayerControls（新建）
└── PlayerVolume（新建）
```

请确认此计划后，我将开始实施具体的代码修改。