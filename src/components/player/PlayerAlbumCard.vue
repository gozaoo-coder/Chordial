<template>
  <div class="player-album-card" ref="root">
    <slot />
  </div>
</template>

<script setup>
/**
 * PlayerAlbumCard - 播放器专辑卡片
 *
 * 容器组件，对根元素做 scaleIn 入场动画（弹性缩放淡入）。
 */
import { onMounted, useTemplateRef } from 'vue'
import { useAnime } from '@/composables/useAnime.js'

const rootRef = useTemplateRef('root')
const { run } = useAnime(() => rootRef.value)

onMounted(() => {
  run(({ animate, presets }) => {
    animate(rootRef.value, { ...presets.scaleIn })
  })
})
</script>

<style scoped>
.player-album-card {
  width: 100%;
  height: 100%;
}
</style>
