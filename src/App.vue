<script setup>
import { onMounted, onUnmounted } from 'vue';
import AppLayout from './components/layout/AppLayout.vue';
import { perf } from '@/utils/performanceMonitor.js';

let stopLongTasks = null;
let stopPaint = null;

onMounted(() => {
  stopLongTasks = perf.watchLongTasks();
  stopPaint = perf.watchPaint();
});

onUnmounted(() => {
  if (stopLongTasks) stopLongTasks();
  if (stopPaint) stopPaint();
});
</script>

<template>
  <AppLayout />
</template>

<style>
</style>
