/**
 * 封面图片加载 Composable
 * 使用 ResourceManager 按需加载封面图片
 */
import { ref, onMounted, onUnmounted, watch } from 'vue';

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
        const url = resource.url || resource.getUrl?.();

        if (url) {
          coverUrl.value = url;
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

  // 监听 item 变化，重新加载封面
  watch(item, () => {
    releaseCover();
    loadCover();
  }, { deep: true });

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

    // 加载新的封面
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

      try {
        const resource = await item.getCoverResource(size);
        if (resource && resource.url) {
          coverUrls.value.set(item.id, resource.url);
          releaseFns.set(item.id, resource.release);
        }
      } catch (err) {
        console.warn(`Failed to load cover for ${item.id}:`, err);
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

  // 监听列表变化，重新加载封面
  watch(itemsRef, () => {
    loadCovers();
  }, { deep: true });

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
