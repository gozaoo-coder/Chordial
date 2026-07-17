/**
 * AmllSettingsStore — AMLL 播放器全部配置项的持久化中心。
 *
 * # 设计
 * - 集中管理 AMLL Jotai atoms 的初始值与持久化
 * - 用 localStorage 落盘，避免每次启动都重置默认值
 * - 提供 applyToStore(store) 把配置一次性写入 Jotai store
 * - 提供 bindFromStore(store) 反向监听 Jotai 变化回写到 localStorage
 *   （这样设置页改 Jotai atom 时能自动持久化）
 *
 * # 字段分组
 * 1. 歌词效果：模糊、缩放、弹簧动画、翻译/音译显示
 * 2. 歌词字体：family、weight、letterSpacing、sizePreset、wordFadeWidth
 * 3. 背景流体：FPS、renderScale、staticMode
 * 4. 显示项：歌名/艺人/专辑/音量条/底部控件/剩余时间
 * 5. 布局：verticalCoverLayout、playerControlsType
 *
 * 配置 schema 版本化，便于未来迁移。
 */
import { reactive, watch } from 'vue'

// ── 持久化键 ──────────────────────────────────────────────────
const STORAGE_KEY = 'chordial_amll_settings'
const SCHEMA_VERSION = 1

// ── 默认值（与 AMLL 原生 default 对齐）──────────────────────────
export const AMLL_DEFAULTS = {
	// 歌词效果
	enableLyricLineBlurEffect: true,
	enableLyricLineScaleEffect: true,
	enableLyricLineSpringAnimation: true,
	enableLyricTranslationLine: true,
	enableLyricRomanLine: false,
	enableLyricSwapTransRomanLine: false,
	lyricWordFadeWidth: 0.5,

	// 歌词字体
	lyricFontFamily: '',
	lyricFontWeight: 600,
	lyricLetterSpacing: 'normal',
	lyricSizePreset: 'medium', // LyricSizePreset.Medium

	// 背景流体
	lyricBackgroundFPS: 60,
	lyricBackgroundRenderScale: 1,
	lyricBackgroundStaticMode: false,
	cssBackgroundProperty: '#111111',

	// 显示项
	showMusicName: true,
	showMusicArtists: true,
	showMusicAlbum: false,
	showVolumeControl: true,
	showBottomControl: true,
	showRemainingTime: true,

	// 布局
	verticalCoverLayout: 'auto', // VerticalCoverLayout.Auto
	playerControlsType: 'controls', // PlayerControlsType.Controls
	hideLyricView: false,
}

// ── 响应式状态（设置页双向绑定）──────────────────────────────
const state = reactive({ ...AMLL_DEFAULTS })

// ── 持久化 ──────────────────────────────────────────────────────
function loadFromStorage() {
	try {
		const raw = localStorage.getItem(STORAGE_KEY)
		if (!raw) return null
		const parsed = JSON.parse(raw)
		if (parsed.__schema !== SCHEMA_VERSION) {
			// schema 变更时丢弃旧数据，回退默认值
			return null
		}
		return parsed.values
	} catch (err) {
		console.warn('[AmllSettings] 加载失败:', err)
		return null
	}
}

function saveToStorage() {
	const payload = {
		__schema: SCHEMA_VERSION,
		values: { ...state },
	}
	try {
		localStorage.setItem(STORAGE_KEY, JSON.stringify(payload))
	} catch (err) {
		console.warn('[AmllSettings] 保存失败:', err)
	}
}

/**
 * 初始化：从 localStorage 读取并合并到 state。
 * 在应用启动时调用一次。
 */
export function initAmllSettings() {
	const saved = loadFromStorage()
	if (saved) {
		// 只合并已知字段，防止旧版本残留污染
		for (const key of Object.keys(AMLL_DEFAULTS)) {
			if (saved[key] !== undefined) {
				state[key] = saved[key]
			}
		}
	}
	// state 字段变化时自动持久化（Vue 深度 watch）
	watch(state, saveToStorage, { deep: true })
}

/**
 * 把当前配置一次性应用到 Jotai store。
 * 在 useAmllBridge 创建 store 后调用。
 *
 * @param {import('jotai').Store} store
 * @param {object} atoms - 从 @applemusic-like-lyrics/react-full 导入的 atoms
 */
export function applyAmllSettings(store, atoms) {
	store.set(atoms.enableLyricLineBlurEffectAtom, state.enableLyricLineBlurEffect)
	store.set(atoms.enableLyricLineScaleEffectAtom, state.enableLyricLineScaleEffect)
	store.set(atoms.enableLyricLineSpringAnimationAtom, state.enableLyricLineSpringAnimation)
	store.set(atoms.enableLyricTranslationLineAtom, state.enableLyricTranslationLine)
	store.set(atoms.enableLyricRomanLineAtom, state.enableLyricRomanLine)
	store.set(atoms.enableLyricSwapTransRomanLineAtom, state.enableLyricSwapTransRomanLine)
	store.set(atoms.lyricWordFadeWidthAtom, state.lyricWordFadeWidth)
	store.set(atoms.lyricFontFamilyAtom, state.lyricFontFamily)
	store.set(atoms.lyricFontWeightAtom, state.lyricFontWeight)
	store.set(atoms.lyricLetterSpacingAtom, state.lyricLetterSpacing)
	store.set(atoms.lyricSizePresetAtom, state.lyricSizePreset)
	store.set(atoms.lyricBackgroundFPSAtom, state.lyricBackgroundFPS)
	store.set(atoms.lyricBackgroundRenderScaleAtom, state.lyricBackgroundRenderScale)
	store.set(atoms.lyricBackgroundStaticModeAtom, state.lyricBackgroundStaticMode)
	store.set(atoms.cssBackgroundPropertyAtom, state.cssBackgroundProperty)
	store.set(atoms.showMusicNameAtom, state.showMusicName)
	store.set(atoms.showMusicArtistsAtom, state.showMusicArtists)
	store.set(atoms.showMusicAlbumAtom, state.showMusicAlbum)
	store.set(atoms.showVolumeControlAtom, state.showVolumeControl)
	store.set(atoms.showBottomControlAtom, state.showBottomControl)
	store.set(atoms.showRemainingTimeAtom, state.showRemainingTime)
	store.set(atoms.verticalCoverLayoutAtom, state.verticalCoverLayout)
	store.set(atoms.playerControlsTypeAtom, state.playerControlsType)
	store.set(atoms.hideLyricViewAtom, state.hideLyricView)
}

/**
 * 反向监听 Jotai store 变化，自动回写到 state（从而触发持久化）。
 * 用于设置页直接改 atom 时同步到 localStorage。
 *
 * @param {import('jotai').Store} store
 * @param {object} atoms
 * @returns {() => void} 取消订阅函数
 */
export function bindAmllSettingsFromStore(store, atoms) {
	const bindings = [
		[atoms.enableLyricLineBlurEffectAtom, 'enableLyricLineBlurEffect'],
		[atoms.enableLyricLineScaleEffectAtom, 'enableLyricLineScaleEffect'],
		[atoms.enableLyricLineSpringAnimationAtom, 'enableLyricLineSpringAnimation'],
		[atoms.enableLyricTranslationLineAtom, 'enableLyricTranslationLine'],
		[atoms.enableLyricRomanLineAtom, 'enableLyricRomanLine'],
		[atoms.enableLyricSwapTransRomanLineAtom, 'enableLyricSwapTransRomanLine'],
		[atoms.lyricWordFadeWidthAtom, 'lyricWordFadeWidth'],
		[atoms.lyricFontFamilyAtom, 'lyricFontFamily'],
		[atoms.lyricFontWeightAtom, 'lyricFontWeight'],
		[atoms.lyricLetterSpacingAtom, 'lyricLetterSpacing'],
		[atoms.lyricSizePresetAtom, 'lyricSizePreset'],
		[atoms.lyricBackgroundFPSAtom, 'lyricBackgroundFPS'],
		[atoms.lyricBackgroundRenderScaleAtom, 'lyricBackgroundRenderScale'],
		[atoms.lyricBackgroundStaticModeAtom, 'lyricBackgroundStaticMode'],
		[atoms.cssBackgroundPropertyAtom, 'cssBackgroundProperty'],
		[atoms.showMusicNameAtom, 'showMusicName'],
		[atoms.showMusicArtistsAtom, 'showMusicArtists'],
		[atoms.showMusicAlbumAtom, 'showMusicAlbum'],
		[atoms.showVolumeControlAtom, 'showVolumeControl'],
		[atoms.showBottomControlAtom, 'showBottomControl'],
		[atoms.showRemainingTimeAtom, 'showRemainingTime'],
		[atoms.verticalCoverLayoutAtom, 'verticalCoverLayout'],
		[atoms.playerControlsTypeAtom, 'playerControlsType'],
		[atoms.hideLyricViewAtom, 'hideLyricView'],
	]

	const unsubs = bindings.map(([atom, key]) =>
		store.sub(atom, () => {
			const value = store.get(atom)
			// 避免回写触发响应式循环（值相同时 Vue 不会触发 watch）
			if (state[key] !== value) {
				state[key] = value
			}
		}),
	)

	return () => unsubs.forEach((unsub) => unsub())
}

export const AmllSettingsStore = {
	state,
	init: initAmllSettings,
	apply: applyAmllSettings,
	bind: bindAmllSettingsFromStore,
}

export default AmllSettingsStore
