//! 文件系统监听 — 使用 notify 实现文件夹变化增量同步。
//!
//! 通过 [`start_watcher`] 启动一个后台监听线程，对用户添加的所有音乐文件夹
//! 进行递归监听。当检测到文件增删改时，自动调用 [`LocalMusicSource`] 的
//! `index_file` / `unindex_file` / `reindex_file` 方法实现单点增量更新。
//!
//! # 设计要点
//!
//! - **单 watcher 多目录**：使用一个 notify watcher 监听所有文件夹，避免重复扫描。
//! - **事件去重**：使用简单的延时去重（同一文件 500ms 内的重复事件合并）。
//! - **穿透同步**：文件变化 → watcher 事件 → LocalMusicSource → MusicLibrary。

use notify::event::{CreateKind, ModifyKind, RemoveKind};
use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::source::LocalMusicSource;

/// 文件事件去重记录。
struct PendingEvent {
    /// 事件到达时间
    received_at: Instant,
    /// 事件类型（简化：create / modify / remove）
    kind: SimpleEventKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SimpleEventKind {
    Create,
    Modify,
    Remove,
}

/// 启动文件系统监听器（阻塞式，应在独立线程中运行）。
///
/// # 参数
/// - `source`: 本地音乐来源的共享引用
/// - `folders`: 需要监听的文件夹路径列表
///
/// # 行为
/// 此函数会阻塞当前线程，持续监听文件夹变化并同步到音乐库。
/// 建议在 `std::thread::spawn` 中调用。
///
/// 对于每个文件事件：
/// - **Create** → `source.index_file(path)`
/// - **Modify** → `source.reindex_file(path)`
/// - **Remove** → `source.unindex_file(path)`
///
/// # 事件去重
/// 同一文件在 500ms 内的重复事件（如编辑器保存触发的 Remove+Create）会被合并，
/// 只执行最终状态对应的操作。
pub fn start_watcher(source: Arc<LocalMusicSource>, folders: Vec<PathBuf>) -> Result<(), String> {
    let (tx, rx) = mpsc::channel::<Result<Event, notify::Error>>();

    let mut watcher = notify::recommended_watcher(tx)
        .map_err(|e| format!("创建文件监听器失败: {}", e))?;

    // 监听所有文件夹（递归）
    for folder in &folders {
        if folder.exists() {
            watcher
                .watch(folder, RecursiveMode::Recursive)
                .map_err(|e| format!("监听文件夹失败 '{}': {}", folder.display(), e))?;
        }
    }

    // 事件去重缓冲：file_path → PendingEvent
    let mut pending: HashMap<PathBuf, PendingEvent> = HashMap::new();
    let dedup_window = Duration::from_millis(500);

    // 事件处理循环
    loop {
        // 收集本批次事件（非阻塞获取，至少等一个）
        match rx.recv() {
            Ok(Ok(event)) => {
                handle_raw_event(&event, &mut pending);
            }
            Ok(Err(e)) => {
                eprintln!("[local_watcher] 监听错误: {}", e);
            }
            Err(mpsc::RecvError) => {
                // 通道关闭，退出
                break;
            }
        }

        // 处理所有就绪的已去重事件
        let now = Instant::now();
        let ready: Vec<(PathBuf, SimpleEventKind)> = pending
            .iter()
            .filter(|(_, e)| now.duration_since(e.received_at) >= dedup_window)
            .map(|(p, e)| (p.clone(), e.kind.clone()))
            .collect();

        for (path, kind) in ready {
            pending.remove(&path);

            let result = match kind {
                SimpleEventKind::Create => source.index_file(&path),
                SimpleEventKind::Modify => source.reindex_file(&path),
                SimpleEventKind::Remove => source.unindex_file(&path),
            };

            if let Err(e) = result {
                eprintln!("[local_watcher] 同步失败 '{}': {}", path.display(), e);
            }
        }

        // 非阻塞排空积压事件
        while let Ok(event) = rx.try_recv() {
            match event {
                Ok(ev) => handle_raw_event(&ev, &mut pending),
                Err(e) => eprintln!("[local_watcher] 监听错误: {}", e),
            }
        }
    }

    Ok(())
}

/// 处理原始 notify 事件，提取文件路径并加入去重缓冲。
fn handle_raw_event(event: &Event, pending: &mut HashMap<PathBuf, PendingEvent>) {
    let kind = match event.kind {
        EventKind::Create(CreateKind::File) => SimpleEventKind::Create,
        EventKind::Modify(ModifyKind::Data(_)) => SimpleEventKind::Modify,
        EventKind::Remove(RemoveKind::File) => SimpleEventKind::Remove,
        _ => return, // 忽略文件夹事件、访问事件、元数据事件等
    };

    for path in &event.paths {
        // 忽略非音频文件
        if !super::scanner::is_supported_audio(path) {
            continue;
        }

        let now = Instant::now();
        pending
            .entry(path.clone())
            .and_modify(|e| {
                e.received_at = now;
                e.kind = kind.clone();
            })
            .or_insert(PendingEvent {
                received_at: now,
                kind: kind.clone(),
            });
    }
}
