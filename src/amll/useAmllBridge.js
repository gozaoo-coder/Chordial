/**
 * useAmllBridge — Vue ↔ React(AMLL) 状态桥接层
 *
 * 核心设计：
 *  1. 创建 Jotai store 作为 Vue ↔ React 通信层
 *  2. Vue watchers 将 PlayerStore 状态推送到 Jotai atoms（单向）
 *  3. Jotai store.sub 监听 shuffle/repeat 变化回写 Vue（反向，用于 UI 按钮）
 *  4. 回调 atoms 一次性绑定到 PlayerStore actions
 *  5. 配置 atoms 一次性设置默认值
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
	musicLyricLinesAtom, lowFreqVolumeAtom,
	// 回调 atoms
	onPlayOrResumeAtom, onRequestPrevSongAtom, onRequestNextSongAtom,
	onSeekPositionAtom, onChangeVolumeAtom,
	onRequestOpenMenuAtom, onClickControlThumbAtom,
	// 控制 atoms
	repeatModeAtom, isShuffleActiveAtom, isShuffleEnabledAtom, isRepeatEnabledAtom,
	// 配置 atoms
	enableLyricLineBlurEffectAtom, enableLyricLineScaleEffectAtom,
	enableLyricLineSpringAnimationAtom, enableLyricTranslationLineAtom,
	enableLyricRomanLineAtom, lyricWordFadeWidthAtom,
	showMusicNameAtom, showMusicArtistsAtom, showMusicAlbumAtom,
	showVolumeControlAtom, showBottomControlAtom, showRemainingTimeAtom,
	isLyricPageOpenedAtom, hideLyricViewAtom,
	verticalCoverLayoutAtom, lyricBackgroundFPSAtom,
	lyricBackgroundRenderScaleAtom, lyricBackgroundStaticModeAtom,
	lyricSizePresetAtom, playerControlsTypeAtom,
	// 枚举
	RepeatMode, VerticalCoverLayout, LyricSizePreset, PlayerControlsType,
} from '@applemusic-like-lyrics/react-full'
import { PlayerStore, PlayMode } from '@/stores/player.js'
import { parseLyrics } from '@/utils/lyricConverter.js'

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
 * 创建并初始化 AMLL Jotai store，建立 Vue ↔ React 双向同步
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

	// ── 2. 配置 atoms 默认值（一次性）──
	store.set(enableLyricLineBlurEffectAtom, true)
	store.set(enableLyricLineScaleEffectAtom, true)
	store.set(enableLyricLineSpringAnimationAtom, true)
	store.set(enableLyricTranslationLineAtom, true)
	store.set(enableLyricRomanLineAtom, false)
	store.set(lyricWordFadeWidthAtom, 0.12)
	store.set(showMusicNameAtom, true)
	store.set(showMusicArtistsAtom, true)
	store.set(showMusicAlbumAtom, true)
	store.set(showVolumeControlAtom, true)
	store.set(showBottomControlAtom, true)
	store.set(showRemainingTimeAtom, true)
	store.set(isLyricPageOpenedAtom, true)
	store.set(hideLyricViewAtom, false)
	store.set(verticalCoverLayoutAtom, VerticalCoverLayout.Auto)
	store.set(lyricSizePresetAtom, LyricSizePreset.Medium)
	store.set(playerControlsTypeAtom, PlayerControlsType.Controls)
	store.set(lyricBackgroundFPSAtom, 60)
	store.set(lyricBackgroundRenderScaleAtom, 1)
	store.set(lyricBackgroundStaticModeAtom, false)
	store.set(isShuffleEnabledAtom, true)
	store.set(isRepeatEnabledAtom, true)
	store.set(musicCoverIsVideoAtom, false)
	store.set(lowFreqVolumeAtom, 0.5)

	// ── 3. Vue → Jotai 状态同步（watchers）──

	// 当前曲目变化 → 更新所有元数据
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

			// 异步加载封面
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

	// ── 4. Jotai → Vue 反向同步（shuffle/repeat 按钮点击）──
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

	// ── 5. 清理 ──
	onScopeDispose(() => {
		stopTrackWatch()
		stopPlayingWatch()
		stopPositionWatch()
		stopDurationWatch()
		stopVolumeWatch()
		stopLyricsWatch()
		stopPlayModeWatch()
		unsubRepeat()
		unsubShuffle()
	})

	return store
}
