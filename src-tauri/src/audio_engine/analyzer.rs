use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};

use super::decoder::SymphoniaDecoder;
use super::beat_detection::{BeatDetector, AnalysisResult};
use super::analysis_cache::AnalysisCache;

/// 音频分析器 - 整合解码和节拍检测
pub struct AudioAnalyzer {
    cache: Arc<Mutex<AnalysisCache>>,
}

impl AudioAnalyzer {
    /// 创建新的音频分析器
    pub fn new(cache: AnalysisCache) -> Self {
        Self {
            cache: Arc::new(Mutex::new(cache)),
        }
    }
    
    /// 使用应用数据目录创建分析器
    pub fn with_app_data() -> anyhow::Result<Self> {
        let cache = AnalysisCache::in_app_data()?;
        Ok(Self::new(cache))
    }
    
    /// 分析音频文件（带缓存）
    pub fn analyze_file(&self, file_path: &str) -> anyhow::Result<AnalysisResult> {
        let cache = self.cache.lock().unwrap();
        
        cache.get_or_analyze(file_path, || {
            // 解码音频文件
            let (pcm_data, sample_rate) = self.decode_to_mono(file_path)?;
            
            // 执行节拍检测
            let detector = BeatDetector::new(sample_rate);
            let result = detector.analyze(&pcm_data);
            
            Ok(result)
        })
    }
    
    /// 强制重新分析（忽略缓存）
    pub fn analyze_file_force(&self, file_path: &str) -> anyhow::Result<AnalysisResult> {
        // 使缓存失效
        {
            let cache = self.cache.lock().unwrap();
            let _ = cache.invalidate(file_path);
        }
        
        // 重新分析
        self.analyze_file(file_path)
    }
    
    /// 解码音频文件为单声道f32 PCM
    /// 返回 (PCM数据, 采样率)
    fn decode_to_mono(&self, file_path: &str) -> anyhow::Result<(Vec<f32>, u32)> {
        let mut decoder = SymphoniaDecoder::new(file_path)
            .map_err(|e| anyhow::anyhow!("解码器创建失败: {}", e))?;
        
        let sample_rate = decoder.sample_rate();
        
        let mut mono_samples = Vec::new();
        
        // 解码所有帧
        loop {
            match decoder.next_frame() {
                Ok(Some(frame)) => {
                    // 转换为单声道（如果是立体声则取平均）
                    let channels = decoder.channels() as usize;
                    if channels == 1 {
                        mono_samples.extend_from_slice(&frame.samples);
                    } else {
                        // 多声道转单声道
                        for chunk in frame.samples.chunks_exact(channels) {
                            let sum: f32 = chunk.iter().sum();
                            mono_samples.push(sum / channels as f32);
                        }
                    }
                }
                Ok(None) => break,  // 解码完成
                Err(e) => return Err(anyhow::anyhow!("解码错误: {}", e)),
            }
        }
        
        if mono_samples.is_empty() {
            return Err(anyhow::anyhow!("未能解码任何音频数据"));
        }
        
        Ok((mono_samples, sample_rate))
    }
    
    /// 获取缓存统计
    pub fn get_cache_stats(&self) -> anyhow::Result<super::analysis_cache::CacheStats> {
        let cache = self.cache.lock().unwrap();
        cache.get_stats()
    }
    
    /// 清空缓存
    pub fn clear_cache(&self) -> anyhow::Result<()> {
        let cache = self.cache.lock().unwrap();
        cache.clear_all()
    }
    
    /// 批量分析文件
    pub fn batch_analyze(
        &self,
        paths: Vec<String>,
        app_handle: Option<AppHandle>,
    ) -> Vec<(String, anyhow::Result<AnalysisResult>)> {
        use rayon::prelude::*;
        use std::sync::atomic::{AtomicUsize, Ordering};
        
        let total = paths.len();
        let processed = Arc::new(AtomicUsize::new(0));
        let cache = Arc::clone(&self.cache);
        
        paths
            .into_par_iter()
            .map(|path| {
                // 先尝试从缓存加载（短暂持锁）
                let cached_result = {
                    match cache.lock() {
                        Ok(cache_guard) => cache_guard.load(&path).ok().flatten(),
                        Err(_) => None,
                    }
                };
                
                // 如果缓存有效，直接返回
                let result = if let Some(result) = cached_result {
                    Ok(result)
                } else {
                    // 执行分析（无锁状态）
                    let analysis_result = (|| {
                        let (pcm_data, sample_rate) = self.decode_to_mono(&path)?;
                        let detector = BeatDetector::new(sample_rate);
                        Ok(detector.analyze(&pcm_data))
                    })();
                    
                    // 保存到缓存（短暂持锁）
                    if let Ok(ref result) = analysis_result {
                        if let Ok(cache_guard) = cache.lock() {
                            let _ = cache_guard.save(&path, result);
                        }
                    }
                    
                    analysis_result
                };
                
                // 更新进度
                let count = processed.fetch_add(1, Ordering::SeqCst) + 1;
                
                // 发送进度事件
                if let Some(ref handle) = app_handle {
                    let _ = handle.emit(
                        "analysis_progress",
                        serde_json::json!({
                            "current": count,
                            "total": total,
                            "percent": (count as f32 / total as f32) * 100.0,
                            "file": path.clone(),
                        }),
                    );
                }
                
                (path, result)
            })
            .collect()
    }
    
    /// 获取建议的切歌点
    pub fn find_mix_points(
        &self,
        file_path: &str,
    ) -> anyhow::Result<MixPoints> {
        let result = self.analyze_file(file_path)?;
        let duration = self.get_audio_duration(file_path)?;
        
        // 找到切出点（歌曲结尾前10秒内的最后一个Beat）
        let mix_out_point = result.beat_positions
            .iter()
            .filter(|&&t| t < duration - 10.0)
            .last()
            .copied();
        
        // 找到切入点（第一拍或第5拍）
        let mix_in_point = result.downbeat_position
            .or_else(|| result.beat_positions.get(0).copied());
        
        Ok(MixPoints {
            bpm: result.bpm,
            mix_in_point,
            mix_out_point,
            all_beats: result.beat_positions,
            duration,
        })
    }
    
    /// 获取音频时长（秒）
    fn get_audio_duration(&self, file_path: &str) -> anyhow::Result<f64> {
        let decoder = SymphoniaDecoder::new(file_path)
            .map_err(|e| anyhow::anyhow!("解码器创建失败: {}", e))?;
        
        // 尝试从元数据获取时长
        if let Some(duration) = decoder.duration() {
            Ok(duration)
        } else {
            // 估算时长
            let sample_rate = decoder.sample_rate() as f64;
            let channels = decoder.channels() as f64;
            
            // 获取文件大小并估算
            let metadata = std::fs::metadata(file_path)?;
            let file_size = metadata.len() as f64;
            
            // 粗略估算（假设平均比特率）
            let estimated_duration = file_size / (sample_rate * channels * 2.0);
            Ok(estimated_duration)
        }
    }
}

/// 混音点信息
#[derive(Debug, Clone)]
pub struct MixPoints {
    pub bpm: f64,
    pub mix_in_point: Option<f64>,
    pub mix_out_point: Option<f64>,
    pub all_beats: Vec<f64>,
    pub duration: f64,
}



/// 可共享的音频分析器
pub type SharedAudioAnalyzer = Arc<Mutex<AudioAnalyzer>>;

/// 创建共享分析器
pub fn create_shared_analyzer() -> anyhow::Result<SharedAudioAnalyzer> {
    let analyzer = AudioAnalyzer::with_app_data()?;
    Ok(Arc::new(Mutex::new(analyzer)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_analyzer_creation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_cache.db");
        let cache = AnalysisCache::new(&db_path).unwrap();
        
        let analyzer = AudioAnalyzer::new(cache);
        let stats = analyzer.get_cache_stats().unwrap();
        
        assert_eq!(stats.entry_count, 0);
    }
}
