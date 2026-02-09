问题：`more-btn` 按钮当前与 `track-info` 处于同一 flex 容器下，其 `position: absolute` 参考的是 `.player-album-card`，导致位置不正确。

修复方案：将 `more-btn` 移动到 `.cover-container` 内部，使其相对于封面容器定位。

修改文件：src/components/player/PlayerAlbumCard.vue

具体变更：

1. 将 `more-btn` 从根容器移动到 `.cover-container` 内部（放在 `cover-wrapper` 之后）
2. 更新 CSS 选择器，确保样式仍然生效
3. 删除根容器的 `position: relative`（如果不需要）

