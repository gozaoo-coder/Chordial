<template>
  <div class="automix-controls">
    <!-- Automix 主开关 -->
    <div class="automix-main-control">
      <button
        class="automix-toggle"
        :class="{ active: automixEnabled }"
        @click="toggleAutomix"
        :title="automixEnabled ? '关闭智能混音' : '开启智能混音'"
      >
        <svg viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 3v10.55c-.59-.34-1.27-.55-2-.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z"/>
        </svg>
        <span class="toggle-label">AutoMix</span>
        <span class="toggle-status">{{ automixEnabled ? '开启' : '关闭' }}</span>
      </button>
    </div>

    <!-- Automix 详细控制（仅在开启时显示） -->
    <div v-if="automixEnabled" class="automix-details">
      <!-- BPM 信息显示 -->
      <div class="bpm-info">
        <div class="bpm-display">
          <span class="bpm-label">BPM</span>
          <span class="bpm-value">{{ formattedBpm }}</span>
        </div>
        <div v-if="isAnalyzing" class="analysis-status">
          <span class="analyzing-indicator">分析中...</span>
        </div>
      </div>

      <!-- 交叉淡化控制 -->
      <div class="crossfade-control">
        <div class="control-header">
          <span class="control-label">交叉淡化</span>
          <span class="control-value">{{ crossfadeDuration }}s</span>
        </div>
        <input
          type="range"
          class="slider"
          min="1"
          max="30"
          step="1"
          :value="crossfadeDuration"
          @input="onCrossfadeDurationChange"
        />
        <div class="curve-selector">
          <button
            v-for="curve in curveOptions"
            :key="curve.value"
            class="curve-btn"
            :class="{ active: crossfadeCurve === curve.value }"
            @click="setCrossfadeCurve(curve.value)"
          >
            {{ curve.label }}
          </button>
        </div>
      </div>

      <!-- BPM 同步控制 -->
      <div class="bpm-sync-control">
        <button
          class="sync-toggle"
          :class="{ active: bpmSyncEnabled }"
          @click="toggleBpmSync"
        >
          <svg viewBox="0 0 24 24" fill="currentColor">
            <path d="M12 4V1L8 5l4 4V6c3.31 0 6 2.69 6 6 0 1.01-.25 1.97-.7 2.8l1.46 1.46C19.54 15.03 20 13.57 20 12c0-4.42-3.58-8-8-8zm0 14c-3.31 0-6-2.69-6-6 0-1.01.25-1.97.7-2.8L5.24 7.74C4.46 8.97 4 10.43 4 12c0 4.42 3.58 8 8 8v3l4-4-4-4v3z"/>
          </svg>
          <span>BPM 同步</span>
          <span class="sync-status">{{ bpmSyncEnabled ? '开启' : '关闭' }}</span>
        </button>
        <div v-if="bpmSyncEnabled" class="speed-display">
          速度: {{ formattedSpeed }}
        </div>
      </div>

      <!-- 分析进度条 -->
      <div v-if="isAnalyzing && analysisProgress > 0" class="analysis-progress">
        <div class="progress-bar">
          <div class="progress-fill" :style="{ width: `${analysisProgress}%` }"></div>
        </div>
        <span class="progress-text">{{ Math.round(analysisProgress) }}%</span>
      </div>
    </div>
  </div>
</template>

<script>
import { computed } from 'vue';
import PlayerStore from '@/stores/player.js';

export default {
  name: 'AutomixControls',
  setup() {
    // 从 PlayerStore 获取状态
    const automixEnabled = computed(() => PlayerStore.state.automixEnabled);
    const crossfadeEnabled = computed(() => PlayerStore.state.crossfadeEnabled);
    const crossfadeDuration = computed(() => PlayerStore.state.crossfadeDuration);
    const crossfadeCurve = computed(() => PlayerStore.state.crossfadeCurve);
    const bpmSyncEnabled = computed(() => PlayerStore.state.bpmSyncEnabled);
    const playbackSpeed = computed(() => PlayerStore.state.playbackSpeed);
    const currentBpm = computed(() => PlayerStore.state.currentBpm);
    const isAnalyzing = computed(() => PlayerStore.state.isAnalyzing);
    const analysisProgress = computed(() => PlayerStore.state.analysisProgress);

    // 计算属性
    const formattedBpm = computed(() => {
      if (currentBpm.value) {
        return Math.round(currentBpm.value).toString();
      }
      return '--';
    });

    const formattedSpeed = computed(() => {
      return `${Math.round(playbackSpeed.value * 100)}%`;
    });

    // 曲线选项
    const curveOptions = [
      { value: 'linear', label: '线性' },
      { value: 'logarithmic', label: '对数' },
      { value: 's_curve', label: 'S型' }
    ];

    // 方法
    const toggleAutomix = async () => {
      await PlayerStore.toggleAutomix();
    };

    const onCrossfadeDurationChange = (e) => {
      const duration = parseInt(e.target.value);
      PlayerStore.setCrossfade(crossfadeEnabled.value, duration, crossfadeCurve.value);
    };

    const setCrossfadeCurve = (curve) => {
      PlayerStore.setCrossfade(crossfadeEnabled.value, crossfadeDuration.value, curve);
    };

    const toggleBpmSync = async () => {
      await PlayerStore.setBpmSync(!bpmSyncEnabled.value);
    };

    return {
      // 状态
      automixEnabled,
      crossfadeEnabled,
      crossfadeDuration,
      crossfadeCurve,
      bpmSyncEnabled,
      playbackSpeed,
      currentBpm,
      isAnalyzing,
      analysisProgress,
      
      // 计算属性
      formattedBpm,
      formattedSpeed,
      curveOptions,
      
      // 方法
      toggleAutomix,
      onCrossfadeDurationChange,
      setCrossfadeCurve,
      toggleBpmSync
    };
  }
};
</script>

<style scoped>
.automix-controls {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 12px;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 12px;
  backdrop-filter: blur(10px);
}

/* Automix 主开关 */
.automix-main-control {
  display: flex;
  justify-content: center;
}

.automix-toggle {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 20px;
  border: none;
  border-radius: 24px;
  background: rgba(255, 255, 255, 0.1);
  color: white;
  cursor: pointer;
  transition: all 0.3s ease;
}

.automix-toggle:hover {
  background: rgba(255, 255, 255, 0.15);
}

.automix-toggle.active {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  box-shadow: 0 4px 15px rgba(102, 126, 234, 0.4);
}

.automix-toggle svg {
  width: 20px;
  height: 20px;
}

.toggle-label {
  font-weight: 600;
  font-size: 14px;
}

.toggle-status {
  font-size: 12px;
  opacity: 0.8;
  padding: 2px 8px;
  background: rgba(255, 255, 255, 0.2);
  border-radius: 10px;
}

/* Automix 详细控制 */
.automix-details {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding-top: 12px;
  border-top: 1px solid rgba(255, 255, 255, 0.1);
}

/* BPM 信息 */
.bpm-info {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 8px;
}

.bpm-display {
  display: flex;
  align-items: baseline;
  gap: 6px;
}

.bpm-label {
  font-size: 12px;
  opacity: 0.7;
}

.bpm-value {
  font-size: 20px;
  font-weight: 700;
  color: #667eea;
}

.analysis-status {
  font-size: 12px;
  color: #f39c12;
}

.analyzing-indicator {
  animation: pulse 1.5s infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

/* 交叉淡化控制 */
.crossfade-control {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.control-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 13px;
}

.control-label {
  opacity: 0.9;
}

.control-value {
  font-weight: 600;
  color: #667eea;
}

.slider {
  width: 100%;
  height: 4px;
  border-radius: 2px;
  background: rgba(255, 255, 255, 0.1);
  outline: none;
  -webkit-appearance: none;
}

.slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: #667eea;
  cursor: pointer;
  transition: all 0.2s ease;
}

.slider::-webkit-slider-thumb:hover {
  transform: scale(1.2);
  box-shadow: 0 2px 8px rgba(102, 126, 234, 0.5);
}

.curve-selector {
  display: flex;
  gap: 6px;
}

.curve-btn {
  flex: 1;
  padding: 6px 8px;
  border: none;
  border-radius: 6px;
  background: rgba(255, 255, 255, 0.1);
  color: white;
  font-size: 11px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.curve-btn:hover {
  background: rgba(255, 255, 255, 0.15);
}

.curve-btn.active {
  background: #667eea;
}

/* BPM 同步控制 */
.bpm-sync-control {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.sync-toggle {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 14px;
  border: none;
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.1);
  color: white;
  cursor: pointer;
  transition: all 0.2s ease;
}

.sync-toggle:hover {
  background: rgba(255, 255, 255, 0.15);
}

.sync-toggle.active {
  background: rgba(102, 126, 234, 0.3);
  border: 1px solid #667eea;
}

.sync-toggle svg {
  width: 18px;
  height: 18px;
}

.sync-toggle span {
  flex: 1;
  text-align: left;
  font-size: 13px;
}

.sync-status {
  font-size: 11px !important;
  opacity: 0.8;
  text-align: right !important;
}

.speed-display {
  font-size: 12px;
  color: #667eea;
  text-align: center;
  padding: 4px;
  background: rgba(102, 126, 234, 0.1);
  border-radius: 4px;
}

/* 分析进度条 */
.analysis-progress {
  display: flex;
  align-items: center;
  gap: 8px;
}

.progress-bar {
  flex: 1;
  height: 4px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 2px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, #667eea 0%, #764ba2 100%);
  border-radius: 2px;
  transition: width 0.3s ease;
}

.progress-text {
  font-size: 11px;
  color: #667eea;
  min-width: 36px;
  text-align: right;
}

/* 响应式 */
@media (max-width: 767px) {
  .automix-controls {
    padding: 10px;
    gap: 10px;
  }

  .automix-toggle {
    padding: 8px 16px;
  }

  .toggle-label {
    font-size: 13px;
  }

  .bpm-value {
    font-size: 18px;
  }

  .curve-btn {
    font-size: 10px;
    padding: 5px 6px;
  }
}
</style>
