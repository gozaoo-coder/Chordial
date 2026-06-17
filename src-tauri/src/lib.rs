mod module;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|_app| {
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            crate::module::commands::storage_get,
            crate::module::commands::storage_set,
            crate::module::commands::storage_remove,
            crate::module::commands::storage_has,
            crate::module::commands::storage_keys,
            crate::module::commands::storage_clear,
            crate::module::commands::storage_save,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
