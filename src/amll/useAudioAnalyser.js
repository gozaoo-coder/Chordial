/**
 * useAudioAnalyser — 将 HTMLAudioElement 接入 Web Audio API，
 * 实时计算低频音量（lowFreqVolume）和频域数据（fftData），
 * 推送给 AMLL Jotai store，驱动流体背景随音乐跳动。
 *
 * # 设计要点
 * - AnalyserNode 单例：audio 元素切换 src 时不需重建 AudioContext，
 *   只在首次调用时惰性创建。
 * - requestAnimationFrame 节流：浏览器原生对齐渲染帧，避免空转。
 * - 仅在 PlayerView 打开时启动分析，关闭后立即停止，零后台开销。
 * - fftSize = 256：32 个频段，足够驱动背景动画，CPU 开销极低。
 * - lowFreqVolume 取 80-120Hz 频段均值，归一化到 0-1。
 *
 * # CORS 注意
 * chordial:// 协议返回的音频流必须带 CORS 头，否则 crossOrigin="anonymous"
 * 的 audio 元素会被 Tauri WebView 拒绝读取频谱数据（ImageData tainted）。
 * 媒体协议层已在 to_tauri_response 注入 Access-Control-Allow-Origin: *。
 */
import { PlayerStore } from '@/stores/player.js'

// ── 常量 ──────────────────────────────────────────────────────
const FFT_SIZE = 256              // AnalyserNode.fftSize（必须是 2 的幂）
const BIN_COUNT = FFT_SIZE / 2    // 频段数 = 128，但只用前 32 个（低频段）
const SAMPLE_RATE = 44100         // 假设采样率；AnalyserNode 实际值会覆盖
const LOW_FREQ_HZ = 80            // 低频下限
const HIGH_FREQ_HZ = 120          // 低频上限（AMLL 推荐范围）
const SMOOTHING = 0.8             // AnalyserNode.smoothingTimeConstant

// ── 单例状态（模块级，多次调用 useAudioAnalyser 复用同一实例）──
let audioCtx = null
let analyser = null
let sourceNode = null
let freqDataBuffer = null         // Uint8Array(FFT_SIZE / 2)
let rafId = 0
let running = false

// 当前 audio 元素引用，用于判断是否需要重新绑定 sourceNode
let boundAudioEl = null

/**
 * 取得或创建 AudioContext + AnalyserNode 单例。
 * 之所以惰性创建：浏览器要求 AudioContext 必须由用户手势触发 resume，
 * 在页面加载时就建会导致 context state = 'suspended'。
 */
function ensureContext() {
	if (audioCtx) return audioCtx
	// 兼容 webkitAudioContext（旧 iOS）
	const Ctor = window.AudioContext || window.webkitAudioContext
	if (!Ctor) {
		console.warn('[useAudioAnalyser] Web Audio API 不可用')
		return null
	}
	audioCtx = new Ctor()
	analyser = audioCtx.createAnalyser()
	analyser.fftSize = FFT_SIZE
	analyser.smoothingTimeConstant = SMOOTHING
	freqDataBuffer = new Uint8Array(analyser.frequencyBinCount)
	return audioCtx
}

/**
 * 把 audio 元素接到 AnalyserNode 上。
 * MediaElementSourceNode 一旦创建就被"消耗"，同一元素不能 createMediaElementSource 两次，
 * 所以缓存 boundAudioEl 避免重复绑定。
 */
function bindAudioElement(audioEl) {
	if (!audioCtx || !analyser) return
	if (boundAudioEl === audioEl) return

	// 如果之前绑过别的元素，先断开旧 sourceNode
	if (sourceNode) {
		try { sourceNode.disconnect() } catch {}
	}

	try {
		sourceNode = audioCtx.createMediaElementSource(audioEl)
		sourceNode.connect(analyser)
		analyser.connect(audioCtx.destination)
		boundAudioEl = audioEl
	} catch (err) {
		// 常见错误：同一元素被重复 createMediaElementSource
		console.warn('[useAudioAnalyser] 绑定 audio 元素失败:', err)
	}
}

/**
 * 从频域数据计算 lowFreqVolume（0-1）。
 * AMLL 期望 80-120Hz 范围的音量均值，用于背景鼓点跳动。
 */
function computeLowFreqVolume(freqData) {
	if (!audioCtx || !analyser) return 0
	const nyquist = audioCtx.sampleRate / 2
	const binHz = nyquist / freqData.length
	const lowBin = Math.max(1, Math.floor(LOW_FREQ_HZ / binHz))
	const highBin = Math.min(freqData.length - 1, Math.ceil(HIGH_FREQ_HZ / binHz))

	let sum = 0
	let count = 0
	for (let i = lowBin; i <= highBin; i++) {
		sum += freqData[i]
		count++
	}
	if (count === 0) return 0
	// Uint8Array 取值 0-255，归一化到 0-1
	return sum / count / 255
}

/**
 * 抽取 fftData：取前 BIN_COUNT 个频段，归一化到 0-1。
 * AMLL 的 fftDataAtom 期望 number[]，长度无严格要求（一般 32-64）。
 */
function computeFftData(freqData) {
	// 取前 32 个频段（低中频，对背景可视化最有信息量）
	const out = new Array(BIN_COUNT)
	for (let i = 0; i < BIN_COUNT; i++) {
		out[i] = freqData[i] / 255
	}
	return out
}

/**
 * 启动音频分析循环。
 * @param {import('jotai').Store} store - AMLL Jotai store
 * @param {object} atoms - 需要写入的 atoms 引用
 * @param {import('@applemusic-like-lyrics/react-full').PrimitiveAtom<number>} atoms.lowFreqVolumeAtom
 * @param {import('@applemusic-like-lyrics/react-full').PrimitiveAtom<number[]>} atoms.fftDataAtom
 */
function startLoop(store, atoms) {
	if (running) return
	running = true

	const tick = () => {
		if (!running || !analyser || !freqDataBuffer) {
			running = false
			return
		}
		analyser.getByteFrequencyData(freqDataBuffer)
		store.set(atoms.lowFreqVolumeAtom, computeLowFreqVolume(freqDataBuffer))
		store.set(atoms.fftDataAtom, computeFftData(freqDataBuffer))
		rafId = requestAnimationFrame(tick)
	}
	rafId = requestAnimationFrame(tick)
}

/** 停止分析循环 */
function stopLoop() {
	running = false
	if (rafId) {
		cancelAnimationFrame(rafId)
		rafId = 0
	}
}

/**
 * 启动音频分析（PlayerView 挂载时调用）。
 * 会自动接管 PlayerStore 的 audioElement。
 */
export function startAudioAnalyser(store, atoms) {
	const audioEl = PlayerStore.getAudioElement()
	if (!audioEl) {
		console.warn('[useAudioAnalyser] PlayerStore.audioElement 尚未就绪')
		return
	}
	const ctx = ensureContext()
	if (!ctx) return

	// AudioContext 可能因自动播放策略处于 suspended 状态
	if (ctx.state === 'suspended') {
		ctx.resume().catch(() => {})
	}

	// audioEl crossOrigin 必须设为 anonymous 才能读取频谱数据
	// （Tauri 协议层已返回 CORS 头）
	if (!audioEl.crossOrigin) audioEl.crossOrigin = 'anonymous'

	bindAudioElement(audioEl)
	startLoop(store, atoms)
}

/** 停止音频分析（PlayerView 卸载时调用） */
export function stopAudioAnalyser() {
	stopLoop()
}

export default { startAudioAnalyser, stopAudioAnalyser }
