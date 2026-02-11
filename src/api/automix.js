/**
 * Automix API 封装层
 * 封装 Tauri 后端命令，提供前端友好的接口
 */

import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

/**
 * 分析音频文件的 BPM 和节拍
 * @param {string} filePath - 音频文件路径
 * @returns {Promise<{bpm: number, beat_positions: number[], downbeat_position: number|null}>}
 */
export async function analyzeAudioBeat(filePath) {
  return await invoke('analyze_audio_beat', { file_path: filePath });
}

/**
 * 强制重新分析音频（忽略缓存）
 * @param {string} filePath - 音频文件路径
 * @returns {Promise<{bpm: number, beat_positions: number[], downbeat_position: number|null}>}
 */
export async function reanalyzeAudioBeat(filePath) {
  return await invoke('reanalyze_audio_beat', { file_path: filePath });
}

/**
 * 获取音频的混音点建议
 * @param {string} filePath - 音频文件路径
 * @returns {Promise<{bpm: number, mix_in_point: number|null, mix_out_point: number|null, duration: number}>}
 */
export async function getMixPoints(filePath) {
  return await invoke('get_mix_points', { file_path: filePath });
}

/**
 * 批量分析音频文件
 * @param {string[]} filePaths - 音频文件路径数组
 * @returns {Promise<Array<{file_path: string, success: boolean, bpm?: number, beat_count?: number, downbeat?: number, error?: string}>>}
 */
export async function batchAnalyzeAudio(filePaths) {
  return await invoke('batch_analyze_audio', { file_paths: filePaths });
}

/**
 * 获取分析缓存统计
 * @returns {Promise<{entry_count: number, total_data_size: number}>}
 */
export async function getAnalysisCacheStats() {
  return await invoke('get_analysis_cache_stats');
}

/**
 * 清空分析缓存
 * @returns {Promise<void>}
 */
export async function clearAnalysisCache() {
  return await invoke('clear_analysis_cache');
}

/**
 * 设置交叉淡化启用状态
 * @param {boolean} enabled - 是否启用
 * @returns {Promise<void>}
 */
export async function setCrossfadeEnabled(enabled) {
  return await invoke('set_crossfade_enabled', { enabled });
}

/**
 * 检查交叉淡化是否启用
 * @returns {Promise<boolean>}
 */
export async function isCrossfadeEnabled() {
  return await invoke('is_crossfade_enabled');
}

/**
 * 设置交叉淡化配置
 * @param {number} durationSecs - 淡化时长（秒）
 * @param {'linear'|'logarithmic'|'s_curve'} curveType - 曲线类型
 * @returns {Promise<void>}
 */
export async function setCrossfadeConfig(durationSecs, curveType = 's_curve') {
  return await invoke('set_crossfade_config', {
    duration_secs: durationSecs,
    curve_type: curveType
  });
}

/**
 * 预加载下一首音频
 * @param {string} filePath - 音频文件路径
 * @returns {Promise<void>}
 */
export async function preloadNextAudio(filePath) {
  return await invoke('preload_next_audio', { file_path: filePath });
}

/**
 * 获取下一首音频路径
 * @returns {Promise<string|null>}
 */
export async function getNextAudioPath() {
  return await invoke('get_next_audio_path');
}

/**
 * 设置当前播放音频的 BPM 信息
 * @param {number} bpm - BPM 值
 * @param {number[]} beatPositions - Beat 位置数组（秒）
 * @returns {Promise<void>}
 */
export async function setCurrentTrackBpm(bpm, beatPositions) {
  return await invoke('set_current_track_bpm', {
    bpm,
    beat_positions: beatPositions
  });
}

/**
 * 设置下一首音频的 BPM 信息
 * @param {number} bpm - BPM 值
 * @param {number[]} beatPositions - Beat 位置数组（秒）
 * @returns {Promise<void>}
 */
export async function setNextTrackBpm(bpm, beatPositions) {
  return await invoke('set_next_track_bpm', {
    bpm,
    beat_positions: beatPositions
  });
}

/**
 * 启用/禁用 BPM 同步
 * @param {boolean} enabled - 是否启用
 * @returns {Promise<void>}
 */
export async function setBpmSyncEnabled(enabled) {
  return await invoke('set_bpm_sync_enabled', { enabled });
}

/**
 * 检查 BPM 同步是否启用
 * @returns {Promise<boolean>}
 */
export async function isBpmSyncEnabled() {
  return await invoke('is_bpm_sync_enabled');
}

/**
 * 获取当前播放速度比率
 * @returns {Promise<number>}
 */
export async function getPlaybackSpeedRatio() {
  return await invoke('get_playback_speed_ratio');
}

/**
 * 设置播放速度
 * @param {number} speedRatio - 速度比率 (0.5 - 2.0)
 * @returns {Promise<void>}
 */
export async function setPlaybackSpeed(speedRatio) {
  return await invoke('set_playback_speed', { speed_ratio: speedRatio });
}

/**
 * 监听分析进度事件
 * @param {Function} callback - 回调函数，接收 {current, total, percent, file} 参数
 * @returns {Promise<() => void>} 取消监听函数
 */
export async function listenAnalysisProgress(callback) {
  const unlisten = await listen('analysis_progress', (event) => {
    callback(event.payload);
  });
  return unlisten;
}

/**
 * Automix 配置对象
 */
export const AutomixConfig = {
  // 交叉淡化曲线类型
  CurveType: {
    LINEAR: 'linear',
    LOGARITHMIC: 'logarithmic',
    S_CURVE: 's_curve'
  },

  // 默认配置
  defaults: {
    crossfadeDuration: 10, // 秒
    crossfadeCurve: 's_curve',
    bpmSyncEnabled: false,
    playbackSpeed: 1.0
  }
};

/**
 * 分析结果缓存管理
 */
export const AnalysisCache = {
  /**
   * 获取缓存统计
   */
  async getStats() {
    return await getAnalysisCacheStats();
  },

  /**
   * 清空缓存
   */
  async clear() {
    return await clearAnalysisCache();
  },

  /**
   * 格式化缓存大小
   * @param {number} bytes - 字节数
   * @returns {string} 格式化后的字符串
   */
  formatSize(bytes) {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(2)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
  }
};

export default {
  analyzeAudioBeat,
  reanalyzeAudioBeat,
  getMixPoints,
  batchAnalyzeAudio,
  getAnalysisCacheStats,
  clearAnalysisCache,
  setCrossfadeEnabled,
  isCrossfadeEnabled,
  setCrossfadeConfig,
  preloadNextAudio,
  getNextAudioPath,
  setCurrentTrackBpm,
  setNextTrackBpm,
  setBpmSyncEnabled,
  isBpmSyncEnabled,
  getPlaybackSpeedRatio,
  setPlaybackSpeed,
  listenAnalysisProgress,
  AutomixConfig,
  AnalysisCache
};
