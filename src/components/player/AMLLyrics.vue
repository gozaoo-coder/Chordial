<template>
  <div class="amll-lyrics">
    <!-- AMLL 歌词组件占位 - 已被 AMLLLyricPlayer 替代 -->
    <AMLLLyricPlayer
      :lyric-lines="lyricLines"
      :current-time="currentTime"
      :playing="playing"
    />
  </div>
</template>

<script setup>
/**
 * AMLLLyrics - AMLL 歌词组件（兼容旧版本）
 * 
 * 注意：请直接使用 AMLLLyricPlayer 组件
 */
import { AMLLLyricPlayer } from '@/components/player'

defineProps({
  lyricLines: {
    type: Array,
    default: () => []
  },
  currentTime: {
    type: Number,
    default: 0
  },
  playing: {
    type: Boolean,
    default: false
  }
})
</script>

<style scoped>
.amll-lyrics {
  width: 100%;
  height: 100%;
}
</style>
