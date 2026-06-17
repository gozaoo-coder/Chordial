/**
 * Player 组件模块
 * 
 * 提供音乐播放器相关的 Vue 组件
 * 采用解耦架构，各组件可独立使用
 */

// AMLL 原生组件封装
export { default as AMLLBackgroundRenderer } from './AMLLBackgroundRenderer.vue'
export { default as AMLLLyricPlayer } from './AMLLLyricPlayer.vue'
export { default as AMLLPlayerContainer } from './AMLLPlayerContainer.vue'

// 原有组件
export { default as PlayerAlbumCard } from './PlayerAlbumCard.vue'
export { default as PlayerControlBar } from './PlayerControlBar.vue'
export { default as PlayerControls } from './PlayerControls.vue'
export { default as PlayerProgress } from './PlayerProgress.vue'
export { default as PlayerVolume } from './PlayerVolume.vue'
