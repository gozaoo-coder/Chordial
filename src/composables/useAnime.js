/**
 * anime.js v4 Vue 3 集成 composable — 全 spring 物理 easing 版本
 *
 * 职责：
 *  - 自动创建 createScope 限定到组件根节点，避免污染全局
 *  - 组件卸载时自动 revert（停止动画 + 清理内联样式）
 *  - 暴露 animate / stagger / createTimeline / createAnimatable / createLayout / createDraggable 等
 *  - 提供 spring() / enter() / exit() 字符串 API（便于组件声明式调用）
 *  - 提供项目级预设（ANIME_PRESETS / ANIME_STAGGER / ANIME_LOOP / ANIME_LAYOUT / ANIME_SPRINGS）
 *
 * 用法 1：直接 animate
 *   <script setup>
 *   const rootRef = useTemplateRef('root');
 *   const { run } = useAnime(() => rootRef.value);
 *   onMounted(() => {
 *     run(({ animate, stagger, presets }) => {
 *       animate('.item', { ...presets.fadeInUp, delay: stagger(60) });
 *     });
 *   });
 *   </script>
 *
 * 用法 2：字符串 API（推荐用于 enter/exit 场景）
 *   const { enter, exit, spring } = useAnime();
 *   enter(el, 'scale', { duration: 400 });          // 等价于 ANIME_PRESETS.scaleIn
 *   exit(el, 'fade');                                // 等价于 ANIME_PRESETS.fadeOut
 *   animate(el, { opacity: [0,1], ease: spring('bouncy') });
 *
 * 用法 3：createLayout FLIP
 *   const { useLayout } = useAnime();
 *   const layout = useLayout(() => listRef.value, ANIME_LAYOUT.list);
 *   watch(items, async () => {
 *     layout.record();
 *     await nextTick();
 *     layout.animate();
 *   });
 *
 * 注意：
 *  - 不要用 ref() 包裹 anime.js 返回的 Animation/Timeline 实例（会被 Vue 深代理）
 *  - 在 <transition :css="false"> 的 JS hook 中可直接调用 animate()
 *  - createLayout 与 CSS content-visibility: auto / 虚拟列表 transform 定位冲突
 */
import { onScopeDispose } from 'vue';
import {
  animate as animeAnimate,
  stagger as animeStagger,
  createScope,
  createTimeline,
  createAnimatable,
  createTimer,
  createDraggable,
  createLayout,
  utils,
} from 'animejs';
import {
  ANIME_PRESETS,
  ANIME_STAGGER,
  ANIME_LOOP,
  ANIME_LAYOUT,
  ANIME_SPRINGS,
  ANIME_DURATIONS,
  ANIME_ENTERS,
  ANIME_EXITS,
  getSpring,
} from '@/utils/animePresets.js';

/**
 * @param {() => Element | undefined | null} rootRefGetter  组件根节点 ref 的 getter
 */
export function useAnime(rootRefGetter) {
  let scope = null;
  // 跟踪通过 useLayout 创建的 AutoLayout 实例
  const layouts = new Set();

  const ensureScope = () => {
    if (scope) return scope;
    const root = typeof rootRefGetter === 'function' ? rootRefGetter() : rootRefGetter;
    scope = createScope(root ? { root } : undefined);
    return scope;
  };

  /**
   * 在组件作用域内执行动画工厂。所有 animate/createTimeline 调用应放在 factory 内，
   * 以确保被 scope 管理，组件卸载时统一 revert。
   */
  const run = (factory) => {
    const s = ensureScope();
    s.add(() =>
      factory({
        animate: animeAnimate,
        stagger: animeStagger,
        createTimeline,
        createAnimatable,
        createTimer,
        createDraggable,
        createLayout,
        utils,
        presets: ANIME_PRESETS,
        staggerPresets: ANIME_STAGGER,
        loopPresets: ANIME_LOOP,
        layoutPresets: ANIME_LAYOUT,
        springs: ANIME_SPRINGS,
        easings: ANIME_SPRINGS, // 向后兼容别名
        durations: ANIME_DURATIONS,
        spring: getSpring,
      })
    );
    return scope;
  };

  /**
   * 创建受作用域管理的 createLayout 实例，组件卸载时自动 revert。
   *
   * Vue 集成模式（DOM 由 Vue 响应式控制，无法用 layout.update(callback)）：
   *   const layout = useLayout(() => listRef.value, ANIME_LAYOUT.list);
   *   watch(items, async () => {
   *     layout.record();           // pre-flush：DOM 未更新，记录旧位置
   *     await nextTick();          // 等 Vue 更新 DOM
   *     layout.animate();          // FLIP 从旧位置到新位置
   *   });
   */
  const useLayout = (rootGetter, params) => {
    let layout = null;
    const ensure = () => {
      if (layout) return layout;
      const root = typeof rootGetter === 'function' ? rootGetter() : rootGetter;
      if (!root) return null;
      layout = createLayout(root, params);
      layouts.add(layout);
      return layout;
    };
    return {
      record: () => ensure()?.record(),
      animate: (p) => ensure()?.animate(p),
      update: (cb, p) => ensure()?.update(cb, p),
      revert: () => {
        if (layout) {
          layout.revert();
          layouts.delete(layout);
          layout = null;
        }
      },
    };
  };

  /**
   * 命名入场动画。返回 Promise，动画完成时 resolve。
   * @param {Element} el
   * @param {keyof typeof ANIME_ENTERS} name  'fade' | 'fadeUp' | 'fadeDown' | 'scale' | 'pop' | 'slideRight' | 'slideLeft' | 'listItem' | 'sheetUp'
   * @param {object} [overrides] 覆盖预设参数（如 duration/springName）
   */
  const enter = (el, name = 'fade', overrides = {}) => {
    if (!el) return Promise.resolve();
    const preset = ANIME_ENTERS[name] || ANIME_ENTERS.fade;
    return new Promise((resolve) => {
      animeAnimate(el, {
        ...preset,
        ...overrides,
        // 允许 overrides.springName 指定 spring
        ease: overrides.springName ? getSpring(overrides.springName) : preset.ease,
        onComplete: () => { resolve(); overrides.onComplete?.(); },
      });
    });
  };

  /**
   * 命名退场动画。返回 Promise，动画完成时 resolve。
   * @param {Element} el
   * @param {keyof typeof ANIME_EXITS} name  'fade' | 'fadeUp' | 'fadeDown' | 'scale' | 'sheetDown'
   * @param {object} [overrides]
   */
  const exit = (el, name = 'fade', overrides = {}) => {
    if (!el) return Promise.resolve();
    const preset = ANIME_EXITS[name] || ANIME_EXITS.fade;
    return new Promise((resolve) => {
      animeAnimate(el, {
        ...preset,
        ...overrides,
        ease: overrides.springName ? getSpring(overrides.springName) : preset.ease,
        onComplete: () => { resolve(); overrides.onComplete?.(); },
      });
    });
  };

  /**
   * 按名称获取 spring easing。
   * @param {'default'|'sensitive'|'bouncy'|'powerful'} name
   */
  const spring = (name = 'default') => getSpring(name);

  // 组件作用域销毁时清理
  onScopeDispose(() => {
    layouts.forEach((l) => l.revert());
    layouts.clear();
    scope?.revert();
    scope = null;
  });

  return {
    run,
    // 直接暴露（用于 <transition> JS hook 等无法放入 factory 的场景）
    animate: animeAnimate,
    stagger: animeStagger,
    createTimeline,
    createAnimatable,
    createTimer,
    createDraggable,
    createLayout,
    utils,
    // 字符串 API
    enter,
    exit,
    spring,
    // 预设
    presets: ANIME_PRESETS,
    staggerPresets: ANIME_STAGGER,
    loopPresets: ANIME_LOOP,
    layoutPresets: ANIME_LAYOUT,
    springs: ANIME_SPRINGS,
    easings: ANIME_SPRINGS, // 向后兼容别名
    durations: ANIME_DURATIONS,
    // createLayout 便捷封装（自动 cleanup）
    useLayout,
    // 手动清理（一般无需调用）
    revert: () => {
      layouts.forEach((l) => l.revert());
      layouts.clear();
      scope?.revert();
      scope = null;
    },
  };
}
