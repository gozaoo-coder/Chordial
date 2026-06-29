/**
 * performanceMonitor.js — 前端性能监控工具
 *
 * 基于 Performance API (performance.mark / performance.measure / performance.now)，
 * 为页面加载、数据请求、用户交互等操作提供统一的性能打点与日志输出。
 *
 * 启用/禁用：
 *   - 开发环境 (import.meta.env.DEV) 默认启用
 *   - 生产环境可通过 localStorage.setItem('chordial_perf_monitor', '1') 手动开启
 *   - localStorage.setItem('chordial_perf_monitor', '0') 可强制关闭
 *
 * 日志格式：
 *   ⏱ [Perf] <label>  duration_ms  { ...metadata }
 *   📍 [Perf] <label>  { ...metadata }
 *
 * 用法示例：
 *   import { perf } from '@/utils/performanceMonitor.js';
 *
 *   // 手动计时
 *   perf.start('loadHomeData');
 *   await library.getCached();
 *   perf.end('loadHomeData', { tracks: 1234 });
 *
 *   // 包装异步操作
 *   const data = await perf.measureAsync('fetchTracks', library.getCached());
 *
 *   // 单次事件日志
 *   perf.log('playTrack', { trackId: 'xxx', source: 'click' });
 */

// ── 是否启用 ──────────────────────────────────────────────────────────────────

function isEnabled() {
  const stored = localStorage.getItem('chordial_perf_monitor');
  if (stored !== null) {
    return stored === '1';
  }
  // 开发环境默认开启
  try {
    if (import.meta.env.DEV) return true;
  } catch {
    // import.meta 不可用时静默
  }
  return false;
}

// ── 检查 Performance API 可用性 ───────────────────────────────────────────────

const hasPerformance =
  typeof performance !== 'undefined' &&
  typeof performance.mark === 'function' &&
  typeof performance.measure === 'function' &&
  typeof performance.now === 'function';

// ── 计时器存储 ────────────────────────────────────────────────────────────────

/** @type {Map<string, number>} label → performance.now() 起始时间 */
const timers = new Map();

/** 用于生成唯一 ID 的计数器 */
let idCounter = 0;

// ── 格式化输出 ────────────────────────────────────────────────────────────────

const STYLE_LABEL = 'font-weight: bold; color: #0ea5e9;';
const STYLE_DURATION = 'font-weight: bold; color: #10b981;';
const STYLE_META = 'color: #6b7280;';

/**
 * 格式化持续时间
 * @param {number} ms
 * @returns {string}
 */
function formatDuration(ms) {
  if (ms < 1) return (ms * 1000).toFixed(1) + 'μs';
  if (ms < 1000) return ms.toFixed(1) + 'ms';
  return (ms / 1000).toFixed(2) + 's';
}

// ── 核心 API ──────────────────────────────────────────────────────────────────

export const perf = {
  /**
   * 开始计时
   * @param {string} label - 计时标签（同一标签可多次使用，内部自动去重）
   * @returns {string} 去重后的标签名（可用于 end）
   */
  start(label) {
    if (!isEnabled() || !hasPerformance) return label;

    // 如果标签已存在，生成唯一后缀
    const uniqueLabel = timers.has(label) ? `${label}#${++idCounter}` : label;
    const markName = `${uniqueLabel}_start`;

    try {
      performance.mark(markName);
    } catch {
      // mark 失败时静默降级
    }

    timers.set(uniqueLabel, performance.now());
    return uniqueLabel;
  },

  /**
   * 结束计时并输出日志
   * @param {string} label - 与 start() 返回值一致的标签
   * @param {object} [meta] - 附加元数据
   * @returns {number} 持续时间（毫秒），-1 表示未启用
   */
  end(label, meta) {
    const duration = performance.now() - (timers.get(label) ?? 0);
    timers.delete(label);

    if (!isEnabled() || !hasPerformance) return duration;

    const markName = `${label}_end`;
    try {
      performance.mark(markName);
      performance.measure(label, `${label}_start`, markName);

      // 清理 marks（避免内存泄漏）
      performance.clearMarks(`${label}_start`);
      performance.clearMarks(markName);
      performance.clearMeasures(label);
    } catch {
      // 清理失败不阻塞
    }

    const metaStr = meta ? ' ' + JSON.stringify(meta) : '';
    console.log(
      `%c⏱ %c[Perf]%c ${label}  %c${formatDuration(duration)}%c${metaStr}`,
      '', STYLE_LABEL, '', STYLE_DURATION, STYLE_META
    );

    return duration;
  },

  /**
   * 包装一个异步操作并自动计时
   * @template T
   * @param {string} label - 计时标签
   * @param {Promise<T>} promise - 待计时的 Promise
   * @param {object} [meta] - 附加元数据（在 Promise 完成后合并 duration）
   * @returns {Promise<T>}
   */
  async measureAsync(label, promise, meta) {
    const uniqueLabel = this.start(label);
    try {
      const result = await promise;
      this.end(uniqueLabel, meta);
      return result;
    } catch (error) {
      this.end(uniqueLabel, { ...meta, error: error?.message ?? String(error) });
      throw error;
    }
  },

  /**
   * 记录一个点事件（无持续时间）
   * @param {string} label - 事件标签
   * @param {object} [meta] - 附加元数据
   */
  log(label, meta) {
    if (!isEnabled()) return;

    const metaStr = meta ? ' ' + JSON.stringify(meta) : '';
    console.log(
      `%c📍 %c[Perf]%c ${label}%c${metaStr}`,
      '', STYLE_LABEL, '', STYLE_META
    );
  },

  /**
   * 创建一个分组，分组内的日志在控制台折叠展示
   * @param {string} label
   * @param {() => void | Promise<void>} fn
   */
  async group(label, fn) {
    if (!isEnabled()) {
      await fn();
      return;
    }
    const uniqueLabel = this.start(label);
    console.groupCollapsed(`%c⏱ %c[Perf]%c ${label}`, '', STYLE_LABEL, '');
    try {
      await fn();
    } finally {
      console.groupEnd();
      this.end(uniqueLabel);
    }
  },

  /** 检查监控是否已启用 */
  get enabled() {
    return isEnabled();
  },
};

/**
 * Vue composable：为组件提供便捷的计时方法
 * @param {string} componentName - 组件/页面名称（用作标签前缀）
 * @returns {{ start: function, end: function, log: function, measureAsync: function, group: function }}
 */
export function usePerf(componentName) {
  const prefix = (label) => `${componentName}.${label}`;

  return {
    start: (label) => perf.start(prefix(label)),
    end: (label, meta) => perf.end(prefix(label), meta),
    log: (label, meta) => perf.log(prefix(label), meta),
    measureAsync: (label, promise, meta) => perf.measureAsync(prefix(label), promise, meta),
    group: (label, fn) => perf.group(prefix(label), fn),
  };
}

export default perf;
