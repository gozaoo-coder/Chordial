//! 音频元数据核心类型定义
//!
//! 定义音频元数据结构、格式枚举、错误类型和 Trait

use std::fmt;
use std::path::Path;
use std::time::Duration;

/// 音频格式枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFormat {
    /// FLAC 格式
    Flac,
    /// MP3 格式
    Mp3,
    /// M4A/AAC 格式
    M4a,
    /// OGG/Vorbis 格式
    Ogg,
    /// WAV 格式
    Wav,
    /// 未知格式
    Unknown,
}

impl AudioFormat {
    /// 从文件扩展名检测音频格式
    pub fn from_extension(path: &Path) -> Self {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();

        match ext.as_str() {
            "flac" => AudioFormat::Flac,
            "mp3" => AudioFormat::Mp3,
            "m4a" | "mp4" | "aac" => AudioFormat::M4a,
            "ogg" | "oga" => AudioFormat::Ogg,
            "wav" | "wave" => AudioFormat::Wav,
            _ => AudioFormat::Unknown,
        }
    }

    /// 从文件头魔数检测音频格式
    pub fn from_magic_bytes(data: &[u8]) -> Self {
        if data.len() < 12 {
            return AudioFormat::Unknown;
        }

        // FLAC: "fLaC"
        if &data[0..4] == b"fLaC" {
            return AudioFormat::Flac;
        }

        // MP3: ID3 标签或 MP3 帧同步
        if &data[0..3] == b"ID3" {
            return AudioFormat::Mp3;
        }

        // M4A/MP4: ftyp 盒子
        if data.len() >= 12 && &data[4..8] == b"ftyp" {
            let brand = &data[8..12];
            if brand == b"M4A " || brand == b"mp42" || brand == b"isom" {
                return AudioFormat::M4a;
            }
        }

        // OGG: "OggS"
        if &data[0..4] == b"OggS" {
            return AudioFormat::Ogg;
        }

        // WAV: "RIFF" + "WAVE"
        if &data[0..4] == b"RIFF" && data.len() >= 8 && &data[8..12] == b"WAVE" {
            return AudioFormat::Wav;
        }

        // 检查 MP3 帧同步 (0xFFE0)
        if data.len() >= 2 {
            let sync_word = u16::from_be_bytes([data[0], data[1]]);
            if sync_word & 0xFFE0 == 0xFFE0 {
                return AudioFormat::Mp3;
            }
        }

        AudioFormat::Unknown
    }
}

impl fmt::Display for AudioFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AudioFormat::Flac => write!(f, "FLAC"),
            AudioFormat::Mp3 => write!(f, "MP3"),
            AudioFormat::M4a => write!(f, "M4A"),
            AudioFormat::Ogg => write!(f, "OGG"),
            AudioFormat::Wav => write!(f, "WAV"),
            AudioFormat::Unknown => write!(f, "Unknown"),
        }
    }
}

/// 图片类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PictureType {
    /// 其他
    Other,
    /// 32x32 像素文件图标
    FileIcon,
    /// 其他文件图标
    OtherFileIcon,
    /// 封面（正面）
    CoverFront,
    /// 封面（背面）
    CoverBack,
    /// 宣传单页
    LeafletPage,
    /// 媒体（如 CD 标签）
    Media,
    /// 主唱/表演者
    LeadArtist,
    /// 艺术家/表演者
    Artist,
    /// 指挥
    Conductor,
    /// 乐队/乐团
    Band,
    /// 作曲
    Composer,
    /// 作词/编剧
    Lyricist,
    /// 录音地点
    RecordingLocation,
    /// 录音期间
    DuringRecording,
    /// 表演期间
    DuringPerformance,
    /// 电影/视频截图
    MovieScreenCapture,
    /// 彩色鱼（？）
    ColoredFish,
    /// 插图
    Illustration,
    /// 乐队/艺术家标志
    BandLogo,
    /// 出版商/工作室标志
    PublisherLogo,
}

impl PictureType {
    /// 从 ID3v2 APIC 类型代码创建
    pub fn from_id3v2_type(code: u8) -> Self {
        match code {
            0x00 => PictureType::Other,
            0x01 => PictureType::FileIcon,
            0x02 => PictureType::OtherFileIcon,
            0x03 => PictureType::CoverFront,
            0x04 => PictureType::CoverBack,
            0x05 => PictureType::LeafletPage,
            0x06 => PictureType::Media,
            0x07 => PictureType::LeadArtist,
            0x08 => PictureType::Artist,
            0x09 => PictureType::Conductor,
            0x0A => PictureType::Band,
            0x0B => PictureType::Composer,
            0x0C => PictureType::Lyricist,
            0x0D => PictureType::RecordingLocation,
            0x0E => PictureType::DuringRecording,
            0x0F => PictureType::DuringPerformance,
            0x10 => PictureType::MovieScreenCapture,
            0x11 => PictureType::ColoredFish,
            0x12 => PictureType::Illustration,
            0x13 => PictureType::BandLogo,
            0x14 => PictureType::PublisherLogo,
            _ => PictureType::Other,
        }
    }

    /// 转换为 FLAC/Vorbis 评论中的图片类型字符串
    pub fn to_vorbis_string(&self) -> &'static str {
        match self {
            PictureType::Other => "Other",
            PictureType::FileIcon => "File Icon",
            PictureType::OtherFileIcon => "Other File Icon",
            PictureType::CoverFront => "Cover (front)",
            PictureType::CoverBack => "Cover (back)",
            PictureType::LeafletPage => "Leaflet page",
            PictureType::Media => "Media (e.g. label side of CD)",
            PictureType::LeadArtist => "Lead artist/lead performer/soloist",
            PictureType::Artist => "Artist/performer",
            PictureType::Conductor => "Conductor",
            PictureType::Band => "Band/Orchestra",
            PictureType::Composer => "Composer",
            PictureType::Lyricist => "Lyricist/text writer",
            PictureType::RecordingLocation => "Recording Location",
            PictureType::DuringRecording => "During recording",
            PictureType::DuringPerformance => "During performance",
            PictureType::MovieScreenCapture => "Movie/video screen capture",
            PictureType::ColoredFish => "A bright coloured fish",
            PictureType::Illustration => "Illustration",
            PictureType::BandLogo => "Band/artist logotype",
            PictureType::PublisherLogo => "Publisher/Studio logotype",
        }
    }
}

/// 内嵌图片结构
#[derive(Debug, Clone)]
pub struct Picture {
    /// 图片类型
    pub picture_type: PictureType,
    /// MIME 类型
    pub mime_type: String,
    /// 描述
    pub description: String,
    /// 图片数据
    pub data: Vec<u8>,
    /// 宽度（像素）
    pub width: Option<u32>,
    /// 高度（像素）
    pub height: Option<u32>,
    /// 颜色深度
    pub color_depth: Option<u32>,
    /// 索引颜色数
    pub indexed_colors: Option<u32>,
}

impl Picture {
    /// 创建新的图片
    pub fn new(
        picture_type: PictureType,
        mime_type: String,
        description: String,
        data: Vec<u8>,
    ) -> Self {
        Self {
            picture_type,
            mime_type,
            description,
            data,
            width: None,
            height: None,
            color_depth: None,
            indexed_colors: None,
        }
    }

    /// 检查是否为封面图片
    pub fn is_cover(&self) -> bool {
        self.picture_type == PictureType::CoverFront
    }

    /// 获取图片格式扩展名
    pub fn format_extension(&self) -> &str {
        match self.mime_type.as_str() {
            "image/jpeg" | "image/jpg" => "jpg",
            "image/png" => "png",
            "image/gif" => "gif",
            "image/bmp" => "bmp",
            "image/webp" => "webp",
            _ => "bin",
        }
    }
}

/// 歌词行结构
#[derive(Debug, Clone)]
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

    /// 获取时间戳（毫秒）
    pub fn timestamp_millis(&self) -> u64 {
        self.timestamp.as_millis() as u64
    }
}

/// 音频元数据结构
#[derive(Debug, Clone)]
pub struct AudioMetadata {
    /// 音频格式
    pub format: AudioFormat,
    /// 标题
    pub title: Option<String>,
    /// 艺术家
    pub artist: Option<String>,
    /// 专辑
    pub album: Option<String>,
    /// 专辑艺术家
    pub album_artist: Option<String>,
    /// 年份
    pub year: Option<u32>,
    /// 音轨号
    pub track_number: Option<u32>,
    /// 总音轨数
    pub total_tracks: Option<u32>,
    /// 碟片号
    pub disc_number: Option<u32>,
    /// 总碟片数
    pub total_discs: Option<u32>,
    /// 流派
    pub genre: Option<String>,
    /// 作曲
    pub composer: Option<String>,
    /// 作词
    pub lyricist: Option<String>,
    /// 评论
    pub comment: Option<String>,
    /// 时长
    pub duration: Option<Duration>,
    /// 比特率（kbps）
    pub bitrate: Option<u32>,
    /// 采样率（Hz）
    pub sample_rate: Option<u32>,
    /// 声道数
    pub channels: Option<u8>,
    /// 内嵌图片列表
    pub pictures: Vec<Picture>,
    /// 普通歌词
    pub lyrics: Option<String>,
    /// 同步歌词
    pub synced_lyrics: Option<Vec<LyricLine>>,
}

impl AudioMetadata {
    /// 创建空的音频元数据
    pub fn new(format: AudioFormat) -> Self {
        Self {
            format,
            title: None,
            artist: None,
            album: None,
            album_artist: None,
            year: None,
            track_number: None,
            total_tracks: None,
            disc_number: None,
            total_discs: None,
            genre: None,
            composer: None,
            lyricist: None,
            comment: None,
            duration: None,
            bitrate: None,
            sample_rate: None,
            channels: None,
            pictures: Vec::new(),
            lyrics: None,
            synced_lyrics: None,
        }
    }

    /// 获取主标题（如果没有标题则返回文件名）
    pub fn title_or_filename(&self, path: &Path) -> String {
        self.title.clone().unwrap_or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown")
                .to_string()
        })
    }

    /// 获取显示艺术家（如果没有艺术家则返回"Unknown Artist"）
    pub fn artist_or_unknown(&self) -> String {
        self.artist.clone().unwrap_or_else(|| "Unknown Artist".to_string())
    }

    /// 获取封面图片
    pub fn cover_picture(&self) -> Option<&Picture> {
        self.pictures
            .iter()
            .find(|p| p.is_cover())
            .or_else(|| self.pictures.first())
    }

    /// 添加图片
    pub fn add_picture(&mut self, picture: Picture) {
        self.pictures.push(picture);
    }

    /// 设置普通歌词
    pub fn set_lyrics(&mut self, lyrics: String) {
        self.lyrics = Some(lyrics);
    }

    /// 设置同步歌词
    pub fn set_synced_lyrics(&mut self, lyrics: Vec<LyricLine>) {
        self.synced_lyrics = Some(lyrics);
    }
}

impl Default for AudioMetadata {
    fn default() -> Self {
        Self::new(AudioFormat::Unknown)
    }
}
