/**
 * useAmllBridge — Vue ↔ React(AMLL) 状态桥接层
 *
 * # 同步架构
 * Vue → Jotai（程序化同步）：watch PlayerStore.state → store.set(atom)
 * Jotai → Vue（用户交互同步）：回调 atoms onEmit → PlayerStore actions
 *
 * # 媒体控件绑定
 * - 播放/暂停/上一首/下一首：onPlayOrResume / onRequestPrev / onRequestNext
 * - 进度条拖动：onSeekPosition（毫秒 → 秒）
 * - 音量滑块：onChangeVolume（0-1）
 * - 随机按钮：onToggleShuffle → toggleShufflePlayMode
 * - 循环按钮：onCycleRepeatMode → cyclePlayMode（Off→All→One→Off）
 * - 歌词行点击：onLyricLineClick → seek(line.startTime)
 * - 顶部横条点击：onClickControlThumb → closePlayerView
 *
 * # 歌词数据链路
 * PlayerStore.loadLyrics → state.lyricsData.syncedLyrics (LRC/YRC/QRC/TTML)
 * → watch → parseToAmllLyricLines → musicLyricLinesAtom (LyricLine[])
 * 纯文本歌词兜底：无时间戳时每行一个 LyricLine，时间戳递增占位
 *
 * # 时间单位
 * AMLL 使用毫秒，PlayerStore 使用秒，桥接层负责转换。
 */
import { watch, onScopeDispose } from 'vue'
import { createStore } from 'jotai'
import {
	// 数据 atoms
	musicIdAtom, musicNameAtom, musicArtistsAtom, musicAlbumNameAtom,
	musicCoverAtom, musicCoverIsVideoAtom, musicDurationAtom,
	musicPlayingAtom, musicPlayingPositionAtom, musicVolumeAtom,
	musicLyricLinesAtom, lowFreqVolumeAtom, fftDataAtom,
	// 回调 atoms（媒体控件入口）
	onPlayOrResumeAtom, onRequestPrevSongAtom, onRequestNextSongAtom,
	onSeekPositionAtom, onChangeVolumeAtom,
	onRequestOpenMenuAtom, onClickControlThumbAtom,
	onClickAudioQualityTagAtom,
	onLyricLineClickAtom, onLyricLineContextMenuAtom,
	onToggleShuffleAtom, onCycleRepeatModeAtom,
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
		if (!parsed || parsed.length === 0) {
			// 兜底：纯文本歌词（无时间戳）→ 每行一个 LyricLine，时间戳从 0 递增
			// 让 AMLL 至少能展示文本，不会一片空白
			return parsePlainTextToLyricLines(lyricString)
		}
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

/**
 * 纯文本歌词兜底：无时间戳时按行切分，每行一个 LyricLine。
 * 时间戳从 0 开始按 5 秒递增（仅用于占位，实际不会驱动高亮）。
 */
function parsePlainTextToLyricLines(text) {
	const lines = text.split(/\r?\n/).map(l => l.trim()).filter(Boolean)
	if (lines.length === 0) return []
	return lines.map((text, i) => {
		const startTime = i * 5000
		const endTime = startTime + 5000
		return {
			words: [{ word: text, startTime, endTime, romanWord: '' }],
			startTime,
			endTime,
			translatedLyric: '',
			romanLyric: '',
			isBG: false,
			isDuet: false,
		}
	})
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

/**
 * 用户点 AMLL 循环按钮时的 PlayMode 循环逻辑。
 * AMLL 的 RepeatMode 循环：Off → All → One → Off
 * 映射到 PlayMode：SEQUENCE → LOOP → LOOP_ONE → SEQUENCE
 * 注意：RANDOM 模式下点循环按钮，应切到 LOOP（退出随机）
 */
function cyclePlayMode(currentPlayMode) {
	switch (currentPlayMode) {
		case PlayMode.SEQUENCE: return PlayMode.LOOP
		case PlayMode.LOOP:     return PlayMode.LOOP_ONE
		case PlayMode.LOOP_ONE: return PlayMode.SEQUENCE
		case PlayMode.RANDOM:   return PlayMode.LOOP // 退出随机，进入列表循环
		default:                return PlayMode.LOOP
	}
}

/**
 * 用户点 AMLL 随机按钮时的 PlayMode 切换逻辑。
 * 当前是 RANDOM → 切到 SEQUENCE；否则切到 RANDOM。
 */
function toggleShufflePlayMode(currentPlayMode) {
	return currentPlayMode === PlayMode.RANDOM ? PlayMode.SEQUENCE : PlayMode.RANDOM
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

	// ── 1. 媒体控件回调 atoms 绑定 ──
	// 所有 AMLL UI 按钮点击都通过回调 atoms 流回 PlayerStore，
	// 这是"用户交互入口"的权威通道；Vue→Jotai watcher 负责程序化状态同步。

	// 播放/暂停按钮
	store.set(onPlayOrResumeAtom, { onEmit: () => PlayerStore.togglePlay() })
	// 上一首 / 下一首
	store.set(onRequestPrevSongAtom, { onEmit: () => PlayerStore.playPrevious() })
	store.set(onRequestNextSongAtom, { onEmit: () => PlayerStore.playNext() })
	// 进度条拖动（毫秒 → 秒）
	store.set(onSeekPositionAtom, { onEmit: (ms) => PlayerStore.seek(ms / 1000) })
	// 音量滑块（0-1）
	store.set(onChangeVolumeAtom, { onEmit: (v) => PlayerStore.setVolume(v) })

	// 随机按钮：用户点击时切换 PlayMode 的 RANDOM 状态
	// 不依赖 AMLL 内部 atom 更新时序，直接基于 PlayerStore 当前状态计算新值
	store.set(onToggleShuffleAtom, {
		onEmit: () => {
			const newMode = toggleShufflePlayMode(PlayerStore.state.playMode)
			PlayerStore.setPlayMode(newMode)
			// Vue watcher 会把新 PlayMode 同步回 Jotai atoms，纠正 AMLL 内部状态
		},
	})

	// 循环按钮：Off → All → One → Off，映射到 PlayMode 循环
	store.set(onCycleRepeatModeAtom, {
		onEmit: () => {
			const newMode = cyclePlayMode(PlayerStore.state.playMode)
			PlayerStore.setPlayMode(newMode)
		},
	})

	// 歌词行左键点击 → 跳转到该行开始时间
	// LyricLineMouseEvent.line.startTime 是毫秒
	store.set(onLyricLineClickAtom, {
		onEmit: (evt) => {
			const startTime = evt?.line?.startTime
			if (typeof startTime === 'number' && !isNaN(startTime)) {
				PlayerStore.seek(startTime / 1000)
			}
		},
	})

	// 歌词行右键点击（暂无功能，预留）
	store.set(onLyricLineContextMenuAtom, { onEmit: () => {} })

	// 顶部控制横条点击 → 关闭 PlayerView（与 PlayerView 的关闭按钮一致）
	store.set(onClickControlThumbAtom, {
		onEmit: () => PlayerStore.closePlayerView(),
	})

	// 菜单按钮 / 音质标签（暂无功能，预留）
	store.set(onRequestOpenMenuAtom, { onEmit: () => {} })
	store.set(onClickAudioQualityTagAtom, { onEmit: () => {} })

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
	// 这是单向同步：Vue PlayMode 变化 → Jotai atoms。
	// 反向（用户点 AMLL 按钮）通过 onToggleShuffleAtom / onCycleRepeatModeAtom 回调处理，
	// 回调里 setPlayMode 后会再次触发这个 watcher，把权威状态写回 atoms。
	const stopPlayModeWatch = watch(
		() => PlayerStore.state.playMode,
		(mode) => {
			const { repeatMode, isShuffle } = playModeToAmll(mode)
			store.set(repeatModeAtom, repeatMode)
			store.set(isShuffleActiveAtom, isShuffle)
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

	// ── 6. Jotai → Vue 反向同步 ──
	// 已改用回调 atoms（onToggleShuffleAtom / onCycleRepeatModeAtom）作为用户点击入口，
	// 不再需要 store.sub 监听 repeatModeAtom / isShuffleActiveAtom。
	// 回调里直接基于 PlayerStore 当前状态计算新值并 setPlayMode，
	// Vue watcher 会把新 PlayMode 同步回 Jotai atoms，纠正 AMLL 内部状态。
	// 这样避免了 AMLL 内部初始化或程序化改 atom 时误触发反向同步。

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
		unbindSettings()
		stopAudioAnalyser()
	})

	return store
}
