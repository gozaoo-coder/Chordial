/**
 * anime.js v4 Vue 3 集成 composable
 *
 * 职责：
 *  - 自动创建 createScope 限定到组件根节点，避免污染全局
 *  - 组件卸载时自动 revert（停止动画 + 清理内联样式）
 *  - 暴露 animate / stagger / createTimeline / createAnimatable 等方法
 *  - 提供项目级预设（ANIME_PRESETS / ANIME_STAGGER / ANIME_LOOP）
 *
 * 用法：
 *   <script setup>
 *   const rootRef = useTemplateRef('root');
 *   const { run, animate, stagger } = useAnime(() => rootRef.value);
 *
 *   onMounted(() => {
 *     run(({ animate, stagger }) => {
 *       animate('.item', { ...ANIME_PRESETS.fadeInUp, delay: stagger(60) });
 *     });
 *   });
 *   </script>
 *
 * 注意：
 *  - 不要用 ref() 包裹 anime.js 返回的 Animation/Timeline 实例（会被 Vue 深代理）
 *  - 同一元素的 CSS transition 与 anime.js 动画不要同时使用，避免抖动
 *  - 在 <transition :css="false"> 的 JS hook 中可直接调用 animate()
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
  utils,
} from 'animejs';
import {
  ANIME_PRESETS,
  ANIME_STAGGER,
  ANIME_LOOP,
  ANIME_EASINGS,
  ANIME_DURATIONS,
} from '@/utils/animePresets.js';

/**
 * @param {() => Element | undefined | null} rootRefGetter  组件根节点 ref 的 getter
 * @returns {{
 *   run: (factory: (ctx: {animate, stagger, createTimeline, createAnimatable, createTimer, createDraggable, utils, presets}) => void) => void,
 *   animate: typeof animeAnimate,
 *   stagger: typeof animeStagger,
 *   createTimeline: typeof createTimeline,
 *   createAnimatable: typeof createAnimatable,
 *   createTimer: typeof createTimer,
 *   createDraggable: typeof createDraggable,
 *   utils: typeof utils,
 *   presets: typeof ANIME_PRESETS,
 *   staggerPresets: typeof ANIME_STAGGER,
 *   loopPresets: typeof ANIME_LOOP,
 *   easings: typeof ANIME_EASINGS,
 *   durations: typeof ANIME_DURATIONS,
 *   revert: () => void,
 * }}
 */
export function useAnime(rootRefGetter) {
  let scope = null;

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
        utils,
        presets: ANIME_PRESETS,
        staggerPresets: ANIME_STAGGER,
        loopPresets: ANIME_LOOP,
        easings: ANIME_EASINGS,
        durations: ANIME_DURATIONS,
      })
    );
    return scope;
  };

  // 组件作用域销毁时清理：停止所有动画 + 还原 DOM 内联样式
  onScopeDispose(() => {
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
    utils,
    // 预设
    presets: ANIME_PRESETS,
    staggerPresets: ANIME_STAGGER,
    loopPresets: ANIME_LOOP,
    easings: ANIME_EASINGS,
    durations: ANIME_DURATIONS,
    // 手动清理（一般无需调用，onScopeDispose 会自动处理）
    revert: () => {
      scope?.revert();
      scope = null;
    },
  };
}
