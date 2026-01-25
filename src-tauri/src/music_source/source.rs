//! 音樂源定義
//!
//! 定義本地文件夾和網盤源的結構

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

/// 音樂源類型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceType {
    /// 本地文件夾
    LocalFolder,
    /// 網盤源 (webdev 協議)
    WebDisk,
}

/// 音樂源配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceConfig {
    /// 源唯一標識
    pub id: String,
    /// 源類型
    pub source_type: SourceType,
    /// 源路徑 (本地文件夾) 或 URL (網盤)
    pub path: PathBuf,
    /// 源名稱 (可自定義)
    pub name: String,
    /// 是否啟用
    pub enabled: bool,
    /// 選項
    pub options: SourceOptions,
    /// 創建時間
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 最後掃描時間
    pub last_scanned_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// 源選項
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceOptions {
    /// 是否遞歸掃描子文件夾 (本地文件夾)
    pub recursive: bool,
    /// 網盤認證信息 (網盤源)
    pub auth: Option<WebDiskAuth>,
    /// 文件擴展名過濾
    pub extensions: Vec<String>,
    /// 排除的路徑模式
    pub exclude_patterns: Vec<String>,
}

/// 網盤認證信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebDiskAuth {
    /// 用戶名
    pub username: String,
    /// 密碼或令牌
    pub password: String,
}

/// 音樂源trait
pub trait MusicSource {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn source_type(&self) -> SourceType;
    fn path(&self) -> &PathBuf;
    fn is_enabled(&self) -> bool;
    fn set_enabled(&mut self, enabled: bool);
    fn options(&self) -> &SourceOptions;
    fn options_mut(&mut self) -> &mut SourceOptions;
    fn created_at(&self) -> chrono::DateTime<chrono::Utc>;
    fn last_scanned_at(&self) -> Option<chrono::DateTime<chrono::Utc>>;
    fn set_last_scanned_at(&mut self, time: chrono::DateTime<chrono::Utc>);
}

impl SourceConfig {
    /// 創建新的本地文件夾源
    pub fn new_local_folder(path: PathBuf, name: Option<String>, recursive: bool) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            source_type: SourceType::LocalFolder,
            path: path.clone(),
            name: name.unwrap_or_else(|| path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("本地文件夾")
                .to_string()),
            enabled: true,
            options: SourceOptions {
                recursive,
                auth: None,
                extensions: vec!["mp3".to_string(), "flac".to_string(), "m4a".to_string(), "ogg".to_string(), "wav".to_string()],
                exclude_patterns: vec![],
            },
            created_at: chrono::Utc::now(),
            last_scanned_at: None,
        }
    }

    /// 創建新的網盤源
    pub fn new_web_disk(url: PathBuf, name: Option<String>, auth: Option<WebDiskAuth>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            source_type: SourceType::WebDisk,
            path: url.clone(),
            name: name.unwrap_or_else(|| url.to_string_lossy().to_string()),
            enabled: true,
            options: SourceOptions {
                recursive: true,
                auth,
                extensions: vec!["mp3".to_string(), "flac".to_string(), "m4a".to_string(), "ogg".to_string(), "wav".to_string()],
                exclude_patterns: vec![],
            },
            created_at: chrono::Utc::now(),
            last_scanned_at: None,
        }
    }
}

impl MusicSource for SourceConfig {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn source_type(&self) -> SourceType {
        self.source_type.clone()
    }

    fn path(&self) -> &PathBuf {
        &self.path
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn options(&self) -> &SourceOptions {
        &self.options
    }

    fn options_mut(&mut self) -> &mut SourceOptions {
        &mut self.options
    }

    fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.created_at
    }

    fn last_scanned_at(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.last_scanned_at
    }

    fn set_last_scanned_at(&mut self, time: chrono::DateTime<chrono::Utc>) {
        self.last_scanned_at = Some(time);
    }
}
