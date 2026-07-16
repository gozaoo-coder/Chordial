/**
 * anime.js v4 项目级预设 — 全 spring 物理 easing
 *
 * 设计理念：
 *  全面采用 anime.js v4 Spring 物理 easing（SwiftUI 风格 bounce+duration API），
 *  不再使用 cubicBezier / 字符串 easing。
 *
 * 4 种 spring profile：
 *  - 默认弹簧 (default)  : bounce 0,    duration 500  — 平滑无回弹，通用过渡
 *  - 灵敏弹簧 (sensitive) : bounce 0,    duration 280  — 快速响应，按钮反馈/小控件
 *  - 弹跳弹簧 (bouncy)    : bounce 0.65, duration 650  — 可见回弹，强调入场/弹窗
 *  - 强力弹簧 (powerful)  : bounce 0.15, duration 850  — 强力到位，大位移/模态展开
 *
 * 与 src/app.css 的关系：
 *  CSS 变量 --transition-* 仍保留为 cubicBezier（CSS 不支持物理 spring），
 *  但仅用于纯 CSS 上下文（如 :hover 颜色变化等无法 JS 化的微小过渡）。
 *  所有由 JS/anime.js 驱动的动画必须使用本文件的 spring 预设。
 */
import { spring, stagger } from 'animejs';

/**
 * 4 种 spring profile 工厂。
 * 用法：
 *   import { ANIME_SPRINGS } from '@/utils/animePresets.js';
 *   animate(el, { opacity: [0, 1], ease: ANIME_SPRINGS.default });
 *   // 或通过 useAnime().spring('default')
 */
export const ANIME_SPRINGS = {
  // 默认弹簧 — 平滑无回弹
  default:   spring({ bounce: 0,    duration: 500 }),
  // 灵敏弹簧 — 快速响应
  sensitive: spring({ bounce: 0,    duration: 280 }),
  // 弹跳弹簧 — 可见回弹
  bouncy:    spring({ bounce: 0.65, duration: 650 }),
  // 强力弹簧 — 强力到位
  powerful:  spring({ bounce: 0.15, duration: 850 }),
};

/**
 * 按名称获取 spring。便于 BottomSheet 等组件用字符串 API：
 *   spring('bouncy') → ANIME_SPRINGS.bouncy
 */
export function getSpring(name = 'default') {
  return ANIME_SPRINGS[name] || ANIME_SPRINGS.default;
}

/**
 * 与 --transition-* 对齐的 duration（毫秒）。
 * spring 模式下 duration 由 spring solver 计算，这里仅作为非 spring 备用与 stagger 间隔。
 */
export const ANIME_DURATIONS = {
  fast: 150,
  normal: 280,
  slow: 450,
  spring: 600,
};

/**
 * 入场/退场预设。每个预设可直接展开进 animate() 的参数对象。
 * 例：animate(el, { ...ANIME_PRESETS.fadeInUp })
 *
 * 所有 ease 字段使用 ANIME_SPRINGS。
 */
export const ANIME_PRESETS = {
  // 基础淡入
  fadeIn: {
    opacity: [0, 1],
    duration: ANIME_DURATIONS.normal,
    ease: ANIME_SPRINGS.default,
  },
  // 向上淡入（列表项、卡片入场常用）
  fadeInUp: {
    opacity: [0, 1],
    translateY: [16, 0],
    duration: ANIME_DURATIONS.slow,
    ease: ANIME_SPRINGS.bouncy,
  },
  // 向下淡入（下拉菜单、通知）
  fadeInDown: {
    opacity: [0, 1],
    translateY: [-16, 0],
    duration: ANIME_DURATIONS.slow,
    ease: ANIME_SPRINGS.bouncy,
  },
  // 缩放淡入（弹窗、对话框）
  scaleIn: {
    opacity: [0, 1],
    scale: [0.92, 1],
    duration: ANIME_DURATIONS.spring,
    ease: ANIME_SPRINGS.bouncy,
  },
  // 弹性弹入（按钮反馈、强调元素）
  popIn: {
    scale: [0.6, 1],
    opacity: [0, 1],
    duration: ANIME_DURATIONS.spring,
    ease: ANIME_SPRINGS.bouncy,
  },
  // 从右滑入（抽屉、侧边面板）
  slideInRight: {
    opacity: [0, 1],
    translateX: [40, 0],
    duration: ANIME_DURATIONS.slow,
    ease: ANIME_SPRINGS.default,
  },
  // 从左滑入
  slideInLeft: {
    opacity: [0, 1],
    translateX: [-40, 0],
    duration: ANIME_DURATIONS.slow,
    ease: ANIME_SPRINGS.default,
  },
  // 列表项入场（配合 stagger 使用）
  listItemEnter: {
    opacity: [0, 1],
    translateY: [12, 0],
    duration: ANIME_DURATIONS.slow,
    ease: ANIME_SPRINGS.default,
  },
  // 退场预设
  fadeOut: {
    opacity: [1, 0],
    duration: ANIME_DURATIONS.fast,
    ease: ANIME_SPRINGS.sensitive,
  },
  fadeOutUp: {
    opacity: [1, 0],
    translateY: [0, -16],
    duration: ANIME_DURATIONS.fast,
    ease: ANIME_SPRINGS.sensitive,
  },
  fadeOutDown: {
    opacity: [1, 0],
    translateY: [0, 16],
    duration: ANIME_DURATIONS.fast,
    ease: ANIME_SPRINGS.sensitive,
  },
  scaleOut: {
    opacity: [1, 0],
    scale: [1, 0.92],
    duration: ANIME_DURATIONS.fast,
    ease: ANIME_SPRINGS.sensitive,
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
    ease: 'linear', // 旋转必须 linear，物理 spring 不适合循环
    loop: true,
  },
  // 脉冲（呼吸效果）
  pulse: {
    opacity: [1, 0.5],
    duration: 1000,
    ease: 'inOutQuad', // 呼吸用对称 ease，不用 spring
    alternate: true,
    loop: true,
  },
};

/**
 * createLayout 预设。用于 FLIP 布局动画（列表项增删/排序、模态 display 切换、
 * 父元素交换等场景）。配合 useAnime().useLayout(root, preset) 使用。
 *
 * 字段说明：
 *  - enterFrom: 新元素入场起点（opacity/scale/translateY...）
 *  - leaveTo:   旧元素退场终点
 *  - swapAt:    父元素交换时同时存在的两元素动画
 *  - duration/ease/delay: 全局默认时长与缓动
 *
 * 注意：与 CSS content-visibility: auto / 虚拟列表 transform 定位冲突，仅用于
 * 普通 DOM 列表（如 P2pMatchDialog 的请求列表）。
 */
export const ANIME_LAYOUT = {
  // 模态对话框 display 切换：进入缩放淡入，离开缩放淡出
  modal: {
    duration: ANIME_DURATIONS.spring,
    ease: ANIME_SPRINGS.bouncy,
    enterFrom: {
      opacity: [0, 1],
      scale: [0.92, 1],
      duration: ANIME_DURATIONS.spring,
      ease: ANIME_SPRINGS.bouncy,
    },
    leaveTo: {
      opacity: [1, 0],
      scale: [1, 0.92],
      duration: ANIME_DURATIONS.fast,
      ease: ANIME_SPRINGS.sensitive,
    },
  },
  // 列表项增删 FLIP：新项从透明缩放进入，旧项淡出缩小
  list: {
    duration: ANIME_DURATIONS.normal,
    ease: ANIME_SPRINGS.default,
    enterFrom: {
      opacity: [0, 1],
      scale: [0.85, 1],
      duration: ANIME_DURATIONS.normal,
      ease: ANIME_SPRINGS.default,
    },
    leaveTo: {
      opacity: [1, 0],
      scale: [1, 0.85],
      duration: ANIME_DURATIONS.fast,
      ease: ANIME_SPRINGS.sensitive,
    },
  },
  // 父元素交换（crossfade 同时存在两元素）
  swap: {
    duration: ANIME_DURATIONS.normal,
    ease: ANIME_SPRINGS.default,
    swapAt: {
      opacity: [0, 1],
      duration: ANIME_DURATIONS.normal,
      ease: ANIME_SPRINGS.default,
    },
  },
  // BottomSheet 内部列表项 FLIP（更轻量）
  sheetList: {
    duration: ANIME_DURATIONS.normal,
    ease: ANIME_SPRINGS.default,
    enterFrom: {
      opacity: [0, 1],
      translateY: [12, 0],
      duration: ANIME_DURATIONS.normal,
      ease: ANIME_SPRINGS.default,
    },
    leaveTo: {
      opacity: [1, 0],
      scale: [1, 0.92],
      duration: ANIME_DURATIONS.fast,
      ease: ANIME_SPRINGS.sensitive,
    },
  },
};

/**
 * 命名入场动画（用于 useAnime().enter(el, name)）。
 * 与 ANIME_PRESETS 一一对应，但提供字符串 API 便于组件声明式调用。
 */
export const ANIME_ENTERS = {
  fade: ANIME_PRESETS.fadeIn,
  fadeUp: ANIME_PRESETS.fadeInUp,
  fadeDown: ANIME_PRESETS.fadeInDown,
  scale: ANIME_PRESETS.scaleIn,
  pop: ANIME_PRESETS.popIn,
  slideRight: ANIME_PRESETS.slideInRight,
  slideLeft: ANIME_PRESETS.slideInLeft,
  listItem: ANIME_PRESETS.listItemEnter,
  // BottomSheet 专用
  sheetUp: {
    opacity: [0, 1],
    translateY: ['100%', 0],
    duration: ANIME_DURATIONS.spring,
    ease: ANIME_SPRINGS.bouncy,
  },
};

/**
 * 命名退场动画（用于 useAnime().exit(el, name)）。
 */
export const ANIME_EXITS = {
  fade: ANIME_PRESETS.fadeOut,
  fadeUp: ANIME_PRESETS.fadeOutUp,
  fadeDown: ANIME_PRESETS.fadeOutDown,
  scale: ANIME_PRESETS.scaleOut,
  // BottomSheet 专用
  sheetDown: {
    opacity: [1, 0],
    translateY: [0, '100%'],
    duration: ANIME_DURATIONS.normal,
    ease: ANIME_SPRINGS.sensitive,
  },
};
