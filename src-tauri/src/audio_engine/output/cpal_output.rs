use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, SampleRate, Stream, StreamConfig};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

/// 音频输出回调函数类型
pub type AudioCallback = Box<dyn FnMut(&mut [f32]) + Send>;

/// CPAL 音频输出封装
pub struct CpalOutput {
    /// 音频流
    stream: Option<Stream>,
    /// 设备
    device: cpal::Device,
    /// 配置
    config: StreamConfig,
    /// 采样格式
    sample_format: SampleFormat,
    /// 播放状态
    is_playing: Arc<AtomicBool>,
    /// 音量 (0.0 - 1.0)
    volume: Arc<AtomicU32>,
    /// 回调函数
    callback: Arc<Mutex<Option<AudioCallback>>>,
}

impl CpalOutput {
    /// 创建新的音频输出
    pub fn new() -> anyhow::Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| anyhow::anyhow!("No default output device available"))?;

        let config = device.default_output_config()?;
        let sample_format = config.sample_format();
        
        // 使用默认配置，但统一采样率为 48000
        let mut stream_config: StreamConfig = config.config();
        stream_config.sample_rate = SampleRate(48000);

        Ok(Self {
            stream: None,
            device,
            config: stream_config,
            sample_format,
            is_playing: Arc::new(AtomicBool::new(false)),
            volume: Arc::new(AtomicU32::new(1.0f32.to_bits())),
            callback: Arc::new(Mutex::new(None)),
        })
    }

    /// 获取采样率
    pub fn sample_rate(&self) -> u32 {
        self.config.sample_rate.0
    }

    /// 获取声道数
    pub fn channels(&self) -> u16 {
        self.config.channels
    }

    /// 设置音频回调
    pub fn set_callback(&self, callback: AudioCallback) {
        let mut cb = self.callback.lock().unwrap();
        *cb = Some(callback);
    }

    /// 开始播放
    pub fn start(&mut self) -> anyhow::Result<()> {
        if self.stream.is_some() {
            return Ok(());
        }

        let is_playing = Arc::clone(&self.is_playing);
        let volume = Arc::clone(&self.volume);
        let callback = Arc::clone(&self.callback);

        let stream = match self.sample_format {
            SampleFormat::F32 => self.build_stream_f32(is_playing, volume, callback)?,
            SampleFormat::I16 => self.build_stream_i16(is_playing, volume, callback)?,
            SampleFormat::U16 => self.build_stream_u16(is_playing, volume, callback)?,
            _ => return Err(anyhow::anyhow!("Unsupported sample format")),
        };

        stream.play()?;
        self.is_playing.store(true, Ordering::Relaxed);
        self.stream = Some(stream);

        Ok(())
    }

    /// 暂停播放
    pub fn pause(&self) {
        self.is_playing.store(false, Ordering::Relaxed);
    }

    /// 恢复播放
    pub fn resume(&self) {
        self.is_playing.store(true, Ordering::Relaxed);
    }

    /// 停止播放
    pub fn stop(&mut self) {
        self.is_playing.store(false, Ordering::Relaxed);
        self.stream = None;
    }

    /// 设置音量 (0.0 - 1.0)
    pub fn set_volume(&self, volume: f32) {
        let clamped = volume.clamp(0.0, 1.0);
        self.volume.store(clamped.to_bits(), Ordering::Relaxed);
    }

    /// 获取音量
    pub fn get_volume(&self) -> f32 {
        f32::from_bits(self.volume.load(Ordering::Relaxed))
    }

    /// 构建 f32 格式的音频流
    fn build_stream_f32(
        &self,
        is_playing: Arc<AtomicBool>,
        volume: Arc<AtomicU32>,
        callback: Arc<Mutex<Option<AudioCallback>>>,
    ) -> anyhow::Result<Stream> {
        let config = self.config.clone();

        let stream = self.device.build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                // 如果未播放，输出静音
                if !is_playing.load(Ordering::Relaxed) {
                    for sample in data.iter_mut() {
                        *sample = 0.0;
                    }
                    return;
                }

                // 获取音量
                let vol = f32::from_bits(volume.load(Ordering::Relaxed));

                // 调用回调获取音频数据
                if let Ok(mut cb) = callback.lock() {
                    if let Some(ref mut callback_fn) = *cb {
                        callback_fn(data);
                        // 应用音量
                        for sample in data.iter_mut() {
                            *sample *= vol;
                        }
                    } else {
                        // 没有回调，输出静音
                        for sample in data.iter_mut() {
                            *sample = 0.0;
                        }
                    }
                }
            },
            |err| eprintln!("Audio stream error: {}", err),
            None,
        )?;

        Ok(stream)
    }

    /// 构建 i16 格式的音频流
    fn build_stream_i16(
        &self,
        is_playing: Arc<AtomicBool>,
        volume: Arc<AtomicU32>,
        callback: Arc<Mutex<Option<AudioCallback>>>,
    ) -> anyhow::Result<Stream> {
        let config = self.config.clone();

        let stream = self.device.build_output_stream(
            &config,
            move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                if !is_playing.load(Ordering::Relaxed) {
                    for sample in data.iter_mut() {
                        *sample = 0;
                    }
                    return;
                }

                let vol = f32::from_bits(volume.load(Ordering::Relaxed));

                // 临时缓冲区用于 f32 数据
                let mut f32_buffer = vec![0.0f32; data.len()];

                if let Ok(mut cb) = callback.lock() {
                    if let Some(ref mut callback_fn) = *cb {
                        callback_fn(&mut f32_buffer);
                        // 转换为 i16 并应用音量
                        for (i, sample) in data.iter_mut().enumerate() {
                            let scaled = f32_buffer[i] * vol * i16::MAX as f32;
                            *sample = scaled.clamp(i16::MIN as f32, i16::MAX as f32) as i16;
                        }
                    } else {
                        for sample in data.iter_mut() {
                            *sample = 0;
                        }
                    }
                }
            },
            |err| eprintln!("Audio stream error: {}", err),
            None,
        )?;

        Ok(stream)
    }

    /// 构建 u16 格式的音频流
    fn build_stream_u16(
        &self,
        is_playing: Arc<AtomicBool>,
        volume: Arc<AtomicU32>,
        callback: Arc<Mutex<Option<AudioCallback>>>,
    ) -> anyhow::Result<Stream> {
        let config = self.config.clone();

        let stream = self.device.build_output_stream(
            &config,
            move |data: &mut [u16], _: &cpal::OutputCallbackInfo| {
                if !is_playing.load(Ordering::Relaxed) {
                    for sample in data.iter_mut() {
                        *sample = u16::MAX / 2;
                    }
                    return;
                }

                let vol = f32::from_bits(volume.load(Ordering::Relaxed));
                let mut f32_buffer = vec![0.0f32; data.len()];

                if let Ok(mut cb) = callback.lock() {
                    if let Some(ref mut callback_fn) = *cb {
                        callback_fn(&mut f32_buffer);
                        for (i, sample) in data.iter_mut().enumerate() {
                            let scaled = f32_buffer[i] * vol * i16::MAX as f32;
                            let biased = scaled + (u16::MAX / 2) as f32;
                            *sample = biased.clamp(0.0, u16::MAX as f32) as u16;
                        }
                    } else {
                        for sample in data.iter_mut() {
                            *sample = u16::MAX / 2;
                        }
                    }
                }
            },
            |err| eprintln!("Audio stream error: {}", err),
            None,
        )?;

        Ok(stream)
    }
}

impl Drop for CpalOutput {
    fn drop(&mut self) {
        self.stop();
    }
}

/// 音频输出设备管理器
pub struct AudioDeviceManager;

impl AudioDeviceManager {
    /// 获取默认输出设备名称
    pub fn default_device_name() -> Option<String> {
        let host = cpal::default_host();
        host.default_output_device()
            .and_then(|d| d.name().ok())
    }

    /// 列出所有可用的输出设备
    pub fn list_devices() -> Vec<String> {
        let host = cpal::default_host();
        host.output_devices()
            .map(|devices| {
                devices
                    .filter_map(|d| d.name().ok())
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpal_output_creation() {
        let output = CpalOutput::new();
        assert!(output.is_ok());
    }

    #[test]
    fn test_list_devices() {
        let devices = AudioDeviceManager::list_devices();
        println!("Available audio devices: {:?}", devices);
    }
}
