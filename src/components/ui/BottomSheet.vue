<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch, useTemplateRef } from "vue";
import { useAnime } from "@/composables/useAnime";
import { ANIME_LAYOUT } from "@/utils/animePresets";

type SheetDetent = "medium" | "large";

const props = withDefaults(
  defineProps<{
    visible: boolean;
    title?: string;
    showBack?: boolean;
    detents?: SheetDetent[];
    defaultDetent?: SheetDetent;
    /** 内部列表项选择器（如 '.song-item'），提供后自动监听 DOM 变化并执行 FLIP 动画 */
    listItemSelector?: string;
  }>(),
  {
    title: "",
    showBack: false,
    detents: () => ["medium", "large"] as SheetDetent[],
    defaultDetent: "medium",
    listItemSelector: "",
  },
);

const emit = defineEmits<{
  close: [];
  back: [];
  "update:visible": [v: boolean];
}>();

const currentDetent = ref<SheetDetent>(props.defaultDetent);
const dragState = ref<{
  active: boolean;
  startY: number;
  startDetent: SheetDetent;
  offset: number;
} | null>(null);

const maskEl = useTemplateRef<HTMLElement>("mask");
const sheetEl = useTemplateRef<HTMLElement>("sheet");
const contentEl = useTemplateRef<HTMLElement>("content");
// 延迟移除：visible 变 false 时先播退场动画，onComplete 再卸载 DOM
const internalVisible = ref(props.visible);

const isDesktop = ref(false);
let resizeObserver: ResizeObserver | null = null;

const { animate, enter, exit, spring, useLayout } = useAnime();

// ── 内部列表项 FLIP ──────────────────────────────────────────
// 通过 MutationObserver 自动监听 content 子树变化，配合 createLayout 执行 FLIP
const listLayout = useLayout(() => contentEl.value, ANIME_LAYOUT.sheetList);
let listObserver: MutationObserver | null = null;
let flipDebounce: number | null = null;

function setupListFlip() {
  if (!props.listItemSelector) return;
  listObserver = new MutationObserver(() => {
    // 防抖：连续 DOM 变化只触发一次 FLIP
    if (flipDebounce !== null) {
      clearTimeout(flipDebounce);
    }
    flipDebounce = window.setTimeout(async () => {
      flipDebounce = null;
      if (!contentEl.value || !internalVisible.value) return;
      listLayout.record();
      await nextTick();
      listLayout.animate();
    }, 16);
  });
  nextTick(() => {
    if (contentEl.value && listObserver) {
      listObserver.observe(contentEl.value, { childList: true, subtree: true });
    }
  });
}

// ── 视口检测 ──────────────────────────────────────────────────
function updateViewport() {
  isDesktop.value = window.innerWidth >= 768;
}

onMounted(() => {
  updateViewport();
  resizeObserver = new ResizeObserver(updateViewport);
  resizeObserver.observe(document.documentElement);
  if (props.visible) {
    document.body.style.overflow = "hidden";
    nextTick(() => playEnter());
  }
  setupListFlip();
});

onBeforeUnmount(() => {
  resizeObserver?.disconnect();
  listObserver?.disconnect();
  if (flipDebounce !== null) clearTimeout(flipDebounce);
});

// ── 入场 / 退场动画 ───────────────────────────────────────────
function playEnter() {
  if (maskEl.value) {
    enter(maskEl.value, "fade", { duration: 250 });
  }
  if (!sheetEl.value) return;
  if (isDesktop.value) {
    // 桌面端：居中模态缩放淡入（createLayout modal 风格）
    enter(sheetEl.value, "scale", { duration: 400 });
  } else {
    // 移动端：从底部滑入
    enter(sheetEl.value, "sheetUp", { duration: 500 });
  }
}

async function playExit(): Promise<void> {
  const tasks: Promise<unknown>[] = [];
  if (maskEl.value) {
    tasks.push(exit(maskEl.value, "fade", { duration: 200 }));
  }
  if (sheetEl.value) {
    if (isDesktop.value) {
      tasks.push(exit(sheetEl.value, "scale", { duration: 250 }));
    } else {
      tasks.push(exit(sheetEl.value, "sheetDown", { duration: 300 }));
    }
  }
  await Promise.all(tasks);
}

// ── visible 监听 ──────────────────────────────────────────────
watch(
  () => props.visible,
  async (v) => {
    if (v) {
      currentDetent.value = props.defaultDetent;
      document.body.style.overflow = "hidden";
      internalVisible.value = true;
      await nextTick();
      playEnter();
    } else {
      document.body.style.overflow = "";
      await playExit();
      internalVisible.value = false;
    }
  },
);

watch(
  () => props.defaultDetent,
  (d) => {
    if (props.visible) currentDetent.value = d;
  },
);

// ── Detent 几何 ───────────────────────────────────────────────
const hasLarge = computed(() => props.detents.includes("large"));
const hasMedium = computed(() => props.detents.includes("medium"));

const sheetStyle = computed(() => {
  if (isDesktop.value) {
    return {};
  }
  const drag = dragState.value;
  if (drag?.active) {
    return {
      transform: `translateY(${drag.offset}px)`,
      transition: "none",
    };
  }
  // 非拖拽时让 anime 完全接管 transform，不设 transition
  return {};
});

// 标记拖拽结束导致的 detent 变化，避免 watch 重复触发归零动画
let suppressDetentWatch = false;

watch(currentDetent, () => {
  if (suppressDetentWatch) return;
  if (dragState.value?.active) return;
  if (!sheetEl.value) return;
  if (isDesktop.value) return;
  // detent 切换：CSS 高度已变，animate 归零 translateY
  animate(sheetEl.value, {
    translateY: 0,
    ease: spring("bouncy"),
    duration: 500,
  });
});

// ── 内容滚动检测 ──────────────────────────────────────────────
function contentAtTop(): boolean {
  const el = contentEl.value;
  if (!el) return true;
  return el.scrollTop <= 0;
}

function contentAtBottom(): boolean {
  const el = contentEl.value;
  if (!el) return true;
  return el.scrollTop + el.clientHeight >= el.scrollHeight - 1;
}

// ── 拖拽手势（pointer events + anime.js spring snap）──────────
function onHandlePointerDown(e: PointerEvent) {
  startDrag(e);
}

function onHeaderPointerDown(e: PointerEvent) {
  if ((e.target as HTMLElement).closest("button")) return;
  startDrag(e);
}

function startDrag(e: PointerEvent) {
  if (isDesktop.value) return;
  const target = e.currentTarget as HTMLElement;
  target.setPointerCapture(e.pointerId);
  dragState.value = {
    active: true,
    startY: e.clientY,
    startDetent: currentDetent.value,
    offset: 0,
  };
}

function onPointerMove(e: PointerEvent) {
  const ds = dragState.value;
  if (!ds?.active) return;
  const dy = e.clientY - ds.startY;
  // 限制：large detent 下不能向上拖（无更高 detent）
  if (dy < 0 && ds.startDetent === "large" && !hasMedium.value) {
    ds.offset = 0;
    return;
  }
  if (dy < 0 && ds.startDetent === "large") {
    // large 下向上拖：阻尼（防止拖出顶部边界）
    ds.offset = dy * 0.3;
    return;
  }
  ds.offset = dy;
}

function onPointerUp(e: PointerEvent) {
  const ds = dragState.value;
  if (!ds?.active) return;
  const dy = e.clientY - ds.startY;
  const target = e.currentTarget as HTMLElement;
  target.releasePointerCapture?.(e.pointerId);
  const currentOffset = ds.offset;
  dragState.value = null;

  const threshold = 60;
  // 关闭：从 medium 向下拖超过阈值
  if (dy > threshold * 1.8 && ds.startDetent === "medium") {
    emit("close");
    emit("update:visible", false);
    return;
  }
  // 关闭：从 large 向下拖超过阈值且无 medium detent
  if (dy > threshold * 1.8 && ds.startDetent === "large" && !hasMedium.value) {
    emit("close");
    emit("update:visible", false);
    return;
  }

  // detent 切换由 onPointerUp 统一归零，抑制 watch 重复 animate
  suppressDetentWatch = true;
  if (dy < -threshold && ds.startDetent === "medium" && hasLarge.value) {
    currentDetent.value = "large";
  } else if (dy > threshold && ds.startDetent === "large" && hasMedium.value) {
    currentDetent.value = "medium";
  } else {
    currentDetent.value = ds.startDetent;
  }
  nextTick(() => {
    suppressDetentWatch = false;
  });

  // anime 接管归零：用 [currentOffset, 0] 显式 from，避免 Vue 移除 inline transform 后跳跃
  if (sheetEl.value) {
    animate(sheetEl.value, {
      translateY: [currentOffset, 0],
      ease: spring("bouncy"),
      duration: 500,
    });
  }
}

function onContentWheel(e: WheelEvent) {
  if (isDesktop.value) return;
  const el = contentEl.value;
  if (!el) return;
  const atTop = contentAtTop();
  const atBottom = contentAtBottom();
  if (e.deltaY < 0 && atTop && currentDetent.value === "medium" && hasLarge.value) {
    e.preventDefault();
    currentDetent.value = "large";
  } else if (e.deltaY > 0 && atTop && currentDetent.value === "medium") {
    e.preventDefault();
    emit("close");
    emit("update:visible", false);
  } else if (e.deltaY > 0 && atBottom && currentDetent.value === "large") {
    // bounce, do nothing
  }
}

// ── UI 事件 ───────────────────────────────────────────────────
function onMaskClick() {
  emit("close");
  emit("update:visible", false);
}

function onBack() {
  emit("back");
}

function onClose() {
  emit("close");
  emit("update:visible", false);
}

// ── 暴露 FLIP 手动触发（父组件可直接调用）─────────────────────
defineExpose({
  /** 手动触发内部列表 FLIP 动画 */
  flip: async () => {
    if (!contentEl.value) return;
    listLayout.record();
    await nextTick();
    listLayout.animate();
  },
  /** 切换 detent */
  setDetent: (d: SheetDetent) => {
    if (props.detents.includes(d)) {
      currentDetent.value = d;
    }
  },
});
</script>

<template>
  <Teleport to="body">
    <div
      v-if="internalVisible"
      ref="mask"
      class="bs-mask"
      :class="{ 'bs-mask--desktop': isDesktop }"
      @click.self="onMaskClick"
    >
      <div
        ref="sheet"
        class="bs-sheet clean-card"
        :class="[
          `bs-sheet--${currentDetent}`,
          { 'bs-sheet--desktop': isDesktop },
        ]"
        :style="sheetStyle"
      >
        <div
          class="bs-handle-area"
          @pointerdown="onHandlePointerDown"
          @pointermove="onPointerMove"
          @pointerup="onPointerUp"
          @pointercancel="onPointerUp"
        >
          <div class="bs-handle" />
        </div>

        <div
          class="bs-header"
          @pointerdown="onHeaderPointerDown"
          @pointermove="onPointerMove"
          @pointerup="onPointerUp"
          @pointercancel="onPointerUp"
        >
          <div class="bs-header-left">
            <button v-if="showBack" class="bs-icon-btn" @click="onBack" aria-label="返回">
              <i class="bi bi-chevron-left" style="font-size:18px"></i>
            </button>
            <h3 class="bs-title">{{ title }}</h3>
          </div>
          <button class="bs-icon-btn bs-close" @click="onClose" aria-label="关闭">
            <i class="bi bi-x-lg" style="font-size:16px"></i>
          </button>
        </div>

        <div
          ref="content"
          class="bs-content"
          @wheel="onContentWheel"
        >
          <slot />
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.bs-mask {
  position: fixed;
  inset: 0;
  z-index: 400;
  background: rgba(0, 0, 0, 0.4);
  display: flex;
  align-items: flex-end;
  justify-content: center;
  /* 修复：动画过程中底部边缘溢出 — 裁剪超出视口的 sheet 部分 */
  overflow: hidden;
}

.bs-sheet {
  width: 100%;
  max-width: 480px;
  background: var(--bg-50);
  border-radius: var(--radius-2xl) var(--radius-2xl) 0 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  box-shadow: 0 -8px 40px rgba(0, 0, 0, 0.15);
  overflow: hidden;
  touch-action: none;
}

.bs-sheet--medium {
  height: 60vh;
  max-height: 60vh;
}

.bs-sheet--large {
  height: calc(100vh - 40px - env(safe-area-inset-top, 0px));
  max-height: calc(100vh - 40px - env(safe-area-inset-top, 0px));
}

.bs-handle-area {
  flex-shrink: 0;
  padding: var(--space-2) 0 var(--space-1);
  display: flex;
  justify-content: center;
  cursor: grab;
  touch-action: none;
}

.bs-handle-area:active {
  cursor: grabbing;
}

.bs-handle {
  width: 40px;
  height: 4px;
  background: var(--bg-300);
  border-radius: var(--radius-full);
}

.bs-header {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-2) var(--space-4) var(--space-3);
  cursor: grab;
  user-select: none;
  touch-action: none;
}

.bs-header-left {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  flex: 1;
  min-width: 0;
}

.bs-title {
  font-size: var(--text-lg);
  font-weight: var(--fw-bold);
  color: var(--color-text);
  margin: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.bs-icon-btn {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  border: none;
  background: var(--bg-200);
  color: var(--color-text-secondary);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  padding: 0;
  flex-shrink: 0;
  transition: background 0.15s;
}

.bs-icon-btn:active {
  transform: scale(0.92);
  background: var(--bg-300);
}

.bs-close {
  margin-left: auto;
}

.bs-content {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  -webkit-overflow-scrolling: touch;
  overscroll-behavior: contain;
  padding: 0 var(--space-4) var(--space-5);
  touch-action: pan-y;
}

/* Desktop centered dialog mode */
.bs-mask--desktop {
  align-items: center;
  overflow: hidden;
}

.bs-sheet--desktop {
  width: 480px;
  max-width: 90vw;
  min-height: 320px;
  max-height: 90vh;
  height: 560px;
  border-radius: var(--radius-2xl);
}

.bs-sheet--desktop.bs-sheet--large {
  height: 90vh;
  max-height: 90vh;
}
</style>
