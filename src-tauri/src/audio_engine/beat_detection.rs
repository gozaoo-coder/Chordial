use realfft::RealFftPlanner;
use std::f32::consts::PI;

/// 音频分析结果
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    /// 检测到的BPM
    pub bpm: f64,
    /// Beat位置（秒为单位）
    pub beat_positions: Vec<f64>,
    /// Onset强度曲线（用于调试和可视化）
    pub onset_curve: Vec<f32>,
    /// 第一拍位置（秒）
    pub downbeat_position: Option<f64>,
}

/// 节拍检测器 - 基于多频带Spectral Flux算法
pub struct BeatDetector {
    sample_rate: u32,
    /// FFT窗口大小
    fft_size: usize,
    /// 帧移（hop size）
    hop_size: usize,
    /// 多频带权重配置: (start_bin, end_bin, weight)
    freq_bands: Vec<(usize, usize, f32)>,
}

impl BeatDetector {
    /// 创建新的节拍检测器
    pub fn new(sample_rate: u32) -> Self {
        let fft_size = 2048;
        let hop_size = 512;
        
        // 定义频带权重：低频（Kick鼓）权重更高
        let freq_bands = vec![
            // 低频 20-150Hz，权重1.0（Kick鼓主要频段）
            (1, (150.0 * fft_size as f32 / sample_rate as f32) as usize, 1.0),
            // 中低频 150-400Hz，权重0.6（Snare主要频段）
            ((150.0 * fft_size as f32 / sample_rate as f32) as usize,
             (400.0 * fft_size as f32 / sample_rate as f32) as usize, 0.6),
            // 高频 400Hz-奈奎斯特频率，权重0.2
            ((400.0 * fft_size as f32 / sample_rate as f32) as usize,
             fft_size / 2, 0.2),
        ];
        
        Self {
            sample_rate,
            fft_size,
            hop_size,
            freq_bands,
        }
    }

    /// 分析音频数据，返回BPM和Beat位置
    /// 
    /// # Arguments
    /// * `audio` - 单声道f32音频数据，范围[-1.0, 1.0]
    pub fn analyze(&self, audio: &[f32]) -> AnalysisResult {
        // 1. 计算Onset Strength Curve
        let onset_curve = self.compute_onset_curve(audio);
        
        // 2. 从Onset Curve提取Tempo和Beat位置
        let (bpm, beat_positions) = self.extract_beats(&onset_curve);
        
        // 3. 检测第一拍位置
        let downbeat_position = self.find_downbeat(&onset_curve, bpm);
        
        AnalysisResult {
            bpm,
            beat_positions,
            onset_curve,
            downbeat_position,
        }
    }

    /// 计算Onset Strength Curve（核心算法）
    /// 
    /// 使用多频带Spectral Flux检测频谱能量的突然增加
    fn compute_onset_curve(&self, audio: &[f32]) -> Vec<f32> {
        let num_frames = (audio.len().saturating_sub(self.fft_size)) / self.hop_size + 1;
        let mut onset_curve = Vec::with_capacity(num_frames);
        
        // 初始化FFT规划器
        let mut real_planner = RealFftPlanner::<f32>::new();
        let fft = real_planner.plan_fft_forward(self.fft_size);
        
        // 创建汉宁窗（减少频谱泄漏）
        let window: Vec<f32> = (0..self.fft_size)
            .map(|i| 0.5 * (1.0 - (2.0 * PI * i as f32 / self.fft_size as f32).cos()))
            .collect();
        
        let mut prev_spectrum: Option<Vec<f32>> = None;
        let mut input_buffer = vec![0.0f32; self.fft_size];
        let mut spectrum_buffer = fft.make_output_vec();
        
        for frame_idx in 0..num_frames {
            let start = frame_idx * self.hop_size;
            
            // 加窗
            for (i, win) in window.iter().enumerate() {
                input_buffer[i] = audio.get(start + i).copied().unwrap_or(0.0) * win;
            }
            
            // 执行FFT
            fft.process(&mut input_buffer, &mut spectrum_buffer).unwrap();
            
            // 计算幅度谱
            let magnitude: Vec<f32> = spectrum_buffer.iter()
                .map(|c| c.norm())
                .collect();
            
            // 计算加权Spectral Flux
            let flux = if let Some(ref prev) = prev_spectrum {
                self.compute_weighted_flux(&magnitude, prev)
            } else {
                0.0
            };
            
            onset_curve.push(flux);
            prev_spectrum = Some(magnitude);
        }
        
        // 归一化和压缩
        self.normalize_and_compress(&mut onset_curve);
        onset_curve
    }

    /// 计算多频带加权Spectral Flux
    /// 
    /// Spectral Flux = 相邻帧频谱差值的正数部分之和
    fn compute_weighted_flux(&self, curr: &[f32], prev: &[f32]) -> f32 {
        let mut total_flux = 0.0f32;
        
        for (start, end, weight) in &self.freq_bands {
            let end_idx = (*end).min(curr.len());
            let start_idx = (*start).min(end_idx);
            
            let band_flux: f32 = curr[start_idx..end_idx]
                .iter()
                .zip(&prev[start_idx..end_idx])
                .map(|(c, p)| (c - p).max(0.0))  // 只取正增量（Rectify）
                .sum();
            
            total_flux += band_flux * weight;
        }
        
        total_flux
    }

    /// 对数压缩和Z-score归一化
    /// 
    /// 让强弱歌曲的onset都能被检测
    fn normalize_and_compress(&self, curve: &mut [f32]) {
        if curve.is_empty() { return; }
        
        // 对数压缩：增强弱onset，压缩强onset
        for val in curve.iter_mut() {
            *val = (1.0 + *val).ln();
        }
        
        // Z-score归一化
        let mean = curve.iter().sum::<f32>() / curve.len() as f32;
        let variance = curve.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f32>() / curve.len() as f32;
        let std = variance.sqrt().max(0.0001);  // 防止除零
        
        for val in curve.iter_mut() {
            *val = (*val - mean) / std;
        }
    }

    /// 提取BPM和Beat位置
    /// 
    /// 使用Comb Filter方法找到最佳Tempo
    fn extract_beats(&self, onset_curve: &[f32]) -> (f64, Vec<f64>) {
        let mut best_score = f32::MIN;
        let mut best_bpm = 120.0f64;
        
        // 粗搜索：步进2 BPM
        for bpm in (60..=200).step_by(2) {
            let period = self.bpm_to_period_frames(bpm as f64);
            let score = self.evaluate_tempo(onset_curve, period);
            
            if score > best_score {
                best_score = score;
                best_bpm = bpm as f64;
            }
        }
        
        // 细搜索：在最佳BPM附近±2精细搜索
        for bpm in ((best_bpm as i32 - 2).max(60))..=((best_bpm as i32 + 2).min(200)) {
            let period = self.bpm_to_period_frames(bpm as f64);
            let score = self.evaluate_tempo(onset_curve, period);
            
            if score > best_score {
                best_score = score;
                best_bpm = bpm as f64;
            }
        }
        
        // 根据最佳BPM提取Beat位置
        let beat_frames = self.find_beat_positions(onset_curve, best_bpm);
        let beat_seconds: Vec<f64> = beat_frames.iter()
            .map(|&f| f as f64 * self.hop_size as f64 / self.sample_rate as f64)
            .collect();
        
        (best_bpm, beat_seconds)
    }

    /// BPM转换为帧周期
    fn bpm_to_period_frames(&self, bpm: f64) -> usize {
        (60.0 / bpm * self.sample_rate as f64 / self.hop_size as f64) as usize
    }

    /// Comb Filter评估：检查该Period下的能量聚集程度
    /// 
    /// 原理：正确的BPM会让onset energy在beat位置聚集
    fn evaluate_tempo(&self, onset_curve: &[f32], period: usize) -> f32 {
        if period == 0 || onset_curve.len() < period {
            return 0.0;
        }
        
        let num_pulses = onset_curve.len() / period;
        let mut best_score = 0.0f32;
        
        // 尝试不同的相位偏移
        for offset in 0..period.min(16) {  // 限制搜索范围
            let mut pulse_sum = 0.0f32;
            for i in 0..num_pulses {
                let idx = offset + i * period;
                if idx < onset_curve.len() {
                    pulse_sum += onset_curve[idx].max(0.0);
                }
            }
            best_score = best_score.max(pulse_sum);
        }
        
        best_score
    }

    /// 查找Beat位置（基于峰值检测）
    fn find_beat_positions(&self, onset_curve: &[f32], bpm: f64) -> Vec<usize> {
        let period = self.bpm_to_period_frames(bpm);
        let mut beats = Vec::new();
        
        if period == 0 || onset_curve.is_empty() {
            return beats;
        }
        
        let tolerance = period / 4;  // 允许±25%的偏差
        let threshold = 0.5f32;  // 峰值阈值（已归一化）
        let window = 3usize;  // 局部最大值窗口
        
        let mut last_beat = 0usize;
        let start_frame = period.saturating_sub(tolerance);
        
        for i in start_frame..onset_curve.len() {
            // 确保间隔足够
            if i.saturating_sub(last_beat) < period.saturating_sub(tolerance) {
                continue;
            }
            
            // 检查是否是局部最大值
            let is_peak = self.is_local_peak(onset_curve, i, window);
            
            if is_peak && onset_curve[i] > threshold {
                beats.push(i);
                last_beat = i;
            }
        }
        
        beats
    }

    /// 检查指定位置是否是局部峰值
    fn is_local_peak(&self, curve: &[f32], idx: usize, window: usize) -> bool {
        if curve.is_empty() || idx >= curve.len() {
            return false;
        }
        
        let start = idx.saturating_sub(window);
        let end = (idx + window + 1).min(curve.len());
        
        let current_val = curve[idx];
        (start..end).all(|j| j == idx || current_val >= curve[j])
    }

    /// 检测第一拍位置（Downbeat Detection）
    /// 
    /// 使用复数域Comb Filter检测哪个相位有最强的能量聚集
    fn find_downbeat(&self, onset_curve: &[f32], bpm: f64) -> Option<f64> {
        let period = self.bpm_to_period_frames(bpm);
        
        if period == 0 || onset_curve.len() < period * 4 {
            return None;
        }
        
        // 尝试4个不同的相位偏移（假设4/4拍）
        let best_offset = (0..4)
            .map(|i| {
                let offset = i * period / 4;
                let score: f32 = (0..(onset_curve.len() - offset) / period)
                    .map(|j| onset_curve[offset + j * period])
                    .sum();
                (offset, score)
            })
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(offset, _)| offset)?;
        
        // 转换为秒
        Some(best_offset as f64 * self.hop_size as f64 / self.sample_rate as f64)
    }

    /// 获取建议的切歌点（歌曲结尾前10秒内的最后一个Beat）
    pub fn find_mix_out_point(&self, result: &AnalysisResult, duration_secs: f64) -> Option<f64> {
        let cutoff = duration_secs - 10.0;
        result.beat_positions
            .iter()
            .filter(|&&t| t < cutoff)
            .last()
            .copied()
    }
}

/// 流式节拍检测器（用于实时分析）
pub struct StreamingBeatDetector {
    detector: BeatDetector,
    buffer: Vec<f32>,
    min_analysis_duration: usize,  // 最小分析时长（samples）
}

impl StreamingBeatDetector {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            detector: BeatDetector::new(sample_rate),
            buffer: Vec::new(),
            min_analysis_duration: sample_rate as usize * 6,  // 至少6秒
        }
    }

    /// 接收音频块（来自解码器）
    pub fn feed(&mut self, chunk: &[f32]) {
        self.buffer.extend_from_slice(chunk);
    }

    /// 尝试分析（当缓冲区足够时）
    pub fn try_analyze(&mut self) -> Option<AnalysisResult> {
        if self.buffer.len() >= self.min_analysis_duration {
            let result = self.detector.analyze(&self.buffer);
            Some(result)
        } else {
            None
        }
    }

    /// 清空缓冲区
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// 获取当前缓冲区时长（秒）
    pub fn buffered_seconds(&self, sample_rate: u32) -> f64 {
        self.buffer.len() as f64 / sample_rate as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beat_detector_creation() {
        let detector = BeatDetector::new(44100);
        assert_eq!(detector.sample_rate, 44100);
        assert_eq!(detector.fft_size, 2048);
        assert_eq!(detector.hop_size, 512);
    }

    #[test]
    fn test_bpm_to_period() {
        let detector = BeatDetector::new(44100);
        let period = detector.bpm_to_period_frames(120.0);
        // 120 BPM = 0.5秒/拍 = 22050 samples @ 44.1kHz
        // 22050 / 512 hop_size ≈ 43 frames
        assert!(period > 40 && period < 50);
    }

    #[test]
    fn test_local_peak_detection() {
        let detector = BeatDetector::new(44100);
        let curve = vec![0.1, 0.5, 1.0, 0.5, 0.1];
        
        assert!(detector.is_local_peak(&curve, 2, 2));
        assert!(!detector.is_local_peak(&curve, 0, 2));
        assert!(!detector.is_local_peak(&curve, 4, 2));
    }
}
