<script setup>
import AppHeader from './AppHeader.vue';
import AppSidebar from './AppSidebar.vue';
import AppBottomNav from './AppBottomNav.vue';
import { PlayerControlBar } from '@/components/player';
</script>

<template>
  <div class="app-layout">
    <AppHeader />
    
    <div class="layout-body">
      <AppSidebar class="sidebar-desktop" />
      
      <main class="main-content">
        <div class="content-wrapper">
          <router-view v-slot="{ Component }">
            <transition name="fade" mode="out-in">
              <component :is="Component" />
            </transition>
          </router-view>
        </div>
      </main>
    </div>
    
    <AppBottomNav />
    
    <!-- 播放器控制栏 -->
    <PlayerControlBar />
  </div>
</template>

<style scoped>
.app-layout {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  background: var(--bg-primary, #F3F3F3);
}

.layout-body {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.sidebar-desktop {
  flex-shrink: 0;
  position: fixed;
  left: 0;
  top: var(--header-height, 60px);
  bottom: var(--player-bar-height, 100px);
  z-index: 50;
}

.main-content {
  flex: 1;
  margin-left: var(--sidebar-width, 200px);
  padding-bottom: var(--player-bar-height, 100px);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.content-wrapper {
  padding: 24px;
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

@media (max-width: 1023px) {
  .sidebar-desktop {
    display: none;
  }

  .main-content {
    margin-left: 0;
    /* 移动端需要同时考虑底栏和播放器的高度 */
    padding-bottom: calc(var(--bottom-nav-height, 60px) + var(--player-bar-height, 100px));
  }

  .content-wrapper {
    padding: 16px;
  }
}

@media (max-width: 767px) {
  .content-wrapper {
    padding: 12px;
  }
}

@media (prefers-color-scheme: dark) {
  .app-layout {
    --bg-primary: #2f2f2f;
  }
}
</style>
