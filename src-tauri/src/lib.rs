//! 音乐元数据读取器库
//!
//! 支持多种音频格式的元数据解析，包括 FLAC、MP3、M4A、OGG、WAV 等格式。

pub mod error;
pub mod audio_metadata;
pub mod lyric_parser;
pub mod lyric_enhancer;
pub mod audio_engine;

pub use audio_metadata::{
    read_metadata,
    batch_read_metadata,
    AudioMetadata,
    AudioFormat,
    LyricLine,
    MetadataReader,
    MetadataError,
};

pub use lyric_enhancer::{
    EnhancedLyrics,
    enhance_metadata_with_lyrics,
    find_lyric_file,
};
pub use lyric_parser::{ParsedLyric, LyricFormat, ParseError};

pub mod music_source;
pub mod cache;
pub mod scanner;

pub use music_source::{
    SourceManager,
    SourceConfig,
    SourceType,
    MusicSource,
    MusicLibrary,
    TrackMetadata,
    Artist,
    ArtistSummary,
    Album,
    AlbumSummary,
    ArtistParser,
    AlbumIdGenerator,
};

pub use cache::{
    CacheManager,
    CacheError,
};

pub use scanner::{
    MusicScanner,
    ScanProgress,
    ScanOptions,
    ScanResult,
};

pub use error::{AppError, AppResult, ResultExt};

#[cfg(test)]
mod tests;

// 新增模块
pub mod constants;
pub mod state;
pub mod commands;

use std::path::PathBuf;
use tauri::Manager;
use audio_engine::SharedAudioPlayer;

/// 示例问候函数
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let cache_path = app.path().local_data_dir()
                .unwrap_or_else(|_| PathBuf::from("./cache"))
                .join("chordial");
            
            if !cache_path.exists() {
                std::fs::create_dir_all(&cache_path).ok();
            }
            
            let audio_player = SharedAudioPlayer::new()
                .map_err(|e| format!("Failed to initialize audio player: {}", e))?;
            
            let audio_analyzer = audio_engine::analyzer::create_shared_analyzer()
                .map_err(|e| format!("Failed to initialize audio analyzer: {}", e))?;
            
            let app_state = state::AppState::new(
                SourceManager::new(),
                CacheManager::with_directory(cache_path),
                MusicScanner::new(),
                audio_player,
                audio_analyzer,
            );
            
            app.manage(app_state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 示例命令
            greet,
            // 音乐源管理命令
            commands::add_local_source,
            commands::add_web_disk_source,
            commands::add_webdev_source,
            commands::remove_source,
            commands::get_all_sources,
            commands::set_source_enabled,
            commands::scan_all_sources,
            commands::refresh_source,
            commands::get_source_from_cache,
            // 音乐库查询命令
            commands::get_cached_library,
            commands::clear_all_cache,
            commands::get_cache_size,
            commands::get_track_info,
            commands::get_tracks_by_ids,
            commands::get_album_art,
            commands::get_music_file,
            commands::get_artist_image,
            commands::get_artist_info,
            commands::get_artist_summary,
            commands::get_album_info,
            commands::get_album_summary,
            commands::get_artists_by_ids,
            commands::get_albums_by_ids,
            commands::get_all_artists,
            commands::get_all_albums,
            commands::get_lyrics,
            commands::parse_lyric_content,
            commands::detect_lyric_format,
            // 音频播放控制命令
            commands::play_audio,
            commands::pause_audio,
            commands::resume_audio,
            commands::stop_audio,
            commands::seek_audio,
            commands::set_audio_volume,
            commands::get_audio_volume,
            commands::get_audio_position,
            commands::get_audio_duration,
            commands::get_audio_state,
            commands::is_audio_playing,
            // 双缓冲与无缝切换命令
            commands::preload_next_audio,
            commands::get_next_audio_path,
            commands::set_crossfade_enabled,
            commands::is_crossfade_enabled,
            commands::set_crossfade_config,
            // BPM同步与时间拉伸命令
            commands::set_current_track_bpm,
            commands::set_next_track_bpm,
            commands::set_bpm_sync_enabled,
            commands::is_bpm_sync_enabled,
            commands::get_playback_speed_ratio,
            commands::set_playback_speed,
            // 节拍检测与BPM分析命令
            commands::analyze_audio_beat,
            commands::reanalyze_audio_beat,
            commands::get_mix_points,
            commands::batch_analyze_audio,
            commands::get_analysis_cache_stats,
            commands::clear_analysis_cache,
            // 窗口控制命令
            commands::toggle_always_on_top,
            commands::close_window,
            commands::minimize_window,
            commands::toggle_maximize,
            commands::get_window_position,
            commands::set_window_position,
            commands::get_window_size,
            commands::set_window_size,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
