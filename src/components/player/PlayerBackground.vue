<template>
  <div class="player-background" :class="{ 'has-cover': hasCover && !webglFailed }">
    <!-- AMLL 流体背景 -->
    <div v-if="hasCover && !webglFailed" class="amll-background-wrapper">
      <BackgroundRender
        :album="stableCoverUrl"
        :playing="isPlaying"
        :flow-speed="flowSpeed"
        :fps="fps"
        :has-lyric="hasLyric"
        :render-scale="renderScale"
        :low-freq-volume="lowFreqVolume"
        :static-mode="staticMode"
        class="amll-background"
        @error="onBackgroundError"
      />
    </div>

    <!-- WebGL 失败或加载失败时的备用背景 -->
    <div v-if="!hasCover || webglFailed" class="fallback-background">
      <div class="cover-blur-bg" :style="coverBgStyle"></div>
      <div class="gradient-bg"></div>
    </div>

    <!-- 遮罩层 -->
    <div class="background-overlay"></div>
  </div>
</template>

<script>
import { computed, ref, watch, onUnmounted } from 'vue';
import { BackgroundRender } from '@applemusic-like-lyrics/vue';

/**
 * PlayerBackground - 播放器背景组件
 * 封装 AMLL BackgroundRender，提供统一的背景渲染接口
 */
export default {
  name: 'PlayerBackground',
  components: {
    BackgroundRender
  },
  props: {
    // 封面图片 URL
    coverUrl: {
      type: String,
      default: ''
    },
    // 是否正在播放
    isPlaying: {
      type: Boolean,
      default: false
    },
    // 背景流动速度
    flowSpeed: {
      type: Number,
      default: 2
    },
    // 帧率
    fps: {
      type: Number,
      default: 30
    },
    // 是否有歌词
    hasLyric: {
      type: Boolean,
      default: true
    },
    // 渲染缩放比例
    renderScale: {
      type: Number,
      default: 0.5
    },
    // 低频音量 (0-1)
    lowFreqVolume: {
      type: Number,
      default: 1.0
    },
    // 是否启用静态模式（节省性能）
    staticMode: