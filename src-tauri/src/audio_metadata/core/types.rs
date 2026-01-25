//! 通用数据结构定义

use std::time::Duration;
use std::fmt;

use super::AudioFormat;

/// 音频元数据结构
#[derive(Debug, Clone, Default)]
pub struct AudioMetadata {
    /// 标题
    pub title: Option<String>,
    /// 艺术家
    pub artist: Option<String>,
    /// 专辑
    pub album: Option<String>,
    /// 专辑艺术家
    pub album_artist: Option<String>,
    /// 作曲家
    pub composer: Option<String>,
    /// 年份
    pub year: Option<u32>,
    /// 音轨号
    pub track_number: Option<u32>,
    /// 总音轨数
    pub total_tracks: Option<u32>,
    /// 光盘号
    pub disc_number: Option<u32>,
    /// 总光盘数
    pub total_discs: Option<u32>,
    /// 流派
    pub genre: Option<String>,
    /// 评论
    pub comment: Option<String>,
    /// 歌词（非同步）
    pub lyrics: Option<String>,
    /// 歌词（同步，带时间戳）
    pub synced_lyrics: Option<Vec<LyricLine>>,
    /// 持续时间
    pub duration: Option<Duration>,
    /// 比特率 (kbps)
    pub bitrate: Option<u32>,
    /// 采样率 (Hz)
    pub sample_rate: Option<u32>,
    /// 声道数
    pub channels: Option<u32>,
    /// 文件格式
    pub format: AudioFormat,
    /// 封面图片
    pub pictures: Vec<Picture>,
    /// 额外的自定义标签
    pub extra_tags: std::collections::HashMap<String, String>,
}

impl AudioMetadata {
    /// 创建新的元数据实例
    pub fn new(format: AudioFormat) -> Self {
        Self {
            format,
            ..Default::default()
        }
    }

    /// 检查是否有基本的标签信息
    pub fn has_basic_tags(&self) -> bool {
        self.title.is_some() || self.artist.is_some() || self.album.is_some()
    }

    /// 获取完整的艺术家信息（包含专辑艺术家）
    pub fn get_artist_display(&self) -> Option<String> {
        if let Some(artist) = &self.artist {
            if let Some(album_artist) = &self.album_artist {
                if artist != album_artist {
                    return Some(format!("{} ({})", artist, album_artist));
                }
            }
            Some(artist.clone())
        } else {
            self.album_artist.clone()
        }
    }

    /// 获取音轨信息显示
    pub fn get_track_display(&self) -> Option<String> {
        match (self.track_number, self.total_tracks) {
            (Some(num), Some(total)) => Some(format!("{}/{}", num, total)),
            (Some(num), None) => Some(num.to_string()),
            _ => None,
        }
    }

    /// 获取光盘信息显示
    pub fn get_disc_display(&self) -> Option<String> {
        match (self.disc_number, self.total_discs) {
            (Some(num), Some(total)) => Some(format!("{}/{}", num, total)),
            (Some(num), None) => Some(num.to_string()),
            _ => None,
        }
    }
}

impl fmt::Display for AudioMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AudioMetadata {{ ")?;
        
        if let Some(title) = &self.title {
            write!(f, "title: \"{}\", ", title)?;
        }
        if let Some(artist) = &self.artist {
            write!(f, "artist: \"{}\", ", artist)?;
        }
        if let Some(album) = &self.album {
            write!(f, "album: \"{}\", ", album)?;
        }
        if let Some(duration) = self.duration {
            write!(f, "duration: {:?}, ", duration)?;
        }
        
        write!(f, "format: {:?} }}", self.format)
    }
}

/// 图片数据结构
#[derive(Debug, Clone)]
pub struct Picture {
    /// 图片类型（封面、艺术家等）
    pub picture_type: PictureType,
    /// MIME类型
    pub mime_type: String,
    /// 图片描述
    pub description: Option<String>,
    /// 图片宽度
    pub width: Option<u32>,
    /// 图片高度
    pub height: Option<u32>,
    /// 图片数据
    pub data: Vec<u8>,
}

impl Picture {
    /// 创建新的图片实例
    pub fn new(picture_type: PictureType, mime_type: String, data: Vec<u8>) -> Self {
        Self {
            picture_type,
            mime_type,
            description: None,
            width: None,
            height: None,
            data,
        }
    }

    /// 检查是否为封面图片
    pub fn is_cover(&self) -> bool {
        matches!(self.picture_type, PictureType::CoverFront)
    }

    /// 获取图片大小（字节）
    pub fn size(&self) -> usize {
        self.data.len()
    }
}

/// 同步歌词行
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LyricLine {
    /// 时间戳
    pub timestamp: Duration,
    /// 歌词文本
    pub text: String,
}

impl LyricLine {
    /// 创建新的歌词行
    pub fn new(timestamp: Duration, text: String) -> Self {
        Self { timestamp, text }
    }

    /// 格式化时间戳为 [mm:ss.xx] 格式
    pub fn format_timestamp(&self) -> String {
        let total_secs = self.timestamp.as_secs();
        let minutes = total_secs / 60;
        let seconds = total_secs % 60;
        let millis = self.timestamp.subsec_millis();
        format!("[{:02}:{:02}.{:03}]", minutes, seconds, millis)
    }
}

/// 图片类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PictureType {
    /// 其他
    Other,
    /// 文件图标
    FileIcon,
    /// 其他文件图标
    OtherFileIcon,
    /// 封面（正面）
    CoverFront,
    /// 封面（背面）
    CoverBack,
    /// 内页
    Leaflet,
    /// 媒体（如CD标签）
    Media,
    /// 艺术家
    Artist,
    /// 指挥家
    Conductor,
    /// 乐队
    Band,
    /// 作曲家
    Composer,
    /// 作词家
    Lyricist,
    /// 录制地点
    RecordingLocation,
    /// 录制期间
    DuringRecording,
    /// 表演期间
    DuringPerformance,
    /// 视频截图
    VideoScreenshot,
    /// 亮鱼（发光的鱼）
    BrightFish,
    /// 插图
    Illustration,
    /// 艺术家图片
    ArtistLogo,
    /// 出版商Logo
    PublisherLogo,
}

impl Default for PictureType {
    fn default() -> Self {
        PictureType::Other
    }
}

impl PictureType {
    /// 从ID3图片类型代码转换
    pub fn from_id3_code(code: u8) -> Self {
        match code {
            0 => PictureType::Other,
            1 => PictureType::FileIcon,
            2 => PictureType::OtherFileIcon,
            3 => PictureType::CoverFront,
            4 => PictureType::CoverBack,
            5 => PictureType::Leaflet,
            6 => PictureType::Media,
            7 => PictureType::Artist,
            8 => PictureType::Conductor,
            9 => PictureType::Band,
            10 => PictureType::Composer,
            11 => PictureType::Lyricist,
            12 => PictureType::RecordingLocation,
            13 => PictureType::DuringRecording,
            14 => PictureType::DuringPerformance,
            15 => PictureType::VideoScreenshot,
            16 => PictureType::BrightFish,
            17 => PictureType::Illustration,
            18 => PictureType::ArtistLogo,
            19 => PictureType::PublisherLogo,
            _ => PictureType::Other,
        }
    }

    /// 转换为ID3图片类型代码
    pub fn to_id3_code(&self) -> u8 {
        match self {
            PictureType::Other => 0,
            PictureType::FileIcon => 1,
            PictureType::OtherFileIcon => 2,
            PictureType::CoverFront => 3,
            PictureType::CoverBack => 4,
            PictureType::Leaflet => 5,
            PictureType::Media => 6,
            PictureType::Artist => 7,
            PictureType::Conductor => 8,
            PictureType::Band => 9,
            PictureType::Composer => 10,
            PictureType::Lyricist => 11,
            PictureType::RecordingLocation => 12,
            PictureType::DuringRecording => 13,
            PictureType::DuringPerformance => 14,
            PictureType::VideoScreenshot => 15,
            PictureType::BrightFish => 16,
            PictureType::Illustration => 17,
            PictureType::ArtistLogo => 18,
            PictureType::PublisherLogo => 19,
        }
    }
}
