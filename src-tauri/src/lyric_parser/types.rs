use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LyricWord {
    pub start_time: u64,      // 起始时间（毫秒）
    pub end_time: u64,        // 结束时间（毫秒）
    pub word: String,         // 歌词文字
    pub roman_word: Option<String>, // 音译文字
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LyricLine {
    pub words: Vec<LyricWord>,        // 单词数组
    pub translated_lyric: Option<String>, // 翻译歌词
    pub roman_lyric: Option<String>,    // 音译歌词
    pub is_bg: bool,                  // 是否为背景歌词
    pub is_duet: bool,                // 是否为对唱歌词
    pub start_time: u64,              // 行起始时间
    pub end_time: u64,                // 行结束时间
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedLyric {
    pub lines: Vec<LyricLine>,
    pub metadata: Option<LyricMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LyricMetadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub author: Option<String>,
    pub offset: Option<i64>,      // 时间偏移（毫秒）
    pub length: Option<u64>,     // 总时长（毫秒）
}

#[derive(Debug, Clone)]
pub enum LyricFormat {
    Lrc,    // 标准 LRC 格式
    Yrc,    // 网易云音乐 YRC 格式
    Qrc,    // QQ 音乐 QRC 格式
    Ttml,   // TTML 格式
    Unknown,
}

impl LyricFormat {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "lrc" => LyricFormat::Lrc,
            "yrc" => LyricFormat::Yrc,
            "qrc" => LyricFormat::Qrc,
            "ttml" => LyricFormat::Ttml,
            _ => LyricFormat::Unknown,
        }
    }
    
    pub fn from_content(content: &str) -> Self {
        if content.contains("[00:") && content.contains("]") {
            LyricFormat::Lrc
        } else if content.contains("[") && content.contains("]") && content.contains("(") {
            if content.contains(",0)") {
                LyricFormat::Yrc
            } else {
                LyricFormat::Qrc
            }
        } else if content.contains("<?xml") && content.contains("tt") {
            LyricFormat::Ttml
        } else {
            LyricFormat::Unknown
        }
    }
}

#[derive(Debug)]
pub enum ParseError {
    InvalidFormat(String),
    InvalidTimestamp(String),
    IoError(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::InvalidFormat(msg) => write!(f, "Invalid lyric format: {}", msg),
            ParseError::InvalidTimestamp(msg) => write!(f, "Invalid timestamp: {}", msg),
            ParseError::IoError(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl std::error::Error for ParseError {}

impl ParsedLyric {
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            metadata: None,
        }
    }
    
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
    
    pub fn duration(&self) -> u64 {
        self.lines.last().map(|line| line.end_time).unwrap_or(0)
    }
    
    pub fn find_line_by_time(&self, time_ms: u64) -> Option<&LyricLine> {
        self.lines
            .iter()
            .find(|line| time_ms >= line.start_time && time_ms <= line.end_time)
    }
    
    pub fn find_current_line_index(&self, time_ms: u64) -> Option<usize> {
        self.lines
            .iter()
            .position(|line| time_ms >= line.start_time && time_ms <= line.end_time)
    }
}