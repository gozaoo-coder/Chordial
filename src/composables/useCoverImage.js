/**
 * 封面图片加载 Composable
 * 使用 ResourceManager 按需加载封面图片
 *
 * # 性能优化
 * - 实现并发控制，限制同时加载的图片数量
 * - 优化 watch 监听，避免深度监听大对象
 * - 自动管理资源生命周期，组件卸载时释放资源
 */
import { ref, onMounted, onUnmounted, watch, computed } from 'vue';

// 全局并发控制 - 限制同时加载的封面数量
const MAX_CONCURRENT_LOADS = 5;
let currentLoadCount = 0;
const loadQueue = [];

/**
 * 获取加载许可（并发控制）
 * @returns {Promise<void>}
 */
async function acquireLoadPermit() {
  if (currentLoadCount < MAX_CONCURRENT_LOADS) {
    currentLoadCount++;
    return;
  }

  // 等待队列
  return new Promise((resolve) => {
    loadQueue.push(resolve);
  });
}

/**
 * 释放加载许可
 */
function releaseLoadPermit() {
  currentLoadCount--;
  // 唤醒队列中的下一个
  if (loadQueue.length > 0) {
    const next = loadQueue.shift();
    currentLoadCount++;
    next();
  }
}

/**
 * 从对象获取封面资源
 * 优先使用 getCoverResource 或 acquireCoverResource 方法，其次使用 coverData
 * @param {Object} item - 专辑/曲目/歌手对象
 * @param {string} size - 图片尺寸
 * @returns {Promise<{url: string, release: Function}|null>}
 */
async function acquireCoverResource(item, size = 'medium') {
  if (!item) return null;

  // 优先级1: 使用 acquireCoverResource 方法（统一接口）
  if (typeof item.acquireCoverResource === 'function') {
    return await item.acquireCoverResource(size);
  }

  // 优先级2: 使用 getCoverResource 方法
  if (typeof item.getCoverResource === 'function') {
    return await item.getCoverResource(size);
  }

  // 优先级3: 使用 coverData（Data URL，不需要释放）
  if (item.coverData) {
    return {
      url: item.coverData,
      release: () => {} // Data URL 不需要释放
    };
  }

  // 优先级4: 使用 albumCoverData（Track 对象）
  if (item.albumCoverData) {
    return {
      url: item.albumCoverData,
      release: () => {} // Data URL 不需要释放
    };
  }

  return null;
}

/**
 * 加载专辑或歌手封面图片
 * @param {Object} item - 专辑或歌手对象 (AlbumSummary, Album, ArtistSummary, Artist)
 * @param {string} size - 图片尺寸 ('small', 'medium', 'large')
 * @returns {Object} { coverUrl, isLoading, error, release }
 */
export function useCoverImage(item, size = 'medium') {
  const coverUrl = ref('');
  const isLoading = ref(false);
  const error = ref(null);
  let releaseFn = null;
  let isLoadingCover = false;

  const loadCover = async () => {
    if (!item.value) return;
    if (isLoadingCover) {
      return;
    }

    isLoadingCover = true;
    isLoading.value = true;
    error.value = null;

    try {
      const resource = await acquireCoverResource(item.value, size);

      if (resource && resource.url) {
        if (releaseFn) {
          releaseFn();
          releaseFn = null;
        }

        coverUrl.value = resource.url;
        releaseFn = typeof resource.release === 'function'
          ? () => resource.release()
          : null;
      }
    } catch (err) {
      console.warn('Failed to load cover:', err);
      error.value = err;
    } finally {
      isLoadingCover = false;
      isLoading.value = false;
    }
  };

  const releaseCover = () => {
    if (releaseFn) {
      releaseFn();
      releaseFn = null;
    }
  };

  let lastItemId = null;

  onMounted(() => {
    lastItemId = item.value?.id;
    loadCover();
  });

  watch(() => item.value?.id, (newId, oldId) => {
    if (newId !== lastItemId) {
      lastItemId = newId;
      loadCover();
    }
  }, { immediate: false });

  onUnmounted(() => {
    releaseCover();
  });

  return {
    coverUrl,
    isLoading,
    error,
    reload: loadCover,
    release: releaseCover
  };
}

/**
 * 批量加载封面图片（用于列表）
 * @param {Ref<Array>} itemsRef - 专辑或歌手列表的 ref
 * @param {string} size - 图片尺寸
 * @returns {Object} { coverUrls, isLoading }
 */
export function useCoverImages(itemsRef, size = 'medium') {
  const coverUrls = ref(new Map());
  const isLoading = ref(false);
  const releaseFns = new Map();

  const loadCovers = async () => {
    if (!itemsRef.value || itemsRef.value.length === 0) return;

    isLoading.value = true;

    // 加载新的封面（带并发控制），先收集到临时 Map，最后一次性替换（避免 N+1 次重渲染）
    const newCoverUrls = new Map();
    const newReleaseFns = new Map();

    const promises = itemsRef.value.map(async (item) => {
      if (!item) return;

      // 获取加载许可（并发控制）
      await acquireLoadPermit();

      try {
        // 使用统一的资源获取函数
        const resource = await acquireCoverResource(item, size);
        if (resource && resource.url) {
          newCoverUrls.set(item.id, resource.url);
          newReleaseFns.set(item.id, resource.release);
        }
      } catch (err) {
        console.warn(`Failed to load cover for ${item.id}:`, err);
      } finally {
        // 释放加载许可
        releaseLoadPermit();
      }
    });

    await Promise.all(promises);

    // 释放旧的资源
    releaseFns.forEach((release) => release());
    releaseFns.clear();

    // 一次性替换整个 Map，只触发一次响应式更新
    coverUrls.value = newCoverUrls;
    // 更新 releaseFns 引用
    releaseFns.clear();
    newReleaseFns.forEach((fn, id) => releaseFns.set(id, fn));

    isLoading.value = false;
  };

  const releaseAll = () => {
    releaseFns.forEach((release) => release());
    releaseFns.clear();
    coverUrls.value.clear();
  };

  // 监听列表长度和 ID 变化，避免深度监听大对象
  const itemsKey = computed(() => {
    const items = itemsRef.value;
    if (!items) return '';
    return `${items.length}_${items.map(i => i?.id).join(',')}`;
  });
  watch(itemsKey, () => {
    loadCovers();
  });

  // 组件挂载时加载封面
  onMounted(() => {
    loadCovers();
  });

  // 组件卸载时释放所有资源
  onUnmounted(() => {
    releaseAll();
  });

  return {
    coverUrls,
    isLoading,
    reload: loadCovers,
    releaseAll
  };
}

export default useCoverImage;
