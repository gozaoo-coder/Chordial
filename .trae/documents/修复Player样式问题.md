## 问题分析

根据截图，存在以下样式问题需要修复：

1. **进度条区域不对称、不美观**
   - 当前布局：时间(左) + 音质(中) + 时间(右) + 进度条(使用order:-1)
   - 问题：flex布局导致元素排列不整齐，音质标签位置不居中
   - 修复：改为上下结构，上面是进度条，下面是时间+音质的对称布局

2. **控制按钮样式杂乱、不统一**
   - 播放按钮是白底黑字，其他按钮是透明底白字
   - 按钮大小不一致
   - 修复：统一按钮样式，使用一致的视觉风格

3. **底部被截断**
   - player-main有overflow:hidden
   - player-left内容可能超出容器
   - 修复：调整布局，确保内容不被截断

## 具体修复方案

### 1. 修改PlayerProgress.vue
- 改为上下两行布局
- 第一行：进度条（全宽）
- 第二行：当前时间 | 进度条下方居中 | 音质标签 | 总时间（右对齐）
- 使用更合理的flex布局

### 2. 修改PlayerControls.vue
- 统一所有按钮为圆形透明背景
- 播放按钮使用白色背景，但保持相同尺寸比例
- 调整按钮间距，使其更紧凑
- 统一图标大小

### 3. 修改PlayerView.vue
- 调整player-left的padding和gap
- 移除可能导致截断的样式
- 优化响应式布局

### 4. 修改PlayerVolume.vue
- 调整音量滑块样式，使其与进度条风格一致
- 统一按钮大小

## 文件变更清单
1. 修改: src/components/player/PlayerProgress.vue
2. 修改: src/components/player/PlayerControls.vue
3. 修改: src/components/player/PlayerVolume.vue
4. 修改: src/views/PlayerView.vue