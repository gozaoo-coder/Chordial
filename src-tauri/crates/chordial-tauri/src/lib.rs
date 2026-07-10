//! Chordial front 层（Tauri 宿主）— 库调用形式。
//!
//! 本 crate 链接 [`chordial_core`]，在 Tauri `setup` 阶段构建 [`AppContext`] 并通过
//! `app.manage()` 注入为 Tauri State。前端通过 `invoke` 调用的命令（[`commands`]）以
//! `State<'_, Arc<AppContext>>` 提取上下文，委托给 core 同步方法 —— 全程进程内，无网络。
//!
//! `chordial://` 协议（[`media_protocol`]）同样委托给 core 的 `media::handle`。

mod commands;
mod media_protocol;

use chordial_core::AppContext;
use std::sync::Arc;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init());

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    let builder = builder.plugin(tauri_plugin_window_state::Builder::default().build());

    builder
        .setup(|app| {
            // 构建 server 层上下文（音乐库 + 来源系统 + 缓存 + 配置）
            let ctx = Arc::new(
                AppContext::new_default_dir()
                    .expect("初始化 Chordial server 层上下文失败"),
            );

            // 媒体协议桥接：注入来源注册器
            media_protocol::init(ctx.registrar.clone());

            // 注入为 Tauri State，供各命令通过 State<'_, Arc<AppContext>> 提取
            app.manage(ctx);
            Ok(())
        })
        .register_asynchronous_uri_scheme_protocol("chordial", |_ctx, request, responder| {
            media_protocol::handle_protocol(request, responder);
        })
        .invoke_handler(tauri::generate_handler![
            // Config — 自动防抖落盘
            commands::config_get,
            commands::config_set,
            commands::config_remove,
            commands::config_has,
            commands::config_keys,
            commands::config_clear,
            commands::config_flush,
            commands::config_reload,
            // Storage — 手动落盘
            commands::storage_get,
            commands::storage_set,
            commands::storage_remove,
            commands::storage_has,
            commands::storage_keys,
            commands::storage_clear,
            commands::storage_save,
            // Cache — 纯内存 TTL
            commands::cache_get,
            commands::cache_set,
            commands::cache_remove,
            commands::cache_has,
            commands::cache_keys,
            commands::cache_clear,
            commands::cache_clear_expired,
            commands::cache_touch,
            // Blob Cache — 磁盘文件 + 内存 TTL
            commands::cache_enable_blob_storage,
            commands::cache_blob_storage_enabled,
            commands::cache_set_blob,
            commands::cache_get_blob,
            commands::cache_remove_blob,
            commands::cache_has_blob,
            commands::cache_blob_keys,
            commands::cache_clear_blobs,
            commands::cache_clear_expired_blobs,
            // Blob Storage — 持久化二进制文件
            commands::storage_set_blob,
            commands::storage_get_blob,
            commands::storage_remove_blob,
            commands::storage_has_blob,
            commands::storage_blob_keys,
            commands::storage_clear_blobs,
            // Music Source — 资源获取
            commands::get_song_file,
            commands::get_album_picture,
            commands::get_lyric_text,
            // Local Source — 文件夹管理
            commands::local_stats,
            commands::local_add_folder,
            commands::local_remove_folder,
            commands::local_get_folders,
            commands::local_rescan,
            // MusicLibrary — 持久化
            commands::library_save,
            commands::library_cleanup_empty_entities,
            // MusicLibrary — Song CRUD + 搜索
            commands::library_song_count,
            commands::library_get_song,
            commands::library_get_all_songs,
            commands::library_get_songs_page,
            commands::library_search_songs,
            // MusicLibrary — Artist CRUD + 搜索
            commands::library_artist_count,
            commands::library_get_artist,
            commands::library_get_all_artists,
            commands::library_get_artists_page,
            commands::library_search_artists,
            // MusicLibrary — Album CRUD + 搜索
            commands::library_album_count,
            commands::library_get_album,
            commands::library_get_all_albums,
            commands::library_get_albums_page,
            commands::library_search_albums,
            // MusicLibrary — Home
            commands::library_get_home_stats,
            // MusicLibrary — Lyric CRUD + 搜索
            commands::library_lyric_count,
            commands::library_get_lyric,
            commands::library_get_all_lyrics,
            commands::library_search_lyrics,
            // MusicLibrary — Relations
            commands::library_get_artists_of_song,
            commands::library_get_album_of_song,
            commands::library_get_lyric_of_song,
            commands::library_get_songs_by_artist,
            commands::library_get_albums_by_artist,
            commands::library_get_songs_in_album,
            commands::library_get_source_ids_of_song,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
