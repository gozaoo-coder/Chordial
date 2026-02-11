//! 音频播放控制命令

use tauri::State;
use crate::state::AppState;
use crate::lock_state;
use crate::lock_state_unwrap;
use crate::audio_engine::{PlaybackState, CrossfadeCurve, CrossfadeConfig};

/// 播放音频文件
#[tauri::command(rename_all = "snake_case")]
pub fn play_audio(
    state: State<AppState>,
    file_path: String,
) -> Result<(), String> {
    let player = lock_state!(state, audio_player)?;
    player.play(&file_path).map_err(|e| e.to_string())
}

/// 暂停播放
#[tauri::command]
pub fn pause_audio(state: State<AppState>) {
    let player = lock_state_unwrap!(state, audio_player);
    player.pause();
}

/// 恢复播放
#[tauri::command]
pub fn resume_audio(state: State<AppState>) {
    let player = lock_state_unwrap!(state, audio_player);
    player.resume();
}

/// 停止播放
#[tauri::command]
pub fn stop_audio(state: State<AppState>) {
    let player = lock_state_unwrap!(state, audio_player);
    player.stop();
}

/// 跳转到指定位置（秒）
#[tauri::command(rename_all = "snake_case")]
pub fn seek_audio(
    state: State<AppState>,
    position: f64,
) -> Result<(), String> {
    let player = lock_state!(state, audio_player)?;
    player.seek(position).map_err(|e| e.to_string())
}

/// 设置音量 (0.0 - 1.0)
#[tauri::command(rename_all = "snake_case")]
pub fn set_audio_volume(
    state: State<AppState>,
    volume: f32,
) {
    let player = lock_state_unwrap!(state, audio_player);
    player.set_volume(volume.clamp(0.0, 1.0));
}

/// 获取当前音量
#[tauri::command]
pub fn get_audio_volume(state: State<AppState>) -> f32 {
    let player = lock_state_unwrap!(state, audio_player);
    player.get_volume()
}

/// 获取当前播放位置（秒）
#[tauri::command]
pub fn get_audio_position(state: State<AppState>) -> f32 {
    let player = lock_state_unwrap!(state, audio_player);
    player.get_position()
}

/// 获取音频总时长（秒）
#[tauri::command]
pub fn get_audio_duration(state: State<AppState>) -> f32 {
    let player = lock_state_unwrap!(state, audio_player);
    player.get_duration()
}

/// 获取播放状态
#[tauri::command]
pub fn get_audio_state(state: State<AppState>) -> String {
    let player = lock_state_unwrap!(state, audio_player);
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
pub fn is_audio_playing(state: State<AppState>) -> bool {
    let player = lock_state_unwrap!(state, audio_player);
    player.is_playing()
}

/// 预加载下一首音频
#[tauri::command(rename_all = "snake_case")]
pub fn preload_next_audio(
    state: State<AppState>,
    file_path: String,
) -> Result<(), String> {
    let player = lock_state!(state, audio_player)?;
    player.preload_next(&file_path).map_err(|e| e.to_string())
}

/// 获取下一首音频路径
#[tauri::command]
pub fn get_next_audio_path(state: State<AppState>) -> Option<String> {
    let player = lock_state_unwrap!(state, audio_player);
    player.get_next_path()
}

/// 设置交叉淡化启用状态
#[tauri::command(rename_all = "snake_case")]
pub fn set_crossfade_enabled(
    state: State<AppState>,
    enabled: bool,
) {
    let player = lock_state_unwrap!(state, audio_player);
    player.set_crossfade_enabled(enabled);
}

/// 检查交叉淡化是否启用
#[tauri::command]
pub fn is_crossfade_enabled(state: State<AppState>) -> bool {
    let player = lock_state_unwrap!(state, audio_player);
    player.is_crossfade_enabled()
}

/// 设置交叉淡化配置
#[tauri::command(rename_all = "snake_case")]
pub fn set_crossfade_config(
    state: State<AppState>,
    duration_secs: f32,
    curve_type: String,
) -> Result<(), String> {
    let curve = match curve_type.as_str() {
        "linear" => CrossfadeCurve::Linear,
        "logarithmic" => CrossfadeCurve::Logarithmic,
        "s_curve" => CrossfadeCurve::SCurve,
        _ => return Err("Invalid curve type".to_string()),
    };
    
    let config = CrossfadeConfig {
        duration_secs: duration_secs.max(1.0).min(30.0),
        curve,
    };
    
    let player = lock_state!(state, audio_player)?;
    player.set_crossfade_config(config);
    Ok(())
}

// ========== BPM同步与时间拉伸命令 ==========

/// 设置当前播放音频的BPM信息
#[tauri::command(rename_all = "snake_case")]
pub fn set_current_track_bpm(
    state: State<AppState>,
    bpm: f64,
    beat_positions: Vec<f64>,
) -> Result<(), String> {
    let player = lock_state!(state, audio_player)?;
    player.set_current_bpm(bpm, beat_positions);
    Ok(())
}

/// 设置下一首音频的BPM信息
#[tauri::command(rename_all = "snake_case")]
pub fn set_next_track_bpm(
    state: State<AppState>,
    bpm: f64,
    beat_positions: Vec<f64>,
) -> Result<(), String> {
    let player = lock_state!(state, audio_player)?;
    player.set_next_bpm(bpm, beat_positions);
    Ok(())
}

/// 启用/禁用BPM同步
#[tauri::command(rename_all = "snake_case")]
pub fn set_bpm_sync_enabled(
    state: State<AppState>,
    enabled: bool,
) -> Result<(), String> {
    let player = lock_state!(state, audio_player)?;
    player.set_bpm_sync(enabled);
    Ok(())
}

/// 检查BPM同步是否启用
#[tauri::command]
pub fn is_bpm_sync_enabled(state: State<AppState>) -> bool {
    let player = lock_state_unwrap!(state, audio_player);
    player.is_bpm_sync()
}

/// 获取当前播放速度比率
#[tauri::command]
pub fn get_playback_speed_ratio(state: State<AppState>) -> f64 {
    let player = lock_state_unwrap!(state, audio_player);
    player.speed_ratio()
}

/// 设置播放速度（覆盖BPM同步）
#[tauri::command(rename_all = "snake_case")]
pub fn set_playback_speed(
    state: State<AppState>,
    speed_ratio: f64,
) -> Result<(), String> {
    let player = lock_state!(state, audio_player)?;
    player.set_speed(speed_ratio).map_err(|e| e.to_string())
}
