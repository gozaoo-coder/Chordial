//! 音乐源管理命令

use std::path::PathBuf;
use tauri::State;
use crate::state::AppState;
use crate::lock_state;
use crate::music_source::{SourceConfig, MusicLibrary, TrackMetadata, MusicSource};
use crate::scanner::ScanOptions;

/// 添加本地文件夹源
#[tauri::command]
pub fn add_local_source(
    state: State<AppState>,
    path: String,
    recursive: bool,
) -> Result<SourceConfig, String> {
    let mut source_manager = lock_state!(state, source_manager)?;
    let path = PathBuf::from(path);
    source_manager.add_local_folder(path, recursive)
}

/// 添加 WebDisk 源
#[tauri::command]
pub fn add_web_disk_source(
    state: State<AppState>,
    url: String,
    username: Option<String>,
    password: Option<String>,
) -> Result<SourceConfig, String> {
    let mut source_manager = lock_state!(state, source_manager)?;
    let auth = match (username, password) {
        (Some(u), Some(p)) => Some((u, p)),
        _ => None,
    };
    source_manager.add_web_disk(PathBuf::from(url), auth)
}

/// 添加 WebDev 源
#[tauri::command]
pub fn add_webdev_source(
    state: State<AppState>,
    api_base_url: String,
    name: Option<String>,
    api_key: Option<String>,
    auth_token: Option<String>,
) -> Result<SourceConfig, String> {
    let mut source_manager = lock_state!(state, source_manager)?;
    source_manager.add_web_dev(api_base_url, name, api_key, auth_token)
}

/// 移除音乐源
#[tauri::command]
pub fn remove_source(
    state: State<AppState>,
    id: String,
) -> Result<bool, String> {
    let mut source_manager = lock_state!(state, source_manager)?;
    let cache_manager = lock_state!(state, cache_manager)?;
    
    if source_manager.remove_source(&id).is_some() {
        let _ = cache_manager.delete_source_cache(&id);
        Ok(true)
    } else {
        Err("源不存在".to_string())
    }
}

/// 获取所有音乐源
#[tauri::command]
pub fn get_all_sources(
    state: State<AppState>,
) -> Result<Vec<SourceConfig>, String> {
    let source_manager = lock_state!(state, source_manager)?;
    Ok(source_manager.get_all_sources().to_vec())
}

/// 启用/禁用音乐源
#[tauri::command]
pub fn set_source_enabled(
    state: State<AppState>,
    id: String,
    enabled: bool,
) -> Result<bool, String> {
    let mut source_manager = lock_state!(state, source_manager)?;
    Ok(source_manager.set_source_enabled(&id, enabled))
}

/// 扫描所有音乐源
#[tauri::command]
pub fn scan_all_sources(
    state: State<AppState>,
) -> Result<MusicLibrary, String> {
    // 先获取源列表和扫描器，然后立即释放锁
    let (sources, scanner) = {
        let source_manager = lock_state!(state, source_manager)?;
        let scanner = lock_state!(state, scanner)?;
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
        let mut source_manager = lock_state!(state, source_manager)?;
        let cache_manager = lock_state!(state, cache_manager)?;

        library.sources = source_manager.get_all_sources().to_vec();

        // 用于合并 Artist 和 Album 数据
        let mut artists_map: std::collections::HashMap<String, crate::music_source::Artist> = std::collections::HashMap::new();
        let mut albums_map: std::collections::HashMap<String, crate::music_source::Album> = std::collections::HashMap::new();

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

/// 刷新指定源
#[tauri::command]
pub fn refresh_source(
    state: State<AppState>,
    source_id: String,
) -> Result<Vec<TrackMetadata>, String> {
    let cache_manager = lock_state!(state, cache_manager)?;
    let scanner = lock_state!(state, scanner)?;
    
    // 先获取源的克隆，释放锁
    let source = {
        let source_manager = lock_state!(state, source_manager)?;
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
        let mut source_manager = lock_state!(state, source_manager)?;
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

/// 从缓存获取指定源的曲目
#[tauri::command]
pub fn get_source_from_cache(
    state: State<AppState>,
    source_id: String,
) -> Result<Option<Vec<TrackMetadata>>, String> {
    let scanner = lock_state!(state, scanner)?;
    Ok(scanner.load_from_cache(&source_id))
}
