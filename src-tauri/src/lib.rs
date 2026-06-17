mod module;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .setup(|_app| {
            // 初始化音乐来源系统（音乐库 + 来源管理器 + 注册器 + Blob 缓存）
            crate::module::commands::init_music_system();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Config — 自动防抖落盘
            crate::module::commands::config_get,
            crate::module::commands::config_set,
            crate::module::commands::config_remove,
            crate::module::commands::config_has,
            crate::module::commands::config_keys,
            crate::module::commands::config_clear,
            crate::module::commands::config_flush,
            crate::module::commands::config_reload,
            // Storage — 手动落盘
            crate::module::commands::storage_get,
            crate::module::commands::storage_set,
            crate::module::commands::storage_remove,
            crate::module::commands::storage_has,
            crate::module::commands::storage_keys,
            crate::module::commands::storage_clear,
            crate::module::commands::storage_save,
            // Cache — 纯内存 TTL
            crate::module::commands::cache_get,
            crate::module::commands::cache_set,
            crate::module::commands::cache_remove,
            crate::module::commands::cache_has,
            crate::module::commands::cache_keys,
            crate::module::commands::cache_clear,
            crate::module::commands::cache_clear_expired,
            crate::module::commands::cache_touch,
            // Blob Cache — 磁盘文件 + 内存 TTL
            crate::module::commands::cache_enable_blob_storage,
            crate::module::commands::cache_blob_storage_enabled,
            crate::module::commands::cache_set_blob,
            crate::module::commands::cache_get_blob,
            crate::module::commands::cache_remove_blob,
            crate::module::commands::cache_has_blob,
            crate::module::commands::cache_blob_keys,
            crate::module::commands::cache_clear_blobs,
            crate::module::commands::cache_clear_expired_blobs,
            // Blob Storage — 持久化二进制文件
            crate::module::commands::storage_set_blob,
            crate::module::commands::storage_get_blob,
            crate::module::commands::storage_remove_blob,
            crate::module::commands::storage_has_blob,
            crate::module::commands::storage_blob_keys,
            crate::module::commands::storage_clear_blobs,
            // Music Source — 资源获取
            crate::module::commands::get_song_file,
            crate::module::commands::get_album_picture,
            crate::module::commands::get_lyric_text,
            // Local Source — 文件夹管理
            crate::module::commands::local_stats,
            crate::module::commands::local_add_folder,
            crate::module::commands::local_remove_folder,
            crate::module::commands::local_get_folders,
            crate::module::commands::local_rescan,
            // MusicLibrary — 持久化
            crate::module::commands::library_save,
            crate::module::commands::library_cleanup_empty_entities,
            // MusicLibrary — Song CRUD + 搜索
            crate::module::commands::library_song_count,
            crate::module::commands::library_get_song,
            crate::module::commands::library_get_all_songs,
            crate::module::commands::library_search_songs,
            // MusicLibrary — Artist CRUD + 搜索
            crate::module::commands::library_artist_count,
            crate::module::commands::library_get_artist,
            crate::module::commands::library_get_all_artists,
            crate::module::commands::library_search_artists,
            // MusicLibrary — Album CRUD + 搜索
            crate::module::commands::library_album_count,
            crate::module::commands::library_get_album,
            crate::module::commands::library_get_all_albums,
            crate::module::commands::library_search_albums,
            // MusicLibrary — Lyric CRUD + 搜索
            crate::module::commands::library_lyric_count,
            crate::module::commands::library_get_lyric,
            crate::module::commands::library_get_all_lyrics,
            crate::module::commands::library_search_lyrics,
            // MusicLibrary — Relations
            crate::module::commands::library_get_artists_of_song,
            crate::module::commands::library_get_album_of_song,
            crate::module::commands::library_get_lyric_of_song,
            crate::module::commands::library_get_songs_by_artist,
            crate::module::commands::library_get_albums_by_artist,
            crate::module::commands::library_get_songs_in_album,
            crate::module::commands::library_get_source_ids_of_song,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
