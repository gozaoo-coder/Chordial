//! EQ滤波器模块 - 实现二阶IIR滤波器（Biquad）

/// 滤波器类型
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FilterType {
    /// 低通滤波器
    LowPass,
    /// 高通滤波器
    HighPass,
    /// 带通滤波器
    BandPass,
    /// 陷波滤波器
    Notch,
    /// 峰值滤波器
    Peak,
    /// 低架滤波器
    LowShelf,
    /// 高架滤波器
    HighShelf,
}

/// 二阶IIR滤波器（Biquad）
/// 
/// 使用标准双二阶滤波器公式：
/// y[n] = b0*x[n] + b1*x[n-1] + b2*x[n-2] - a1*y[n-1] - a2*y[n-2]
#[derive(Clone, Debug)]
pub struct BiquadFilter {
    /// 滤波器系数
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    /// 状态变量（每个声道一组）
    z1: Vec<f32>, // x[n-1] 延迟
    z2: Vec<f32>, // x[n-2] 延迟
    y1: Vec<f32>, // y[n-1] 延迟
    y2: Vec<f32>, // y[n-2] 延迟
    /// 采样率
    sample_rate: f32,
    /// 滤波器类型
    filter_type: FilterType,
    /// 截止/中心频率
    frequency: f32,
    /// Q值（品质因数）
    q: f32,
    /// 增益（dB，用于Peak/Shelf滤波器）
    gain_db: f32,
    /// 声道数
    channels: usize,
}

impl BiquadFilter {
    /// 创建新的滤波器
    pub fn new(
        filter_type: FilterType,
        sample_rate: f32,
        frequency: f32,
        q: f32,
        gain_db: f32,
        channels: usize,
    ) -> Self {
        let mut filter = Self {
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            z1: vec![0.0; channels],
            z2: vec![0.0; channels],
            y1: vec![0.0; channels],
            y2: vec![0.0; channels],
            sample_rate,
            filter_type,
            frequency,
            q,
            gain_db,
            channels,
        };
        
        filter.calculate_coefficients();
        filter
    }
    
    /// 计算滤波器系数
    fn calculate_coefficients(&mut self) {
        let w0 = 2.0 * std::f32::consts::PI * self.frequency / self.sample_rate;
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();
        let alpha = sin_w0 / (2.0 * self.q);
        
        match self.filter_type {
            FilterType::LowPass => {
                self.b0 = (1.0 - cos_w0) / 2.0;
                self.b1 = 1.0 - cos_w0;
                self.b2 = (1.0 - cos_w0) / 2.0;
                self.a1 = -2.0 * cos_w0;
                self.a2 = 1.0 - alpha;
            }
            FilterType::HighPass => {
                self.b0 = (1.0 + cos_w0) / 2.0;
                self.b1 = -(1.0 + cos_w0);
                self.b2 = (1.0 + cos_w0) / 2.0;
                self.a1 = -2.0 * cos_w0;
                self.a2 = 1.0 - alpha;
            }
            FilterType::BandPass => {
                self.b0 = alpha;
                self.b1 = 0.0;
                self.b2 = -alpha;
                self.a1 = -2.0 * cos_w0;
                self.a2 = 1.0 - alpha;
            }
            FilterType::Notch => {
                self.b0 = 1.0;
                self.b1 = -2.0 * cos_w0;
                self.b2 = 1.0;
                self.a1 = -2.0 * cos_w0;
                self.a2 = 1.0 - alpha;
            }
            FilterType::Peak => {
                let a = 10.0f32.powf(self.gain_db / 40.0);
                self.b0 = 1.0 + alpha * a;
                self.b1 = -2.0 * cos_w0;
                self.b2 = 1.0 - alpha * a;
                self.a1 = -2.0 * cos_w0;
                self.a2 = 1.0 - alpha / a;
            }
            FilterType::LowShelf => {
                let a = 10.0f32.powf(self.gain_db / 40.0);
                let sqrt_a = a.sqrt();
                self.b0 = a * ((a + 1.0) - (a - 1.0) * cos_w0 + 2.0 * sqrt_a * alpha);
                self.b1 = 2.0 * a * ((a - 1.0) - (a + 1.0) * cos_w0);
                self.b2 = a * ((a + 1.0) - (a - 1.0) * cos_w0 - 2.0 * sqrt_a * alpha);
                self.a1 = -2.0 * ((a - 1.0) + (a + 1.0) * cos_w0);
                self.a2 = (a + 1.0) + (a - 1.0) * cos_w0 - 2.0 * sqrt_a * alpha;
            }
            FilterType::HighShelf => {
                let a = 10.0f32.powf(self.gain_db / 40.0);
                let sqrt_a = a.sqrt();
                self.b0 = a * ((a + 1.0) + (a - 1.0) * cos_w0 + 2.0 * sqrt_a * alpha);
                self.b1 = -2.0 * a * ((a - 1.0) + (a + 1.0) * cos_w0);
                self.b2 = a * ((a + 1.0) + (a - 1.0) * cos_w0 - 2.0 * sqrt_a * alpha);
                self.a1 = -2.0 * ((a - 1.0) - (a + 1.0) * cos_w0);
                self.a2 = (a + 1.0) - (a - 1.0) * cos_w0 - 2.0 * sqrt_a * alpha;
            }
        }
        
        // 归一化（使a0 = 1）
        let a0 = 1.0 + alpha;
        self.b0 /= a0;
        self.b1 /= a0;
        self.b2 /= a0;
        self.a1 /= a0;
        self.a2 /= a0;
    }
    
    /// 设置截止频率
    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency.clamp(20.0, self.sample_rate / 2.0);
        self.calculate_coefficients();
    }
    
    /// 设置Q值
    pub fn set_q(&mut self, q: f32) {
        self.q = q.max(0.1);
        self.calculate_coefficients();
    }
    
    /// 设置增益（dB）
    pub fn set_gain(&mut self, gain_db: f32) {
        self.gain_db = gain_db;
        self.calculate_coefficients();
    }
    
    /// 处理单个样本
    fn process_sample(&mut self, input: f32, channel: usize) -> f32 {
        // 双二阶滤波器公式
        let output = self.b0 * input 
                   + self.b1 * self.z1[channel] 
                   + self.b2 * self.z2[channel]
                   - self.a1 * self.y1[channel]
                   - self.a2 * self.y2[channel];
        
        // 更新状态
        self.z2[channel] = self.z1[channel];
        self.z1[channel] = input;
        self.y2[channel] = self.y1[channel];
        self.y1[channel] = output;
        
        output
    }
    
    /// 处理音频缓冲区（交错格式）
    pub fn process(&mut self, buffer: &mut [f32]) {
        for (i, sample) in buffer.iter_mut().enumerate() {
            let channel = i % self.channels;
            *sample = self.process_sample(*sample, channel);
        }
    }
    
    /// 处理音频缓冲区并返回新缓冲区
    pub fn process_buffer(&mut self, input: &[f32]) -> Vec<f32> {
        input.iter().enumerate()
            .map(|(i, &sample)| {
                let channel = i % self.channels;
                self.process_sample(sample, channel)
            })
            .collect()
    }
    
    /// 重置滤波器状态
    pub fn reset(&mut self) {
        for i in 0..self.channels {
            self.z1[i] = 0.0;
            self.z2[i] = 0.0;
            self.y1[i] = 0.0;
            self.y2[i] = 0.0;
        }
    }
    
    /// 获取当前频率
    pub fn frequency(&self) -> f32 {
        self.frequency
    }
    
    /// 获取滤波器类型
    pub fn filter_type(&self) -> FilterType {
        self.filter_type
    }
}

/// 3段EQ（低频、中频、高频）
pub struct ThreeBandEQ {
    /// 低频滤波器（Low Shelf）
    low_band: BiquadFilter,
    /// 中频滤波器（Peak）
    mid_band: BiquadFilter,
    /// 高频滤波器（High Shelf）
    high_band: BiquadFilter,
    /// 采样率
    sample_rate: f32,
    /// 声道数
    channels: usize,
}

impl ThreeBandEQ {
    /// 创建新的3段EQ
    /// 
    /// 默认频段：
    /// - 低频: 100Hz (Low Shelf)
    /// - 中频: 1000Hz (Peak)
    /// - 高频: 10000Hz (High Shelf)
    pub fn new(sample_rate: f32, channels: usize) -> Self {
        Self {
            low_band: BiquadFilter::new(
                FilterType::LowShelf,
                sample_rate,
                100.0,
                0.707,
                0.0,
                channels,
            ),
            mid_band: BiquadFilter::new(
                FilterType::Peak,
                sample_rate,
                1000.0,
                1.0,
                0.0,
                channels,
            ),
            high_band: BiquadFilter::new(
                FilterType::HighShelf,
                sample_rate,
                10000.0,
                0.707,
                0.0,
                channels,
            ),
            sample_rate,
            channels,
        }
    }
    
    /// 设置低频增益（dB）
    pub fn set_low_gain(&mut self, gain_db: f32) {
        self.low_band.set_gain(gain_db);
    }
    
    /// 设置中频增益（dB）
    pub fn set_mid_gain(&mut self, gain_db: f32) {
        self.mid_band.set_gain(gain_db);
    }
    
    /// 设置高频增益（dB）
    pub fn set_high_gain(&mut self, gain_db: f32) {
        self.high_band.set_gain(gain_db);
    }
    
    /// 处理音频缓冲区
    pub fn process(&mut self, buffer: &mut [f32]) {
        // 依次通过三个频段
        self.low_band.process(buffer);
        self.mid_band.process(buffer);
        self.high_band.process(buffer);
    }
    
    /// 重置所有滤波器
    pub fn reset(&mut self) {
        self.low_band.reset();
        self.mid_band.reset();
        self.high_band.reset();
    }
}

/// DJ风格的过渡EQ
/// 
/// 在交叉淡化时动态调整EQ：
/// - 淡出轨道：逐渐降低高频（Hi-Cut效果）
/// - 淡入轨道：逐渐提升高频
pub struct TransitionEQ {
    /// 轨道A的滤波器
    filter_a: BiquadFilter,
    /// 轨道B的滤波器
    filter_b: BiquadFilter,
    /// 采样率
    sample_rate: f32,
    /// 声道数
    channels: usize,
}

impl TransitionEQ {
    /// 创建新的过渡EQ
    pub fn new(sample_rate: f32, channels: usize) -> Self {
        Self {
            filter_a: BiquadFilter::new(
                FilterType::LowPass,
                sample_rate,
                20000.0, // 初始为全通
                0.707,
                0.0,
                channels,
            ),
            filter_b: BiquadFilter::new(
                FilterType::HighPass,
                sample_rate,
                20.0, // 初始为全通
                0.707,
                0.0,
                channels,
            ),
            sample_rate,
            channels,
        }
    }
    
    /// 更新过渡状态
    /// 
    /// # Arguments
    /// * `progress` - 过渡进度 (0.0 - 1.0)
    /// * `is_track_a` - true表示处理轨道A，false表示处理轨道B
    pub fn update_transition(&mut self, progress: f32, is_track_a: bool) {
        let progress = progress.clamp(0.0, 1.0);
        
        if is_track_a {
            // 轨道A：从全频逐渐降低高频（20kHz -> 200Hz）
            let cutoff = 20000.0 * (1.0 - progress) + 200.0 * progress;
            self.filter_a.set_frequency(cutoff);
        } else {
            // 轨道B：从低频逐渐提升到全频（200Hz -> 20kHz）
            let cutoff = 200.0 * (1.0 - progress) + 20000.0 * progress;
            self.filter_b.set_frequency(cutoff);
        }
    }
    
    /// 处理轨道A的音频
    pub fn process_track_a(&mut self, buffer: &mut [f32]) {
        self.filter_a.process(buffer);
    }
    
    /// 处理轨道B的音频
    pub fn process_track_b(&mut self, buffer: &mut [f32]) {
        self.filter_b.process(buffer);
    }
    
    /// 重置
    pub fn reset(&mut self) {
        self.filter_a.set_frequency(20000.0);
        self.filter_b.set_frequency(20.0);
        self.filter_a.reset();
        self.filter_b.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_biquad_creation() {
        let filter = BiquadFilter::new(
            FilterType::LowPass,
            44100.0,
            1000.0,
            0.707,
            0.0,
            2,
        );
        
        assert_eq!(filter.frequency(), 1000.0);
        assert_eq!(filter.filter_type(), FilterType::LowPass);
    }
    
    #[test]
    fn test_frequency_change() {
        let mut filter = BiquadFilter::new(
            FilterType::LowPass,
            44100.0,
            1000.0,
            0.707,
            0.0,
            2,
        );
        
        filter.set_frequency(2000.0);
        assert_eq!(filter.frequency(), 2000.0);
    }
    
    #[test]
    fn test_three_band_eq() {
        let mut eq = ThreeBandEQ::new(44100.0, 2);
        
        eq.set_low_gain(6.0);
        eq.set_mid_gain(-3.0);
        eq.set_high_gain(3.0);
        
        let mut buffer = vec![0.5, 0.5, 0.5, 0.5];
        eq.process(&mut buffer);
        
        // 处理后应该有变化
        assert_ne!(buffer, vec![0.5, 0.5, 0.5, 0.5]);
    }
    
    #[test]
    fn test_transition_eq() {
        let mut transition = TransitionEQ::new(44100.0, 2);
        
        // 开始过渡
        transition.update_transition(0.0, true);
        transition.update_transition(0.0, false);
        
        let mut buffer_a = vec![0.5, 0.5, 0.5, 0.5];
        let mut buffer_b = vec![0.3, 0.3, 0.3, 0.3];
        
        transition.process_track_a(&mut buffer_a);
        transition.process_track_b(&mut buffer_b);
        
        // 中间状态
        transition.update_transition(0.5, true);
        transition.update_transition(0.5, false);
    }
    
    #[test]
    fn test_filter_reset() {
        let mut filter = BiquadFilter::new(
            FilterType::LowPass,
            44100.0,
            1000.0,
            0.707,
            0.0,
            2,
        );
        
        // 处理一些样本
        let mut buffer = vec![1.0, 1.0, 1.0, 1.0];
        filter.process(&mut buffer);
        
        // 重置
        filter.reset();
        
        // 再次处理应该得到相同结果
        let result1 = buffer.clone();
        let mut buffer2 = vec![1.0, 1.0, 1.0, 1.0];
        filter.process(&mut buffer2);
        
        assert_eq!(buffer, buffer2);
    }
}
