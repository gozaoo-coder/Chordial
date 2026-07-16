<template>
  <div class="player-controls" ref="root">
    <slot />
  </div>
</template>

<script setup>
/**
 * PlayerControls - 播放器控制组件
 *
 * 容器组件，对 slot 内的控制按钮做 popIn + stagger 入场动画。
 */
import { onMounted, useTemplateRef } from 'vue'
import { useAnime } from '@/composables/useAnime.js'

const rootRef = useTemplateRef('root')
const { run } = useAnime(() => rootRef.value)

onMounted(() => {
  run(({ animate, stagger, presets }) => {
    const children = rootRef.value?.children
    if (children && children.length > 0) {
      animate(children, {
        ...presets.popIn,
        delay: stagger(60),
      })
    }
  })
})
</script>

<style scoped>
.player-controls {
  width: 100%;
  height: 100%;
}
</style>
