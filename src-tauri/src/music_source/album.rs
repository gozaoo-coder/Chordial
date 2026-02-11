//! 专辑类型定义
//!
//! 定义专辑完整信息和摘要信息类型
//!
//! # 性能优化
//! - 封面数据使用 Arc<String> 共享，避免大体积二进制数据克隆

use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// 专辑完整信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Album {
    /// 专辑唯一标识 (通常为专辑名稱+歌手名的哈希)
    pub id: String,
    /// 专辑名稱
    pub title: String,
    /// 歌手ID
    pub artist_id: String,
    /// 歌手名稱
    pub artist_name: String,
    /// 发行年份
    pub year: Option<u32>,
    /// 流派列表
    pub genres: Vec<String>,
    /// 封面图片数据 (Base64 Data URL)
    /// 使用 Arc 共享大体积二进制数据，避免克隆开销
    /// 不序列化到缓存，按需从音乐文件读取
    #[serde(skip)]
    pub cover_data: Option<Arc<String>>,
    /// 歌曲ID列表 (按曲目顺序)
    pub track_ids: Vec<String>,
    /// 总时长（秒）
    pub total_duration: u64,
}

impl Album {
    /// 创建新的专辑
    pub fn new(id: String, title: String, artist_id: String, artist_name: String) -> Self {
        Self {
            id,
            title,
            artist_id,
            artist_name,
            year: None,
            genres: Vec::new(),
            cover_data: None,
            track_ids: Vec::new(),
            total_duration: 0,
        }
    }

    /// 添加歌曲
    pub fn add_track(&mut self, track_id: String) {
        if !self.track_ids.contains(&track_id) {
            self.track_ids.push(track_id);
        }
    }

    /// 获取歌曲数量
    pub fn track_count(&self) -> usize {
        self.track_ids.len()
    }

    /// 生成摘要信息
    pub fn to_summary(&self) -> AlbumSummary {
        AlbumSummary {
            id: self.id.clone(),
            title: self.title.clone(),
            artist_id: self.artist_id.clone(),
            artist_name: self.artist_name.clone(),
            cover_data: self.cover_data.clone(),
            year: self.year,
            track_count: self.track_count(),
        }
    }
}

/// 专辑摘要信息
/// 用于列表展示等场景，减少数据传输
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumSummary {
    /// 专辑唯一标识
    pub id: String,
    /// 专辑名稱
    pub title: String,
    /// 歌手ID
    pub artist_id: String,
    /// 歌手名稱
    pub artist_name: String,
    /// 封面图片数据 (Base64 Data URL)
    /// 使用 Arc 共享大体积二进制数据
    pub cover_data: Option<Arc<String>>,
    /// 发行年份
    pub year: Option<u32>,
    /// 歌曲数量
    pub track_count: usize,
}

impl AlbumSummary {
    /// 创建新的专辑摘要
    pub fn new(id: String, title: String, artist_id: String, artist_name: String) -> Self {
        Self {
            id,
            title,
            artist_id,
            artist_name,
            cover_data: None,
            year: None,
            track_count: 0,
        }
    }

    /// 从完整专辑信息创建摘要
    pub fn from_album(album: &Album) -> Self {
        album.to_summary()
    }
}

/// 专辑ID生成工具
pub struct AlbumIdGenerator;

impl AlbumIdGenerator {
    /// 生成专辑ID
    /// 基于专辑名稱和歌手名稱的哈希
    pub fn generate_id(title: &str, artist_name: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let normalized_title = Self::normalize_name(title);
        let normalized_artist = Self::normalize_name(artist_name);
        let combined = format!("{} - {}", normalized_title, normalized_artist);

        let mut hasher = DefaultHasher::new();
        combined.hash(&mut hasher);
        format!("album_{:x}", hasher.finish())
    }

    /// 规范化名稱
    /// - 去除首尾空白
    /// - 转换为小写（可选，用于提高匹配率）
    pub fn normalize_name(name: &str) -> String {
        name.trim().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_album_new() {
        let album = Album::new(
            "album_123".to_string(),
            "范特西".to_string(),
            "artist_456".to_string(),
            "周杰伦".to_string(),
        );

        assert_eq!(album.id, "album_123");
        assert_eq!(album.title, "范特西");
        assert_eq!(album.artist_id, "artist_456");
        assert_eq!(album.artist_name, "周杰伦");
        assert_eq!(album.track_count(), 0);
    }

    #[test]
    fn test_album_add_track() {
        let mut album = Album::new(
            "album_123".to_string(),
            "范特西".to_string(),
            "artist_456".to_string(),
            "周杰伦".to_string(),
        );

        album.add_track("track_1".to_string());
        album.add_track("track_2".to_string());
        album.add_track("track_1".to_string()); // 重复添加应该被忽略

        assert_eq!(album.track_count(), 2);
        assert_eq!(album.track_ids, vec!["track_1", "track_2"]);
    }

    #[test]
    fn test_album_to_summary() {
        let album = Album {
            id: "album_123".to_string(),
            title: "范特西".to_string(),
            artist_id: "artist_456".to_string(),
            artist_name: "周杰伦".to_string(),
            year: Some(2001),
            genres: vec!["流行".to_string()],
            cover_data: Some(Arc::new("data:image/jpeg;base64,...".to_string())),
            track_ids: vec!["track_1".to_string(), "track_2".to_string()],
            total_duration: 3600,
        };

        let summary = album.to_summary();
        assert_eq!(summary.id, "album_123");
        assert_eq!(summary.title, "范特西");
        assert_eq!(summary.artist_id, "artist_456");
        assert_eq!(summary.artist_name, "周杰伦");
        assert_eq!(summary.year, Some(2001));
        assert_eq!(summary.track_count, 2);
        assert_eq!(summary.cover_data, Some(Arc::new("data:image/jpeg;base64,...".to_string())));
    }

    #[test]
    fn test_album_summary_from_album() {
        let album = Album::new(
            "album_123".to_string(),
            "范特西".to_string(),
            "artist_456".to_string(),
            "周杰伦".to_string(),
        );

        let summary = AlbumSummary::from_album(&album);
        assert_eq!(summary.id, "album_123");
        assert_eq!(summary.title, "范特西");
    }

    #[test]
    fn test_generate_id() {
        let id1 = AlbumIdGenerator::generate_id("范特西", "周杰伦");
        let id2 = AlbumIdGenerator::generate_id("范特西", "周杰伦");
        let id3 = AlbumIdGenerator::generate_id("叶惠美", "周杰伦");

        assert_eq!(id1, id2); // 相同输入应该产生相同ID
        assert_ne!(id1, id3); // 不同输入应该产生不同ID
        assert!(id1.starts_with("album_"));
    }
}
