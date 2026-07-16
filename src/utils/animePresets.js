/**
 * anime.js v4 项目级预设
 *
 * 与 src/app.css 的 --transition-* 变量保持一一对应：
 *   --transition-fast:   0.15s cubic-bezier(0.4, 0, 0.2, 1)   → ANIME_DURATIONS.fast    + ANIME_EASINGS.standard
 *   --transition-normal: 0.25s cubic-bezier(0.4, 0, 0.2, 1)   → ANIME_DURATIONS.normal  + ANIME_EASINGS.standard
 *   --transition-slow:   0.35s cubic-bezier(0.4, 0, 0.2, 1)   → ANIME_DURATIONS.slow    + ANIME_EASINGS.standard
 *   --transition-spring: 0.4s  cubic-bezier(0.34, 1.56, 0.64, 1) → ANIME_DURATIONS.spring + ANIME_EASINGS.spring
 *
 * 修改本文件时请同步修改 app.css，反之亦然。
 */
import { cubicBezier, stagger } from 'animejs';

/** 与 --transition-* 对齐的 easing 函数 */
export const ANIME_EASINGS = {
  // cubic-bezier(0.4, 0, 0.2, 1) — 标准 Material/Apple 过渡
  standard: cubicBezier(0.4, 0, 0.2, 1),
  // cubic-bezier(0.34, 1.56, 0.64, 1) — 弹性回弹
  spring: cubicBezier(0.34, 1.56, 0.64, 1),
  // Apple 风格入场（easeOutExpo-ish，减速明显）
  entrance: cubicBezier(0.16, 1, 0.3, 1),
  // Apple 风格退场（easeInExpo-ish，加速明显）
  exit: cubicBezier(0.7, 0, 0.84, 0),
  // 内置字符串 easing 的别名（anime.js v4 原生支持）
  linear: 'linear',
  easeOutQuad: 'easeOutQuad',
  easeInQuad: 'easeInQuad',
  easeInOutQuad: 'easeInOutQuad',
};

/** 与 --transition-* 对齐的 duration（毫秒） */
export const ANIME_DURATIONS = {
  fast: 150,
  normal: 250,
  slow: 350,
  spring: 400,
};

/**
 * 入场/退场预设。每个预设可直接展开进 animate() 的参数对象。
 * 例：animate(el, { ...ANIME_PRESETS.fadeInUp })
 */
export const ANIME_PRESETS = {
  // 基础淡入
  fadeIn: {
    opacity: [0, 1],
    duration: ANIME_DURATIONS.normal,
    easing: ANIME_EASINGS.standard,
  },
  // 向上淡入（列表项、卡片入场常用）
  fadeInUp: {
    opacity: [0, 1],
    translateY: [16, 0],
    duration: ANIME_DURATIONS.normal,
    easing: ANIME_EASINGS.standard,
  },
  // 向下淡入（下拉菜单、通知）
  fadeInDown: {
    opacity: [0, 1],
    translateY: [-16, 0],
    duration: ANIME_DURATIONS.normal,
    easing: ANIME_EASINGS.standard,
  },
  // 缩放淡入（弹窗、对话框）
  scaleIn: {
    opacity: [0, 1],
    scale: [0.92, 1],
    duration: ANIME_DURATIONS.slow,
    easing: ANIME_EASINGS.spring,
  },
  // 弹性弹入（按钮反馈、强调元素）
  popIn: {
    scale: [0.6, 1],
    opacity: [0, 1],
    duration: ANIME_DURATIONS.spring,
    easing: ANIME_EASINGS.spring,
  },
  // 从右滑入（抽屉、侧边面板）
  slideInRight: {
    opacity: [0, 1],
    translateX: [40, 0],
    duration: ANIME_DURATIONS.slow,
    easing: ANIME_EASINGS.standard,
  },
  // 从左滑入
  slideInLeft: {
    opacity: [0, 1],
    translateX: [-40, 0],
    duration: ANIME_DURATIONS.slow,
    easing: ANIME_EASINGS.standard,
  },
  // 列表项入场（配合 stagger 使用）
  listItemEnter: {
    opacity: [0, 1],
    translateY: [12, 0],
    duration: 300,
    easing: ANIME_EASINGS.easeOutQuad,
  },
  // 退场预设
  fadeOut: {
    opacity: [1, 0],
    duration: ANIME_DURATIONS.fast,
    easing: ANIME_EASINGS.standard,
  },
  fadeOutUp: {
    opacity: [1, 0],
    translateY: [0, -16],
    duration: ANIME_DURATIONS.fast,
    easing: ANIME_EASINGS.standard,
  },
  fadeOutDown: {
    opacity: [1, 0],
    translateY: [0, 16],
    duration: ANIME_DURATIONS.fast,
    easing: ANIME_EASINGS.standard,
  },
  scaleOut: {
    opacity: [1, 0],
    scale: [1, 0.92],
    duration: ANIME_DURATIONS.fast,
    easing: ANIME_EASINGS.standard,
  },
};

/**
 * stagger 工厂函数。配合 animate() 的 delay 参数使用。
 * 例：animate('.item', { ...ANIME_PRESETS.listItemEnter, delay: ANIME_STAGGER.normal() })
 */
export const ANIME_STAGGER = {
  // 快速错峰（小列表、紧凑网格）
  fast: (from = 'first') => stagger(40, { from }),
  // 标准错峰（默认推荐）
  normal: (from = 'first') => stagger(70, { from }),
  // 从中心扩散（网格布局）
  grid: (cols, from = 'center') =>
    stagger([50, 25], { grid: [cols, Infinity], from }),
};

/**
 * 无限循环动画预设（loading spinner、ambient 装饰）。
 * 配合 loop: true 使用。
 */
export const ANIME_LOOP = {
  // 旋转（spinner）
  spin: {
    rotate: '1turn',
    duration: 800,
    easing: ANIME_EASINGS.linear,
    loop: true,
  },
  // 脉冲（呼吸效果）
  pulse: {
    opacity: [1, 0.5],
    duration: 1000,
    easing: ANIME_EASINGS.easeInOutQuad,
    alternate: true,
    loop: true,
  },
};
