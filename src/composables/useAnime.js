/**
 * anime.js v4 Vue 3 集成 composable
 *
 * 职责：
 *  - 自动创建 createScope 限定到组件根节点，避免污染全局
 *  - 组件卸载时自动 revert（停止动画 + 清理内联样式）
 *  - 暴露 animate / stagger / createTimeline / createAnimatable / createLayout 等方法
 *  - 提供项目级预设（ANIME_PRESETS / ANIME_STAGGER / ANIME_LOOP / ANIME_LAYOUT）
 *
 * 用法：
 *   <script setup>
 *   const rootRef = useTemplateRef('root');
 *   const { run, animate, stagger, useLayout } = useAnime(() => rootRef.value);
 *
 *   onMounted(() => {
 *     run(({ animate, stagger }) => {
 *       animate('.item', { ...ANIME_PRESETS.fadeInUp, delay: stagger(60) });
 *     });
 *   });
 *   </script>
 *
 * createLayout 用法（FLIP 布局动画，如列表增删）：
 *   const layout = useLayout(() => listRef.value, ANIME_LAYOUT.list);
 *   watch(items, async () => {
 *     layout.record();           // 记录旧位置（pre-flush，DOM 未更新）
 *     await nextTick();          // 等 Vue 更新 DOM
 *     layout.animate();          // FLIP 到新位置
 *   });
 *
 * 注意：
 *  - 不要用 ref() 包裹 anime.js 返回的 Animation/Timeline 实例（会被 Vue 深代理）
 *  - 同一元素的 CSS transition 与 anime.js 动画不要同时使用，避免抖动
 *  - 在 <transition :css="false"> 的 JS hook 中可直接调用 animate()
 *  - createLayout 与 CSS content-visibility: auto / 虚拟列表 transform 定位冲突，
 *    仅用于普通 DOM 列表
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
  ANIME_EASINGS,
  ANIME_DURATIONS,
  ANIME_LAYOUT,
} from '@/utils/animePresets.js';

/**
 * @param {() => Element | undefined | null} rootRefGetter  组件根节点 ref 的 getter
 * @returns {{
 *   run: (factory: (ctx: {animate, stagger, createTimeline, createAnimatable, createTimer, createDraggable, createLayout, utils, presets, layoutPresets}) => void) => void,
 *   animate: typeof animeAnimate,
 *   stagger: typeof animeStagger,
 *   createTimeline: typeof createTimeline,
 *   createAnimatable: typeof createAnimatable,
 *   createTimer: typeof createTimer,
 *   createDraggable: typeof createDraggable,
 *   createLayout: typeof createLayout,
 *   utils: typeof utils,
 *   presets: typeof ANIME_PRESETS,
 *   staggerPresets: typeof ANIME_STAGGER,
 *   loopPresets: typeof ANIME_LOOP,
 *   layoutPresets: typeof ANIME_LAYOUT,
 *   easings: typeof ANIME_EASINGS,
 *   durations: typeof ANIME_DURATIONS,
 *   useLayout: (rootGetter: () => Element | null, params?: object) => { record: () => void, animate: (params?: object) => unknown, update: (cb: (l: unknown) => void, params?: object) => unknown, revert: () => void },
 *   revert: () => void,
 * }}
 */
export function useAnime(rootRefGetter) {
  let scope = null;
  // 跟踪通过 useLayout 创建的 AutoLayout 实例，scope revert 不足以清理它们
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
        easings: ANIME_EASINGS,
        durations: ANIME_DURATIONS,
      })
    );
    return scope;
  };

  /**
   * 创建受作用域管理的 createLayout 实例，组件卸载时自动 revert。
   * 用于 FLIP 布局动画（列表项增删/排序、模态 display 切换等）。
   *
   * Vue 集成模式（DOM 由 Vue 响应式控制，无法用 layout.update(callback)）：
   *   const layout = useLayout(() => listRef.value, ANIME_LAYOUT.list);
   *   watch(items, async () => {
   *     layout.record();           // pre-flush：DOM 未更新，记录旧位置
   *     await nextTick();          // 等 Vue 更新 DOM
   *     layout.animate();          // FLIP 从旧位置到新位置
   *   });
   *
   * @param {() => Element | null} rootGetter  layout 根节点的 getter
   * @param {object} [params]  createLayout 参数（可用 ANIME_LAYOUT 预设）
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

  // 组件作用域销毁时清理：停止所有动画 + 还原 DOM 内联样式
  onScopeDispose(() => {
    // 先 revert 所有 createLayout 实例（清理 layout 内联样式与 timeline）
    layouts.forEach((l) => l.revert());
    layouts.clear();
    // 再 revert scope（清理所有通过 run() 注册的动画）
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
    // 预设
    presets: ANIME_PRESETS,
    staggerPresets: ANIME_STAGGER,
    loopPresets: ANIME_LOOP,
    layoutPresets: ANIME_LAYOUT,
    easings: ANIME_EASINGS,
    durations: ANIME_DURATIONS,
    // createLayout 便捷封装（自动 cleanup）
    useLayout,
    // 手动清理（一般无需调用，onScopeDispose 会自动处理）
    revert: () => {
      layouts.forEach((l) => l.revert());
      layouts.clear();
      scope?.revert();
      scope = null;
    },
  };
}
