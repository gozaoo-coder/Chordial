use std::fs::File;
use std::path::Path;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::{FormatOptions, FormatReader};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

/// 解码后的音频帧
#[derive(Clone, Debug)]
pub struct AudioFrame {
    /// 采样数据（交错格式：LRLRLR...）
    pub samples: Vec<f32>,
    /// 采样率
    pub sample_rate: u32,
    /// 声道数
    pub channels: u16,
    /// 当前帧的时间戳（秒）
    pub timestamp: f64,
}

/// 音频解码器封装
pub struct SymphoniaDecoder {
    /// 格式读取器
    format: Box<dyn FormatReader>,
    /// 解码器
    decoder: Box<dyn symphonia::core::codecs::Decoder>,
    /// 当前音轨ID
    track_id: u32,
    /// 采样缓冲区
    sample_buffer: Option<SampleBuffer<f32>>,
    /// 总时长（秒）
    duration: Option<f64>,
    /// 当前播放位置（秒）
    current_position: f64,
    /// 采样率
    sample_rate: u32,
    /// 声道数
    channels: u16,
}

impl SymphoniaDecoder {
    /// 从文件路径创建解码器
    pub fn new<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let path = path.as_ref();
        
        // 打开文件
        let file = File::open(path)?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());
        
        // 创建提示
        let mut hint = Hint::new();
        if let Some(extension) = path.extension() {
            hint.with_extension(extension.to_str().unwrap_or(""));
        }
        
        // 探测格式
        let format_opts = FormatOptions::default();
        let metadata_opts = MetadataOptions::default();
        let probe_result = symphonia::default::get_probe()
            .format(&hint, mss, &format_opts, &metadata_opts)
            .map_err(|e| anyhow::anyhow!("Failed to probe format: {}", e))?;
        
        let format = probe_result.format;
        
        // 找到第一个音频轨道
        let track = format.tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .ok_or_else(|| anyhow::anyhow!("No audio track found"))?;
        
        let track_id = track.id;
        let codec_params = &track.codec_params;
        
        // 获取音频参数
        let sample_rate = codec_params.sample_rate.ok_or_else(|| anyhow::anyhow!("Unknown sample rate"))?;
        let channels = codec_params.channels
            .map(|c| c.count() as u16)
            .unwrap_or(2);
        
        // 计算总时长
        let duration = codec_params.n_frames
            .map(|frames| frames as f64 / sample_rate as f64);
        
        // 创建解码器
        let decoder_opts = DecoderOptions::default();
        let decoder = symphonia::default::get_codecs()
            .make(codec_params, &decoder_opts)
            .map_err(|e| anyhow::anyhow!("Failed to create decoder: {}", e))?;
        
        Ok(Self {
            format,
            decoder,
            track_id,
            sample_buffer: None,
            duration,
            current_position: 0.0,
            sample_rate,
            channels,
        })
    }
    
    /// 获取采样率
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
    
    /// 获取声道数
    pub fn channels(&self) -> u16 {
        self.channels
    }
    
    /// 获取总时长（秒）
    pub fn duration(&self) -> Option<f64> {
        self.duration
    }
    
    /// 获取当前播放位置（秒）
    pub fn current_position(&self) -> f64 {
        self.current_position
    }
    
    /// 获取剩余时长（秒）
    pub fn remaining(&self) -> Option<f64> {
        self.duration.map(|d| d - self.current_position)
    }
    
    /// 解码下一帧
    pub fn next_frame(&mut self) -> anyhow::Result<Option<AudioFrame>> {
        loop {
            // 获取下一个数据包
            let packet = match self.format.next_packet() {
                Ok(packet) => packet,
                Err(SymphoniaError::IoError(_)) => return Ok(None), // 文件结束
                Err(e) => return Err(anyhow::anyhow!("Read error: {}", e)),
            };
            
            // 只处理当前音轨
            if packet.track_id() != self.track_id {
                continue;
            }
            
            // 更新当前位置
            let ts = packet.ts();
            self.current_position = ts as f64 / self.sample_rate as f64;
            let current_pos = self.current_position;
            
            // 解码
            match self.decoder.decode(&packet) {
                Ok(decoded) => {
                    // 获取音频规格
                    let spec = *decoded.spec();
                    
                    // 创建新的采样缓冲区
                    let duration = decoded.capacity() as u64;
                    let mut sample_buffer = SampleBuffer::<f32>::new(duration, spec);
                    sample_buffer.copy_planar_ref(decoded);
                    
                    return Ok(Some(AudioFrame {
                        samples: sample_buffer.samples().to_vec(),
                        sample_rate: spec.rate,
                        channels: spec.channels.count() as u16,
                        timestamp: current_pos,
                    }));
                }
                Err(SymphoniaError::DecodeError(_)) => continue, // 跳过解码错误
                Err(e) => return Err(anyhow::anyhow!("Decode error: {}", e)),
            }
        }
    }
    
    /// 跳转到指定位置（秒）
    /// 
    /// # Arguments
    /// * `position` - 目标位置（秒），必须 >= 0
    pub fn seek(&mut self, position: f64) -> anyhow::Result<()> {
        use symphonia::core::formats::SeekMode;
        use symphonia::core::formats::SeekTo;
        
        // 检查位置是否有效
        if position < 0.0 {
            return Err(anyhow::anyhow!("Seek position must be non-negative: {}", position));
        }
        
        // 检查是否超出时长
        if let Some(duration) = self.duration {
            if position > duration {
                return Err(anyhow::anyhow!(
                    "Seek position {} exceeds duration {}",
                    position, duration
                ));
            }
        }
        
        let seek_to = SeekTo::Time {
            time: symphonia::core::units::Time::new(
                position as u64,
                position.fract()
            ),
            track_id: Some(self.track_id),
        };
        
        match self.format.seek(SeekMode::Accurate, seek_to) {
            Ok(_) => {
                self.current_position = position;
                // 重置解码器状态
                self.decoder.reset();
                Ok(())
            }
            Err(e) => Err(anyhow::anyhow!("Seek error: {}", e)),
        }
    }
    
    /// 批量解码指定时长的音频（用于分析）
    pub fn decode_duration(&mut self, duration_secs: f64) -> anyhow::Result<Vec<f32>> {
        let target_samples = (duration_secs * self.sample_rate as f64) as usize * self.channels as usize;
        let mut all_samples = Vec::with_capacity(target_samples);
        
        while all_samples.len() < target_samples {
            match self.next_frame()? {
                Some(frame) => {
                    all_samples.extend_from_slice(&frame.samples);
                }
                None => break, // 文件结束
            }
        }
        
        // 截断到目标长度
        all_samples.truncate(target_samples);
        Ok(all_samples)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_decoder_creation() {
        // 这个测试需要一个实际的音频文件
        // 在实际测试时提供文件路径
    }
}
