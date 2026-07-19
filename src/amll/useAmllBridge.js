/**
 * useAmllBridge — Vue ↔ React(AMLL) 状态桥接层
 *
 * 核心设计：
 *  1. 创建 Jotai store 作为 Vue ↔ React 通信层
 *  2. 应用 AmllSettingsStore 持久化配置（首次启动从 localStorage 读取）
 *  3. 反向绑定 Jotai → AmllSettingsStore，使设置页改 atom 时自动落盘
 *  4. Vue watchers 将 PlayerStore 状态推送到 Jotai atoms（单向）
 *  5. Jotai store.sub 监听 shuffle/repeat 变化回写 Vue（反向）
 *  6. 回调 atoms 一次性绑定到 PlayerStore actions
 *  7. 启动音频分析器，把 lowFreqVolume / fftData 实时推给 Jotai，
 *     驱动 AMLL 流体背景随音乐跳动
 *
 * 时间单位：AMLL 使用毫秒，PlayerStore 使用秒，桥接层负责转换。
 */
import { watch, onScopeDispose } from 'vue'
import { createStore } from 'jotai'
import {
	// 数据 atoms
	musicIdAtom, musicNameAtom, musicArtistsAtom, musicAlbumNameAtom,
	musicCoverAtom, musicCoverIsVideoAtom, musicDurationAtom,
	musicPlayingAtom, musicPlayingPositionAtom, musicVolumeAtom,
	musicLyricLinesAtom, lowFreqVolumeAtom, fftDataAtom,
	// 回调 atoms
	onPlayOrResumeAtom, onRequestPrevSongAtom, onRequestNextSongAtom,
	onSeekPositionAtom, onChangeVolumeAtom,
	onRequestOpenMenuAtom, onClickControlThumbAtom,
	// 控制 atoms
	repeatModeAtom, isShuffleActiveAtom, isShuffleEnabledAtom, isRepeatEnabledAtom,
	// 配置 atoms（需要在 AmllSettingsStore.apply / .bind 中使用）
	enableLyricLineBlurEffectAtom, enableLyricLineScaleEffectAtom,
	enableLyricLineSpringAnimationAtom, enableLyricTranslationLineAtom,
	enableLyricRomanLineAtom, enableLyricSwapTransRomanLineAtom,
	lyricWordFadeWidthAtom, lyricFontFamilyAtom, lyricFontWeightAtom,
	lyricLetterSpacingAtom, lyricSizePresetAtom,
	lyricBackgroundFPSAtom, lyricBackgroundRenderScaleAtom,
	lyricBackgroundStaticModeAtom, cssBackgroundPropertyAtom,
	showMusicNameAtom, showMusicArtistsAtom, showMusicAlbumAtom,
	showVolumeControlAtom, showBottomControlAtom, showRemainingTimeAtom,
	hideLyricViewAtom, verticalCoverLayoutAtom, playerControlsTypeAtom,
	isLyricPageOpenedAtom,
	// 枚举
	RepeatMode,
} from '@applemusic-like-lyrics/react-full'
import { PlayerStore, PlayMode } from '@/stores/player.js'
import { parseLyrics } from '@/utils/lyricConverter.js'
import { AmllSettingsStore } from '@/stores/amllSettings.js'
import { startAudioAnalyser, stopAudioAnalyser, setLowFreqRange } from '@/amll/useAudioAnalyser.js'

// VSync 开启时使用的 FPS 上限。
// AMLL 的 MeshGradientRenderer 用 `e < 1e3 / maxFPS` 节流：
// - maxFPS=0 会因 1000/0=Infinity 导致永不渲染
// - maxFPS=240 时每帧最小间隔 4.17ms，rAF 本身被浏览器限制在显示器刷新率（60/120Hz），
//   所以实际就是垂直同步。
const VSYNC_FPS = 240

// 收集所有配置 atoms，供 AmllSettingsStore.apply / .bind 使用
// （放在模块顶层避免每次调用 useAmllBridge 重建对象）
const CONFIG_ATOMS = {
	enableLyricLineBlurEffectAtom,
	enableLyricLineScaleEffectAtom,
	enableLyricLineSpringAnimationAtom,
	enableLyricTranslationLineAtom,
	enableLyricRomanLineAtom,
	enableLyricSwapTransRomanLineAtom,
	lyricWordFadeWidthAtom,
	lyricFontFamilyAtom,
	lyricFontWeightAtom,
	lyricLetterSpacingAtom,
	lyricSizePresetAtom,
	lyricBackgroundFPSAtom,
	lyricBackgroundRenderScaleAtom,
	lyricBackgroundStaticModeAtom,
	cssBackgroundPropertyAtom,
	showMusicNameAtom,
	showMusicArtistsAtom,
	showMusicAlbumAtom,
	showVolumeControlAtom,
	showBottomControlAtom,
	showRemainingTimeAtom,
	verticalCoverLayoutAtom,
	playerControlsTypeAtom,
	hideLyricViewAtom,
}

// 音频分析器需要的 atoms
const ANALYSER_ATOMS = { lowFreqVolumeAtom, fftDataAtom }

/**
 * 将 lyricConverter 输出的歌词行转为 AMLL LyricLine 格式
 * 复用 useLyricParser.js 的逻辑，但以纯函数形式供桥接层使用
 */
function toLyricLine(line, nextStartTime) {
	const startTime = line.time ?? 0
	const endTime = line.endTime
		?? (nextStartTime > startTime ? nextStartTime : startTime + (line.duration || 5000))

	const words = (line.words && line.words.length > 0)
		? line.words.map(w => ({
				startTime: w.time ?? startTime,
				endTime: w.time + (w.duration || 200),
				word: w.text || '',
				romanWord: '',
			}))
		: [{
				word: line.text || '',
				startTime,
				endTime,
				romanWord: '',
			}]

	return {
		words,
		startTime,
		endTime,
		translatedLyric: line.translation || '',
		romanLyric: '',
		isBG: false,
		isDuet: false,
	}
}

/** 将 LRC/YRC/QRC/TTML 字符串解析为 AMLL LyricLine[] */
function parseToAmllLyricLines(lyricString) {
	if (!lyricString || typeof lyricString !== 'string') return []
	try {
		const parsed = parseLyrics(lyricString)
		if (!parsed || parsed.length === 0) return []
		parsed.sort((a, b) => (a.time ?? 0) - (b.time ?? 0))
		return parsed.map((line, i) => {
			const nextLine = parsed[i + 1]
			const nextStartTime = nextLine ? (nextLine.time ?? 0) : undefined
			return toLyricLine(line, nextStartTime)
		})
	} catch (err) {
		console.warn('[useAmllBridge] 歌词解析失败:', err)
		return []
	}
}

/** PlayMode → { repeatMode, isShuffle } */
function playModeToAmll(playMode) {
	switch (playMode) {
		case PlayMode.RANDOM:   return { repeatMode: RepeatMode.Off, isShuffle: true }
		case PlayMode.LOOP:     return { repeatMode: RepeatMode.All, isShuffle: false }
		case PlayMode.LOOP_ONE: return { repeatMode: RepeatMode.One, isShuffle: false }
		default:                return { repeatMode: RepeatMode.Off, isShuffle: false }
	}
}

/** { repeatMode, isShuffle } → PlayMode */
function amllToPlayMode(repeatMode, isShuffle) {
	if (isShuffle) return PlayMode.RANDOM
	switch (repeatMode) {
		case RepeatMode.One: return PlayMode.LOOP_ONE
		case RepeatMode.All: return PlayMode.LOOP
		default:             return PlayMode.SEQUENCE
	}
}

/**
 * 创建并初始化 AMLL Jotai store，建立 Vue ↔ React 双向同步。
 *
 * 同时做四件事：
 *  1. 绑定回调 atoms 到 PlayerStore actions
 *  2. 应用 AmllSettingsStore 持久化配置到 store
 *  3. 反向监听配置 atoms 变化回写到 AmllSettingsStore（自动持久化）
 *  4. 建立 7 个 Vue watchers + 2 个 Jotai subs
 *  5. 启动音频分析器驱动流体背景
 *
 * @returns {import('jotai').Store} Jotai store 实例
 */
export function useAmllBridge() {
	const store = createStore()
	let syncing = false // 防止双向同步循环

	// ── 1. 回调 atoms 绑定（一次性）──
	store.set(onPlayOrResumeAtom, { onEmit: () => PlayerStore.togglePlay() })
	store.set(onRequestPrevSongAtom, { onEmit: () => PlayerStore.playPrevious() })
	store.set(onRequestNextSongAtom, { onEmit: () => PlayerStore.playNext() })
	store.set(onSeekPositionAtom, { onEmit: (ms) => PlayerStore.seek(ms / 1000) })
	store.set(onChangeVolumeAtom, { onEmit: (v) => PlayerStore.setVolume(v) })
	store.set(onRequestOpenMenuAtom, { onEmit: () => {} })
	store.set(onClickControlThumbAtom, { onEmit: () => {} })

	// ── 2. 应用持久化配置（从 localStorage 读取后写入 Jotai）──
	AmllSettingsStore.apply(store, CONFIG_ATOMS)

	// ── 3. 反向绑定 Jotai → AmllSettingsStore（设置页改 atom 时自动持久化）──
	const unbindSettings = AmllSettingsStore.bind(store, CONFIG_ATOMS)

	// ── 4. 一次性配置（非持久化项）──
	store.set(isLyricPageOpenedAtom, true)
	store.set(isShuffleEnabledAtom, true)
	store.set(isRepeatEnabledAtom, true)
	store.set(musicCoverIsVideoAtom, false)
	store.set(lowFreqVolumeAtom, 0)

	// ── 4.1 VSync 初始化 ──
	// VSync 开启时覆盖 apply 写入的 FPS atom 为高值（实现跟随显示器刷新率）
	const applyVSync = (enabled) => {
		store.set(lyricBackgroundFPSAtom, enabled ? VSYNC_FPS : AmllSettingsStore.state.lyricBackgroundFPS)
	}
	applyVSync(AmllSettingsStore.state.lyricBackgroundVSync)

	// ── 4.2 lowFreqVolume 频段初始化 ──
	setLowFreqRange(AmllSettingsStore.state.lowFreqVolumeRange)

	// ── 5. Vue → Jotai 状态同步（watchers）──

	// 当前曲目变化 → 更新所有元数据 + 异步加载封面
	const stopTrackWatch = watch(
		() => PlayerStore.state.currentTrack,
		async (track) => {
			if (!track) {
				store.set(musicIdAtom, '')
				store.set(musicNameAtom, '')
				store.set(musicArtistsAtom, [])
				store.set(musicAlbumNameAtom, '')
				store.set(musicCoverAtom, '')
				store.set(musicLyricLinesAtom, [])
				return
			}

			store.set(musicIdAtom, track.id || '')
			store.set(musicNameAtom, track.title || '未知歌曲')
			store.set(musicArtistsAtom, (track.artistNames || []).map((name, i) => ({
				name: name || '未知歌手',
				id: track.artistIds?.[i] || name || '',
			})))
			store.set(musicAlbumNameAtom, track.albumTitle || '')

			// 异步加载封面（acquireCoverResource 返回 chordial:// URL）
			try {
				const { url } = await track.acquireCoverResource('large')
				store.set(musicCoverAtom, url)
			} catch {
				store.set(musicCoverAtom, '')
			}
		},
		{ immediate: true },
	)

	// 播放状态
	const stopPlayingWatch = watch(
		() => PlayerStore.state.isPlaying,
		(isPlaying) => store.set(musicPlayingAtom, isPlaying),
		{ immediate: true },
	)

	// 播放进度（秒 → 毫秒）
	const stopPositionWatch = watch(
		() => PlayerStore.state.currentTime,
		(sec) => store.set(musicPlayingPositionAtom, sec * 1000),
		{ immediate: true },
	)

	// 总时长（秒 → 毫秒）
	const stopDurationWatch = watch(
		() => PlayerStore.state.duration,
		(sec) => store.set(musicDurationAtom, sec * 1000),
		{ immediate: true },
	)

	// 音量
	const stopVolumeWatch = watch(
		() => PlayerStore.state.volume,
		(v) => store.set(musicVolumeAtom, v),
		{ immediate: true },
	)

	// 歌词数据 → 解析为 AMLL LyricLine[]
	// PlayerStore.loadLyrics() 把 LRC 字符串写入 lyricsData.syncedLyrics
	// 桥接层负责 LRC/YRC/QRC/TTML → AMLL LyricLine[] 转换
	const stopLyricsWatch = watch(
		() => PlayerStore.state.lyricsData?.syncedLyrics || PlayerStore.state.lyricsData?.plainLyrics || '',
		(lyricString) => store.set(musicLyricLinesAtom, parseToAmllLyricLines(lyricString)),
		{ immediate: true },
	)

	// 播放模式 → RepeatMode + isShuffle
	const stopPlayModeWatch = watch(
		() => PlayerStore.state.playMode,
		(mode) => {
			if (syncing) return
			const { repeatMode, isShuffle } = playModeToAmll(mode)
			syncing = true
			store.set(repeatModeAtom, repeatMode)
			store.set(isShuffleActiveAtom, isShuffle)
			syncing = false
		},
		{ immediate: true },
	)

	// VSync 开关变化 → 覆盖 FPS atom
	// 开启：设为高值，由 rAF 原生节流到显示器刷新率
	// 关闭：恢复用户选择的 FPS
	const stopVSyncWatch = watch(
		() => AmllSettingsStore.state.lyricBackgroundVSync,
		(enabled) => applyVSync(enabled),
	)

	// 用户改 FPS 选择 → VSync 关闭时同步到 atom
	// （VSync 开启时 FPS 选择器在 UI 上被禁用，但为防误改仍加保护）
	const stopFpsWatch = watch(
		() => AmllSettingsStore.state.lyricBackgroundFPS,
		(fps) => {
			if (!AmllSettingsStore.state.lyricBackgroundVSync) {
				store.set(lyricBackgroundFPSAtom, fps)
			}
		},
	)

	// lowFreqVolume 频段变化 → 实时更新音频分析器
	const stopFreqRangeWatch = watch(
		() => AmllSettingsStore.state.lowFreqVolumeRange,
		(range) => setLowFreqRange(range),
		{ deep: true },
	)

	// ── 6. Jotai → Vue 反向同步（shuffle/repeat 按钮点击）──
	const unsubRepeat = store.sub(repeatModeAtom, () => {
		if (syncing) return
		syncing = true
		const mode = amllToPlayMode(store.get(repeatModeAtom), store.get(isShuffleActiveAtom))
		if (PlayerStore.state.playMode !== mode) {
			PlayerStore.setPlayMode(mode)
		}
		syncing = false
	})

	const unsubShuffle = store.sub(isShuffleActiveAtom, () => {
		if (syncing) return
		syncing = true
		const mode = amllToPlayMode(store.get(repeatModeAtom), store.get(isShuffleActiveAtom))
		if (PlayerStore.state.playMode !== mode) {
			PlayerStore.setPlayMode(mode)
		}
		syncing = false
	})

	// ── 7. 启动音频分析器（驱动流体背景）──
	// PlayerView 挂载时 audio 元素已就绪，分析器会自动接管
	startAudioAnalyser(store, ANALYSER_ATOMS)

	// ── 8. 清理 ──
	onScopeDispose(() => {
		stopTrackWatch()
		stopPlayingWatch()
		stopPositionWatch()
		stopDurationWatch()
		stopVolumeWatch()
		stopLyricsWatch()
		stopPlayModeWatch()
		stopVSyncWatch()
		stopFpsWatch()
		stopFreqRangeWatch()
		unsubRepeat()
		unsubShuffle()
		unbindSettings()
		stopAudioAnalyser()
	})

	return store
}
