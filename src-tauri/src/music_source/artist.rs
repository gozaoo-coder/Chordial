//! 歌手类型定义
//!
//! 定义歌手完整信息和摘要信息类型

use serde::{Deserialize, Serialize};

/// 歌手完整信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artist {
    /// 歌手唯一标识 (通常为名稱的哈希或规范化后的名稱)
    pub id: String,
    /// 歌手名稱
    pub name: String,
    /// 歌手简介
    pub bio: Option<String>,
    /// 流派列表
    pub genres: Vec<String>,
    /// 封面图片数据 (Base64 Data URL)
    pub cover_data: Option<String>,
    /// 专辑ID列表
    pub album_ids: Vec<String>,
    /// 歌曲ID列表
    pub track_ids: Vec<String>,
}

impl Artist {
    /// 创建新的歌手
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            bio: None,
            genres: Vec::new(),
            cover_data: None,
            album_ids: Vec::new(),
            track_ids: Vec::new(),
        }
    }

    /// 添加专辑
    pub fn add_album(&mut self, album_id: String) {
        if !self.album_ids.contains(&album_id) {
            self.album_ids.push(album_id);
        }
    }

    /// 添加歌曲
    pub fn add_track(&mut self, track_id: String) {
        if !self.track_ids.contains(&track_id) {
            self.track_ids.push(track_id);
        }
    }

    /// 获取专辑数量
    pub fn album_count(&self) -> usize {
        self.album_ids.len()
    }

    /// 获取歌曲数量
    pub fn track_count(&self) -> usize {
        self.track_ids.len()
    }

    /// 生成摘要信息
    pub fn to_summary(&self) -> ArtistSummary {
        ArtistSummary {
            id: self.id.clone(),
            name: self.name.clone(),
            cover_data: self.cover_data.clone(),
            album_count: self.album_count(),
            track_count: self.track_count(),
        }
    }
}

/// 歌手摘要信息
/// 用于列表展示等场景，减少数据传输
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistSummary {
    /// 歌手唯一标识
    pub id: String,
    /// 歌手名稱
    pub name: String,
    /// 封面图片数据 (Base64 Data URL)
    pub cover_data: Option<String>,
    /// 专辑数量
    pub album_count: usize,
    /// 歌曲数量
    pub track_count: usize,
}

impl ArtistSummary {
    /// 创建新的歌手摘要
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            cover_data: None,
            album_count: 0,
            track_count: 0,
        }
    }

    /// 从完整歌手信息创建摘要
    pub fn from_artist(artist: &Artist) -> Self {
        artist.to_summary()
    }
}

/// 多歌手解析工具
pub struct ArtistParser;

impl ArtistParser {
    /// 解析可能包含多个歌手的字符串
    /// 支持 "/" 和 "&" 作为分隔符
    ///
    /// # 示例
    /// ```
    /// let artists = ArtistParser::parse("周杰伦/费玉清");
    /// assert_eq!(artists, vec!["周杰伦", "费玉清"]);
    ///
    /// let artists = ArtistParser::parse("Taylor Swift & Ed Sheeran");
    /// assert_eq!(artists, vec!["Taylor Swift", "Ed Sheeran"]);
    /// ```
    pub fn parse(artist_str: &str) -> Vec<String> {
        if artist_str.is_empty() {
            return Vec::new();
        }

        // 先尝试 "/" 分隔符
        let parts: Vec<&str> = if artist_str.contains('/') {
            artist_str.split('/').collect()
        } else if artist_str.contains('&') {
            artist_str.split('&').collect()
        } else {
            vec![artist_str]
        };

        parts
            .into_iter()
            .map(|s| Self::normalize_name(s))
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// 规范化歌手名稱
    /// - 去除首尾空白
    /// - 去除多余空格
    pub fn normalize_name(name: &str) -> String {
        name.trim().split_whitespace().collect::<Vec<_>>().join(" ")
    }

    /// 生成歌手ID
    /// 使用规范化后的名稱作为ID基础
    pub fn generate_id(name: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let normalized = Self::normalize_name(name);
        let mut hasher = DefaultHasher::new();
        normalized.hash(&mut hasher);
        format!("artist_{:x}", hasher.finish())
    }

    /// 从多个歌手名生成组合ID
    pub fn generate_combined_id(names: &[String]) -> String {
        if names.len() == 1 {
            return Self::generate_id(&names[0]);
        }

        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let combined = names.join(" / ");
        let mut hasher = DefaultHasher::new();
        combined.hash(&mut hasher);
        format!("artists_{:x}", hasher.finish())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_artist() {
        let artists = ArtistParser::parse("周杰伦");
        assert_eq!(artists, vec!["周杰伦"]);
    }

    #[test]
    fn test_parse_slash_separator() {
        let artists = ArtistParser::parse("周杰伦/费玉清");
        assert_eq!(artists, vec!["周杰伦", "费玉清"]);
    }

    #[test]
    fn test_parse_ampersand_separator() {
        let artists = ArtistParser::parse("Taylor Swift & Ed Sheeran");
        assert_eq!(artists, vec!["Taylor Swift", "Ed Sheeran"]);
    }

    #[test]
    fn test_parse_with_whitespace() {
        let artists = ArtistParser::parse("  周杰伦  /  费玉清  ");
        assert_eq!(artists, vec!["周杰伦", "费玉清"]);
    }

    #[test]
    fn test_parse_empty() {
        let artists = ArtistParser::parse("");
        assert!(artists.is_empty());
    }

    #[test]
    fn test_normalize_name() {
        assert_eq!(ArtistParser::normalize_name("  周杰伦  "), "周杰伦");
        assert_eq!(ArtistParser::normalize_name("Taylor   Swift"), "Taylor Swift");
    }

    #[test]
    fn test_artist_summary_from_artist() {
        let artist = Artist {
            id: "artist_123".to_string(),
            name: "周杰伦".to_string(),
            bio: Some("华语流行歌手".to_string()),
            genres: vec!["流行".to_string()],
            cover_data: None,
            album_ids: vec!["album_1".to_string(), "album_2".to_string()],
            track_ids: vec!["track_1".to_string(), "track_2".to_string(), "track_3".to_string()],
        };

        let summary = artist.to_summary();
        assert_eq!(summary.id, "artist_123");
        assert_eq!(summary.name, "周杰伦");
        assert_eq!(summary.album_count, 2);
        assert_eq!(summary.track_count, 3);
    }
}
