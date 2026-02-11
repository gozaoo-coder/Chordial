//! 应用状态管理

use std::sync::Mutex;
use crate::music_source::SourceManager;
use crate::cache::CacheManager;
use crate::scanner::MusicScanner;
use crate::audio_engine::{SharedAudioPlayer, analyzer::SharedAudioAnalyzer};

/// 应用状态结构体
pub struct AppState {
    pub source_manager: Mutex<SourceManager>,
    pub cache_manager: Mutex<CacheManager>,
    pub scanner: Mutex<MusicScanner>,
    pub audio_player: Mutex<SharedAudioPlayer>,
    pub audio_analyzer: Mutex<SharedAudioAnalyzer>,
}

impl AppState {
    /// 创建新的应用状态
    pub fn new(
        source_manager: SourceManager,
        cache_manager: CacheManager,
        scanner: MusicScanner,
        audio_player: SharedAudioPlayer,
        audio_analyzer: SharedAudioAnalyzer,
    ) -> Self {
        Self {
            source_manager: Mutex::new(source_manager),
            cache_manager: Mutex::new(cache_manager),
            scanner: Mutex::new(scanner),
            audio_player: Mutex::new(audio_player),
            audio_analyzer: Mutex::new(audio_analyzer),
        }
    }
}

/// 简化锁获取的宏 - 返回 Result<String, String> 类型，用于 Tauri 命令
#[macro_export]
macro_rules! lock_state {
    ($state:expr, $field:ident) => {
        $state.$field.lock().map_err(|e| format!("锁被污染: {}", e))
    };
}

/// 简化锁获取（用于不返回 Result 的命令）
/// 如果锁被污染，会打印错误并 panic（这是不可恢复的错误）
#[macro_export]
macro_rules! lock_state_unwrap {
    ($state:expr, $field:ident) => {
        match $state.$field.lock() {
            Ok(guard) => guard,
            Err(e) => {
                eprintln!("严重错误: 锁被污染 - {}", e);
                panic!("锁被污染，应用状态不一致: {}", e)
            }
        }
    };
}
