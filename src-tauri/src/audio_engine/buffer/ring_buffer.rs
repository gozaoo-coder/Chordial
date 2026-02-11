use ringbuf::traits::{Consumer, Producer};
use ringbuf::HeapRb;
use std::sync::{Arc, Mutex};

/// 默认交叉淡化持续时间（秒）
const DEFAULT_CROSSFADE_DURATION_SECS: f32 = 10.0;

/// 默认缓冲区容量（秒）
const DEFAULT_BUFFER_CAPACITY_SECS: u32 = 5;

/// 默认音频块大小（样本数）
const DEFAULT_CHUNK_SIZE: usize = 1024;

/// 音频样本块
pub type AudioChunk = Vec<f32>;

/// 双缓冲音频队列
/// 使用两个 RingBuffer 实现无缝切换：
/// - Buffer A: 当前播放轨道
/// - Buffer B: 预加载轨道（下一首）
pub struct DoubleBuffer {
    /// 当前播放缓冲区
    buffer_a: Arc<Mutex<HeapRb<AudioChunk>>>,
    /// 预加载缓冲区
    buffer_b: Arc<Mutex<HeapRb<AudioChunk>>>,
    /// 当前活跃缓冲区 (true = A, false = B)
    active_a: std::sync::atomic::AtomicBool,
    /// 是否正在交叉淡化
    is_crossfading: std::sync::atomic::AtomicBool,
    /// 交叉淡化进度 (0.0 - 1.0)
    crossfade_progress: std::sync::atomic::AtomicU32,
    /// 采样率
    sample_rate: u32,
    /// 声道数
    channels: u16,
}

impl DoubleBuffer {
    /// 创建新的双缓冲区
    /// capacity: 每个缓冲区的块数容量
    pub fn new(capacity: usize, sample_rate: u32, channels: u16) -> Self {
        Self {
            buffer_a: Arc::new(Mutex::new(HeapRb::new(capacity))),
            buffer_b: Arc::new(Mutex::new(HeapRb::new(capacity))),
            active_a: std::sync::atomic::AtomicBool::new(true),
            is_crossfading: std::sync::atomic::AtomicBool::new(false),
            crossfade_progress: std::sync::atomic::AtomicU32::new(0),
            sample_rate,
            channels,
        }
    }

    /// 获取采样率
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// 获取声道数
    pub fn channels(&self) -> u16 {
        self.channels
    }

    /// 向当前活跃缓冲区推送音频块
    pub fn push_active(&self, chunk: AudioChunk) -> Result<(), AudioChunk> {
        let buffer = if self.active_a.load(std::sync::atomic::Ordering::Relaxed) {
            &self.buffer_a
        } else {
            &self.buffer_b
        };

        let mut buf = buffer.lock().unwrap();
        match buf.try_push(chunk) {
            Ok(_) => Ok(()),
            Err(chunk) => Err(chunk),
        }
    }

    /// 向预加载缓冲区推送音频块
    pub fn push_preload(&self, chunk: AudioChunk) -> Result<(), AudioChunk> {
        let buffer = if self.active_a.load(std::sync::atomic::Ordering::Relaxed) {
            &self.buffer_b
        } else {
            &self.buffer_a
        };

        let mut buf = buffer.lock().unwrap();
        match buf.try_push(chunk) {
            Ok(_) => Ok(()),
            Err(chunk) => Err(chunk),
        }
    }

    /// 从当前活跃缓冲区弹出音频块
    pub fn pop_active(&self) -> Option<AudioChunk> {
        let buffer = if self.active_a.load(std::sync::atomic::Ordering::Relaxed) {
            &self.buffer_a
        } else {
            &self.buffer_b
        };

        let mut buf = buffer.lock().unwrap();
        buf.try_pop()
    }

    /// 从预加载缓冲区弹出音频块
    pub fn pop_preload(&self) -> Option<AudioChunk> {
        let buffer = if self.active_a.load(std::sync::atomic::Ordering::Relaxed) {
            &self.buffer_b
        } else {
            &self.buffer_a
        };

        let mut buf = buffer.lock().unwrap();
        buf.try_pop()
    }

    /// 切换活跃缓冲区（用于无缝切换）
    pub fn swap_buffers(&self) {
        let current = self.active_a.load(std::sync::atomic::Ordering::Relaxed);
        self.active_a.store(!current, std::sync::atomic::Ordering::Relaxed);
    }

    /// 开始交叉淡化
    pub fn start_crossfade(&self) {
        self.is_crossfading.store(true, std::sync::atomic::Ordering::Relaxed);
        self.crossfade_progress.store(0, std::sync::atomic::Ordering::Relaxed);
    }

    /// 停止交叉淡化
    pub fn stop_crossfade(&self) {
        self.is_crossfading.store(false, std::sync::atomic::Ordering::Relaxed);
        self.crossfade_progress.store(0, std::sync::atomic::Ordering::Relaxed);
    }

    /// 检查是否正在交叉淡化
    pub fn is_crossfading(&self) -> bool {
        self.is_crossfading.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// 获取交叉淡化进度
    pub fn crossfade_progress(&self) -> f32 {
        f32::from_bits(self.crossfade_progress.load(std::sync::atomic::Ordering::Relaxed))
    }

    /// 更新交叉淡化进度
    pub fn advance_crossfade(&self, delta: f32) {
        let current = self.crossfade_progress();
        let new_progress = (current + delta).min(1.0);
        self.crossfade_progress.store(new_progress.to_bits(), std::sync::atomic::Ordering::Relaxed);

        if new_progress >= 1.0 {
            // 交叉淡化完成，切换缓冲区
            self.swap_buffers();
            self.stop_crossfade();
        }
    }

    /// 获取混合后的音频数据（用于交叉淡化期间）
    /// 返回的样本数是 requested_samples 和可用样本数的最小值
    /// 
    /// # 性能优化
    /// - 预分配输出向量容量，避免动态扩容
    /// - 使用索引循环替代迭代器，减少边界检查
    /// - 使用 resize 替代 extend + repeat 填充零值
    pub fn mix_output(&self, requested_samples: usize) -> Vec<f32> {
        if !self.is_crossfading() {
            // 非交叉淡化状态，只从活跃缓冲区读取
            if let Some(chunk) = self.pop_active() {
                let samples_to_take = requested_samples.min(chunk.len());
                if samples_to_take == chunk.len() {
                    chunk
                } else {
                    // 预分配容量并复制数据
                    let mut result = Vec::with_capacity(requested_samples);
                    result.extend_from_slice(&chunk[..samples_to_take]);
                    // 使用 resize 填充剩余部分
                    result.resize(requested_samples, 0.0);
                    result
                }
            } else {
                // 直接创建零值向量
                vec![0.0f32; requested_samples]
            }
        } else {
            // 交叉淡化状态，混合两个缓冲区
            let chunk_a = self.pop_active().unwrap_or_else(|| vec![0.0f32; requested_samples]);
            let chunk_b = self.pop_preload().unwrap_or_else(|| vec![0.0f32; requested_samples]);

            let progress = self.crossfade_progress();
            let samples_to_take = requested_samples.min(chunk_a.len()).min(chunk_b.len());

            // 预分配输出向量
            let mut result = Vec::with_capacity(requested_samples);
            let fade_out = 1.0 - progress;
            let fade_in = progress;

            // 使用索引循环混合音频（比迭代器更高效）
            for i in 0..samples_to_take {
                let mixed = chunk_a[i] * fade_out + chunk_b[i] * fade_in;
                result.push(mixed);
            }

            // 更新交叉淡化进度
            let crossfade_duration_samples = self.sample_rate as f32 * DEFAULT_CROSSFADE_DURATION_SECS;
            let delta = samples_to_take as f32 / crossfade_duration_samples;
            self.advance_crossfade(delta);

            // 如果还有剩余样本，需要重新推回缓冲区
            if chunk_a.len() > samples_to_take {
                let remaining_a: Vec<f32> = chunk_a.into_iter().skip(samples_to_take).collect();
                let _ = self.push_active(remaining_a);
            }
            if chunk_b.len() > samples_to_take {
                let remaining_b: Vec<f32> = chunk_b.into_iter().skip(samples_to_take).collect();
                let _ = self.push_preload(remaining_b);
            }

            // 使用 resize 填充剩余部分（比 extend + repeat 更高效）
            result.resize(requested_samples, 0.0);
            result
        }
    }

    /// 清空所有缓冲区
    pub fn clear(&self) {
        let mut buf_a = self.buffer_a.lock().unwrap();
        let mut buf_b = self.buffer_b.lock().unwrap();
        
        // 使用 try_pop 清空缓冲区
        while buf_a.try_pop().is_some() {}
        while buf_b.try_pop().is_some() {}
        
        self.active_a.store(true, std::sync::atomic::Ordering::Relaxed);
        self.stop_crossfade();
    }

    /// 获取当前活跃缓冲区的已用空间
    /// 注意：ringbuf 0.4 没有直接提供获取已用空间的方法
    /// 我们通过计数来估算
    pub fn active_buffer_len(&self) -> usize {
        // 返回一个估计值，实际实现需要额外的计数器
        // 这里简化处理，返回 0 表示未知
        0
    }

    /// 获取预加载缓冲区的已用空间
    pub fn preload_buffer_len(&self) -> usize {
        0
    }
}

/// 音频流状态
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StreamState {
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

/// 音频流管理器
pub struct AudioStreamManager {
    /// 双缓冲区
    double_buffer: Arc<DoubleBuffer>,
    /// 当前状态
    state: std::sync::atomic::AtomicU8,
}

// 状态常量
const STATE_IDLE: u8 = 0;
const STATE_PLAYING: u8 = 1;
const STATE_PRELOADING: u8 = 2;
const STATE_CROSSFADING: u8 = 3;
const STATE_PAUSED: u8 = 4;

impl AudioStreamManager {
    /// 创建新的音频流管理器
    pub fn new(sample_rate: u32, channels: u16) -> Self {
        // 每个缓冲区容量：约 DEFAULT_BUFFER_CAPACITY_SECS 秒的音频
        let capacity = (sample_rate as usize * DEFAULT_BUFFER_CAPACITY_SECS as usize) / DEFAULT_CHUNK_SIZE;

        Self {
            double_buffer: Arc::new(DoubleBuffer::new(capacity, sample_rate, channels)),
            state: std::sync::atomic::AtomicU8::new(STATE_IDLE),
        }
    }

    /// 获取双缓冲区引用
    pub fn double_buffer(&self) -> Arc<DoubleBuffer> {
        Arc::clone(&self.double_buffer)
    }

    /// 设置状态
    pub fn set_state(&self, state: StreamState) {
        let value = match state {
            StreamState::Idle => STATE_IDLE,
            StreamState::Playing => STATE_PLAYING,
            StreamState::Preloading => STATE_PRELOADING,
            StreamState::Crossfading => STATE_CROSSFADING,
            StreamState::Paused => STATE_PAUSED,
        };
        self.state.store(value, std::sync::atomic::Ordering::Relaxed);
    }

    /// 获取当前状态
    pub fn get_state(&self) -> StreamState {
        match self.state.load(std::sync::atomic::Ordering::Relaxed) {
            STATE_PLAYING => StreamState::Playing,
            STATE_PRELOADING => StreamState::Preloading,
            STATE_CROSSFADING => StreamState::Crossfading,
            STATE_PAUSED => StreamState::Paused,
            _ => StreamState::Idle,
        }
    }

    /// 开始播放
    pub fn start_playback(&self) {
        self.set_state(StreamState::Playing);
    }

    /// 开始预加载
    pub fn start_preloading(&self) {
        self.set_state(StreamState::Preloading);
    }

    /// 开始交叉淡化
    pub fn start_crossfade(&self) {
        self.double_buffer.start_crossfade();
        self.set_state(StreamState::Crossfading);
    }

    /// 暂停
    pub fn pause(&self) {
        self.set_state(StreamState::Paused);
    }

    /// 恢复播放
    pub fn resume(&self) {
        self.set_state(StreamState::Playing);
    }

    /// 停止并清空
    pub fn stop(&self) {
        self.double_buffer.clear();
        self.set_state(StreamState::Idle);
    }

    /// 检查是否正在播放
    pub fn is_playing(&self) -> bool {
        matches!(self.get_state(), StreamState::Playing | StreamState::Crossfading | StreamState::Preloading)
    }

    /// 检查是否正在预加载
    pub fn is_preloading(&self) -> bool {
        matches!(self.get_state(), StreamState::Preloading)
    }

    /// 检查是否正在交叉淡化
    pub fn is_crossfading(&self) -> bool {
        self.double_buffer.is_crossfading()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_double_buffer_creation() {
        let buffer = DoubleBuffer::new(100, 48000, 2);
        assert_eq!(buffer.sample_rate(), 48000);
        assert_eq!(buffer.channels(), 2);
        assert!(!buffer.is_crossfading());
    }

    #[test]
    fn test_buffer_push_pop() {
        let buffer = DoubleBuffer::new(10, 48000, 2);
        let chunk = vec![0.5f32; 1024];

        assert!(buffer.push_active(chunk.clone()).is_ok());
        // 注意：active_buffer_len() 返回 0 因为 ringbuf 0.4 没有 len() 方法
        // 我们直接测试 pop 是否能获取到数据

        let popped = buffer.pop_active();
        assert!(popped.is_some());
        assert_eq!(popped.unwrap().len(), 1024);
    }

    #[test]
    fn test_crossfade() {
        let buffer = DoubleBuffer::new(10, 48000, 2);

        // 填充两个缓冲区
        let chunk_a = vec![1.0f32; 1024];
        let chunk_b = vec![0.5f32; 1024];

        buffer.push_active(chunk_a).unwrap();
        buffer.push_preload(chunk_b).unwrap();

        // 开始交叉淡化
        buffer.start_crossfade();
        assert!(buffer.is_crossfading());

        // 获取混合输出
        let mixed = buffer.mix_output(512);
        assert_eq!(mixed.len(), 512);

        // 检查混合值（初始 progress=0，所以应该是 1.0）
        // 随着 progress 增加，值会逐渐接近 0.5
        assert!(mixed[0] >= 0.5 && mixed[0] <= 1.0);
        
        // 获取更多输出以推进交叉淡化进度
        let _ = buffer.mix_output(48000 * 5); // 推进约5秒的音频
        let mixed2 = buffer.mix_output(512);
        // 进度应该已经增加，值应该小于初始值
        assert!(mixed2[0] <= mixed[0]);
    }
}
