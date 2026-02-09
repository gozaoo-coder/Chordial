## 问题分析
当前 AMLL 歌词组件中，播放状态的歌词显示在底部而非垂直居中。根据 AMLL (Apple Music Like Lyrics) 组件的研究，问题出在样式和初始化配置上。

## 修复步骤

### 1. 修改 AMLLyrics.vue 组件

文件: `src/components/player/AMLLyrics.vue`

需要修改的内容：

**样式部分** - 添加必要的 CSS 变量和样式覆盖：
```css
.amll-lyrics-container {
  width: 100%;
  height: 100%;
  position: relative;
  overflow: hidden;
  /* 添加 AMLL 需要的 CSS 变量 */
  --amll-lp-color: white;
  --amll-lp-font-size: max(max(5vh, 2.5vw), 12px);
}

/* AMLL 样式覆盖 - 确保歌词居中 */
:deep(.amll-lyric-player) {
  width: 100%;
  height: 100%;
}

:deep(.amll-lyric-player.dom) {
  --amll-lp-line-width-aspect: 0.8;
  --amll-lp-line-padding-x: 1em;
}

:deep(._lyricLine_ut4sn_6) {
  text-align: center !important;
  display: flex !important;
  justify-content: center !important;
}
```

**脚本部分** - 确保对齐参数正确设置：
```javascript
// 在 initLyricPlayer 函数中，确保在设置歌词后更新对齐
lyricPlayer.setAlignPosition(0.5);  // 0.5 表示垂直中心
lyricPlayer.setAlignAnchor('center'); // 向歌词行中心对齐
```

### 2. 检查 PlayerView.vue 的容器样式

文件: `src/views/PlayerView.vue`

确保 `.player-right` 容器有明确的高度：
```css
.player-right {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  min-width: 0;
  /* 确保高度被正确设置 */
  height: 100%;
  overflow: hidden;
}
```

## 预期效果
修复后，当前播放的歌词行将会在歌词显示区域的垂直中心位置显示，而不是出现在底部。