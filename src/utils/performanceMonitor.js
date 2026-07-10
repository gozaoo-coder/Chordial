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

  // ── GPU / 帧率 / 长任务 探针 ───────────────────────────────────────────

  /**
   * 包装一个 requestAnimationFrame 回调，测量单帧耗时（含合成）。
   * 仅 DEV 启用，自动剥离生产环境。
   * @param {string} label
   * @param {(deltaMs: number, frameTs: number) => void} fn - 回调，接收上一帧 delta
   * @returns {number} rafId
   */
  raf(label, fn) {
    if (!isEnabled() || typeof requestAnimationFrame !== 'function') {
      return requestAnimationFrame ? requestAnimationFrame(() => fn(16.7, performance.now())) : 0;
    }
    const start = performance.now();
    return requestAnimationFrame((ts) => {
      const delta = ts - start;
      const STYLE_FRAME = 'font-weight: bold; color: #f59e0b;';
      if (delta > 20) {
        // 超过 20ms 视为掉帧
        console.log(
          `%c🎬 %c[Perf]%c ${label}  %c${formatDuration(delta)} (frame)%c ⚠️ drop`,
          '', STYLE_LABEL, '', STYLE_FRAME, STYLE_META
        );
      } else if (delta > 8) {
        console.log(
          `%c🎬 %c[Perf]%c ${label}  %c${formatDuration(delta)}%c (frame)`,
          '', STYLE_LABEL, '', STYLE_FRAME, STYLE_META
        );
      }
      fn(delta, ts);
    });
  },

  /**
   * 启动一个 FPS 采样器，按 interval 节流输出统计。
   * 返回停止函数。用于监测持续动画区域（blur 背景、AMLL spring 等）。
   * @param {string} label
   * @param {number} [intervalMs=1000] - 输出统计周期
   * @returns {() => void} stop
   */
  startFpsMonitor(label, intervalMs = 1000) {
    if (!isEnabled() || typeof requestAnimationFrame !== 'function') {
      return () => {};
    }
    let frames = 0;
    let lastTs = performance.now();
    let periodStart = lastTs;
    let maxFrame = 0;
    let minFrame = Infinity;
    let stopped = false;
    const STYLE_FPS = 'font-weight: bold; color: #8b5cf6;';

    const tick = (ts) => {
      if (stopped) return;
      const delta = ts - lastTs;
      if (delta > 0) {
        frames++;
        if (delta > maxFrame) maxFrame = delta;
        if (delta < minFrame) minFrame = delta;
      }
      lastTs = ts;
      if (ts - periodStart >= intervalMs) {
        const avg = (ts - periodStart) / Math.max(frames, 1);
        const fps = (1000 / avg);
        console.log(
          `%c📊 %c[Perf]%c ${label}  %c${fps.toFixed(1)} fps%c avg=${formatDuration(avg)} min=${formatDuration(minFrame)} max=${formatDuration(maxFrame)} frames=${frames}`,
          '', STYLE_LABEL, '', STYLE_FPS, STYLE_META
        );
        frames = 0;
        maxFrame = 0;
        minFrame = Infinity;
        periodStart = ts;
      }
      requestAnimationFrame(tick);
    };
    requestAnimationFrame(tick);

    return () => { stopped = true; };
  },

  /**
   * 订阅 PerformanceObserver Long Task API。
   * 返回取消订阅函数。捕获 >50ms 的长任务（通常为 JS 阻塞或 GPU 合成阻塞）。
   * @param {string} [label='longtask']
   * @returns {() => void} unsubscribe
   */
  watchLongTasks(label = 'longtask') {
    if (!isEnabled()) return () => {};
    if (typeof PerformanceObserver === 'undefined') return () => {};
    let observer;
    try {
      observer = new PerformanceObserver((list) => {
        for (const entry of list.getEntries()) {
          console.log(
            `%c⚠️ %c[Perf]%c ${label}  %c${formatDuration(entry.duration)}%c name=${entry.name} startTime=${entry.startTime.toFixed(1)}`,
            '', STYLE_LABEL, '', STYLE_DURATION, STYLE_META
          );
        }
      });
      observer.observe({ entryTypes: ['longtask'] });
    } catch {
      return () => {};
    }
    return () => { try { observer.disconnect(); } catch {} };
  },

  /**
   * 订阅 paint timing（FP/FCP）和 LCP。
   * 返回取消订阅函数。
   * @returns {() => void} unsubscribe
   */
  watchPaint() {
    if (!isEnabled() || typeof PerformanceObserver === 'undefined') return () => {};
    const observers = [];
    try {
      const paintObs = new PerformanceObserver((list) => {
        for (const e of list.getEntries()) {
          console.log(`%c🎨 %c[Perf]%c ${e.name}  %c${formatDuration(e.startTime)}`, '', STYLE_LABEL, '', STYLE_DURATION, '');
        }
      });
      paintObs.observe({ entryTypes: ['paint'] });
      observers.push(paintObs);
    } catch {}
    try {
      const lcpObs = new PerformanceObserver((list) => {
        for (const e of list.getEntries()) {
          console.log(`%c🎯 %c[Perf]%c LCP  %c${formatDuration(e.startTime)}%c element=${e.element?.tagName ?? '?'}`, '', STYLE_LABEL, '', STYLE_DURATION, STYLE_META);
        }
      });
      lcpObs.observe({ entryTypes: ['largest-contentful-paint'] });
      observers.push(lcpObs);
    } catch {}
    try {
      const layoutObs = new PerformanceObserver((list) => {
        for (const e of list.getEntries()) {
          console.log(`%c📐 %c[Perf]%c layout-shift  %c${(e.value * 1000).toFixed(2)}‰%c sources=${e.sources?.length ?? 0}`, '', STYLE_LABEL, '', STYLE_DURATION, STYLE_META);
        }
      });
      layoutObs.observe({ entryTypes: ['layout-shift'] });
      observers.push(layoutObs);
    } catch {}
    return () => observers.forEach((o) => { try { o.disconnect(); } catch {} });
  },

  /**
   * 采样当前 GPU 合成层、内存占用（Chrome 限定，不可用时静默）。
   */
  snapshot() {
    if (!isEnabled()) return;
    const mem = (performance).memory;
    if (mem) {
      console.log(
        `%c💾 %c[Perf]%c js-heap  %c${(mem.usedJSHeapSize / 1048576).toFixed(1)}MB / ${(mem.totalJSHeapSize / 1048576).toFixed(1)}MB%c limit=${(mem.jsHeapSizeLimit / 1048576).toFixed(0)}MB`,
        '', STYLE_LABEL, '', STYLE_DURATION, STYLE_META
      );
    }
    if (hasPerformance && performance.getEntriesByType) {
      const marks = performance.getEntriesByType('mark').length;
      const measures = performance.getEntriesByType('measure').length;
      if (marks + measures > 200) {
        console.log(`%c🧹 %c[Perf]%c perf-entries  marks=${marks} measures=${measures}%c 建议清理`, '', STYLE_LABEL, '', STYLE_META);
      }
    }
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
    raf: (label, fn) => perf.raf(prefix(label), fn),
    startFpsMonitor: (label, intervalMs) => perf.startFpsMonitor(prefix(label), intervalMs),
  };
}

export default perf;
