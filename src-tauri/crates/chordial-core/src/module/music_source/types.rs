use serde::{Deserialize, Serialize};

/// 实体类型 — 标记一个来源引用指向哪种音乐实体。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EntityType {
    Song,
    Artist,
    Album,
    Lyric,
}

/// 来源类型 — 区分本地来源与不同的网络来源。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SourceType {
    /// 本地来源
    Local,
    /// 网络来源（携带来源名称，如 `"netease"`, `"spotify"`）
    Web(String),
}

/// 来源标识 — 描述某个实体在特定来源中的身份。
///
/// 一个实体（如歌曲）可以同时拥有多个 [`SourceId`]，表示它来自多个来源。
///
/// # 示例
///
/// ```ignore
/// SourceId {
///     source_name: "my_local".into(),
///     source_type: SourceType::Local,
///     entity_type: EntityType::Song,
///     entity_id: "song_001".into(),
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SourceId {
    /// 已注册的来源名称，如 `"my_local"`, `"netease"`, `"spotify"`
    pub source_name: String,
    /// 来源类型
    pub source_type: SourceType,
    /// 实体类型
    pub entity_type: EntityType,
    /// 该来源内部的实体 ID
    pub entity_id: String,
}

impl SourceId {
    /// 创建一个新的 [`SourceId`]。
    pub fn new(
        source_name: impl Into<String>,
        source_type: SourceType,
        entity_type: EntityType,
        entity_id: impl Into<String>,
    ) -> Self {
        Self {
            source_name: source_name.into(),
            source_type,
            entity_type,
            entity_id: entity_id.into(),
        }
    }

    /// 创建一个实体类型不同的副本（保留来源名、来源类型和实体 ID）。
    ///
    /// 用于将歌曲的来源引用传播到其关联的艺人和专辑上。
    pub fn with_entity_type(&self, entity_type: EntityType) -> Self {
        Self {
            source_name: self.source_name.clone(),
            source_type: self.source_type.clone(),
            entity_type,
            entity_id: self.entity_id.clone(),
        }
    }
}
