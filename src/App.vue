<script setup>
import AppLayout from './components/layout/AppLayout.vue';
</script>

<template>
  <AppLayout />
</template>

<style>
/* EffiTools 设计风格 - 全局样式变量 */
:root {
  /* 主色调 - 微软蓝 */
  --primary-color: #0078d7;
  --primary-hover: #006cbd;
  --primary-light: rgba(0, 120, 215, 0.1);
  
  /* 背景色 */
  --bg-primary: #F3F3F3;
  --bg-secondary: #FBFBFB;
  --bg-tertiary: rgba(15, 15, 15, 0.05);
  --bg-hover: rgba(15, 15, 15, 0.08);
  --bg-active: rgba(15, 15, 15, 0.1);
  
  /* 文字色 */
  --text-primary: #333;
  --text-secondary: #666;
  --text-tertiary: #999;
  
  /* 边框 */
  --border-color: rgba(0, 0, 0, 0.15);
  --border-light: rgba(0, 0, 0, 0.1);
  
  /* 阴影 */
  --shadow-sm: 0 2px 8px rgba(0, 0, 0, 0.08);
  --shadow-md: 0 8px 32px rgba(0, 0, 0, 0.08);
  --shadow-primary: 0 2px 8px rgba(0, 120, 215, 0.15);
  
  /* 圆角 */
  --radius-sm: 6px;
  --radius-md: 8px;
  --radius-lg: 18px;
  --radius-full: 50%;
  
  /* 字体 */
  --font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
  --font-size-xs: 11px;
  --font-size-sm: 12px;
  --font-size-base: 14px;
  --font-size-md: 16px;
  --font-size-lg: 18px;
  
  /* 间距 */
  --space-1: 4px;
  --space-2: 8px;
  --space-3: 12px;
  --space-4: 16px;
  
  /* 动画 */
  --transition-fast: 0.2s ease;
  --transition-normal: 0.3s ease;
  
  /* 布局 */
  --header-height: 60px;
  --sidebar-width: 200px;
  --bottom-nav-height: 60px;
  --player-bar-height: 100px;
}

/* 深色模式 */
@media (prefers-color-scheme: dark) {
  :root {
    --bg-primary: #2f2f2f;
    --bg-secondary: #1a1a1a;
    --bg-tertiary: rgba(15, 15, 15, 0.6);
    --bg-hover: rgba(255, 255, 255, 0.1);
    --bg-active: rgba(255, 255, 255, 0.15);
    
    --text-primary: #f6f6f6;
    --text-secondary: #999;
    --text-tertiary: #666;
    
    --border-color: rgba(255, 255, 255, 0.15);
    --border-light: rgba(255, 255, 255, 0.1);
    
    --shadow-sm: 0 2px 8px rgba(0, 0, 0, 0.3);
    --shadow-md: 0 8px 32px rgba(0, 0, 0, 0.4);
  }
}

/* 全局样式重置 */
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html {
  font-family: var(--font-family);
  font-size: var(--font-size-base);
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-rendering: optimizeLegibility;
}

body {
  color: var(--text-primary);
  background-color: var(--bg-primary);
  line-height: 1.5;
}

/* 滚动条样式 */
::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: var(--border-light);
  border-radius: var(--radius-full);
}

::-webkit-scrollbar-thumb:hover {
  background: var(--text-tertiary);
}

/* 选中文字样式 */
::selection {
  background: var(--primary-light);
  color: var(--primary-color);
}

/* 图片拖拽 */
img {
  -webkit-user-drag: none;
  user-select: none;
}
</style>
