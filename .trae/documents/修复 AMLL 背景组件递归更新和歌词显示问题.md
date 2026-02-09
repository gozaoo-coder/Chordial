## 问题分析

### 1. BackgroundRender 递归更新错误
错误：`Maximum recursive updates exceeded in component <BackgroundRender>`

根本原因：
- `PlayerBackground.vue` 中的 `hasCover` computed 依赖 `props.coverUrl`
- 当封面从空字符串变为 blob URL 时，`v-if="hasCover"` 条件切换
- `BackgroundRender` 组件内部可能对 `album` prop 的响应式处理有问题，导致递归更新

### 2. 歌词组件不显示
从截图看歌词区域空白，可能是 AMLLyrics 组件初始化或数据传递问题。

## 修复计划

### 修复 1: PlayerBackground.vue
1. 添加 `v-once` 或稳定化 `hasCover` 计算，避免不必要的重新渲染
2. 使用 `shallowRef` 或确保 blob URL 只设置一次
3. 添加 `key` 属性帮助 Vue 更好地追踪组件

### 修复 2: AMLLyrics.vue
1. 检查歌词数据解析逻辑
2. 确保 LyricPlayer 正确初始化
3. 添加更多调试日志确认数据流

### 修复 3: useCoverImage.js
1. 确保 blob URL 只被设置一次，避免重复触发更新

## 具体修改

1. **PlayerBackground.vue**: 
   - 将 `hasCover` 改为使用 `shallowRef` 或添加防抖
   - 给 `BackgroundRender` 添加稳定的 `key`

2. **AMLLyrics.vue**:
   - 检查 `lyricLinesData` computed 是否正确生成
   - 确保 `initLyricPlayer` 只被调用一次

3. **useCoverImage.js**:
   - 确保 `coverUrl` 只在真正变化时才更新