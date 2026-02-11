//! BPM 分析和节拍检测命令

use tauri::{State, AppHandle};
use crate::state::AppState;
use crate::lock_state;

/// 分析音频文件的BPM和节拍（异步执行，避免阻塞主线程）
#[tauri::command(rename_all = "snake_case")]
pub async fn analyze_audio_beat(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<serde_json::Value, String> {
    // 克隆Arc以便在闭包中使用
    let analyzer_arc = lock_state!(state, audio_analyzer)?
        .clone();
    
    // 在线程池中执行CPU密集型分析任务
    let result = tokio::task::spawn_blocking(move || {
        // parking_lot::RwLock 使用 read() 而不是 lock()
        let analyzer = analyzer_arc.read();
        
        analyzer.analyze_file(&file_path)
            .map_err(|e| format!("分析失败: {}", e))
    })
    .await
    .map_err(|e| format!("任务执行失败: {}", e))?;
    
    let result = result?;
    
    Ok(serde_json::json!({
        "bpm": result.bpm,
        "beat_positions": result.beat_positions,
        "downbeat_position": result.downbeat_position,
    }))
}

/// 强制重新分析音频（忽略缓存，异步执行）
#[tauri::command(rename_all = "snake_case")]
pub async fn reanalyze_audio_beat(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<serde_json::Value, String> {
    // 克隆Arc以便在闭包中使用
    let analyzer_arc = lock_state!(state, audio_analyzer)?
        .clone();
    
    // 在线程池中执行CPU密集型分析任务
    let result = tokio::task::spawn_blocking(move || {
        let analyzer = analyzer_arc.read();
        
        analyzer.analyze_file_force(&file_path)
            .map_err(|e| format!("分析失败: {}", e))
    })
    .await
    .map_err(|e| format!("任务执行失败: {}", e))?;
    
    let result = result?;
    
    Ok(serde_json::json!({
        "bpm": result.bpm,
        "beat_positions": result.beat_positions,
        "downbeat_position": result.downbeat_position,
    }))
}

/// 获取音频的混音点建议（异步执行）
#[tauri::command(rename_all = "snake_case")]
pub async fn get_mix_points(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<serde_json::Value, String> {
    // 克隆Arc以便在闭包中使用
    let analyzer_arc = lock_state!(state, audio_analyzer)?
        .clone();
    
    // 在线程池中执行CPU密集型任务
    let result = tokio::task::spawn_blocking(move || {
        let analyzer = analyzer_arc.read();
        
        analyzer.find_mix_points(&file_path)
            .map_err(|e| format!("获取混音点失败: {}", e))
    })
    .await
    .map_err(|e| format!("任务执行失败: {}", e))?;
    
    let mix_points = result?;
    
    Ok(serde_json::json!({
        "bpm": mix_points.bpm,
        "mix_in_point": mix_points.mix_in_point,
        "mix_out_point": mix_points.mix_out_point,
        "duration": mix_points.duration,
    }))
}

/// 批量分析音频文件（异步执行）
#[tauri::command(rename_all = "snake_case")]
pub async fn batch_analyze_audio(
    state: State<'_, AppState>,
    file_paths: Vec<String>,
    app_handle: AppHandle,
) -> Result<Vec<serde_json::Value>, String> {
    // 克隆Arc以便在闭包中使用
    let analyzer_arc = lock_state!(state, audio_analyzer)?
        .clone();
    
    // 在线程池中执行CPU密集型批量分析任务
    let json_results = tokio::task::spawn_blocking(move || {
        let analyzer = analyzer_arc.read();
        
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
        
        Ok::<Vec<serde_json::Value>, String>(json_results)
    })
    .await
    .map_err(|e| format!("任务执行失败: {}", e))??;
    
    Ok(json_results)
}

/// 获取分析缓存统计
#[tauri::command]
pub fn get_analysis_cache_stats(state: State<AppState>) -> Result<serde_json::Value, String> {
    let analyzer = lock_state!(state, audio_analyzer)?;
    // parking_lot::RwLock 使用 read() 而不是 lock()
    let analyzer = analyzer.read();
    
    let stats = analyzer.get_cache_stats()
        .map_err(|e| format!("获取统计失败: {}", e))?;
    
    Ok(serde_json::json!({
        "entry_count": stats.entry_count,
        "total_data_size": stats.total_data_size,
    }))
}

/// 清空分析缓存
#[tauri::command]
pub fn clear_analysis_cache(state: State<AppState>) -> Result<(), String> {
    let analyzer = lock_state!(state, audio_analyzer)?;
    // parking_lot::RwLock 使用 write() 而不是 lock()
    let analyzer = analyzer.write();
    
    analyzer.clear_cache()
        .map_err(|e| format!("清空缓存失败: {}", e))?;
    
    Ok(())
}
