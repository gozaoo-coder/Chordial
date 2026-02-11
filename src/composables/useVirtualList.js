/**
 * 虚拟列表 Composable
 *
 * 高性能渲染大量数据，只渲染可见区域的内容
 * 参考实现：响应式音乐歌单虚拟列表
 *
 * # 性能优化
 * - 只渲染可见区域 + 缓冲区的 DOM 节点
 * - 使用 requestAnimationFrame 节流滚动事件
 * - 二分查找 O(log n) 快速定位可见项
 * - 支持动态高度（通过 itemHeight 函数）
 *
 * @param {Ref<Array>} itemsRef - 数据列表
 * @param {Object} options - 配置选项
 * @returns {Object} 虚拟列表状态和计算属性
 */
import { ref, computed, onMounted, onUnmounted } from 'vue';

export function useVirtualList(itemsRef, options = {}) {
  const {
    itemHeight = 64, // 默认每项高度
    bufferSize = 5, // 缓冲区大小（上下各多渲染几项）
    containerRef = null, // 容器 ref
  } = options;

  // 滚动状态
  const scrollTop = ref(0);
  const containerHeight = ref(0);
  const rafId = ref(null);

  // 计算总高度
  const totalHeight = computed(() => {
    if (!itemsRef.value) return 0;

    if (typeof itemHeight === 'function') {
      return itemsRef.value.reduce((sum, item, index) => {
        return sum + itemHeight(item, index);
      }, 0);
    }

    return itemsRef.value.length * itemHeight;
  });

  // 计算前缀和（用于快速定位）
  const prefixSum = computed(() => {
    if (!itemsRef.value) return [];

    const sums = new Float64Array(itemsRef.value.length);
    let sum = 0;

    for (let i = 0; i < itemsRef.value.length; i++) {
      const height = typeof itemHeight === 'function'
        ? itemHeight(itemsRef.value[i], i)
        : itemHeight;
      sum += height;
      sums[i] = sum;
    }

    return sums;
  });

  // 二分查找：根据滚动位置找到起始索引 O(log n)
  const findStartIndex = (scrollPos) => {
    if (!prefixSum.value.length) return 0;

    let left = 0;
    let right = prefixSum.value.length - 1;
    let ans = 0;

    while (left <= right) {
      const mid = (left + right) >> 1;
      if (prefixSum.value[mid] <= scrollPos) {
        ans = mid;
        left = mid + 1;
      } else {
        right = mid - 1;
      }
    }

    return ans;
  };

  // 计算可见项
  const visibleItems = computed(() => {
    if (!itemsRef.value || !itemsRef.value.length) return [];

    const startIdx = Math.max(0, findStartIndex(scrollTop.value) - bufferSize);
    const endIdx = Math.min(
      itemsRef.value.length,
      findStartIndex(scrollTop.value + containerHeight.value) + bufferSize
    );

    const items = [];
    for (let i = startIdx; i < endIdx; i++) {
      const prevHeight = i > 0 ? prefixSum.value[i - 1] : 0;
      const height = typeof itemHeight === 'function'
        ? itemHeight(itemsRef.value[i], i)
        : itemHeight;

      items.push({
        item: itemsRef.value[i],
        index: i,
        offsetY: prevHeight,
        height,
      });
    }

    return items;
  });

  // 渲染的 DOM 节点数量
  const poolSize = computed(() => visibleItems.value.length);

  // 滚动处理（使用 RAF 节流）
  const onScroll = (e) => {
    if (rafId.value) {
      cancelAnimationFrame(rafId.value);
    }

    rafId.value = requestAnimationFrame(() => {
      scrollTop.value = e.target.scrollTop;
    });
  };

  // 滚动到指定索引
  const scrollToIndex = (index) => {
    if (!containerRef?.value) return;

    let offsetY = 0;
    if (typeof itemHeight === 'function') {
      for (let i = 0; i < index && i < itemsRef.value.length; i++) {
        offsetY += itemHeight(itemsRef.value[i], i);
      }
    } else {
      offsetY = index * itemHeight;
    }

    containerRef.value.scrollTop = offsetY;
  };

  // 滚动到指定偏移
  const scrollTo = (offset) => {
    if (containerRef?.value) {
      containerRef.value.scrollTop = offset;
    }
  };

  // 更新容器高度
  const updateContainerHeight = () => {
    if (containerRef?.value) {
      containerHeight.value = containerRef.value.clientHeight;
    }
  };

  // 生命周期
  onMounted(() => {
    updateContainerHeight();
    window.addEventListener('resize', updateContainerHeight);
  });

  onUnmounted(() => {
    if (rafId.value) {
      cancelAnimationFrame(rafId.value);
    }
    window.removeEventListener('resize', updateContainerHeight);
  });

  return {
    // 状态
    scrollTop,
    containerHeight,
    totalHeight,

    // 计算属性
    visibleItems,
    poolSize,
    prefixSum,

    // 方法
    onScroll,
    scrollToIndex,
    scrollTo,
    updateContainerHeight,
  };
}

export default useVirtualList;
