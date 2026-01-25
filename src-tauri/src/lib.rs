//! 音乐元数据读取器库
//!
//! 支持多种音频格式的元数据解析，包括 FLAC、MP3、M4A、OGG、WAV 等格式。

pub mod audio_metadata;
pub mod lyric_parser;
pub mod lyric_enhancer;

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

#[cfg(test)]
mod tests;

use std::sync::Mutex;
use std::path::PathBuf;
use tauri::{State, Manager};
use tauri::ipc::Response;

struct AppState {
    source_manager: Mutex<SourceManager>,
    cache_manager: Mutex<CacheManager>,
    scanner: Mutex<MusicScanner>,
}

#[tauri::command]
fn add_local_source(
    state: State<AppState>,
    path: String,
    recursive: bool,
) -> Result<SourceConfig, String> {
    let mut source_manager = state.source_manager.lock().map_err(|e| e.to_string())?;
    let path = PathBuf::from(path);
    source_manager.add_local_folder(path, recursive)
}

#[tauri::command]
fn add_web_disk_source(
    state: State<AppState>,
    url: String,
    username: Option<String>,
    password: Option<String>,
) -> Result<SourceConfig, String> {
    let mut source_manager = state.source_manager.lock().map_err(|e| e.to_string())?;
    let auth = match (username, password) {
        (Some(u), Some(p)) => Some((u, p)),
        _ => None,
    };
    source_manager.add_web_disk(PathBuf::from(url), auth)
}

#[tauri::command]
fn remove_source(
    state: State<AppState>,
    id: String,
) -> Result<bool, String> {
    let mut source_manager = state.source_manager.lock().map_err(|e| e.to_string())?;
    let cache_manager = state.cache_manager.lock().map_err(|e| e.to_string())?;
    
    if source_manager.remove_source(&id).is_some() {
        let _ = cache_manager.delete_source_cache(&id);
        Ok(true)
    } else {
        Err("源不存在".to_string())
    }
}

#[tauri::command]
fn get_all_sources(
    state: State<AppState>,
) -> Result<Vec<SourceConfig>, String> {
    let source_manager = state.source_manager.lock().map_err(|e| e.to_string())?;
    Ok(source_manager.get_all_sources().to_vec())
}

#[tauri::command]
fn set_source_enabled(
    state: State<AppState>,
    id: String,
    enabled: bool,
) -> Result<bool, String> {
    let mut source_manager = state.source_manager.lock().map_err(|e| e.to_string())?;
    Ok(source_manager.set_source_enabled(&id, enabled))
}

#[tauri::command]
fn scan_all_sources(
    state: State<AppState>,
) -> Result<MusicLibrary, String> {
    let mut source_manager = state.source_manager.lock().map_err(|e| e.to_string())?;
    let mut cache_manager = state.cache_manager.lock().map_err(|e| e.to_string())?;
    let scanner = state.scanner.lock().map_err(|e| e.to_string())?;
    
    let sources = source_manager.get_all_sources().to_vec();
    let options = ScanOptions {
        force_rescan: true,
        parallel_tasks: 4,
        progress_callback: None,
    };
    
    let results = scanner.scan_sources(&sources, &options);
    
    let mut library = MusicLibrary {
        sources: source_manager.get_all_sources().to_vec(),
        tracks: Vec::new(),
    };
    
    for result in results {
        library.tracks.extend(result.tracks);
        
        if let Some(source) = source_manager.get_source_mut(&result.source.id) {
            source.set_last_scanned_at(chrono::Utc::now());
        }
    }
    
    let _ = cache_manager.save_sources(&source_manager);
    let _ = cache_manager.save_library(&library);
    
    Ok(library)
}

#[tauri::command]
fn get_cached_library(
    state: State<AppState>,
) -> Result<Option<MusicLibrary>, String> {
    let cache_manager = state.cache_manager.lock().map_err(|e| e.to_string())?;
    Ok(cache_manager.load_library().ok())
}

#[tauri::command]
fn get_source_from_cache(
    state: State<AppState>,
    source_id: String,
) -> Result<Option<Vec<TrackMetadata>>, String> {
    let scanner = state.scanner.lock().map_err(|e| e.to_string())?;
    Ok(scanner.load_from_cache(&source_id))
}

#[tauri::command]
fn refresh_source(
    state: State<AppState>,
    source_id: String,
) -> Result<Vec<TrackMetadata>, String> {
    let mut source_manager = state.source_manager.lock().map_err(|e| e.to_string())?;
    let mut cache_manager = state.cache_manager.lock().map_err(|e| e.to_string())?;
    let scanner = state.scanner.lock().map_err(|e| e.to_string())?;
    
    let source = source_manager.get_source(&source_id)
        .ok_or("源不存在")?;
    
    let options = ScanOptions {
        force_rescan: true,
        parallel_tasks: 4,
        progress_callback: None,
    };
    
    let result = scanner.scan_source(source, &options);
    
    if let Some(mut source) = source_manager.get_source_mut(&source_id).cloned() {
        source.set_last_scanned_at(chrono::Utc::now());
        let _ = cache_manager.save_sources(&source_manager);
    }
    
    scanner.save_to_cache(&source_id, &result.tracks);
    
    Ok(result.tracks)
}

#[tauri::command]
fn clear_all_cache(
    state: State<AppState>,
) -> Result<(), String> {
    let cache_manager = state.cache_manager.lock().map_err(|e| e.to_string())?;
    cache_manager.clear_all_cache()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_cache_size(
    state: State<AppState>,
) -> Result<u64, String> {
    let cache_manager = state.cache_manager.lock().map_err(|e| e.to_string())?;
    cache_manager.cache_size().map_err(|e| e.to_string())
}

/// 获取曲目完整信息
#[tauri::command(rename_all = "snake_case")]
fn get_track_info(
    state: State<AppState>,
    track_id: String,
) -> Result<TrackMetadata, String> {
    let source_manager = state.source_manager.lock().map_err(|e| e.to_string())?;
    let cache_manager = state.cache_manager.lock().map_err(|e| e.to_string())?;
    
    // 尝试从缓存加载音乐库
    if let Some(library) = cache_manager.load_library().ok() {
        if let Some(track) = library.tracks.into_iter().find(|t| t.id == track_id) {
            return Ok(track);
        }
    }
    
    Err("曲目不存在".to_string())
}

/// 获取专辑图片（使用 Response 传递二进制数据）
#[tauri::command(rename_all = "snake_case")]
fn get_album_art(
    state: State<AppState>,
    album_id: String,
    size: String,
) -> Result<Response, String> {
    // 现在专辑封面数据已经包含在曲目信息中
    // 这里保留向后兼容：如果前端需要单独的封面获取
    // 可以通过曲目ID查找
    let cache_manager = state.cache_manager.lock().map_err(|e| e.to_string())?;
    
    if let Some(library) = cache_manager.load_library().ok() {
        for track in library.tracks {
            if track.album_id.as_ref() == Some(&album_id) {
                if let Some(cover_data) = &track.album_cover_data {
                    // 解析 Data URL
                    if cover_data.starts_with("data:image/") {
                        let parts: Vec<&str> = cover_data.splitn(2, ",").collect();
                        if parts.len() == 2 {
                            let data = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, parts[1])
                                .map_err(|e| e.to_string())?;
                            return Ok(Response::new(data));
                        }
                    }
                }
            }
        }
    }
    
    // 返回默认封面（1x1 透明像素）
    let default_art = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]; // PNG 头
    return Ok(Response::new(default_art));
}

/// 获取音乐文件（使用二进制响应）
#[tauri::command(rename_all = "snake_case")]
fn get_music_file(
    state: State<AppState>,
    track_id: String,
) -> Result<Response, String> {
    let source_manager = state.source_manager.lock().map_err(|e| e.to_string())?;
    let cache_manager = state.cache_manager.lock().map_err(|e| e.to_string())?;
    
    // 尝试从缓存加载曲目信息
    if let Some(library) = cache_manager.load_library().ok() {
        if let Some(track) = library.tracks.into_iter().find(|t| t.id == track_id) {
            if track.path.exists() {
                let data = std::fs::read(&track.path)
                    .map_err(|e| e.to_string())?;
                println!("成功读取音乐文件: {} ({} bytes)", track_id, data.len());
                return Ok(Response::new(data));
            } else {
                println!("音乐文件不存在: {:?}", track.path);
            }
        } else {
            println!("未找到曲目: {}", track_id);
        }
    } else {
        println!("无法加载音乐库缓存");
    }
    
    Err("音乐文件不存在".to_string())
}

/// 获取歌手图片
#[tauri::command(rename_all = "snake_case")]
fn get_artist_image(
    state: State<AppState>,
    artist_id: String,
) -> Result<Response, String> {
    let file_path = PathBuf::from("./artist_images").join(format!("{}.jpg", artist_id));
    
    if !file_path.exists() {
        // 返回默认图片
        let default_image = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        return Ok(Response::new(default_image));
    }
    
    let data = std::fs::read(&file_path)
        .map_err(|e| e.to_string())?;
    
    Ok(Response::new(data))
}

/// 获取歌词
#[tauri::command(rename_all = "snake_case")]
fn get_lyrics(
    state: State<AppState>,
    track_id: String,
) -> Result<serde_json::Value, String> {
    let cache_manager = state.cache_manager.lock().map_err(|e| e.to_string())?;
    
    if let Some(library) = cache_manager.load_library().ok() {
        if let Some(track) = library.tracks.into_iter().find(|t| t.id == track_id) {
            // 返回歌词信息
            return Ok(serde_json::json!({
                "plain_lyrics": track.lyrics.clone().unwrap_or_default(),
                "synced_lyrics": track.synced_lyrics.clone().unwrap_or_default(),
                "has_synced_lyrics": track.synced_lyrics.is_some(),
                "has_plain_lyrics": track.lyrics.is_some()
            }));
        }
    }
    
    Ok(serde_json::json!({
        "plain_lyrics": "",
        "synced_lyrics": "",
        "has_synced_lyrics": false,
        "has_plain_lyrics": false
    }))
}

/// 解析歌词内容
#[tauri::command]
fn parse_lyric_content(
    content: String,
    format: Option<String>,
) -> Result<ParsedLyric, String> {
    use crate::lyric_parser::{LyricParser, LyricFormat, ParsedLyric};
    
    let parser = LyricParser::new();
    
    let lyric_format = if let Some(fmt) = format {
        match fmt.to_lowercase().as_str() {
            "lrc" => LyricFormat::Lrc,
            "yrc" => LyricFormat::Yrc,
            "qrc" => LyricFormat::Qrc,
            "ttml" => LyricFormat::Ttml,
            _ => LyricFormat::Unknown,
        }
    } else {
        LyricFormat::from_content(&content)
    };
    
    parser.parse(&content, lyric_format)
        .map_err(|e| e.to_string())
}

/// 获取歌词格式
#[tauri::command]
fn detect_lyric_format(
    content: String,
) -> String {
    use crate::lyric_parser::LyricFormat;
    
    match LyricFormat::from_content(&content) {
        LyricFormat::Lrc => "lrc".to_string(),
        LyricFormat::Yrc => "yrc".to_string(),
        LyricFormat::Qrc => "qrc".to_string(),
        LyricFormat::Ttml => "ttml".to_string(),
        LyricFormat::Unknown => "unknown".to_string(),
    }
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let cache_path = app.path().local_data_dir()
                .unwrap_or_else(|_| PathBuf::from("./cache"))
                .join("chordial");
            
            if !cache_path.exists() {
                std::fs::create_dir_all(&cache_path).ok();
            }
            
            let app_state = AppState {
                source_manager: Mutex::new(SourceManager::new()),
                cache_manager: Mutex::new(CacheManager::with_directory(cache_path)),
                scanner: Mutex::new(MusicScanner::new()),
            };
            
            app.manage(app_state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            add_local_source,
            add_web_disk_source,
            remove_source,
            get_all_sources,
            set_source_enabled,
            scan_all_sources,
            get_cached_library,
            get_source_from_cache,
            refresh_source,
            clear_all_cache,
            get_cache_size,
            get_track_info,
            get_album_art,
            get_music_file,
            get_artist_image,
            get_lyrics,
            parse_lyric_content,
            detect_lyric_format,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
