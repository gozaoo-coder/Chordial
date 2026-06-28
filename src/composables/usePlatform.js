/**
 * Platform Detection Composable
 *
 * Detects the current runtime environment using Tauri internals and user agent.
 * Returns module-level singleton reactive refs — platform never changes at runtime.
 *
 * @example
 * import { usePlatform } from '@/composables/usePlatform.js';
 * const { isDesktop, isMobile, isWeb } = usePlatform();
 */

import { ref, computed, readonly } from 'vue';

// Module-level singletons — resolved once, shared across all components
const _tauri = ref(null);
const _mobile = ref(null);

function detectTauri() {
  return !!(window.__TAURI_INTERNALS__ || window.__TAURI__);
}

function detectMobileOS() {
  return /android|iphone|ipad|ipod/i.test(navigator.userAgent);
}

/** @returns {import('vue').Ref<boolean>} */
function getIsTauri() {
  if (_tauri.value === null) {
    _tauri.value = detectTauri();
  }
  return _tauri;
}

/** @returns {import('vue').Ref<boolean>} */
function getIsMobile() {
  if (_mobile.value === null) {
    _mobile.value = detectMobileOS();
  }
  return _mobile;
}

/**
 * @typedef {Object} PlatformInfo
 * @property {import('vue').ComputedRef<boolean>} isTauri  — true when running in Tauri runtime
 * @property {import('vue').ComputedRef<boolean>} isMobile — true when Tauri + mobile OS (Android/iOS)
 * @property {import('vue').ComputedRef<boolean>} isDesktop — true when Tauri + desktop OS
 * @property {import('vue').ComputedRef<boolean>} isWeb — true when plain browser (no Tauri)
 */

/**
 * Use platform detection in any component.
 * All returned values are ComputedRef<boolean> — safe for use in templates and watchers.
 *
 * @returns {PlatformInfo}
 */
export function usePlatform() {
  const isTauriVal = getIsTauri();
  const isMobileVal = getIsMobile();

  const isTauri = computed(() => isTauriVal.value);
  const isMobile = computed(() => isTauriVal.value && isMobileVal.value);
  const isDesktop = computed(() => isTauriVal.value && !isMobileVal.value);
  const isWeb = computed(() => !isTauriVal.value);

  return {
    isTauri: readonly(isTauri),
    isMobile: readonly(isMobile),
    isDesktop: readonly(isDesktop),
    isWeb: readonly(isWeb),
  };
}

/** Direct accessors for non-reactive (imperative) use */
export function platformIsTauri() {
  return getIsTauri().value;
}

export function platformIsMobile() {
  return getIsTauri().value && getIsMobile().value;
}

export function platformIsDesktop() {
  return getIsTauri().value && !getIsMobile().value;
}

export function platformIsWeb() {
  return !getIsTauri().value;
}

export default usePlatform;
