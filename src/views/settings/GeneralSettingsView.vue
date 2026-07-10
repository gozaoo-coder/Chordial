<template>
  <div class="general-view">
    <div class="section-header">
      <h2 class="section-title">通用设置</h2>
      <p class="section-subtitle">自定义您的音乐播放体验</p>
    </div>

    <div class="settings-section">
      <h3 class="block-title">
        <i class="bi bi-music-note-beamed"></i>
        播放器
      </h3>

      <div class="setting-item">
        <div class="setting-info">
          <label class="setting-label">默认音量</label>
          <span class="setting-desc">启动时的默认音量</span>
        </div>
        <div class="setting-control">
          <input type="range" min="0" max="100" v-model="defaultVolume" class="slider" />
          <span class="setting-value">{{ defaultVolume }}%</span>
        </div>
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <label class="setting-label">自动播放</label>
          <span class="setting-desc">选择歌曲后自动开始播放</span>
        </div>
        <div class="setting-control">
          <label class="toggle">
            <input type="checkbox" v-model="autoPlay" />
            <span class="toggle-slider"></span>
          </label>
        </div>
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <label class="setting-label">默认播放模式</label>
          <span class="setting-desc">播放列表的默认播放模式</span>
        </div>
        <div class="setting-control">
          <select v-model="defaultPlayMode" class="select">
            <option value="sequence">顺序播放</option>
            <option value="random">随机播放</option>
            <option value="loop">列表循环</option>
            <option value="loop_one">单曲循环</option>
          </select>
        </div>
      </div>
    </div>

    <div class="settings-section">
      <h3 class="block-title">
        <i class="bi bi-info-circle"></i>
        关于
      </h3>
      <div class="about-content">
        <h4 class="app-name">Chordial</h4>
        <p class="app-version">版本 0.1.0</p>
        <p class="app-description">
          一个现代化的本地音乐播放器，支持多种音频格式和歌词显示。
        </p>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, watch, onMounted } from 'vue';

const defaultVolume = ref(80);
const autoPlay = ref(true);
const defaultPlayMode = ref('sequence');

const saveSettings = () => {
  const settings = {
    defaultVolume: defaultVolume.value,
    autoPlay: autoPlay.value,
    defaultPlayMode: defaultPlayMode.value,
  };
  localStorage.setItem('chordial_settings', JSON.stringify(settings));
};

const loadSettings = () => {
  try {
    const saved = localStorage.getItem('chordial_settings');
    if (saved) {
      const s = JSON.parse(saved);
      defaultVolume.value = s.defaultVolume ?? 80;
      autoPlay.value = s.autoPlay ?? true;
      defaultPlayMode.value = s.defaultPlayMode ?? 'sequence';
    }
  } catch (e) {
    console.error('加载设置失败:', e);
  }
};

watch([defaultVolume, autoPlay, defaultPlayMode], saveSettings, { deep: true });

onMounted(loadSettings);
</script>

<style scoped>
.general-view {
  max-width: 720px;
  margin: 0 auto;
}

.section-header {
  margin-bottom: 24px;
}

.section-title {
  font-size: 24px;
  font-weight: 800;
  margin: 0 0 6px;
  color: var(--text-primary, #333);
}

.section-subtitle {
  font-size: 13px;
  color: var(--text-secondary, #666);
  margin: 0;
}

.settings-section {
  background: var(--bg-secondary, #fff);
  border-radius: 16px;
  padding: 20px 24px;
  margin-bottom: 16px;
  border: 1px solid var(--border-light, #e8e8e8);
}

.block-title {
  font-size: 15px;
  font-weight: 700;
  margin: 0 0 16px;
  color: var(--text-primary, #333);
  display: flex;
  align-items: center;
  gap: 8px;
}

.block-title i {
  color: var(--primary-color, #0078d7);
}

.setting-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 0;
  border-bottom: 1px solid var(--border-light, #e8e8e8);
}

.setting-item:last-child {
  border-bottom: none;
}

.setting-info {
  flex: 1;
  min-width: 0;
}

.setting-label {
  display: block;
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary, #333);
  margin-bottom: 3px;
}

.setting-desc {
  display: block;
  font-size: 12px;
  color: var(--text-secondary, #666);
}

.setting-control {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-shrink: 0;
}

.setting-value {
  font-size: 13px;
  color: var(--text-secondary, #666);
  min-width: 40px;
  text-align: right;
}

.slider {
  width: 120px;
  height: 4px;
  -webkit-appearance: none;
  appearance: none;
  background: var(--border-color, #e8e8e8);
  border-radius: 2px;
  outline: none;
}

.slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 16px;
  height: 16px;
  background: var(--primary-color, #0078d7);
  border-radius: 50%;
  cursor: pointer;
}

.toggle {
  position: relative;
  display: inline-block;
  width: 44px;
  height: 22px;
}

.toggle input {
  opacity: 0;
  width: 0;
  height: 0;
}

.toggle-slider {
  position: absolute;
  cursor: pointer;
  inset: 0;
  background-color: var(--border-color, #ccc);
  transition: 0.3s;
  border-radius: 22px;
}

.toggle-slider:before {
  position: absolute;
  content: "";
  height: 16px;
  width: 16px;
  left: 3px;
  bottom: 3px;
  background-color: white;
  transition: 0.3s;
  border-radius: 50%;
}

.toggle input:checked + .toggle-slider {
  background-color: var(--primary-color, #0078d7);
}

.toggle input:checked + .toggle-slider:before {
  transform: translateX(22px);
}

.select {
  padding: 7px 28px 7px 10px;
  font-size: 13px;
  border: 1px solid var(--border-color, #e8e8e8);
  border-radius: 8px;
  background: var(--bg-primary, #fff);
  color: var(--text-primary, #333);
  cursor: pointer;
  outline: none;
  min-width: 110px;
  appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 12 12'%3E%3Cpath fill='%23666' d='M6 8L1 3h10z'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 10px center;
}

.about-content {
  padding: 4px 0;
}

.app-name {
  font-size: 20px;
  font-weight: 700;
  color: var(--text-primary, #333);
  margin: 0 0 4px;
}

.app-version {
  font-size: 13px;
  color: var(--text-secondary, #666);
  margin: 0 0 8px;
}

.app-description {
  font-size: 13px;
  color: var(--text-secondary, #666);
  line-height: 1.5;
  margin: 0;
}

@media (max-width: 767px) {
  .setting-item {
    flex-direction: column;
    align-items: flex-start;
    gap: 10px;
  }

  .setting-control {
    width: 100%;
    justify-content: space-between;
  }
}
</style>
