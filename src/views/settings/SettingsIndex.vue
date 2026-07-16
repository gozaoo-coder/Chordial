<template>
  <div ref="rootRef" class="settings-index">
    <div class="index-header">
      <h2 class="index-title">设置</h2>
      <p class="index-subtitle">选择一个分类进行配置</p>
    </div>

    <div class="index-cards">
      <router-link :to="{ name: 'SettingsGeneral' }" class="index-card">
        <div class="card-icon"><i class="bi bi-sliders"></i></div>
        <div class="card-body">
          <h3 class="card-title">通用</h3>
          <p class="card-desc">播放器默认音量、自动播放、播放模式</p>
        </div>
        <i class="bi bi-chevron-right card-arrow"></i>
      </router-link>

      <router-link :to="{ name: 'SettingsP2p' }" class="index-card">
        <div class="card-icon"><i class="bi bi-broadcast"></i></div>
        <div class="card-body">
          <h3 class="card-title">P2P 资源共享</h3>
          <p class="card-desc">局域网设备发现、匹配码、可信设备、二维码匹配</p>
        </div>
        <i class="bi bi-chevron-right card-arrow"></i>
      </router-link>
    </div>
  </div>
</template>

<script setup>
import { onMounted, useTemplateRef } from 'vue';
import { useAnime } from '@/composables/useAnime.js';

const rootRef = useTemplateRef('root');
const { run } = useAnime(() => rootRef.value);

onMounted(() => {
  run(({ animate, stagger, presets }) => {
    animate('.index-header', { ...presets.fadeIn });
    // 入场后清除内联 transform，恢复 CSS :hover 的 translateY(-1px) 上浮效果
    animate('.index-card', {
      ...presets.fadeInUp,
      delay: stagger(70),
      onComplete: () => {
        rootRef.value?.querySelectorAll('.index-card').forEach((el) => {
          el.style.transform = '';
        });
      },
    });
  });
});
</script>

<style scoped>
.settings-index {
  max-width: 720px;
  margin: 0 auto;
}

.index-header {
  margin-bottom: 32px;
}

.index-title {
  font-size: 32px;
  font-weight: 800;
  margin: 0 0 8px;
  color: var(--text-primary, #333);
}

.index-subtitle {
  font-size: 14px;
  color: var(--text-secondary, #666);
  margin: 0;
}

.index-cards {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.index-card {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 20px;
  border-radius: 16px;
  background: var(--bg-secondary, #fff);
  border: 1px solid var(--border-light, #e8e8e8);
  text-decoration: none;
  color: inherit;
  transition: all 0.2s ease;
}

.index-card:hover {
  transform: translateY(-1px);
  border-color: var(--primary-color, #0078d7);
  box-shadow: 0 4px 16px rgba(0, 120, 215, 0.12);
}

.card-icon {
  width: 48px;
  height: 48px;
  border-radius: 12px;
  background: var(--primary-light, rgba(0, 120, 215, 0.1));
  color: var(--primary-color, #0078d7);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 22px;
  flex-shrink: 0;
}

.card-body {
  flex: 1;
  min-width: 0;
}

.card-title {
  font-size: 16px;
  font-weight: 600;
  margin: 0 0 4px;
  color: var(--text-primary, #333);
}

.card-desc {
  font-size: 13px;
  color: var(--text-secondary, #666);
  margin: 0;
}

.card-arrow {
  color: var(--text-tertiary, #999);
  font-size: 14px;
}
</style>
