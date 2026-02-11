<script setup>
import AppLayout from './components/layout/AppLayout.vue';
</script>

<template>
  <AppLayout />
</template>

<style>
/* Apple Design 风格 - 全局样式变量 */
:root {
  /* 主色调 - Apple Blue */
  --primary-color: #007AFF;
  --primary-hover: #0056D3;
  --primary-light: rgba(0, 122, 255, 0.15);
  --primary-gradient: linear-gradient(135deg, #007AFF 0%, #5856D6 100%);
  
  /* 背景色 - Apple 风格 */
  --bg-primary: #F5F5F7;
  --bg-secondary: #FFFFFF;
  --bg-tertiary: rgba(120, 120, 128, 0.12);
  --bg-hover: rgba(120, 120, 128, 0.08);
  --bg-active: rgba(120, 120, 128, 0.16);
  --bg-glass: rgba(255, 255, 255, 0.72);
  
  /* 文字色 - Apple 风格 */
  --text-primary: #1D1D1F;
  --text-secondary: #6E6E73;
  --text-tertiary: #86868B;
  --text-quaternary: #A1A1A6;
  
  /* 边框 - Apple 风格 */
  --border-color: rgba(0, 0, 0, 0.1);
  --border-light: rgba(0, 0, 0, 0.06);
  --border-strong: rgba(0, 0, 0, 0.2);
  
  /* 阴影 - Apple 风格柔和阴影 */
  --shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.04), 0 1px 3px rgba(0, 0, 0, 0.02);
  --shadow-md: 0 4px 16px rgba(0, 0, 0, 0.08), 0 2px 8px rgba(0, 0, 0, 0.04);
  --shadow-lg: 0 8px 32px rgba(0, 0, 0, 0.12), 0 4px 16px rgba(0, 0, 0, 0.06);
  --shadow-primary: 0 4px 16px rgba(0, 122, 255, 0.25);
  --shadow-card: 0 2px 12px rgba(0, 0, 0, 0.06);
  
  /* 圆角 - Apple 风格大圆角 */
  --radius-xs: 6px;
  --radius-sm: 8px;
  --radius-md: 12px;
  --radius-lg: 16px;
  --radius-xl: 20px;
  --radius-full: 50%;
  
  /* 字体 - Apple 系统字体 */
  --font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Display', 'SF Pro Text', 'Helvetica Neue', Helvetica, Arial, sans-serif;
  --font-size-xs: 11px;
  --font-size-sm: 12px;
  --font-size-base: 14px;
  --font-size-md: 16px;
  --font-size-lg: 20px;
  --font-size-xl: 24px;
  --font-size-2xl: 32px;
  
  /* 字重 */
  --font-regular: 400;
  --font-medium: 500;
  --font-semibold: 600;
  --font-bold: 700;
  
  /* 间距 - Apple 风格 */
  --space-1: 4px;
  --space-2: 8px;
  --space-3: 12px;
  --space-4: 16px;
  --space-5: 20px;
  --space-6: 24px;
  --space-8: 32px;
  
  /* 动画 - Apple 风格平滑动画 */
  --transition-fast: 0.15s cubic-bezier(0.4, 0, 0.2, 1);
  --transition-normal: 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  --transition-slow: 0.35s cubic-bezier(0.4, 0, 0.2, 1);
  --transition-spring: 0.4s cubic-bezier(0.34, 1.56, 0.64, 1);
  
  /* 布局 */
  --header-height: 64px;
  --sidebar-width: 220px;
  --bottom-nav-height: 64px;
  --player-bar-height: 100px;
}

/* 深色模式 - Apple 风格 */
@media (prefers-color-scheme: dark) {
  :root {
    --primary-color: #0A84FF;
    --primary-hover: #409CFF;
    --primary-light: rgba(10, 132, 255, 0.2);
    --primary-gradient: linear-gradient(135deg, #0A84FF 0%, #5E5CE6 100%);
    
    --bg-primary: #000000;
    --bg-secondary: #1C1C1E;
    --bg-tertiary: rgba(120, 120, 128, 0.24);
    --bg-hover: rgba(120, 120, 128, 0.16);
    --bg-active: rgba(120, 120, 128, 0.24);
    --bg-glass: rgba(28, 28, 30, 0.72);
    
    --text-primary: #FFFFFF;
    --text-secondary: #8E8E93;
    --text-tertiary: #636366;
    --text-quaternary: #48484A;
    
    --border-color: rgba(255, 255, 255, 0.1);
    --border-light: rgba(255, 255, 255, 0.06);
    --border-strong: rgba(255, 255, 255, 0.15);
    
    --shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.3), 0 1px 3px rgba(0, 0, 0, 0.2);
    --shadow-md: 0 4px 16px rgba(0, 0, 0, 0.4), 0 2px 8px rgba(0, 0, 0, 0.3);
    --shadow-lg: 0 8px 32px rgba(0, 0, 0, 0.5), 0 4px 16px rgba(0, 0, 0, 0.4);
    --shadow-primary: 0 4px 16px rgba(10, 132, 255, 0.35);
    --shadow-card: 0 2px 12px rgba(0, 0, 0, 0.3);
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

/* 滚动条样式 - Apple 风格细滚动条 */
::-webkit-scrollbar {
  width: 6px;
  height: 6px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: var(--border-color);
  border-radius: 3px;
  border: 1px solid transparent;
  background-clip: padding-box;
}

::-webkit-scrollbar-thumb:hover {
  background: var(--text-tertiary);
  border: 1px solid transparent;
  background-clip: padding-box;
}

/* 选中文字样式 */
::selection {
  background: var(--primary-light);
  color: var(--primary-color);
}

/* 玻璃拟态效果工具类 */
.glass {
  background: var(--bg-glass);
  backdrop-filter: saturate(180%) blur(20px);
  -webkit-backdrop-filter: saturate(180%) blur(20px);
}

/* Apple 风格卡片 */
.card-apple {
  background: var(--bg-secondary);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-card);
  transition: transform var(--transition-normal), box-shadow var(--transition-normal);
}

.card-apple:hover {
  transform: translateY(-2px);
  box-shadow: var(--shadow-md);
}

/* 图片拖拽 */
img {
  -webkit-user-drag: none;
  user-select: none;
}
</style>
