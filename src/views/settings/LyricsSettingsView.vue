<template>
  <div ref="rootRef" class="lyrics-view">
    <div class="section-header">
      <h2 class="section-title">歌词与播放器</h2>
      <p class="section-subtitle">AMLL 播放器外观、歌词效果、流体背景与显示项</p>
    </div>

    <!-- 歌词效果 -->
    <div class="settings-section">
      <h3 class="block-title">
        <i class="bi bi-music-note-list"></i>
        歌词效果
      </h3>

      <div class="setting-item">
        <div class="setting-info">
          <label class="setting-label">歌词行模糊效果</label>
          <span class="setting-desc">非当前歌词行模糊处理，性能影响较高</span>
        </div>
        <div class="setting-control">
          <label class="toggle">
            <input type="checkbox" v-model="settings.enableLyricLineBlurEffect" />
            <span class="toggle-slider"></span>
          </label>
        </div>
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <label class="setting-label">歌词行缩放效果</label>
          <span class="setting-desc">当前歌词行放大，无性能影响</span>
        </div>
        <div class="setting-control">
          <label class="toggle">
            <input type="checkbox" v-model="settings.enableLyricLineScaleEffect" />
            <span class="toggle-slider"></span>
          </label>
        </div>
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <label class="setting-label">弹簧物理动画</label>
          <span class="setting-desc">歌词切换使用弹簧算法，性能影响较高；关闭后回退到 transition 过渡</span>
        </div>
        <div class="setting-control">
          <label class="toggle">
            <input type="checkbox" v-model="settings.enableLyricLineSpringAnimation" />
            <span class="toggle-slider"></span>
          </label>
        </div>
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <label class="setting-label">显示翻译歌词</label>
          <span class="setting-desc">在原文下方显示翻译</span>
        </div>
        <div class="setting-control">
          <label class="toggle">
            <input type="checkbox" v-model="settings.enableLyricTranslationLine" />
            <span class="toggle-slider"></span>
          </label>
        </div>
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <label class="setting-label">显示音译歌词</label>
          <span class="setting-desc">在原文下方显示罗马音译</span>
        </div>
        <div class="setting-control">
          <label class="toggle">
            <input type="checkbox" v-model="settings.enableLyricRomanLine" />
            <span class="toggle-slider"></span>
          </label>
        </div>
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <label class="setting-label">交换翻译/音译位置</label>
          <span class="setting-desc">音译显示在翻译之上</span>
        </div>
        <div class="setting-control">
          <label class="toggle">
            <input type="checkbox" v-model="settings.enableLyricSwapTransRomanLine" />
            <span class="toggle-slider"></span>
          </label>
        </div>
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <label class="setting-label">逐词渐变宽度</label>
          <span class="setting-desc">0 关闭，0.5 模拟 iPad，1 模拟 Android</span>
        </div>
        <div class="setting-control">
          <input
            type="range" min="0" max="1" step="0.05"
            v-model.number="settings.lyricWordFadeWidth"
            class="slider"
          />
          <span class="setting-value">{{ settings.lyricWordFadeWidth.toFixed(2) }}</span>
        </div>
      </div>
    </div>

    <!-- 歌词字体 -->
    <div class="settings-section">
      <h3 class="block-title">
        <i class="bi bi-type"></i>
        歌词字体
      </h3>

      <div class="setting-item">
        <div class="setting-info">
          <label class="setting-label">字体族</label>
          <span class="setting-desc">留空使用默认字体；CSS font-family 语法</span>
        </div>
        <div class="setting-control">
          <input
            type="text"
            v-model="settings.lyricFontFamily"
            placeholder="例: 'PingFang SC', system-ui"
            class="text-input"
          />
        </div>
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <label class="setting-label">字重</label>
          <span class="setting-desc">CSS font-weight</span>
        </div>
        <div class="setting-control">
          <select v-model.number="settings.lyricFontWeight" class="select">
            <option :value="400">Regular (400)</option>
            <option :value="500">Medium (500)</option>
            <option :value="600">Semibold (600)</option>
            <option :value="700">Bold (700)</option>
            <option :value="800">Heavy (800)</option>
          </select>
        </div>
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <label class="setting-label">字符间距</label>
          <span class="setting-desc">CSS letter-spacing</span>
        </div>
        <div class="setting-control">
          <select v-model="settings.lyricLetterSpacing" class="select">
            <option value="normal">默认</option>
            <option value="0.02em">疏松 (0.02em)</option>
            <option value="0.05em">较疏 (0.05em)</option>
            <option value="0.1em">稀疏 (0.1em)</option>
            <option value="-0.02em">紧凑 (-0.02em)</option>
          </select>
        </div>
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <label class="setting-label">字体大小</label>
          <span class="setting-desc">预设字号档位</span>
        </div>
        <div class="setting-control">
          <select v-model="settings.lyricSizePreset" class="select">
            <option value="tiny">极小</option>
            <option value="extra-small">特小</option>
            <option value="small">小</option>
            <option value="medium">中</option>
            <option value="large">大</option>
            <option value="extra-large">特大</option>
            <option value="huge">巨大</option>
          </select>
        </div>
      </div>
    </div>

    <!-- 流体背景 -->
    <div class="settings-section">
      <h3 class="block-title">
        <i class="bi bi-droplet-half"></i>
        流体背景
      </h3>

      <div class="setting-item">
        <div class="setting-info">
          <label class="setting-label">垂直同步（VSync）</label>
          <span class="setting-desc">跟随显示器刷新率渲染，开启后下方帧率选项失效</span>
        </div>
        <div class="setting-control">
          <label class="toggle">
            <input type="checkbox" v-model="settings.lyricBackgroundVSync" />
            <span class="toggle-slider"></span>
          </label>
        </div>
      </div>

      <div class="setting-item" :class="{ 'is-disabled': settings.lyricBackgroundVSync }">
        <div class="setting-info">
          <label class="setting-label">背景帧率</label>
          <span class="setting-desc">较低可提升性能，性能影响高；垂直同步开启时此选项失效</span>
        </div>
        <div class="setting-control">
          <select
            v-model.number="settings.lyricBackgroundFPS"
            class="select"
            :disabled="settings.lyricBackgroundVSync"
          >
            <option :value="30">30 FPS</option>
            <option :value="45">45 FPS</option>
            <option :value="60">60 FPS</option>
            <option :value="90">90 FPS</option>
            <option :value="120">120 FPS</option>
          </select>
        </div>
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <label class="setting-label">鼓点频段下限</label>
          <span class="setting-desc">驱动背景跳动的低频下限（Hz）。默认 80 对应低音鼓</span>
        </div>
        <div class="setting-control">
          <input
            type="range" min="20" max="500" step="5"
            v-model.number="lowFreqLow"
            class="slider"
          />
          <span class="setting-value">{{ lowFreqLow }} Hz</span>
        </div>
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <label class="setting-label">鼓点频段上限</label>
          <span class="setting-desc">驱动背景跳动的低频上限（Hz）。默认 120；提高可让中频也参与跳动</span>
        </div>
        <div class="setting-control">
          <input
            type="range" :min="lowFreqLow + 5" max="2000" step="5"
            v-model.number="lowFreqHigh"
            class="slider"
          />
          <span class="setting-value">{{ lowFreqHigh }} Hz</span>
        </div>
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <label class="setting-label">渲染缩放</label>
          <span class="setting-desc">0.5 降低一半分辨率以提升性能，1 原始分辨率</span>
        </div>
        <div class="setting-control">
          <input
            type="range" min="0.25" max="1" step="0.05"
            v-model.number="settings.lyricBackgroundRenderScale"
            class="slider"
          />
          <span class="setting-value">{{ settings.lyricBackgroundRenderScale.toFixed(2) }}</span>
        </div>
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <label class="setting-label">静态背景模式</label>
          <span class="setting-desc">封面切换后保持静止，禁用鼓点跳动以节省性能</span>
        </div>
        <div class="setting-control">
          <label class="toggle">
            <input type="checkbox" v-model="settings.lyricBackgroundStaticMode" />
            <span class="toggle-slider"></span>
          </label>
        </div>
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <label class="setting-label">纯色/CSS 背景</label>
          <span class="setting-desc">当背景渲染器为纯色时使用此颜色</span>
        </div>
        <div class="setting-control">
          <input
            type="color"
            v-model="settings.cssBackgroundProperty"
            class="color-input"
          />
          <span class="setting-value">{{ settings.cssBackgroundProperty }}</span>
        </div>
      </div>
    </div>

    <!-- 显示项 -->
    <div class="settings-section">
      <h3 class="block-title">
        <i class="bi bi-ui-checks"></i>
        显示项
      </h3>

      <div class="setting-item">
        <div class="setting-info">
          <label class="setting-label">歌曲名称</label>
        </div>
        <div class="setting-control">
          <label class="toggle">
            <input type="checkbox" v-model="settings.showMusicName" />
            <span class="toggle-slider"></span>
          </label>
        </div>
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <label class="setting-label">艺人</label>
        </div>
        <div class="setting-control">
          <label class="toggle">
            <input type="checkbox" v-model="settings.showMusicArtists" />
            <span class="toggle-slider"></span>
          </label>
        </div>
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <label class="setting-label">专辑名</label>
          <span class="setting-desc">三项全开可能影响布局美观</span>
        </div>
        <div class="setting-control">
          <label class="toggle">
            <input type="checkbox" v-model="settings.showMusicAlbum" />
            <span class="toggle-slider"></span>
          </label>
        </div>
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <label class="setting-label">音量控制条</label>
        </div>
        <div class="setting-control">
          <label class="toggle">
            <input type="checkbox" v-model="settings.showVolumeControl" />
            <span class="toggle-slider"></span>
          </label>
        </div>
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <label class="setting-label">底部控制按钮</label>
          <span class="setting-desc">横屏右下角 / 竖屏播放键下方</span>
        </div>
        <div class="setting-control">
          <label class="toggle">
            <input type="checkbox" v-model="settings.showBottomControl" />
            <span class="toggle-slider"></span>
          </label>
        </div>
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <label class="setting-label">进度条显示剩余时间</label>
        </div>
        <div class="setting-control">
          <label class="toggle">
            <input type="checkbox" v-model="settings.showRemainingTime" />
            <span class="toggle-slider"></span>
          </label>
        </div>
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <label class="setting-label">隐藏歌词视图</label>
          <span class="setting-desc">即使有歌词数据也不显示歌词</span>
        </div>
        <div class="setting-control">
          <label class="toggle">
            <input type="checkbox" v-model="settings.hideLyricView" />
            <span class="toggle-slider"></span>
          </label>
        </div>
      </div>
    </div>

    <!-- 布局 -->
    <div class="settings-section">
      <h3 class="block-title">
        <i class="bi bi-layout-text-window-reverse"></i>
        布局
      </h3>

      <div class="setting-item">
        <div class="setting-info">
          <label class="setting-label">封面布局（隐藏歌词时）</label>
          <span class="setting-desc">控制隐藏歌词时专辑图的布局方式</span>
        </div>
        <div class="setting-control">
          <select v-model="settings.verticalCoverLayout" class="select">
            <option value="auto">自动（根据视频封面）</option>
            <option value="force-normal">强制普通</option>
            <option value="force-immersive">强制沉浸</option>
          </select>
        </div>
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <label class="setting-label">底部控件类型</label>
          <span class="setting-desc">歌曲信息下方的组件类型</span>
        </div>
        <div class="setting-control">
          <select v-model="settings.playerControlsType" class="select">
            <option value="controls">播放控制按钮</option>
            <option value="fft">音频可视化</option>
            <option value="none">不显示</option>
          </select>
        </div>
      </div>
    </div>

    <!-- 重置 -->
    <div class="settings-section">
      <h3 class="block-title">
        <i class="bi bi-arrow-counterclockwise"></i>
        重置
      </h3>
      <div class="setting-item">
        <div class="setting-info">
          <label class="setting-label">恢复默认</label>
          <span class="setting-desc">清除所有 AMLL 配置，恢复出厂默认值</span>
        </div>
        <div class="setting-control">
          <button class="reset-btn" @click="resetToDefaults">恢复默认</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed, onMounted, useTemplateRef } from 'vue';
import { useAnime } from '@/composables/useAnime.js';
import { AmllSettingsStore, AMLL_DEFAULTS } from '@/stores/amllSettings.js';

// 直接绑定到 AmllSettingsStore.state，改动会自动通过 store.set 同步到 AMLL Jotai store
// 并由 AmllSettingsStore.bind 反向监听 atom 变化持久化到 localStorage
const settings = AmllSettingsStore.state;

// 鼓点频段：用 computed 把 [low, high] 数组拆成两个双向绑定的数字
// lowFreqLow 改 → 写回 settings.lowFreqVolumeRange[0]
// lowFreqHigh 改 → 写回 settings.lowFreqVolumeRange[1]
const lowFreqLow = computed({
  get: () => settings.lowFreqVolumeRange[0],
  set: (v) => {
    // 保证下限 < 上限
    const high = settings.lowFreqVolumeRange[1];
    if (v >= high) v = high - 5;
    settings.lowFreqVolumeRange = [Math.max(20, v), high];
  },
});
const lowFreqHigh = computed({
  get: () => settings.lowFreqVolumeRange[1],
  set: (v) => {
    const low = settings.lowFreqVolumeRange[0];
    if (v <= low) v = low + 5;
    settings.lowFreqVolumeRange = [low, Math.min(2000, v)];
  },
});

function resetToDefaults() {
  if (!confirm('确定要清除所有 AMLL 配置并恢复默认值吗？')) return;
  for (const key of Object.keys(AMLL_DEFAULTS)) {
    settings[key] = Array.isArray(AMLL_DEFAULTS[key])
      ? [...AMLL_DEFAULTS[key]]
      : AMLL_DEFAULTS[key];
  }
}

const rootRef = useTemplateRef('root');
const { run } = useAnime(() => rootRef.value);

onMounted(() => {
  run(({ animate, stagger, presets }) => {
    animate('.section-header', { ...presets.fadeIn });
    animate('.settings-section', { ...presets.fadeInUp, delay: stagger(80) });
  });
});
</script>

<style scoped>
.lyrics-view {
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

.setting-item.is-disabled {
  opacity: 0.45;
  pointer-events: none;
}

.setting-info {
  flex: 1;
  min-width: 0;
  padding-right: 16px;
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
  line-height: 1.4;
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
  min-width: 50px;
  text-align: right;
  font-variant-numeric: tabular-nums;
}

.slider {
  width: 140px;
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
  min-width: 130px;
  appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 12 12'%3E%3Cpath fill='%23666' d='M6 8L1 3h10z'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 10px center;
}

.text-input {
  padding: 7px 10px;
  font-size: 13px;
  border: 1px solid var(--border-color, #e8e8e8);
  border-radius: 8px;
  background: var(--bg-primary, #fff);
  color: var(--text-primary, #333);
  outline: none;
  min-width: 200px;
  font-family: inherit;
}

.text-input:focus {
  border-color: var(--primary-color, #0078d7);
}

.color-input {
  width: 36px;
  height: 28px;
  padding: 0;
  border: 1px solid var(--border-color, #e8e8e8);
  border-radius: 6px;
  background: transparent;
  cursor: pointer;
}

.color-input::-webkit-color-swatch-wrapper {
  padding: 2px;
}

.color-input::-webkit-color-swatch {
  border: none;
  border-radius: 4px;
}

.reset-btn {
  padding: 7px 14px;
  font-size: 13px;
  border: 1px solid var(--border-color, #e8e8e8);
  border-radius: 8px;
  background: var(--bg-primary, #fff);
  color: var(--text-primary, #333);
  cursor: pointer;
  transition: all 0.2s;
}

.reset-btn:hover {
  border-color: #e34a4a;
  color: #e34a4a;
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

  .text-input,
  .select {
    min-width: 0;
    flex: 1;
  }
}
</style>
