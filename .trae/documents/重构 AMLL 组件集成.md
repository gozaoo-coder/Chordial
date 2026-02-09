## 问题分析

当前 AMLL 组件集成存在问题：

1. **属性名错误**：AMLL 的 `LyricPlayer` 组件使用 `lyricLines` 属性接收歌词数据，但代码中使用了错误的 `lyric-data`
2. **数据格式不匹配**：AMLL 需要特定的歌词行格式，当前转换逻辑可能不正确
3. **事件名错误**：AMLL 使用 `lineClick` 而不是 `line-click`

## 重构计划

### 1. 修复 AMLLyrics.vue 组件
- 将 `lyric-data` 改为 `lyricLines`
- 将 `line-click` 改为 `lineClick`
- 移除不存在的属性如 `enable-spring`, `enable-blur`, `font-size` 等
- 修正歌词数据格式转换

### 2. 修复 PlayerBackground.vue 组件
- 检查 `BackgroundRender` 的属性是否正确

### 3. 验证 PlayerView.vue 中的使用
- 确保正确传递歌词数据

### 4. 添加调试日志
- 添加日志以便排查问题

## 修改文件
1. `src/components/player/AMLLyrics.vue` - 主要修复
2. `src/components/player/PlayerBackground.vue` - 验证修复