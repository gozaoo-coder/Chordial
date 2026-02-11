//! 音乐库查询命令

use std::path::PathBuf;
use tauri::State;
use tauri::ipc::Response;
use crate::state::AppState;
use crate::lock_state;
use crate::constants::DEFAULT_TRANSPARENT_PNG;
use crate::music_source::{MusicLibrary, TrackMetadata, Artist, ArtistSummary, Album, AlbumSummary, SourceConfig, SourceType, MusicSource};
use crate::audio_metadata::read_metadata;

/// 获取缓存的音乐库
#[tauri::command]
pub fn get_cached_library(
    state: State<AppState>,
) -> Result<Option<MusicLibrary>, String> {
    let cache_manager = lock_state!(state, cache_manager)?;
    Ok(cache_manager.load_library().ok())
}

/// 清空所有缓存
#[tauri::command]
pub fn clear_all_cache(
    state: State<AppState>,
) -> Result<(), String> {
    let cache_manager = lock_state!(state, cache_manager)?;
    cache_manager.clear_all_cache()
        .map_err(|e| e.to_string())
}

/// 获取缓存大小
#[tauri::command]
pub fn get_cache_size(
    state: State<AppState>,
) -> Result<u64, String> {
    let cache_manager = lock_state!(state, cache_manager)?;
    cache_manager.cache_size().map_err(|e| e.to_string())
}

/// 获取曲目完整信息
#[tauri::command(rename_all = "snake_case")]
pub fn get_track_info(
    state: State<AppState>,
    track_id: String,
) -> Result<TrackMetadata, String> {
    let cache_manager = lock_state!(state, cache_manager)?;

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
pub fn get_tracks_by_ids(
    state: State<AppState>,
    track_ids: Vec<String>,
) -> Result<Vec<TrackMetadata>, String> {
    let _source_manager = lock_state!(state, source_manager)?;
    let cache_manager = lock_state!(state, cache_manager)?;

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
pub fn get_album_art(
    state: State<AppState>,
    album_id: String,
    _size: String,
) -> Result<Response, String> {
    let cache_manager = lock_state!(state, cache_manager)?;
    
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
    Ok(Response::new(DEFAULT_TRANSPARENT_PNG.to_vec()))
}

/// 获取音乐文件（使用二进制响应）
#[tauri::command(rename_all = "snake_case")]
pub fn get_music_file(
    state: State<AppState>,
    track_id: String,
) -> Result<Response, String> {
    // 先获取曲目信息和源信息，然后立即释放锁
    let (track_path, source_type, source_config) = {
        let source_manager = lock_state!(state, source_manager)?;
        let cache_manager = lock_state!(state, cache_manager)?;

        // 尝试从缓存加载曲目信息
        let library = cache_manager.load_library()
            .map_err(|e| format!("无法加载音乐库缓存: {}", e))?;
        
        let track = library.tracks.iter()
            .find(|t| t.id == track_id)
            .ok_or_else(|| format!("未找到曲目: {}", track_id))?;
        
        // 获取源信息以确定源类型
        let source = source_manager.get_source(&track.source_id)
            .ok_or_else(|| format!("未找到源: {}", track.source_id))?;
        
        let source_type = source.source_type.clone();
        let source_config = source.clone();
        let track_path = track.path.clone();
        
        (track_path, source_type, source_config)
    }; // 锁在这里释放

    // 根据源类型处理文件
    match source_type {
        SourceType::LocalFolder | SourceType::WebDisk => {
            // 本地文件或网盘源，直接读取文件
            if !track_path.exists() {
                return Err(format!("音乐文件不存在: {:?}", track_path));
            }
            
            let data = std::fs::read(&track_path)
                .map_err(|e| format!("读取音乐文件失败: {}", e))?;
            println!("成功读取音乐文件: {} ({} bytes)", track_id, data.len());
            Ok(Response::new(data))
        }
        SourceType::WebDev => {
            // WebDev 源，从 URL 下载文件
            let file_url = track_path.to_string_lossy().to_string();
            download_webdev_file(&source_config, &file_url)
        }
    }
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
pub fn get_artist_image(
    state: State<AppState>,
    artist_id: String,
) -> Result<Response, String> {
    let cache_manager = lock_state!(state, cache_manager)?;
    
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
pub fn get_artist_info(
    state: State<AppState>,
    artist_id: String,
) -> Result<Artist, String> {
    let cache_manager = lock_state!(state, cache_manager)?;
    
    if let Some(library) = cache_manager.load_library().ok() {
        if let Some(artist) = library.artists.iter().find(|a| a.id == artist_id) {
            return Ok(artist.clone());
        }
    }
    
    Err("歌手不存在".to_string())
}

/// 获取歌手摘要信息
#[tauri::command(rename_all = "snake_case")]
pub fn get_artist_summary(
    state: State<AppState>,
    artist_id: String,
) -> Result<ArtistSummary, String> {
    let cache_manager = lock_state!(state, cache_manager)?;
    
    if let Some(library) = cache_manager.load_library().ok() {
        if let Some(artist) = library.artists.iter().find(|a| a.id == artist_id) {
            return Ok(artist.to_summary());
        }
    }
    
    Err("歌手不存在".to_string())
}

/// 获取专辑完整信息
#[tauri::command(rename_all = "snake_case")]
pub fn get_album_info(
    state: State<AppState>,
    album_id: String,
) -> Result<Album, String> {
    let cache_manager = lock_state!(state, cache_manager)?;
    
    if let Some(library) = cache_manager.load_library().ok() {
        if let Some(album) = library.albums.iter().find(|a| a.id == album_id) {
            return Ok(album.clone());
        }
    }
    
    Err("专辑不存在".to_string())
}

/// 获取专辑摘要信息
#[tauri::command(rename_all = "snake_case")]
pub fn get_album_summary(
    state: State<AppState>,
    album_id: String,
) -> Result<AlbumSummary, String> {
    let cache_manager = lock_state!(state, cache_manager)?;
    
    if let Some(library) = cache_manager.load_library().ok() {
        if let Some(album) = library.albums.iter().find(|a| a.id == album_id) {
            return Ok(album.to_summary());
        }
    }
    
    Err("专辑不存在".to_string())
}

/// 批量获取歌手信息
#[tauri::command(rename_all = "snake_case")]
pub fn get_artists_by_ids(
    state: State<AppState>,
    artist_ids: Vec<String>,
) -> Result<Vec<Artist>, String> {
    let cache_manager = lock_state!(state, cache_manager)?;
    
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
pub fn get_albums_by_ids(
    state: State<AppState>,
    album_ids: Vec<String>,
) -> Result<Vec<Album>, String> {
    let cache_manager = lock_state!(state, cache_manager)?;
    
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
pub fn get_all_artists(
    state: State<AppState>,
) -> Result<Vec<ArtistSummary>, String> {
    let cache_manager = lock_state!(state, cache_manager)?;
    
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
pub fn get_all_albums(
    state: State<AppState>,
) -> Result<Vec<AlbumSummary>, String> {
    let cache_manager = lock_state!(state, cache_manager)?;
    
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
pub fn get_lyrics(
    state: State<AppState>,
    track_id: String,
) -> Result<serde_json::Value, String> {
    let cache_manager = lock_state!(state, cache_manager)?;
    
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
pub fn parse_lyric_content(
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
pub fn detect_lyric_format(
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
