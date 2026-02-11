/**
 * 封面图片加载 Composable
 * 使用 ResourceManager 按需加载封面图片
 *
 * # 性能优化
 * - 实现并发控制，限制同时加载的图片数量
 * - 优化 watch 监听，避免深度监听大对象
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
 * 加载专辑或歌手封面图片
 * @param {Object} item - 专辑或歌手对象 (AlbumSummary, Album, ArtistSummary, Artist)
 * @param {string} size - 图片尺寸 ('small', 'medium', 'large')
 * @returns {Object} { coverUrl, isLoading, error }
 */
export function useCoverImage(item, size = 'medium') {
  const coverUrl = ref('');
  const isLoading = ref(false);
  const error = ref(null);
  let releaseFn = null;

  const loadCover = async () => {
    if (!item.value) return;

    // 如果对象已经有 coverData（旧数据兼容），直接使用
    if (item.value.coverData) {
      coverUrl.value = item.value.coverData;
      return;
    }

    // 检查是否有 getCoverResource 方法
    if (typeof item.value.getCoverResource !== 'function') return;

    isLoading.value = true;
    error.value = null;

    try {
      const resource = await item.value.getCoverResource(size);

      if (resource) {
        // 统一接口：优先使用 getUrl() 方法，否则使用 url 属性
        const url = typeof resource.getUrl === 'function'
          ? resource.getUrl()
          : resource.url;

        if (url) {
          coverUrl.value = url;
          // 统一释放接口
          releaseFn = typeof resource.release === 'function'
            ? () => resource.release()
            : null;
        }
      }
    } catch (err) {
      console.warn('Failed to load cover:', err);
      error.value = err;
      coverUrl.value = '';
    } finally {
      isLoading.value = false;
    }
  };

  const releaseCover = () => {
    if (releaseFn) {
      releaseFn();
      releaseFn = null;
    }
    coverUrl.value = '';
  };

  // 组件挂载时加载封面
  onMounted(() => {
    loadCover();
  });

  // 监听 item.id 变化，避免深度监听大对象
  const itemId = computed(() => item.value?.id);
  watch(itemId, () => {
    releaseCover();
    loadCover();
  });

  // 组件卸载时释放资源
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

    // 释放旧的资源
    releaseFns.forEach((release) => release());
    releaseFns.clear();
    coverUrls.value.clear();

    // 加载新的封面（带并发控制）
    const promises = itemsRef.value.map(async (item) => {
      if (!item) return;

      // 如果对象已经有 coverData，直接使用
      if (item.coverData) {
        coverUrls.value.set(item.id, item.coverData);
        return;
      }

      // 检查是否有 getCoverResource 方法
      if (typeof item.getCoverResource !== 'function') {
        return;
      }

      // 获取加载许可（并发控制）
      await acquireLoadPermit();

      try {
        const resource = await item.getCoverResource(size);
        if (resource && resource.url) {
          coverUrls.value.set(item.id, resource.url);
          releaseFns.set(item.id, resource.release);
        }
      } catch (err) {
        console.warn(`Failed to load cover for ${item.id}:`, err);
      } finally {
        // 释放加载许可
        releaseLoadPermit();
      }
    });

    await Promise.all(promises);
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
