mod module;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .setup(|_app| Ok(()))
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
