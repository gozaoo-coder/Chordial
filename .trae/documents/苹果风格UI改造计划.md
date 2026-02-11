## Chordial 苹果风格UI改造计划

### 第一阶段：全局样式系统改造

#### 1. 更新 CSS 变量（App.vue）
- 主色调：#007AFF (Apple Blue) → 替换现有 #0078d7
- 背景色：#FFFFFF / #F5F5F7 → 替换现有 #F3F3F3 / #FBFBFB
- 圆角：增大到 16px-20px
- 阴影：更柔和的 Apple 风格阴影
- 字体：优化字重层级

#### 2. 全局样式优化
- 滚动条样式优化（更细的 Apple 风格）
- 选中文字颜色调整
- 添加玻璃拟态工具类

### 第二阶段：组件级改造

#### 1. AppHeader.vue（顶部导航栏）
- 搜索框：更大的圆角 (20px)，更柔和的边框
- 图标按钮：hover时添加背景色变化
- 整体高度略微增加，更大气

#### 2. AppSidebar.vue（侧边栏）
- 导航项：增大圆角到 10px
- 激活状态：使用填充色 + 轻微阴影
- 图标与文字间距优化
- 添加微妙的hover动画

#### 3. Home.vue（首页）
- 统计卡片：
  - 更大的圆角 (20px)
  - 更柔和的阴影
  - 数字字体加大加粗
  - hover时轻微上浮效果
- 标题区域：增大间距，优化字体层级
- Section标题：更粗的字重

#### 4. TrackList.vue（歌曲列表）
- 列表项：
  - 更大的内边距
  - hover时添加背景色变化
  - 播放按钮使用 Apple 风格圆形
- 表头：更细的字体，大写字母间距
- 封面图：更大的圆角

#### 5. ArtistList.vue（歌手列表）
- 歌手卡片：
  - 圆形封面保持
  - 信息区域居中对齐
  - hover时添加缩放效果

#### 6. AlbumList.vue（专辑列表）
- 专辑封面：更大的圆角 (12px)
- 播放覆盖层：更柔和的背景
- 信息区域：优化间距

#### 7. PlayerControlBar.vue（播放器控制栏）
- 进度条：
  - 更细的线条 (3px)
  - 拖拽手柄使用 Apple 风格
- 控制按钮：
  - 播放按钮使用填充色
  - 其他按钮使用更细的图标
- 整体背景：玻璃拟态效果

#### 8. AppBottomNav.vue（底部导航）
- 导航项：
  - 激活状态使用填充背景
  - 图标与文字间距优化
- 整体背景：玻璃拟态效果

### 第三阶段：细节优化

#### 1. 过渡动画
- 页面切换：更平滑的淡入淡出
- 列表项：添加 stagger 动画
- 按钮：添加按压效果

#### 2. 深色模式
- 同步更新所有深色模式样式
- 使用 Apple 深色模式配色

### 文件修改清单

| 文件 | 修改类型 | 主要内容 |
|------|----------|----------|
| src/App.vue | 修改 | CSS变量系统 |
| src/components/layout/AppHeader.vue | 修改 | 顶部栏样式 |
| src/components/layout/AppSidebar.vue | 修改 | 侧边栏样式 |
| src/components/layout/AppBottomNav.vue | 修改 | 底部导航样式 |
| src/views/Home.vue | 修改 | 首页样式 |
| src/components/common/TrackList.vue | 修改 | 歌曲列表样式 |
| src/components/common/ArtistList.vue | 修改 | 歌手列表样式 |
| src/components/common/AlbumList.vue | 修改 | 专辑列表样式 |
| src/components/player/PlayerControlBar.vue | 修改 | 播放器样式 |

### 预期效果
- 整体视觉更加现代、精致
- 符合 macOS / iOS 用户的审美习惯
- 保持原有功能不变
- 提升用户体验和视觉愉悦度

请确认此计划后，我将开始逐步实施改造。