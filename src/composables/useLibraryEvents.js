/**
 * useLibraryEvents — 全局音乐库变更事件订阅。
 *
 * 设计要点：
 * - 单一事件源：后端 `commands.rs` 在 `local_add_folder` / `local_remove_folder` /
 *   `local_rescan` 等改变库内容的操作完成后 emit `"library-changed"`。
 * - 全局唯一监听器：在 `main.js` 启动时调用 `initLibraryEvents()` 一次，
 *   避免每个组件各自 `listen` 导致的重复订阅与资源泄漏。
 * - 响应式版本号：每次事件递增 `libraryVersion.value`，组件通过 `watch`
 *   该 ref 实现自动刷新。
 * - 同时清除 `library.js` 中的内存缓存（`invalidateCache`），保证后续
 *   重新拉取的数据是最新版本。
 */

import { ref } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { library } from '@/api/musicSource';

/** 库版本号 — 每收到一次 `library-changed` 事件自增 1 */
const libraryVersion = ref(0);

/** 详细事件载荷 — 描述本次变更的范围，便于组件按需决定是否刷新 */
const lastChange = ref(null);

let unlistenFn = null;
let initPromise = null;

/**
 * 初始化全局 `library-changed` 监听器。应在应用启动时调用一次（如 `main.js`）。
 * 重复调用是幂等的：第二次起直接返回已有的 Promise。
 *
 * @returns {Promise<void>}
 */
export function initLibraryEvents() {
  if (initPromise) return initPromise;

  initPromise = (async () => {
    unlistenFn = await listen('library-changed', (e) => {
      libraryVersion.value += 1;
      lastChange.value = e.payload ?? null;
      // 失效前端缓存，确保下次查询重新拉取最新数据
      library.invalidateCache();
    });
  })();

  return initPromise;
}

/**
 * 组件订阅库变更 — 通过 `watch` 返回的 ref 触发刷新。
 *
 * 使用示例：
 * ```js
 * import { useLibraryEvents } from '@/composables/useLibraryEvents';
 * const { libraryVersion } = useLibraryEvents();
 * watch(libraryVersion, () => loadAlbums());
 * ```
 *
 * @returns {{ libraryVersion: import('vue').Ref<number>, lastChange: import('vue').Ref<any> }}
 */
export function useLibraryEvents() {
  return { libraryVersion, lastChange };
}
