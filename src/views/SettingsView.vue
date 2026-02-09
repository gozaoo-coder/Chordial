<template>
  <div class="settings-view">
    <div class="settings-header">
      <h1 class="settings-title">设置</h1>
      <p class="settings-subtitle">自定义您的音乐播放体验</p>
    </div>
    
    <div class="settings-content">
      <!-- 播放器设置 -->
      <div class="settings-section">
        <h2 class="section-title">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M9 18V5l12-2v13"/>
            <circle cx="6" cy="18" r="3"/>
            <circle cx="18" cy="16" r="3"/>
          </svg>
          播放器设置
        </h2>
        
        <div class="setting-item">
          <div class="setting-info">
            <label class="setting-label">默认音量</label>
            <span class="setting-desc">设置播放器启动时的默认音量</span>
          </div>
          <div class="setting-control">
            <input 
              type="range" 
              min="0" 
              max="100" 
              v-model="defaultVolume"
              class="slider"
            />
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
            <span class="setting-desc">选择播放列表的默认播放模式</span>
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
      
      <!-- 歌词设置 -->
      <div class="settings-section">
        <h2 class="section-title">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
          </svg>
          歌词设置
        </h2>
        
        <div class="setting-item">
          <div class="setting-info">
            <label class="setting-label">歌词字体大小</label>
            <span class="setting-desc">调整歌词显示的字体大小</span>
          </div>
          <div class="setting-control">
            <input 
              type="range" 
              min="16" 
              max="48" 
              v-model="lyricFontSize"
              class="slider"
            />
            <span class="setting-value">{{ lyricFontSize }}px</span>
          </div>
        </div>
        
        <div class="setting-item">
          <div class="setting-info">
            <label class="setting-label">歌词动画效果</label>
            <span class="setting-desc">启用歌词切换的弹簧动画效果</span>
          </div>
          <div class="setting-control">
            <label class="toggle">
              <input type="checkbox" v-model="lyricSpring" />
              <span class="toggle-slider"></span>
            </label>
          </div>
        </div>
        
        <div class="setting-item">
          <div class="setting-info">
            <label class="setting-label">歌词模糊效果</label>
            <span class="setting-desc">启用歌词边缘模糊效果（需要较好的 GPU 性能）</span>
          </div>
          <div class="setting-control">
            <label class="toggle">
              <input type="checkbox" v-model="lyricBlur" />
              <span class="toggle-slider"></span>
            </label>
          </div>
        </div>
        
        <div class="setting-item">
          <div class="setting-info">
            <label class="setting-label">歌词对齐方式</label>
            <span class="setting-desc">选择歌词在屏幕中的对齐位置</span>
          </div>
          <div class="setting-control">
            <select v-model="lyricAlign" class="select">
              <option value="0">顶部</option>
              <option value="0.3">偏上</option>
              <option value="0.5">居中</option>
              <option value="0.7">偏下</option>
              <option value="1">底部</option>
            </select>
          </div>
        </div>
      </div>
      
      <!-- 背景效果设置 -->
      <div class="settings-section">
        <h2 class="section-title">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="3" y="3" width="18" height="18" rx="2" ry="2"/>
            <circle cx="8.5" cy="8.5" r="1.5"/>
            <polyline points="21 15 16 10 5 21"/>
          </svg>
          背景效果设置
        </h2>
        
        <div class="setting-item">
          <div class="setting-info">
            <label class="setting-label">背景效果强度</label>
            <span class="setting-desc">调整流体背景效果的强度</span>
          </div>
          <div class="setting-control">
            <input 
              type="range" 
              min="0" 
              max="100" 
              v-model="backgroundIntensity"
              class="slider"
            />
            <span class="setting-value">{{ backgroundIntensity }}%</span>
          </div>
        </div>
        
        <div class="setting-item">
          <div class="setting-info">
            <label class="setting-label">背景模糊程度</label>
            <span class="setting-desc">调整背景模糊的程度</span>
          </div>
          <div class="setting-control">
            <input 
              type="range" 
              min="0" 
              max="200" 
              v-model="backgroundBlur"
              class="slider"
            />
            <span class="setting-value">{{ backgroundBlur }}px</span>
          </div>
        </div>
        
        <div class="setting-item">
          <div class="setting-info">
            <label class="setting-label">背景亮度</label>
            <span class="setting-desc">调整背景效果的亮度</span>
          </div>
          <div class="setting-control">
            <input 
              type="range" 
              min="0" 
              max="100" 
              v-model="backgroundBrightness"
              class="slider"
            />
            <span class="setting-value">{{ backgroundBrightness }}%</span>
          </div>
        </div>
        
        <div class="setting-item">
          <div class="setting-info">
            <label class="setting-label">帧率限制</label>
            <span class="setting-desc">设置背景动画的最大帧率（降低可节省电量）</span>
          </div>
          <div class="setting-control">
            <select v-model="fps" class="select">
              <option value="30">30 FPS</option>
              <option value="60">60 FPS</option>
            </select>
          </div>
        </div>
      </div>
      
      <!-- 关于 -->
      <div class="settings-section">
        <h2 class="section-title">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10"/>
            <line x1="12" y1="16" x2="12" y2="12"/>
            <line x1="12" y1="8" x2="12.01" y2="8"/>
          </svg>
          关于
        </h2>
        
        <div class="about-content">
          <div class="app-info">
            <h3 class="app-name">Chordial</h3>
            <p class="app-version">版本 0.1.0</p>
            <p class="app-description">
              一个现代化的本地音乐播放器，支持多种音频格式和歌词显示。
            </p>
          </div>
          
          <div class="credits">
            <p>使用了以下开源项目：</p>
            <ul>
              <li>AMLL (Apple Music Like Lyrics) - 歌词显示组件</li>
              <li>Vue 3 - 前端框架</li>
              <li>Tauri - 桌面应用框架</li>
              <li>Rust - 后端开发语言</li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import { ref, watch, onMounted } from 'vue';

export default {
  name: 'SettingsView',
  setup() {
    // 播放器设置
    const defaultVolume = ref(80);
    const autoPlay = ref(true);
    const defaultPlayMode = ref('sequence');
    
    // 歌词设置
    const lyricFontSize = ref(24);
    const lyricSpring = ref(true);
    const lyricBlur = ref(true);
    const lyricAlign = ref('0.5');
    
    // 背景效果设置
    const backgroundIntensity = ref(50);
    const backgroundBlur = ref(100);
    const backgroundBrightness = ref(50);
    const fps = ref('60');
    
    // 保存设置到 localStorage
    const saveSettings = () => {
      const settings = {
        defaultVolume: defaultVolume.value,
        autoPlay: autoPlay.value,
        defaultPlayMode: defaultPlayMode.value,
        lyricFontSize: lyricFontSize.value,
        lyricSpring: lyricSpring.value,
        lyricBlur: lyricBlur.value,
        lyricAlign: lyricAlign.value,
        backgroundIntensity: backgroundIntensity.value,
        backgroundBlur: backgroundBlur.value,
        backgroundBrightness: backgroundBrightness.value,
        fps: fps.value
      };
      localStorage.setItem('chordial_settings', JSON.stringify(settings));
    };
    
    // 加载设置
    const loadSettings = () => {
      try {
        const saved = localStorage.getItem('chordial_settings');
        if (saved) {
          const settings = JSON.parse(saved);
          defaultVolume.value = settings.defaultVolume ?? 80;
          autoPlay.value = settings.autoPlay ?? true;
          defaultPlayMode.value = settings.defaultPlayMode ?? 'sequence';
          lyricFontSize.value = settings.lyricFontSize ?? 24;
          lyricSpring.value = settings.lyricSpring ?? true;
          lyricBlur.value = settings.lyricBlur ?? true;
          lyricAlign.value = settings.lyricAlign ?? '0.5';
          backgroundIntensity.value = settings.backgroundIntensity ?? 50;
          backgroundBlur.value = settings.backgroundBlur ?? 100;
          backgroundBrightness.value = settings.backgroundBrightness ?? 50;
          fps.value = settings.fps ?? '60';
        }
      } catch (e) {
        console.error('加载设置失败:', e);
      }
    };
    
    // 监听设置变化并保存
    watch([
      defaultVolume, autoPlay, defaultPlayMode,
      lyricFontSize, lyricSpring, lyricBlur, lyricAlign,
      backgroundIntensity, backgroundBlur, backgroundBrightness, fps
    ], saveSettings, { deep: true });
    
    onMounted(() => {
      loadSettings();
    });
    
    return {
      defaultVolume,
      autoPlay,
      defaultPlayMode,
      lyricFontSize,
      lyricSpring,
      lyricBlur,
      lyricAlign,
      backgroundIntensity,
      backgroundBlur,
      backgroundBrightness,
      fps
    };
  }
};
</script>

<style scoped>
.settings-view {
  max-width: 800px;
  margin: 0 auto;
  padding: 24px;
}

.settings-header {
  margin-bottom: 32px;
}

.settings-title {
  font-size: 32px;
  font-weight: 700;
  color: var(--text-primary, #333);
  margin: 0 0 8px 0;
}

.settings-subtitle {
  font-size: 16px;
  color: var(--text-secondary, #666);
  margin: 0;
}

.settings-content {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.settings-section {
  background: var(--bg-secondary, #fff);
  border-radius: 16px;
  padding: 24px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
}

.section-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary, #333);
  margin: 0 0 20px 0;
  display: flex;
  align-items: center;
  gap: 10px;
}

.section-title svg {
  width: 24px;
  height: 24px;
  color: var(--primary-color, #0078d7);
}

.setting-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 0;
  border-bottom: 1px solid var(--border-color, #e8e8e8);
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
  font-size: 15px;
  font-weight: 500;
  color: var(--text-primary, #333);
  margin-bottom: 4px;
}

.setting-desc {
  display: block;
  font-size: 13px;
  color: var(--text-secondary, #666);
}

.setting-control {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-shrink: 0;
}

.setting-value {
  font-size: 14px;
  color: var(--text-secondary, #666);
  min-width: 50px;
  text-align: right;
}

/* 滑块样式 */
.slider {
  width: 120px;
  height: 4px;
  -webkit-appearance: none;
  appearance: none;
  background: var(--border-color, #e8e8e8);
  border-radius: 2px;
  outline: none;
  cursor: pointer;
}

.slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 16px;
  height: 16px;
  background: var(--primary-color, #0078d7);
  border-radius: 50%;
  cursor: pointer;
  transition: transform 0.2s;
}

.slider::-webkit-slider-thumb:hover {
  transform: scale(1.2);
}

.slider::-moz-range-thumb {
  width: 16px;
  height: 16px;
  background: var(--primary-color, #0078d7);
  border-radius: 50%;
  cursor: pointer;
  border: none;
}

/* 切换开关 */
.toggle {
  position: relative;
  display: inline-block;
  width: 48px;
  height: 24px;
}

.toggle input {
  opacity: 0;
  width: 0;
  height: 0;
}

.toggle-slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: var(--border-color, #ccc);
  transition: 0.3s;
  border-radius: 24px;
}

.toggle-slider:before {
  position: absolute;
  content: "";
  height: 18px;
  width: 18px;
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
  transform: translateX(24px);
}

/* 下拉选择 */
.select {
  padding: 8px 32px 8px 12px;
  font-size: 14px;
  border: 1px solid var(--border-color, #e8e8e8);
  border-radius: 8px;
  background: var(--bg-primary, #fff);
  color: var(--text-primary, #333);
  cursor: pointer;
  outline: none;
  min-width: 120px;
  appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 12 12'%3E%3Cpath fill='%23666' d='M6 8L1 3h10z'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 12px center;
}

.select:focus {
  border-color: var(--primary-color, #0078d7);
}

/* 关于部分 */
.about-content {
  padding: 8px 0;
}

.app-info {
  margin-bottom: 24px;
}

.app-name {
  font-size: 24px;
  font-weight: 700;
  color: var(--text-primary, #333);
  margin: 0 0 4px 0;
}

.app-version {
  font-size: 14px;
  color: var(--text-secondary, #666);
  margin: 0 0 12px 0;
}

.app-description {
  font-size: 14px;
  color: var(--text-secondary, #666);
  line-height: 1.6;
  margin: 0;
}

.credits {
  font-size: 14px;
  color: var(--text-secondary, #666);
}

.credits p {
  margin: 0 0 8px 0;
}

.credits ul {
  margin: 0;
  padding-left: 20px;
}

.credits li {
  margin-bottom: 4px;
}

/* 深色模式 */
@media (prefers-color-scheme: dark) {
  .settings-title,
  .section-title,
  .setting-label {
    color: var(--text-primary, #f0f0f0);
  }
  
  .settings-subtitle,
  .setting-desc,
  .setting-value,
  .app-version,
  .app-description,
  .credits {
    color: var(--text-secondary, #aaa);
  }
  
  .settings-section {
    background: var(--bg-secondary, #3a3a3a);
  }
  
  .setting-item {
    border-color: var(--border-color, #4a4a4a);
  }
  
  .slider {
    background: var(--border-color, #4a4a4a);
  }
  
  .toggle-slider {
    background-color: var(--border-color, #4a4a4a);
  }
  
  .select {
    background-color: var(--bg-primary, #2a2a2a);
    border-color: var(--border-color, #4a4a4a);
    color: var(--text-primary, #f0f0f0);
  }
}

/* 移动端适配 */
@media (max-width: 767px) {
  .settings-view {
    padding: 16px;
  }
  
  .settings-title {
    font-size: 24px;
  }
  
  .settings-section {
    padding: 16px;
  }
  
  .setting-item {
    flex-direction: column;
    align-items: flex-start;
    gap: 12px;
  }
  
  .setting-control {
    width: 100%;
    justify-content: space-between;
  }
  
  .slider {
    flex: 1;
  }
}
</style>
