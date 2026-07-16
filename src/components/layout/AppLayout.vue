<script setup>
import { computed } from 'vue'
import { animate } from 'animejs';
import { ANIME_PRESETS } from '@/utils/animePresets.js';
import AppHeader from './AppHeader.vue';
import AppSidebar from './AppSidebar.vue';
import AppBottomNav from './AppBottomNav.vue';
import P2pMatchDialog from '@/components/p2p/P2pMatchDialog.vue';
import { PlayerControlBar } from '@/components/player';
import { PlayerStore } from '@/stores/player.js'
import { useCoverImage } from '@/composables/useCoverImage';

const currentTrack = computed(() => PlayerStore.state.currentTrack)
const { coverUrl: albumCoverUrl } = useCoverImage(currentTrack, 'small')
const isPlaying = computed(() => PlayerStore.state.isPlaying)

// 路由切换动画：旧页面淡出，新页面从下淡入
function onRouteEnter(el, done) {
  animate(el, {
    ...ANIME_PRESETS.fadeInUp,
    onComplete: done,
  });
}

function onRouteLeave(el, done) {
  animate(el, {
    ...ANIME_PRESETS.fadeOut,
    onComplete: done,
  });
}
</script>

<template>
  <div class="app-layout">
    <AppHeader />
    
    <div class="layout-body">
      <AppSidebar class="sidebar-desktop" />
      
      <main class="main-content">
        <div class="content-wrapper">
          <router-view v-slot="{ Component }">
            <transition :css="false" mode="out-in" @enter="onRouteEnter" @leave="onRouteLeave">
              <component :is="Component" />
            </transition>
          </router-view>
        </div>
      </main>
    </div>
    
    <AppBottomNav />
    
    <!-- 播放器控制栏 -->
    <PlayerControlBar
      :album-cover-url="albumCoverUrl"
      :is-playing="isPlaying"
      @play="PlayerStore.resume"
      @pause="PlayerStore.pause"
      @next="PlayerStore.playNext"
    />

    <!-- P2P 匹配对话框 + 事件桥接 -->
    <P2pMatchDialog />
  </div>
</template>

<style scoped>
.app-layout {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  background: var(--bg-primary);
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
  padding: 28px 32px;
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
}

@media (max-width: 1023px) {
  .sidebar-desktop {
    display: none;
  }

  .main-content {
    margin-left: 0;
    /* 移动端需要同时考虑底栏、播放器和系统导航栏的高度 */
    padding-bottom: calc(var(--bottom-nav-height) + var(--player-bar-height) + var(--safe-area-bottom));
  }

  .content-wrapper {
    padding: 20px 24px;
  }
}

@media (max-width: 767px) {
  .content-wrapper {
    padding: 16px 20px;
  }
}

@media (prefers-color-scheme: dark) {
  .app-layout {
    background: var(--bg-primary);
  }
}
</style>
