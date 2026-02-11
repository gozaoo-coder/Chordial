//! 窗口控制命令

use tauri::Window;

/// 切换窗口置顶状态
#[tauri::command]
pub fn toggle_always_on_top(window: Window) -> bool {
    let is_always_on_top = window.is_always_on_top().unwrap_or(false);
    let _ = window.set_always_on_top(!is_always_on_top);
    !is_always_on_top
}

/// 关闭窗口
#[tauri::command]
pub fn close_window(window: Window) {
    let _ = window.close();
}

/// 最小化窗口
#[tauri::command]
pub fn minimize_window(window: Window) {
    let _ = window.minimize();
}

/// 切换窗口最大化状态
#[tauri::command]
pub fn toggle_maximize(window: Window) -> bool {
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
pub fn get_window_position(window: Window) -> (i32, i32) {
    let position = window.outer_position().unwrap_or(tauri::PhysicalPosition { x: 0, y: 0 });
    (position.x, position.y)
}

/// 设置窗口位置
#[tauri::command]
pub fn set_window_position(window: Window, x: i32, y: i32) {
    let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }));
}

/// 获取窗口尺寸
#[tauri::command]
pub fn get_window_size(window: Window) -> (u32, u32) {
    let size = window.inner_size().unwrap_or(tauri::PhysicalSize { width: 800, height: 600 });
    (size.width, size.height)
}

/// 设置窗口尺寸
#[tauri::command]
pub fn set_window_size(window: Window, width: u32, height: u32) {
    let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize { width, height }));
}
