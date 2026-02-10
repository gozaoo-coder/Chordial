use rubato::{SincFixedIn, Resampler, SincInterpolationType, SincInterpolationParameters, WindowFunction};

/// 时间拉伸处理器 - 使用 Rubato 进行高质量音频重采样
pub struct TimeStretcher {
    /// 重采样器
    resampler: SincFixedIn<f32>,
    /// 当前速度比率 (1.0 = 原速)
    speed_ratio: f64,
    /// 输入采样率
    input_sample_rate: u32,
    /// 输出采样率
    output_sample_rate: u32,
    /// 声道数
    channels: usize,
}

impl TimeStretcher {
    /// 创建新的时间拉伸处理器
    /// 
    /// # Arguments
    /// * `input_sample_rate` - 输入音频采样率
    /// * `channels` - 声道数
    pub fn new(input_sample_rate: u32, channels: usize) -> anyhow::Result<Self> {
        let speed_ratio = 1.0;
        let output_sample_rate = input_sample_rate;
        
        // 创建 Sinc 重采样器（高质量）
        let resampler = Self::create_resampler(
            speed_ratio,
            input_sample_rate,
            channels,
        )?;
        
        Ok(Self {
            resampler,
            speed_ratio,
            input_sample_rate,
            output_sample_rate,
            channels,
        })
    }
    
    /// 创建重采样器
    fn create_resampler(
        speed_ratio: f64,
        sample_rate: u32,
        channels: usize,
    ) -> anyhow::Result<SincFixedIn<f32>> {
        // 计算重采样比率
        // 速度 > 1.0 表示加速（输出采样率 > 输入采样率）
        // 速度 < 1.0 表示减速（输出采样率 < 输入采样率）
        let resample_ratio = 1.0 / speed_ratio;
        
        let params = SincInterpolationParameters {
            sinc_len: 256,
            f_cutoff: 0.95,
            interpolation: SincInterpolationType::Linear,
            oversampling_factor: 256,
            window: WindowFunction::BlackmanHarris2,
        };
        
        SincFixedIn::new(
            resample_ratio,
            2.0,  // 最大重采样比率
            params,
            1024, // 块大小
            channels,
        ).map_err(|e| anyhow::anyhow!("创建重采样器失败: {}", e))
    }
    
    /// 设置播放速度
    /// 
    /// # Arguments
    /// * `speed_ratio` - 速度比率 (0.5 - 2.0, 1.0 = 原速)
    /// 
    /// 当速度变化超过 3% 时，重新创建重采样器
    pub fn set_speed(&mut self, speed_ratio: f64) -> anyhow::Result<()> {
        let speed_ratio = speed_ratio.clamp(0.5, 2.0);
        
        // 如果变化小于 3%，不重新创建
        if (speed_ratio - self.speed_ratio).abs() < 0.03 {
            return Ok(());
        }
        
        self.speed_ratio = speed_ratio;
        self.output_sample_rate = (self.input_sample_rate as f64 * speed_ratio) as u32;
        
        // 重新创建重采样器
        self.resampler = Self::create_resampler(
            speed_ratio,
            self.input_sample_rate,
            self.channels,
        )?;
        
        Ok(())
    }
    
    /// 获取当前速度比率
    pub fn speed_ratio(&self) -> f64 {
        self.speed_ratio
    }
    
    /// 处理音频数据
    /// 
    /// # Arguments
    /// * `input` - 输入音频数据（交错格式）
    /// 
    /// # Returns
    /// 处理后的音频数据
    pub fn process(&mut self, input: &[f32]) -> anyhow::Result<Vec<f32>> {
        if input.is_empty() {
            return Ok(Vec::new());
        }
        
        // 确保输入数据长度是声道数的整数倍
        let frame_count = input.len() / self.channels;
        let valid_input_len = frame_count * self.channels;
        let input = &input[..valid_input_len];
        
        // 将交错数据转换为平面格式（Rubato 要求）
        let mut wave_in: Vec<Vec<f32>> = vec![Vec::with_capacity(frame_count); self.channels];
        for (i, sample) in input.iter().enumerate() {
            let channel = i % self.channels;
            wave_in[channel].push(*sample);
        }
        
        // 执行重采样
        let wave_out = self.resampler.process(&wave_in, None)
            .map_err(|e| anyhow::anyhow!("重采样失败: {}", e))?;
        
        // 将平面格式转换回交错格式
        let output_frame_count = wave_out[0].len();
        let mut output = Vec::with_capacity(output_frame_count * self.channels);
        
        for frame_idx in 0..output_frame_count {
            for ch in 0..self.channels {
                output.push(wave_out[ch][frame_idx]);
            }
        }
        
        Ok(output)
    }
    
    /// 处理音频数据（原地处理，用于流式处理）
    /// 
    /// 返回处理后的样本数和输出缓冲区
    pub fn process_into(
        &mut self,
        input: &[f32],
        output: &mut Vec<f32>,
    ) -> anyhow::Result<usize> {
        let processed = self.process(input)?;
        let count = processed.len();
        output.extend_from_slice(&processed);
        Ok(count)
    }
    
    /// 重置处理器状态
    pub fn reset(&mut self) {
        self.resampler.reset();
    }
    
    /// 计算 BPM 匹配所需的速度比率
    /// 
    /// # Arguments
    /// * `source_bpm` - 源音频的 BPM
    /// * `target_bpm` - 目标 BPM
    /// 
    /// # Returns
    /// 速度比率 (source_bpm / target_bpm)
    pub fn calculate_speed_for_bpm_match(source_bpm: f64, target_bpm: f64) -> f64 {
        if target_bpm <= 0.0 {
            return 1.0;
        }
        source_bpm / target_bpm
    }
    
    /// 计算相位对齐的样本偏移
    /// 
    /// 用于将两个音频的 Beat 对齐
    /// 
    /// # Arguments
    /// * `current_position` - 当前播放位置（样本数）
    /// * `beat_interval_samples` - Beat 间隔（样本数）
    /// * `target_beat_position` - 目标 Beat 位置（样本数）
    /// 
    /// # Returns
    /// 需要调整的样本偏移量
    pub fn calculate_phase_alignment(
        current_position: usize,
        beat_interval_samples: usize,
        target_beat_position: usize,
    ) -> i64 {
        if beat_interval_samples == 0 {
            return 0;
        }
        
        // 计算当前位置相对于目标 Beat 的相位
        let current_phase = (current_position as i64 - target_beat_position as i64)
            .rem_euclid(beat_interval_samples as i64);
        
        // 计算需要调整的偏移量
        let adjustment = if current_phase > (beat_interval_samples as i64 / 2) {
            // 向后调整到下一个 Beat
            beat_interval_samples as i64 - current_phase
        } else {
            // 向前调整到上一个 Beat
            -current_phase
        };
        
        adjustment
    }
}

/// BPM 同步管理器
pub struct BpmSyncManager {
    /// 主音频（当前播放）的 BPM
    master_bpm: f64,
    /// 从音频（下一首）的 BPM
    slave_bpm: f64,
    /// 采样率
    sample_rate: u32,
    /// 是否启用同步
    sync_enabled: bool,
}

impl BpmSyncManager {
    /// 创建新的 BPM 同步管理器
    pub fn new(sample_rate: u32) -> Self {
        Self {
            master_bpm: 120.0,
            slave_bpm: 120.0,
            sample_rate,
            sync_enabled: false,
        }
    }
    
    /// 设置主音频 BPM
    pub fn set_master_bpm(&mut self, bpm: f64) {
        self.master_bpm = bpm.max(60.0).min(200.0);
    }
    
    /// 设置从音频 BPM
    pub fn set_slave_bpm(&mut self, bpm: f64) {
        self.slave_bpm = bpm.max(60.0).min(200.0);
    }
    
    /// 获取速度比率（用于时间拉伸）
    pub fn get_speed_ratio(&self) -> f64 {
        if !self.sync_enabled || self.master_bpm <= 0.0 {
            return 1.0;
        }
        
        // 计算从音频需要调整的速度以匹配主音频
        // slave_bpm * speed_ratio = master_bpm
        // speed_ratio = master_bpm / slave_bpm
        self.master_bpm / self.slave_bpm
    }
    
    /// 启用/禁用同步
    pub fn set_sync_enabled(&mut self, enabled: bool) {
        self.sync_enabled = enabled;
    }
    
    /// 检查同步是否启用
    pub fn is_sync_enabled(&self) -> bool {
        self.sync_enabled
    }
    
    /// 计算 Beat 间隔（样本数）
    pub fn beat_interval_samples(&self, bpm: f64) -> usize {
        if bpm <= 0.0 {
            return 0;
        }
        
        // 60 BPM = 1 beat/second
        // samples per beat = sample_rate * 60 / bpm
        ((self.sample_rate as f64 * 60.0) / bpm) as usize
    }
    
    /// 计算下一拍的位置
    pub fn next_beat_position(&self, current_position: usize, bpm: f64) -> usize {
        let interval = self.beat_interval_samples(bpm);
        if interval == 0 {
            return current_position;
        }
        
        let beats_passed = current_position / interval;
        (beats_passed + 1) * interval
    }
    
    /// 计算最近的 Beat 位置
    pub fn nearest_beat_position(&self, current_position: usize, bpm: f64) -> usize {
        let interval = self.beat_interval_samples(bpm);
        if interval == 0 {
            return current_position;
        }
        
        let beats_passed = current_position / interval;
        let remainder = current_position % interval;
        
        if remainder > interval / 2 {
            (beats_passed + 1) * interval
        } else {
            beats_passed * interval
        }
    }
}

/// 相位同步器 - 用于精确对齐两个音频的 Beat
pub struct PhaseSync {
    /// 目标相位偏移（样本数）
    target_offset: i64,
    /// 当前相位偏移
    current_offset: i64,
    /// 同步速度（0.0 - 1.0，越大越快）
    sync_speed: f32,
}

impl PhaseSync {
    /// 创建新的相位同步器
    pub fn new() -> Self {
        Self {
            target_offset: 0,
            current_offset: 0,
            sync_speed: 0.1,
        }
    }
    
    /// 设置目标相位偏移
    pub fn set_target_offset(&mut self, offset: i64) {
        self.target_offset = offset;
    }
    
    /// 更新当前相位（渐进式同步）
    pub fn update(&mut self) -> i64 {
        let diff = self.target_offset - self.current_offset;
        
        if diff.abs() < 10 {
            // 接近目标，直接设置
            self.current_offset = self.target_offset;
        } else {
            // 渐进式调整
            let adjustment = (diff as f32 * self.sync_speed) as i64;
            self.current_offset += adjustment.max(1).min(diff.abs()) * diff.signum();
        }
        
        self.current_offset
    }
    
    /// 检查是否已同步
    pub fn is_synced(&self) -> bool {
        (self.target_offset - self.current_offset).abs() < 10
    }
    
    /// 重置
    pub fn reset(&mut self) {
        self.target_offset = 0;
        self.current_offset = 0;
    }
}

impl Default for PhaseSync {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_time_stretcher_creation() {
        let stretcher = TimeStretcher::new(44100, 2);
        assert!(stretcher.is_ok());
        
        let stretcher = stretcher.unwrap();
        assert_eq!(stretcher.speed_ratio(), 1.0);
    }
    
    #[test]
    fn test_speed_adjustment() {
        let mut stretcher = TimeStretcher::new(44100, 2).unwrap();
        
        // 设置加速
        stretcher.set_speed(1.5).unwrap();
        assert!((stretcher.speed_ratio() - 1.5).abs() < 0.01);
        
        // 设置减速
        stretcher.set_speed(0.75).unwrap();
        assert!((stretcher.speed_ratio() - 0.75).abs() < 0.01);
    }
    
    #[test]
    fn test_bpm_calculation() {
        // 120 BPM -> 128 BPM
        let ratio = TimeStretcher::calculate_speed_for_bpm_match(120.0, 128.0);
        assert!((ratio - 0.9375).abs() < 0.001);
        
        // 128 BPM -> 120 BPM
        let ratio = TimeStretcher::calculate_speed_for_bpm_match(128.0, 120.0);
        assert!((ratio - 1.0667).abs() < 0.001);
    }
    
    #[test]
    fn test_bpm_sync_manager() {
        let mut manager = BpmSyncManager::new(44100);
        
        manager.set_master_bpm(128.0);
        manager.set_slave_bpm(120.0);
        manager.set_sync_enabled(true);
        
        // 从音频需要加速以匹配主音频
        let ratio = manager.get_speed_ratio();
        assert!((ratio - 1.0667).abs() < 0.001);
        
        // 计算 Beat 间隔
        let interval = manager.beat_interval_samples(120.0);
        assert_eq!(interval, 22050); // 44100 * 60 / 120 = 22050
    }
    
    #[test]
    fn test_phase_alignment() {
        // 当前位置 1000，Beat 间隔 441，目标位置 0
        let offset = TimeStretcher::calculate_phase_alignment(1000, 441, 0);
        // 应该向前调整 118 个样本（到位置 882）
        assert_eq!(offset, -118);
    }
    
    #[test]
    fn test_phase_sync() {
        let mut sync = PhaseSync::new();
        
        sync.set_target_offset(100);
        
        // 多次更新应该逐渐接近目标
        for _ in 0..20 {
            sync.update();
        }
        
        assert!(sync.is_synced());
    }
}
