use crate::audio_engine::buffer::DoubleBuffer;
use crate::audio_engine::decoder::{SymphoniaDecoder, AudioFrame};
use crate::audio_engine::time_stretch::{TimeStretcher, BpmSyncManager, PhaseSync};
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

/// 交叉淡化配置
#[derive(Clone, Debug)]
pub struct CrossfadeConfig {
    /// 交叉淡化持续时间（秒）
    pub duration_secs: f32,
    /// 曲线类型
    pub curve: CrossfadeCurve,
}

impl Default for CrossfadeConfig {
    fn default() -> Self {
        Self {
            duration_secs: 10.0, // 默认10秒交叉淡化
            curve: CrossfadeCurve::Linear,
        }
    }
}

/// 交叉淡化曲线类型
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CrossfadeCurve {
    /// 线性
    Linear,
    /// 对数（更自然的听觉感受）
    Logarithmic,
    /// S型曲线（平滑过渡）
    SCurve,
}

impl CrossfadeCurve {
    /// 计算淡化曲线值
    /// progress: 0.0 - 1.0
    /// is_fade_in: true 表示淡入，false 表示淡出
    pub fn calculate(&self, progress: f32, is_fade_in: bool) -> f32 {
        let p = progress.clamp(0.0, 1.0);
        
        let factor = match self {
            CrossfadeCurve::Linear => p,
            CrossfadeCurve::Logarithmic => {
                // 对数曲线: 20 * log10(p)
                // 简化为平方根曲线以获得类似效果
                p.sqrt()
            }
            CrossfadeCurve::SCurve => {
                // S型曲线: 平滑的淡入淡出
                let smooth = p * p * (3.0 - 2.0 * p);
                smooth
            }
        };

        if is_fade_in {
            factor
        } else {
            1.0 - factor
        }
    }
}

/// 播放轨道信息
pub struct Track {
    /// 解码器
    decoder: Option<SymphoniaDecoder>,
    /// 文件路径
    path: String,
    /// 总时长（秒）
    duration: f32,
    /// 当前位置（秒）
    position: AtomicU32,
    /// 是否已预加载完成
    is_preloaded: AtomicBool,
    /// BPM
    bpm: Option<f64>,
    /// Beat位置（秒）
    beat_positions: Vec<f64>,
    /// 时间拉伸处理器
    time_stretcher: Option<TimeStretcher>,
}

impl Track {
    /// 从文件路径创建轨道
    pub fn new<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        let decoder = SymphoniaDecoder::new(&path_str)?;
        let duration = decoder.duration().unwrap_or(0.0) as f32;
        let sample_rate = decoder.sample_rate();
        let channels = decoder.channels();

        // 时间拉伸器初始化失败不应该阻止轨道加载
        let time_stretcher = match TimeStretcher::new(sample_rate, channels as usize) {
            Ok(stretcher) => Some(stretcher),
            Err(e) => {
                eprintln!("Warning: Failed to create time stretcher for {}: {}", path_str, e);
                None
            }
        };

        Ok(Self {
            decoder: Some(decoder),
            path: path_str,
            duration,
            position: AtomicU32::new(0.0f32.to_bits()),
            is_preloaded: AtomicBool::new(false),
            bpm: None,
            beat_positions: Vec::new(),
            time_stretcher,
        })
    }

    /// 设置BPM和Beat位置
    pub fn set_bpm_info(&mut self, bpm: f64, beat_positions: Vec<f64>) {
        self.bpm = Some(bpm);
        self.beat_positions = beat_positions;
    }

    /// 获取BPM
    pub fn bpm(&self) -> Option<f64> {
        self.bpm
    }

    /// 设置播放速度
    pub fn set_speed(&mut self, speed_ratio: f64) -> anyhow::Result<()> {
        if let Some(ref mut stretcher) = self.time_stretcher {
            stretcher.set_speed(speed_ratio)?;
        }
        Ok(())
    }

    /// 获取当前播放速度
    pub fn speed_ratio(&self) -> f64 {
        self.time_stretcher.as_ref()
            .map(|s| s.speed_ratio())
            .unwrap_or(1.0)
    }

    /// 获取文件路径
    pub fn path(&self) -> &str {
        &self.path
    }

    /// 获取总时长
    pub fn duration(&self) -> f32 {
        self.duration
    }

    /// 获取当前位置
    pub fn position(&self) -> f32 {
        f32::from_bits(self.position.load(Ordering::Relaxed))
    }

    /// 设置当前位置
    pub fn set_position(&self, position: f32) {
        self.position.store(position.to_bits(), Ordering::Relaxed);
    }

    /// 解码下一帧
    pub fn next_frame(&mut self) -> anyhow::Result<Option<AudioFrame>> {
        if let Some(ref mut decoder) = self.decoder {
            let frame = decoder.next_frame()?;
            if let Some(ref f) = frame {
                self.set_position(f.timestamp as f32);
            }
            Ok(frame)
        } else {
            Ok(None)
        }
    }

    /// 跳转到指定位置
    pub fn seek(&mut self, position: f64) -> anyhow::Result<()> {
        if let Some(ref mut decoder) = self.decoder {
            decoder.seek(position)?;
            self.set_position(position as f32);
            Ok(())
        } else {
            Err(anyhow::anyhow!("No decoder available"))
        }
    }

    /// 获取剩余时间（秒）
    pub fn remaining(&self) -> f32 {
        self.duration - self.position()
    }

    /// 检查是否已预加载
    pub fn is_preloaded(&self) -> bool {
        self.is_preloaded.load(Ordering::Relaxed)
    }

    /// 标记为已预加载
    pub fn mark_preloaded(&self) {
        self.is_preloaded.store(true, Ordering::Relaxed);
    }
}

/// 混音器状态
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MixerState {
    /// 空闲
    Idle,
    /// 正在播放
    Playing,
    /// 正在预加载
    Preloading,
    /// 正在交叉淡化
    Crossfading,
    /// 已暂停
    Paused,
}

/// 音频混音器
/// 管理双轨道播放和交叉淡化
/// 
/// # 锁顺序规则
/// 当需要同时获取多个锁时，必须按以下顺序：
/// 1. current_track
/// 2. next_track
/// 3. crossfade_config
/// 4. bpm_sync / phase_sync
/// 
/// 违反此顺序可能导致死锁！
pub struct Mixer {
    /// 当前播放轨道（锁顺序：第1）
    current_track: Arc<Mutex<Option<Track>>>,
    /// 下一首预加载轨道（锁顺序：第2）
    next_track: Arc<Mutex<Option<Track>>>,
    /// 双缓冲区
    double_buffer: Arc<DoubleBuffer>,
    /// 当前状态
    state: AtomicU32,
    /// 交叉淡化配置（锁顺序：第3）
    crossfade_config: Mutex<CrossfadeConfig>,
    /// 音量 (0.0 - 1.0)
    volume: AtomicU32,
    /// 预加载触发阈值（秒）
    preload_threshold: AtomicU32, // 默认10秒
    /// BPM同步管理器（锁顺序：第4）
    bpm_sync: Mutex<BpmSyncManager>,
    /// 相位同步器（锁顺序：第4）
    phase_sync: Mutex<PhaseSync>,
    /// 是否启用BPM同步
    bpm_sync_enabled: AtomicBool,
}

// 状态常量
const MIXER_STATE_IDLE: u32 = 0;
const MIXER_STATE_PLAYING: u32 = 1;
const MIXER_STATE_PRELOADING: u32 = 2;
const MIXER_STATE_CROSSFADING: u32 = 3;
const MIXER_STATE_PAUSED: u32 = 4;

impl Mixer {
    /// 创建新的混音器
    pub fn new(sample_rate: u32, channels: u16) -> Self {
        Self {
            current_track: Arc::new(Mutex::new(None)),
            next_track: Arc::new(Mutex::new(None)),
            double_buffer: Arc::new(DoubleBuffer::new(
                (sample_rate as usize * 5) / 1024, // 约5秒容量
                sample_rate,
                channels,
            )),
            state: AtomicU32::new(MIXER_STATE_IDLE),
            crossfade_config: Mutex::new(CrossfadeConfig::default()),
            volume: AtomicU32::new(1.0f32.to_bits()),
            preload_threshold: AtomicU32::new(10.0f32.to_bits()), // 默认10秒
            bpm_sync: Mutex::new(BpmSyncManager::new(sample_rate)),
            phase_sync: Mutex::new(PhaseSync::new()),
            bpm_sync_enabled: AtomicBool::new(false),
        }
    }

    /// 设置当前轨道BPM信息
    pub fn set_current_track_bpm(&self, bpm: f64, beat_positions: Vec<f64>) {
        let Ok(mut current) = self.current_track.lock() else {
            eprintln!("Failed to lock current_track for BPM update");
            return;
        };
        if let Some(ref mut track) = *current {
            track.set_bpm_info(bpm, beat_positions);
            
            // 更新BPM同步管理器
            if let Ok(mut bpm_sync) = self.bpm_sync.lock() {
                bpm_sync.set_master_bpm(bpm);
            }
        }
    }

    /// 设置下一首轨道BPM信息
    pub fn set_next_track_bpm(&self, bpm: f64, beat_positions: Vec<f64>) {
        let Ok(mut next) = self.next_track.lock() else {
            eprintln!("Failed to lock next_track for BPM update");
            return;
        };
        if let Some(ref mut track) = *next {
            track.set_bpm_info(bpm, beat_positions);
            
            // 更新BPM同步管理器
            if let Ok(mut bpm_sync) = self.bpm_sync.lock() {
                bpm_sync.set_slave_bpm(bpm);
            }
        }
    }

    /// 启用/禁用BPM同步
    pub fn set_bpm_sync_enabled(&self, enabled: bool) {
        self.bpm_sync_enabled.store(enabled, Ordering::Relaxed);
        
        let Ok(mut bpm_sync) = self.bpm_sync.lock() else {
            eprintln!("Failed to lock bpm_sync");
            return;
        };
        bpm_sync.set_sync_enabled(enabled);
        
        // 应用速度调整
        if enabled {
            let speed_ratio = bpm_sync.get_speed_ratio();
            drop(bpm_sync); // 释放锁
            
            if let Ok(mut next) = self.next_track.lock() {
                if let Some(ref mut track) = *next {
                    let _ = track.set_speed(speed_ratio);
                }
            }
        }
    }

    /// 检查BPM同步是否启用
    pub fn is_bpm_sync_enabled(&self) -> bool {
        self.bpm_sync_enabled.load(Ordering::Relaxed)
    }

    /// 获取当前速度比率
    pub fn get_speed_ratio(&self) -> f64 {
        let current = self.current_track.lock().unwrap();
        if let Some(ref track) = *current {
            track.speed_ratio()
        } else {
            1.0
        }
    }

    /// 获取双缓冲区
    pub fn double_buffer(&self) -> Arc<DoubleBuffer> {
        Arc::clone(&self.double_buffer)
    }

    /// 加载并播放轨道
    /// 
    /// # 锁顺序
    /// 遵循 Mixer 结构体文档中的锁顺序规则：current_track -> next_track
    pub fn load_track<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        let track = Track::new(path)?;
        
        // 清空缓冲区
        self.double_buffer.clear();
        
        // 设置当前轨道（按锁顺序获取）
        let mut current = self.current_track.lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock current_track: {}", e))?;
        *current = Some(track);
        
        // 清除下一首
        let mut next = self.next_track.lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock next_track: {}", e))?;
        *next = None;
        
        // 设置状态为播放中
        self.set_state(MixerState::Playing);
        
        // 重置位置
        self.state.store(MIXER_STATE_PLAYING, Ordering::Relaxed);
        
        Ok(())
    }

    /// 预加载下一首
    pub fn preload_next<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        let track = Track::new(path)?;
        
        let mut next = self.next_track.lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock next_track: {}", e))?;
        *next = Some(track);
        
        self.set_state(MixerState::Preloading);
        
        Ok(())
    }

    /// 开始交叉淡化
    pub fn start_crossfade(&self) {
        self.double_buffer.start_crossfade();
        self.set_state(MixerState::Crossfading);
    }

    /// 完成交叉淡化（切换到下一首）
    /// 
    /// # 锁顺序
    /// 遵循 Mixer 结构体文档中的锁顺序规则：current_track -> next_track
    pub fn complete_crossfade(&self) {
        // 交换轨道（按锁顺序获取：current -> next）
        let mut current = match self.current_track.lock() {
            Ok(guard) => guard,
            Err(e) => {
                eprintln!("Failed to lock current_track: {}", e);
                return;
            }
        };
        let mut next = match self.next_track.lock() {
            Ok(guard) => guard,
            Err(e) => {
                eprintln!("Failed to lock next_track: {}", e);
                return;
            }
        };
        
        *current = next.take();
        
        // 停止交叉淡化
        self.double_buffer.stop_crossfade();
        
        // 恢复播放状态
        if current.is_some() {
            self.set_state(MixerState::Playing);
        } else {
            self.set_state(MixerState::Idle);
        }
    }

    /// 检查是否需要预加载
    pub fn should_preload(&self) -> bool {
        // 先获取当前轨道信息
        let remaining = if let Ok(current) = self.current_track.lock() {
            if let Some(ref track) = *current {
                track.remaining()
            } else {
                return false;
            }
        } else {
            return false;
        };
        
        let threshold = self.get_preload_threshold();
        
        // 检查下一首是否已加载
        let next_is_none = match self.next_track.lock() {
            Ok(next) => next.is_none(),
            Err(_) => return false,
        };
        
        // 当剩余时间小于阈值且下一首未加载时，需要预加载
        remaining <= threshold && next_is_none
    }

    /// 检查是否应该开始交叉淡化
    pub fn should_start_crossfade(&self) -> bool {
        // 先获取当前轨道信息
        let remaining = if let Ok(current) = self.current_track.lock() {
            if let Some(ref track) = *current {
                track.remaining()
            } else {
                return false;
            }
        } else {
            return false;
        };
        
        // 获取交叉淡化配置
        let duration_secs = match self.crossfade_config.lock() {
            Ok(config) => config.duration_secs,
            Err(_) => return false,
        };
        
        // 检查下一首是否已加载
        let next_is_some = match self.next_track.lock() {
            Ok(next) => next.is_some(),
            Err(_) => return false,
        };
        
        // 当剩余时间小于交叉淡化持续时间时，开始交叉淡化
        // 且下一首已预加载完成
        remaining <= duration_secs && next_is_some && !self.double_buffer.is_crossfading()
    }

    /// 解码当前轨道的一帧到缓冲区
    pub fn decode_current_frame(&self) -> anyhow::Result<bool> {
        let mut current = match self.current_track.lock() {
            Ok(guard) => guard,
            Err(e) => {
                eprintln!("Failed to lock current_track: {}", e);
                return Ok(false);
            }
        };
        
        if let Some(ref mut track) = *current {
            match track.next_frame() {
                Ok(Some(frame)) => {
                    // 将帧数据转换为交错格式并推入缓冲区
                    let chunk = frame.samples;
                    match self.double_buffer.push_active(chunk) {
                        Ok(_) => Ok(true),
                        Err(_) => {
                            // 缓冲区满，稍后重试
                            Ok(true)
                        }
                    }
                }
                Ok(None) => {
                    // 当前轨道结束
                    Ok(false)
                }
                Err(e) => {
                    eprintln!("Decode error for current track: {}", e);
                    Ok(false)
                }
            }
        } else {
            Ok(false)
        }
    }

    /// 解码下一首轨道的一帧到预加载缓冲区
    pub fn decode_next_frame(&self) -> anyhow::Result<bool> {
        let mut next = match self.next_track.lock() {
            Ok(guard) => guard,
            Err(e) => {
                eprintln!("Failed to lock next_track: {}", e);
                return Ok(false);
            }
        };
        
        if let Some(ref mut track) = *next {
            match track.next_frame() {
                Ok(Some(frame)) => {
                    let chunk = frame.samples;
                    match self.double_buffer.push_preload(chunk) {
                        Ok(_) => Ok(true),
                        Err(_) => Ok(true), // 缓冲区满
                    }
                }
                Ok(None) => {
                    track.mark_preloaded();
                    Ok(false)
                }
                Err(e) => {
                    eprintln!("Decode error for next track: {}", e);
                    Ok(false)
                }
            }
        } else {
            Ok(false)
        }
    }

    /// 获取混合输出
    pub fn get_output(&self, requested_samples: usize) -> Vec<f32> {
        // 检查当前状态，如果不是播放状态则返回静音
        let state = self.get_state();
        if state == MixerState::Idle || state == MixerState::Paused {
            return vec![0.0f32; requested_samples];
        }
        
        let samples = self.double_buffer.mix_output(requested_samples);
        
        // 应用音量
        let volume = self.get_volume();
        samples.into_iter().map(|s| s * volume).collect()
    }

    /// 设置状态
    pub fn set_state(&self, state: MixerState) {
        let value = match state {
            MixerState::Idle => MIXER_STATE_IDLE,
            MixerState::Playing => MIXER_STATE_PLAYING,
            MixerState::Preloading => MIXER_STATE_PRELOADING,
            MixerState::Crossfading => MIXER_STATE_CROSSFADING,
            MixerState::Paused => MIXER_STATE_PAUSED,
        };
        self.state.store(value, Ordering::Relaxed);
    }

    /// 获取当前状态
    pub fn get_state(&self) -> MixerState {
        match self.state.load(Ordering::Relaxed) {
            MIXER_STATE_PLAYING => MixerState::Playing,
            MIXER_STATE_PRELOADING => MixerState::Preloading,
            MIXER_STATE_CROSSFADING => MixerState::Crossfading,
            MIXER_STATE_PAUSED => MixerState::Paused,
            _ => MixerState::Idle,
        }
    }

    /// 暂停
    pub fn pause(&self) {
        self.set_state(MixerState::Paused);
    }

    /// 恢复播放
    pub fn resume(&self) {
        self.set_state(MixerState::Playing);
    }

    /// 停止
    pub fn stop(&self) {
        self.double_buffer.clear();
        
        // 按锁顺序获取锁
        if let Ok(mut current) = self.current_track.lock() {
            *current = None;
        }
        if let Ok(mut next) = self.next_track.lock() {
            *next = None;
        }
        self.set_state(MixerState::Idle);
    }

    /// 设置音量
    pub fn set_volume(&self, volume: f32) {
        self.volume.store(volume.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
    }

    /// 获取音量
    pub fn get_volume(&self) -> f32 {
        f32::from_bits(self.volume.load(Ordering::Relaxed))
    }

    /// 设置预加载阈值
    pub fn set_preload_threshold(&self, seconds: f32) {
        self.preload_threshold.store(seconds.to_bits(), Ordering::Relaxed);
    }

    /// 获取预加载阈值
    pub fn get_preload_threshold(&self) -> f32 {
        f32::from_bits(self.preload_threshold.load(Ordering::Relaxed))
    }

    /// 设置交叉淡化配置
    pub fn set_crossfade_config(&self, config: CrossfadeConfig) {
        if let Ok(mut cfg) = self.crossfade_config.lock() {
            *cfg = config;
        }
    }

    /// 获取交叉淡化配置
    pub fn get_crossfade_config(&self) -> Option<CrossfadeConfig> {
        self.crossfade_config.lock().ok().map(|cfg| cfg.clone())
    }

    /// 获取当前轨道位置
    pub fn current_position(&self) -> f32 {
        self.current_track.lock()
            .ok()
            .and_then(|current| current.as_ref().map(|track| track.position()))
            .unwrap_or(0.0)
    }

    /// 获取当前轨道时长
    pub fn current_duration(&self) -> f32 {
        self.current_track.lock()
            .ok()
            .and_then(|current| current.as_ref().map(|track| track.duration()))
            .unwrap_or(0.0)
    }

    /// 获取当前轨道路径
    pub fn current_path(&self) -> Option<String> {
        self.current_track.lock()
            .ok()
            .and_then(|current| current.as_ref().map(|track| track.path().to_string()))
    }

    /// 检查是否正在播放
    pub fn is_playing(&self) -> bool {
        matches!(self.get_state(), MixerState::Playing | MixerState::Preloading | MixerState::Crossfading)
    }

    /// 检查是否已暂停
    pub fn is_paused(&self) -> bool {
        self.get_state() == MixerState::Paused
    }
}

/// 混音器控制器
/// 用于在音频线程中控制混音器
#[derive(Clone)]
pub struct MixerController {
    mixer: Arc<Mixer>,
}

impl MixerController {
    /// 创建新的控制器
    pub fn new(mixer: Arc<Mixer>) -> Self {
        Self { mixer }
    }

    /// 获取输出样本
    pub fn get_samples(&self, count: usize) -> Vec<f32> {
        self.mixer.get_output(count)
    }

    /// 检查是否需要预加载
    pub fn should_preload(&self) -> bool {
        self.mixer.should_preload()
    }

    /// 检查是否应该开始交叉淡化
    pub fn should_start_crossfade(&self) -> bool {
        self.mixer.should_start_crossfade()
    }

    /// 开始交叉淡化
    pub fn start_crossfade(&self) {
        self.mixer.start_crossfade();
    }

    /// 解码当前轨道一帧
    pub fn decode_current(&self) -> anyhow::Result<bool> {
        self.mixer.decode_current_frame()
    }

    /// 解码下一首一帧
    pub fn decode_next(&self) -> anyhow::Result<bool> {
        self.mixer.decode_next_frame()
    }

    /// 完成交叉淡化
    pub fn complete_crossfade(&self) {
        self.mixer.complete_crossfade();
    }

    /// 获取当前状态
    pub fn state(&self) -> MixerState {
        self.mixer.get_state()
    }

    /// 设置当前轨道BPM信息
    pub fn set_current_bpm(&self, bpm: f64, beat_positions: Vec<f64>) {
        self.mixer.set_current_track_bpm(bpm, beat_positions);
    }

    /// 设置下一首轨道BPM信息
    pub fn set_next_bpm(&self, bpm: f64, beat_positions: Vec<f64>) {
        self.mixer.set_next_track_bpm(bpm, beat_positions);
    }

    /// 启用/禁用BPM同步
    pub fn set_bpm_sync(&self, enabled: bool) {
        self.mixer.set_bpm_sync_enabled(enabled);
    }

    /// 检查BPM同步是否启用
    pub fn is_bpm_sync(&self) -> bool {
        self.mixer.is_bpm_sync_enabled()
    }

    /// 获取当前速度比率
    pub fn speed_ratio(&self) -> f64 {
        self.mixer.get_speed_ratio()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crossfade_curve() {
        let curve = CrossfadeCurve::Linear;
        assert_eq!(curve.calculate(0.0, true), 0.0);
        assert_eq!(curve.calculate(1.0, true), 1.0);
        assert_eq!(curve.calculate(0.5, true), 0.5);

        assert_eq!(curve.calculate(0.0, false), 1.0);
        assert_eq!(curve.calculate(1.0, false), 0.0);
        assert_eq!(curve.calculate(0.5, false), 0.5);
    }

    #[test]
    fn test_mixer_creation() {
        let mixer = Mixer::new(48000, 2);
        assert_eq!(mixer.get_state(), MixerState::Idle);
        assert_eq!(mixer.get_volume(), 1.0);
        assert_eq!(mixer.get_preload_threshold(), 10.0);
    }
}
