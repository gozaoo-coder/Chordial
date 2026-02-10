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

use std::sync::Mutex;
use std::path::PathBuf;
use tauri::{State, Manager, AppHandle};
use tauri::ipc::Response;
use audio_engine::{SharedAudioPlayer, PlaybackState};
use audio_engine::analyzer::SharedAudioAnalyzer;

/// 默认透明 PNG 图片数据（1x1 像素）
const DEFAULT_TRANSPARENT_PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
    0x00, 0x00, 0x00, 0x0D, // IHDR chunk length
    0x49, 0x48, 0x44, 0x52, // IHDR
    0x00, 0x00, 0x00, 0x01, // width: 1
    0x00, 0x00, 0x00, 0x01, // height: 1
    0x08, 0x06, 0x00, 0x00, 0x00, // 8-bit RGBA
    0x1F, 0x15, 0xC4, 0x89, // IHDR CRC
    0x00, 0x00, 0x00, 0x0A, // IDAT chunk length
    0x49, 0x44, 0x41, 0x54, // IDAT
    0x78, 0x9C, 0x63, 0x60, 0x00, 0x00, 0x00, 0x02, 0x00, 0x01,
    0xE2, 0x21, 0xBC, 0x33, // IDAT CRC
    0x00, 0x00, 0x00, 0x00, // IEND chunk length
    0x49, 0x45, 0x4E, 0x44, // IEND
    0xAE, 0x42, 0x60, 0x82, // IEND CRC
];

struct AppState {
    source_manager: Mutex<SourceManager>,
    cache_manager: Mutex<CacheManager>,
    scanner: Mutex<MusicScanner>,
    audio_player: Mutex<SharedAudioPlayer>,
    audio_analyzer: Mutex<SharedAudioAnalyzer>,
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
fn add_webdev_source(
    state: State<AppState>,
    api_base_url: String,
    name: Option<String>,
    api_key: Option<String>,
    auth_token: Option<String>,
) -> Result<SourceConfig, String> {
    let mut source_manager = state.source_manager.lock().map_err(|e| e.to_string())?;
    source_manager.add_web_dev(api_base_url, name, api_key, auth_token)
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
    // 先获取源列表和扫描器，然后立即释放锁
    let (sources, scanner) = {
        let source_manager = state.source_manager.lock().map_err(|e| e.to_string())?;
        let scanner = state.scanner.lock().map_err(|e| e.to_string())?;
        (source_manager.get_all_sources().to_vec(), scanner.clone())
    };

    let options = ScanOptions {
        force_rescan: true,
        parallel_tasks: 4,
        progress_callback: None,
    };

    // 执行长时间扫描操作（不持有锁）
    let results = scanner.scan_sources(&sources, &options);

    // 重新获取锁来更新状态和保存结果
    let mut library = MusicLibrary::new();
    {
        let mut source_manager = state.source_manager.lock().map_err(|e| e.to_string())?;
        let cache_manager = state.cache_manager.lock().map_err(|e| e.to_string())?;

        library.sources = source_manager.get_all_sources().to_vec();

        // 用于合并 Artist 和 Album 数据
        let mut artists_map: std::collections::HashMap<String, Artist> = std::collections::HashMap::new();
        let mut albums_map: std::collections::HashMap<String, Album> = std::collections::HashMap::new();

        for result in &results {
            // 合并曲目
            library.tracks.extend(result.tracks.clone());

            // 合并歌手数据
            for artist in &result.artists {
                artists_map.entry(artist.id.clone())
                    .and_modify(|existing| {
                        // 合并专辑和歌曲列表
                        for album_id in &artist.album_ids {
                            existing.add_album(album_id.clone());
                        }
                        for track_id in &artist.track_ids {
                            existing.add_track(track_id.clone());
                        }
                        // 封面数据不再存储到内存，改为按需从音乐文件读取
                    })
                    .or_insert_with(|| artist.clone());
            }

            // 合并专辑数据
            for album in &result.albums {
                albums_map.entry(album.id.clone())
                    .and_modify(|existing| {
                        // 合并歌曲列表
                        for track_id in &album.track_ids {
                            existing.add_track(track_id.clone());
                        }
                        // 更新总时长
                        existing.total_duration += album.total_duration;
                        // 封面数据不再存储到内存，改为按需从音乐文件读取
                    })
                    .or_insert_with(|| album.clone());
            }

            if let Some(source) = source_manager.get_source_mut(&result.source.id) {
                source.set_last_scanned_at(chrono::Utc::now());
            }
        }

        // 将 HashMap 转换为 Vec
        library.artists = artists_map.into_values().collect();
        library.albums = albums_map.into_values().collect();

        let _ = cache_manager.save_sources(&source_manager);
        let _ = cache_manager.save_library(&library);
    }

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
    let cache_manager = state.cache_manager.lock().map_err(|e| e.to_string())?;
    let scanner = state.scanner.lock().map_err(|e| e.to_string())?;
    
    // 先获取源的克隆，释放锁
    let source = {
        let source_manager = state.source_manager.lock().map_err(|e| e.to_string())?;
        source_manager.get_source(&source_id)
            .ok_or("源不存在")?
            .clone()
    };
    
    let options = ScanOptions {
        force_rescan: true,
        parallel_tasks: 4,
        progress_callback: None,
    };
    
    let result = scanner.scan_source(&source, &options);
    
    // 重新获取锁来更新源
    {
        let mut source_manager = state.source_manager.lock().map_err(|e| e.to_string())?;
        if let Some(src) = source_manager.get_source_mut(&source_id) {
            src.set_last_scanned_at(chrono::Utc::now());
        }
        let _ = cache_manager.save_sources(&source_manager);
    }
    
    // 保存到源缓存
    scanner.save_to_cache(&source_id, &result.tracks);
    
    // 更新 library 缓存：移除该源旧的 tracks，添加新的 tracks，重新构建 artists 和 albums
    if let Ok(mut library) = cache_manager.load_library() {
        // 移除该源旧的 tracks
        library.tracks.retain(|t| t.source_id != source_id);
        
        // 添加新的 tracks
        library.tracks.extend(result.tracks.clone());
        
        // 重新构建 artists 和 albums（使用引用避免克隆整个列表）
        let (artists, albums, enriched_tracks) = scanner.build_artist_album_data(&library.tracks);
        library.artists = artists;
        library.albums = albums;
        library.tracks = enriched_tracks;
        
        // 保存更新后的 library
        let _ = cache_manager.save_library(&library);
    }
    
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
    let cache_manager = state.cache_manager.lock().map_err(|e| e.to_string())?;

    // 尝试从缓存加载音乐库
    match cache_manager.load_library() {
        Ok(library) => {
            if let Some(track) = library.tracks.into_iter().find(|t| t.id == track_id) {
                return Ok(track);
            }
        }
        Err(e) => {
            log::warn!("加载音乐库失败: {}", e);
        }
    }

    Err("曲目不存在".to_string())
}

/// 批量获取曲目信息
#[tauri::command(rename_all = "snake_case")]
fn get_tracks_by_ids(
    state: State<AppState>,
    track_ids: Vec<String>,
) -> Result<Vec<TrackMetadata>, String> {
    let _source_manager = state.source_manager.lock().map_err(|e| e.to_string())?;
    let cache_manager = state.cache_manager.lock().map_err(|e| e.to_string())?;

    // 尝试从缓存加载音乐库
    if let Some(library) = cache_manager.load_library().ok() {
        let tracks: Vec<TrackMetadata> = library.tracks
            .into_iter()
            .filter(|t| track_ids.contains(&t.id))
            .collect();
        return Ok(tracks);
    }

    Ok(Vec::new())
}

/// 获取专辑图片（使用 Response 传递二进制数据）
/// 按需从音乐文件读取封面，不依赖缓存
#[tauri::command(rename_all = "snake_case")]
fn get_album_art(
    state: State<AppState>,
    album_id: String,
    _size: String,
) -> Result<Response, String> {
    let cache_manager = state.cache_manager.lock().map_err(|e| e.to_string())?;
    
    // 找到属于该专辑的第一首歌曲
    let track_path = cache_manager.load_library()
        .ok()
        .and_then(|library| {
            library.tracks.into_iter()
                .find(|t| t.album_id.as_ref() == Some(&album_id))
                .map(|t| t.path)
        });
    
    // 如果找到歌曲，从音乐文件读取封面
    if let Some(path) = track_path {
        if let Ok(metadata) = read_metadata(&path) {
            // 获取第一张封面图片
            if let Some(picture) = metadata.pictures.iter()
                .find(|p| p.is_cover())
                .or_else(|| metadata.pictures.first()) {
                return Ok(Response::new(picture.data.clone()));
            }
        }
    }
    
    // 返回默认封面（1x1 透明 PNG）
    return Ok(Response::new(DEFAULT_TRANSPARENT_PNG.to_vec()));
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
        if let Some(track) = library.tracks.iter().find(|t| t.id == track_id) {
            // 获取源信息以确定源类型
            if let Some(source) = source_manager.get_source(&track.source_id) {
                match source.source_type {
                    SourceType::LocalFolder | SourceType::WebDisk => {
                        // 本地文件或网盘源，直接读取文件
                        if track.path.exists() {
                            let data = std::fs::read(&track.path)
                                .map_err(|e| e.to_string())?;
                            println!("成功读取音乐文件: {} ({} bytes)", track_id, data.len());
                            return Ok(Response::new(data));
                        } else {
                            println!("音乐文件不存在: {:?}", track.path);
                        }
                    }
                    SourceType::WebDev => {
                        // WebDev 源，从 URL 下载文件
                        let file_url = track.path.to_string_lossy().to_string();
                        return download_webdev_file(&source, &file_url);
                    }
                }
            }
        } else {
            println!("未找到曲目: {}", track_id);
        }
    } else {
        println!("无法加载音乐库缓存");
    }

    Err("音乐文件不存在".to_string())
}

/// 从 WebDev 源下载文件
fn download_webdev_file(source: &SourceConfig, file_url: &str) -> Result<Response, String> {
    let auth = source.options().webdev_auth.as_ref()
        .ok_or("WebDev 认证信息缺失")?;

    // 创建 HTTP 客户端
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    // 构建请求
    let mut request = client.get(file_url);

    // 添加认证头
    if let Some(api_key) = &auth.api_key {
        request = request.header("X-API-Key", api_key);
    }
    if let Some(auth_token) = &auth.auth_token {
        request = request.header("Authorization", format!("Bearer {}", auth_token));
    }

    // 发送请求
    let response = request.send()
        .map_err(|e| format!("下载文件失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("下载文件失败，状态码: {}", response.status()));
    }

    // 读取响应体
    let data = response.bytes()
        .map_err(|e| format!("读取响应数据失败: {}", e))?;

    println!("成功下载 WebDev 音乐文件: {} ({} bytes)", file_url, data.len());
    Ok(Response::new(data.to_vec()))
}

/// 获取歌手图片（使用二进制响应）
/// 按需从音乐文件读取封面，不依赖缓存
#[tauri::command(rename_all = "snake_case")]
fn get_artist_image(
    state: State<AppState>,
    artist_id: String,
) -> Result<Response, String> {
    let cache_manager = state.cache_manager.lock().map_err(|e| e.to_string())?;
    
    // 找到该歌手的第一首歌曲，从音乐文件读取封面
    if let Some(library) = cache_manager.load_library().ok() {
        // 找到属于该歌手的第一首歌曲
        let track_path = library.tracks.iter()
            .find(|t| t.artist_id.as_ref() == Some(&artist_id))
            .map(|t| t.path.clone());
        
        // 如果找到歌曲，从音乐文件读取封面
        if let Some(path) = track_path {
            if let Ok(metadata) = read_metadata(&path) {
                // 获取第一张封面图片
                if let Some(picture) = metadata.pictures.iter()
                    .find(|p| p.is_cover())
                    .or_else(|| metadata.pictures.first()) {
                    return Ok(Response::new(picture.data.clone()));
                }
            }
        }
    }
    
    // 尝试从本地文件加载
    let file_path = PathBuf::from("./artist_images").join(format!("{}.jpg", artist_id));
    
    if file_path.exists() {
        let data = std::fs::read(&file_path)
            .map_err(|e| e.to_string())?;
        return Ok(Response::new(data));
    }
    
    // 返回默认图片（1x1 透明 PNG）
    Ok(Response::new(DEFAULT_TRANSPARENT_PNG.to_vec()))
}

/// 获取歌手完整信息
#[tauri::command(rename_all = "snake_case")]
fn get_artist_info(
    state: State<AppState>,
    artist_id: String,
) -> Result<Artist, String> {
    let cache_manager = state.cache_manager.lock().map_err(|e| e.to_string())?;
    
    if let Some(library) = cache_manager.load_library().ok() {
        if let Some(artist) = library.artists.iter().find(|a| a.id == artist_id) {
            return Ok(artist.clone());
        }
    }
    
    Err("歌手不存在".to_string())
}

/// 获取歌手摘要信息
#[tauri::command(rename_all = "snake_case")]
fn get_artist_summary(
    state: State<AppState>,
    artist_id: String,
) -> Result<ArtistSummary, String> {
    let cache_manager = state.cache_manager.lock().map_err(|e| e.to_string())?;
    
    if let Some(library) = cache_manager.load_library().ok() {
        if let Some(artist) = library.artists.iter().find(|a| a.id == artist_id) {
            return Ok(artist.to_summary());
        }
    }
    
    Err("歌手不存在".to_string())
}

/// 获取专辑完整信息
#[tauri::command(rename_all = "snake_case")]
fn get_album_info(
    state: State<AppState>,
    album_id: String,
) -> Result<Album, String> {
    let cache_manager = state.cache_manager.lock().map_err(|e| e.to_string())?;
    
    if let Some(library) = cache_manager.load_library().ok() {
        if let Some(album) = library.albums.iter().find(|a| a.id == album_id) {
            return Ok(album.clone());
        }
    }
    
    Err("专辑不存在".to_string())
}

/// 获取专辑摘要信息
#[tauri::command(rename_all = "snake_case")]
fn get_album_summary(
    state: State<AppState>,
    album_id: String,
) -> Result<AlbumSummary, String> {
    let cache_manager = state.cache_manager.lock().map_err(|e| e.to_string())?;
    
    if let Some(library) = cache_manager.load_library().ok() {
        if let Some(album) = library.albums.iter().find(|a| a.id == album_id) {
            return Ok(album.to_summary());
        }
    }
    
    Err("专辑不存在".to_string())
}

/// 批量获取歌手信息
#[tauri::command(rename_all = "snake_case")]
fn get_artists_by_ids(
    state: State<AppState>,
    artist_ids: Vec<String>,
) -> Result<Vec<Artist>, String> {
    let cache_manager = state.cache_manager.lock().map_err(|e| e.to_string())?;
    
    if let Some(library) = cache_manager.load_library().ok() {
        let artists: Vec<Artist> = library.artists
            .into_iter()
            .filter(|a| artist_ids.contains(&a.id))
            .collect();
        return Ok(artists);
    }
    
    Ok(Vec::new())
}

/// 批量获取专辑信息
#[tauri::command(rename_all = "snake_case")]
fn get_albums_by_ids(
    state: State<AppState>,
    album_ids: Vec<String>,
) -> Result<Vec<Album>, String> {
    let cache_manager = state.cache_manager.lock().map_err(|e| e.to_string())?;
    
    if let Some(library) = cache_manager.load_library().ok() {
        let albums: Vec<Album> = library.albums
            .into_iter()
            .filter(|a| album_ids.contains(&a.id))
            .collect();
        return Ok(albums);
    }
    
    Ok(Vec::new())
}

/// 获取所有歌手列表
#[tauri::command(rename_all = "snake_case")]
fn get_all_artists(
    state: State<AppState>,
) -> Result<Vec<ArtistSummary>, String> {
    let cache_manager = state.cache_manager.lock().map_err(|e| e.to_string())?;
    
    if let Some(library) = cache_manager.load_library().ok() {
        let summaries: Vec<ArtistSummary> = library.artists
            .iter()
            .map(|a| a.to_summary())
            .collect();
        return Ok(summaries);
    }
    
    Ok(Vec::new())
}

/// 获取所有专辑列表
#[tauri::command(rename_all = "snake_case")]
fn get_all_albums(
    state: State<AppState>,
) -> Result<Vec<AlbumSummary>, String> {
    let cache_manager = state.cache_manager.lock().map_err(|e| e.to_string())?;
    
    if let Some(library) = cache_manager.load_library().ok() {
        let summaries: Vec<AlbumSummary> = library.albums
            .iter()
            .map(|a| a.to_summary())
            .collect();
        return Ok(summaries);
    }
    
    Ok(Vec::new())
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
) -> Result<serde_json::Value, String> {
    use crate::lyric_parser::{LyricParser, LyricFormat};
    
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
    
    let parsed = parser.parse(&content, lyric_format)
        .map_err(|e| e.to_string())?;
    
    serde_json::to_value(parsed)
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

// ========== 音频播放控制命令 ==========

/// 播放音频文件
#[tauri::command(rename_all = "snake_case")]
fn play_audio(
    state: State<AppState>,
    file_path: String,
) -> Result<(), String> {
    let player = state.audio_player.lock().map_err(|e| e.to_string())?;
    player.play(&file_path).map_err(|e| e.to_string())
}

/// 暂停播放
#[tauri::command]
fn pause_audio(state: State<AppState>) {
    let player = state.audio_player.lock().unwrap();
    player.pause();
}

/// 恢复播放
#[tauri::command]
fn resume_audio(state: State<AppState>) {
    let player = state.audio_player.lock().unwrap();
    player.resume();
}

/// 停止播放
#[tauri::command]
fn stop_audio(state: State<AppState>) {
    let player = state.audio_player.lock().unwrap();
    player.stop();
}

/// 跳转到指定位置（秒）
#[tauri::command(rename_all = "snake_case")]
fn seek_audio(
    state: State<AppState>,
    position: f64,
) -> Result<(), String> {
    let player = state.audio_player.lock().map_err(|e| e.to_string())?;
    player.seek(position).map_err(|e| e.to_string())
}

/// 设置音量 (0.0 - 1.0)
#[tauri::command(rename_all = "snake_case")]
fn set_audio_volume(
    state: State<AppState>,
    volume: f32,
) {
    let player = state.audio_player.lock().unwrap();
    player.set_volume(volume.clamp(0.0, 1.0));
}

/// 获取当前音量
#[tauri::command]
fn get_audio_volume(state: State<AppState>) -> f32 {
    let player = state.audio_player.lock().unwrap();
    player.get_volume()
}

/// 获取当前播放位置（秒）
#[tauri::command]
fn get_audio_position(state: State<AppState>) -> f32 {
    let player = state.audio_player.lock().unwrap();
    player.get_position()
}

/// 获取音频总时长（秒）
#[tauri::command]
fn get_audio_duration(state: State<AppState>) -> f32 {
    let player = state.audio_player.lock().unwrap();
    player.get_duration()
}

/// 获取播放状态
#[tauri::command]
fn get_audio_state(state: State<AppState>) -> String {
    let player = state.audio_player.lock().unwrap();
    match player.get_state() {
        PlaybackState::Playing => "playing".to_string(),
        PlaybackState::Paused => "paused".to_string(),
        PlaybackState::Stopped => "stopped".to_string(),
        PlaybackState::Preloading => "preloading".to_string(),
        PlaybackState::Crossfading => "crossfading".to_string(),
    }
}

/// 检查是否正在播放
#[tauri::command(rename_all = "snake_case")]
fn is_audio_playing(state: State<AppState>) -> bool {
    let player = state.audio_player.lock().unwrap();
    player.is_playing()
}

/// 预加载下一首音频
#[tauri::command(rename_all = "snake_case")]
fn preload_next_audio(
    state: State<AppState>,
    file_path: String,
) -> Result<(), String> {
    let player = state.audio_player.lock().map_err(|e| e.to_string())?;
    player.preload_next(&file_path).map_err(|e| e.to_string())
}

/// 获取下一首音频路径
#[tauri::command]
fn get_next_audio_path(state: State<AppState>) -> Option<String> {
    let player = state.audio_player.lock().unwrap();
    player.get_next_path()
}

/// 设置交叉淡化启用状态
#[tauri::command(rename_all = "snake_case")]
fn set_crossfade_enabled(
    state: State<AppState>,
    enabled: bool,
) {
    let player = state.audio_player.lock().unwrap();
    player.set_crossfade_enabled(enabled);
}

/// 检查交叉淡化是否启用
#[tauri::command]
fn is_crossfade_enabled(state: State<AppState>) -> bool {
    let player = state.audio_player.lock().unwrap();
    player.is_crossfade_enabled()
}

/// 设置交叉淡化配置
#[tauri::command(rename_all = "snake_case")]
fn set_crossfade_config(
    state: State<AppState>,
    duration_secs: f32,
    curve_type: String,
) -> Result<(), String> {
    use audio_engine::CrossfadeCurve;
    
    let curve = match curve_type.as_str() {
        "linear" => CrossfadeCurve::Linear,
        "logarithmic" => CrossfadeCurve::Logarithmic,
        "s_curve" => CrossfadeCurve::SCurve,
        _ => return Err("Invalid curve type".to_string()),
    };
    
    let config = audio_engine::CrossfadeConfig {
        duration_secs: duration_secs.max(1.0).min(30.0),
        curve,
    };
    
    let player = state.audio_player.lock().map_err(|e| e.to_string())?;
    player.set_crossfade_config(config);
    Ok(())
}

// ========== Phase 3: 节拍检测与BPM分析命令 ==========

/// 分析音频文件的BPM和节拍
#[tauri::command(rename_all = "snake_case")]
fn analyze_audio_beat(
    state: State<AppState>,
    file_path: String,
) -> Result<serde_json::Value, String> {
    let analyzer = state.audio_analyzer.lock().map_err(|e| e.to_string())?;
    let analyzer = analyzer.lock().map_err(|e| e.to_string())?;
    
    let result = analyzer.analyze_file(&file_path)
        .map_err(|e| format!("分析失败: {}", e))?;
    
    Ok(serde_json::json!({
        "bpm": result.bpm,
        "beat_positions": result.beat_positions,
        "downbeat_position": result.downbeat_position,
    }))
}

/// 强制重新分析音频（忽略缓存）
#[tauri::command(rename_all = "snake_case")]
fn reanalyze_audio_beat(
    state: State<AppState>,
    file_path: String,
) -> Result<serde_json::Value, String> {
    let analyzer = state.audio_analyzer.lock().map_err(|e| e.to_string())?;
    let analyzer = analyzer.lock().map_err(|e| e.to_string())?;
    
    let result = analyzer.analyze_file_force(&file_path)
        .map_err(|e| format!("分析失败: {}", e))?;
    
    Ok(serde_json::json!({
        "bpm": result.bpm,
        "beat_positions": result.beat_positions,
        "downbeat_position": result.downbeat_position,
    }))
}

/// 获取音频的混音点建议
#[tauri::command(rename_all = "snake_case")]
fn get_mix_points(
    state: State<AppState>,
    file_path: String,
) -> Result<serde_json::Value, String> {
    let analyzer = state.audio_analyzer.lock().map_err(|e| e.to_string())?;
    let analyzer = analyzer.lock().map_err(|e| e.to_string())?;
    
    let mix_points = analyzer.find_mix_points(&file_path)
        .map_err(|e| format!("获取混音点失败: {}", e))?;
    
    Ok(serde_json::json!({
        "bpm": mix_points.bpm,
        "mix_in_point": mix_points.mix_in_point,
        "mix_out_point": mix_points.mix_out_point,
        "duration": mix_points.duration,
    }))
}

/// 批量分析音频文件
#[tauri::command(rename_all = "snake_case")]
fn batch_analyze_audio(
    state: State<AppState>,
    file_paths: Vec<String>,
    app_handle: AppHandle,
) -> Result<Vec<serde_json::Value>, String> {
    let analyzer = state.audio_analyzer.lock().map_err(|e| e.to_string())?;
    let analyzer = analyzer.lock().map_err(|e| e.to_string())?;
    
    let results = analyzer.batch_analyze(file_paths, Some(app_handle));
    
    let json_results: Vec<serde_json::Value> = results
        .into_iter()
        .map(|(path, result)| {
            match result {
                Ok(analysis) => serde_json::json!({
                    "file_path": path,
                    "success": true,
                    "bpm": analysis.bpm,
                    "beat_count": analysis.beat_positions.len(),
                    "downbeat": analysis.downbeat_position,
                }),
                Err(e) => serde_json::json!({
                    "file_path": path,
                    "success": false,
                    "error": e.to_string(),
                }),
            }
        })
        .collect();
    
    Ok(json_results)
}

/// 获取分析缓存统计
#[tauri::command]
fn get_analysis_cache_stats(state: State<AppState>) -> Result<serde_json::Value, String> {
    let analyzer = state.audio_analyzer.lock().map_err(|e| e.to_string())?;
    let analyzer = analyzer.lock().map_err(|e| e.to_string())?;
    
    let stats = analyzer.get_cache_stats()
        .map_err(|e| format!("获取统计失败: {}", e))?;
    
    Ok(serde_json::json!({
        "entry_count": stats.entry_count,
        "total_data_size": stats.total_data_size,
    }))
}

/// 清空分析缓存
#[tauri::command]
fn clear_analysis_cache(state: State<AppState>) -> Result<(), String> {
    let analyzer = state.audio_analyzer.lock().map_err(|e| e.to_string())?;
    let analyzer = analyzer.lock().map_err(|e| e.to_string())?;
    
    analyzer.clear_cache()
        .map_err(|e| format!("清空缓存失败: {}", e))?;
    
    Ok(())
}

// ========== Phase 4: BPM同步与时间拉伸命令 ==========

/// 设置当前播放音频的BPM信息
#[tauri::command(rename_all = "snake_case")]
fn set_current_track_bpm(
    state: State<AppState>,
    bpm: f64,
    beat_positions: Vec<f64>,
) -> Result<(), String> {
    let player = state.audio_player.lock().map_err(|e| e.to_string())?;
    player.set_current_bpm(bpm, beat_positions);
    Ok(())
}

/// 设置下一首音频的BPM信息
#[tauri::command(rename_all = "snake_case")]
fn set_next_track_bpm(
    state: State<AppState>,
    bpm: f64,
    beat_positions: Vec<f64>,
) -> Result<(), String> {
    let player = state.audio_player.lock().map_err(|e| e.to_string())?;
    player.set_next_bpm(bpm, beat_positions);
    Ok(())
}

/// 启用/禁用BPM同步
#[tauri::command(rename_all = "snake_case")]
fn set_bpm_sync_enabled(
    state: State<AppState>,
    enabled: bool,
) -> Result<(), String> {
    let player = state.audio_player.lock().map_err(|e| e.to_string())?;
    player.set_bpm_sync(enabled);
    Ok(())
}

/// 检查BPM同步是否启用
#[tauri::command]
fn is_bpm_sync_enabled(state: State<AppState>) -> bool {
    let player = state.audio_player.lock().unwrap();
    player.is_bpm_sync()
}

/// 获取当前播放速度比率
#[tauri::command]
fn get_playback_speed_ratio(state: State<AppState>) -> f64 {
    let player = state.audio_player.lock().unwrap();
    player.speed_ratio()
}

/// 设置播放速度（覆盖BPM同步）
#[tauri::command(rename_all = "snake_case")]
fn set_playback_speed(
    state: State<AppState>,
    speed_ratio: f64,
) -> Result<(), String> {
    let player = state.audio_player.lock().map_err(|e| e.to_string())?;
    player.set_speed(speed_ratio).map_err(|e| e.to_string())
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// ========== 窗口控制命令 ==========

/// 切换窗口置顶状态
#[tauri::command]
fn toggle_always_on_top(window: tauri::Window) -> bool {
    let is_always_on_top = window.is_always_on_top().unwrap_or(false);
    let _ = window.set_always_on_top(!is_always_on_top);
    !is_always_on_top
}

/// 关闭窗口
#[tauri::command]
fn close_window(window: tauri::Window) {
    let _ = window.close();
}

/// 最小化窗口
#[tauri::command]
fn minimize_window(window: tauri::Window) {
    let _ = window.minimize();
}

/// 切换窗口最大化状态
#[tauri::command]
fn toggle_maximize(window: tauri::Window) -> bool {
    let is_maximized = window.is_maximized().unwrap_or(false);
    if is_maximized {
        let _ = window.unmaximize();
    } else {
        let _ = window.maximize();
    }
    !is_maximized
}

/// 获取窗口位置
#[tauri::command]
fn get_window_position(window: tauri::Window) -> (i32, i32) {
    let position = window.outer_position().unwrap_or(tauri::PhysicalPosition { x: 0, y: 0 });
    (position.x, position.y)
}

/// 设置窗口位置
#[tauri::command]
fn set_window_position(window: tauri::Window, x: i32, y: i32) {
    let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }));
}

/// 获取窗口尺寸
#[tauri::command]
fn get_window_size(window: tauri::Window) -> (u32, u32) {
    let size = window.inner_size().unwrap_or(tauri::PhysicalSize { width: 800, height: 600 });
    (size.width, size.height)
}

/// 设置窗口尺寸
#[tauri::command]
fn set_window_size(window: tauri::Window, width: u32, height: u32) {
    let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize { width, height }));
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
            
            let app_state = AppState {
                source_manager: Mutex::new(SourceManager::new()),
                cache_manager: Mutex::new(CacheManager::with_directory(cache_path)),
                scanner: Mutex::new(MusicScanner::new()),
                audio_player: Mutex::new(audio_player),
                audio_analyzer: Mutex::new(audio_analyzer),
            };
            
            app.manage(app_state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            add_local_source,
            add_web_disk_source,
            add_webdev_source,
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
            get_tracks_by_ids,
            get_album_art,
            get_music_file,
            get_artist_image,
            get_lyrics,
            parse_lyric_content,
            detect_lyric_format,
            get_artist_info,
            get_artist_summary,
            get_album_info,
            get_album_summary,
            get_artists_by_ids,
            get_albums_by_ids,
            get_all_artists,
            get_all_albums,
            // 音频播放控制命令
            play_audio,
            pause_audio,
            resume_audio,
            stop_audio,
            seek_audio,
            set_audio_volume,
            get_audio_volume,
            get_audio_position,
            get_audio_duration,
            get_audio_state,
            is_audio_playing,
            // Phase 2: 双缓冲与无缝切换命令
            preload_next_audio,
            get_next_audio_path,
            set_crossfade_enabled,
            is_crossfade_enabled,
            set_crossfade_config,
            // Phase 3: 节拍检测与BPM分析命令
            analyze_audio_beat,
            reanalyze_audio_beat,
            get_mix_points,
            batch_analyze_audio,
            get_analysis_cache_stats,
            clear_analysis_cache,
            // Phase 4: BPM同步与时间拉伸命令
            set_current_track_bpm,
            set_next_track_bpm,
            set_bpm_sync_enabled,
            is_bpm_sync_enabled,
            get_playback_speed_ratio,
            set_playback_speed,
            // 窗口控制命令
            toggle_always_on_top,
            close_window,
            minimize_window,
            toggle_maximize,
            get_window_position,
            set_window_position,
            get_window_size,
            set_window_size,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
