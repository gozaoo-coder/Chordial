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

/// 简化锁获取的宏
#[macro_export]
macro_rules! lock_state {
    ($state:expr, $field:ident) => {
        $state.$field.lock().map_err(|e| e.to_string())
    };
}

/// 简化锁获取（使用 unwrap，适用于不返回 Result 的命令）
#[macro_export]
macro_rules! lock_state_unwrap {
    ($state:expr, $field:ident) => {
        $state.$field.lock().unwrap()
    };
}
