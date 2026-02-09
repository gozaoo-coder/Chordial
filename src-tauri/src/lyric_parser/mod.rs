//! 歌词解析器模块
//!
//! 支持多种歌词格式：LRC、YRC、QRC、TTML

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;

// 缓存的正则表达式
static LRC_TIMESTAMP_REGEX: OnceLock<Regex> = OnceLock::new();
static LRC_META_REGEX: OnceLock<Regex> = OnceLock::new();
static YRC_LINE_REGEX: OnceLock<Regex> = OnceLock::new();
static YRC_WORD_REGEX: OnceLock<Regex> = OnceLock::new();
static QRC_LINE_REGEX: OnceLock<Regex> = OnceLock::new();
static QRC_WORD_REGEX: OnceLock<Regex> = OnceLock::new();
static TTML_TIME_FULL_REGEX: OnceLock<Regex> = OnceLock::new();
static TTML_TIME_SHORT_REGEX: OnceLock<Regex> = OnceLock::new();
static TTML_P_REGEX: OnceLock<Regex> = OnceLock::new();
static TTML_SPAN_REGEX: OnceLock<Regex> = OnceLock::new();
static TTML_HTML_TAG_REGEX: OnceLock<Regex> = OnceLock::new();

/// 默认歌词行持续时间（毫秒）
const DEFAULT_LINE_DURATION_MS: u64 = 5000;
/// 默认单词持续时间（毫秒）
const DEFAULT_WORD_DURATION_MS: u64 = 200;
/// 新行时间间隔阈值（毫秒）
const NEW_LINE_THRESHOLD_MS: u64 = 10000;

/// 歌词格式枚举
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LyricFormat {
    /// 标准 LRC 格式
    Lrc,
    /// 网易云音乐 YRC 格式（逐字歌词）
    Yrc,
    /// QQ 音乐 QRC 格式
    Qrc,
    /// TTML 格式（时序文本标记语言）
    Ttml,
    /// 未知格式
    Unknown,
}

impl LyricFormat {
    /// 从内容自动检测歌词格式
    pub fn from_content(content: &str) -> Self {
        let content = content.trim();

        // 检测 YRC 格式（网易云音乐）
        if content.contains("[ver:v1]") && content.contains("[by:网易云]")
            || content.contains("yrc") || content.contains("YRC")
            || (content.contains('[') && content.contains("<") && content.contains('>'))
        {
            return LyricFormat::Yrc;
        }

        // 检测 QRC 格式（QQ 音乐）
        if content.contains("[ver:qrc]") || content.contains("QRC")
            || (content.contains('[') && content.contains(']') && content.contains("(") && content.contains(")"))
        {
            return LyricFormat::Qrc;
        }

        // 检测 TTML 格式
        if content.starts_with("<?xml") || content.contains("<tt") || content.contains("<ttml")
            || content.contains("<p begin=") || content.contains("<span begin=")
        {
            return LyricFormat::Ttml;
        }

        // 检测 LRC 格式
        let lrc_timestamp_regex = LRC_TIMESTAMP_REGEX.get_or_init(|| {
            Regex::new(r"\[\d{2}:\d{2}[\.:]\d{2,3}\]").expect("LRC timestamp regex should be valid")
        });
        if content.contains("[ti:") || content.contains("[ar:") || content.contains("[al:")
            || lrc_timestamp_regex.is_match(content)
        {
            return LyricFormat::Lrc;
        }

        LyricFormat::Unknown
    }
}

/// 解析错误类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// 空内容
    EmptyContent,
    /// 格式错误
    InvalidFormat(String),
    /// 解析失败
    ParseFailed(String),
    /// 不支持的歌词格式
    UnsupportedFormat,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::EmptyContent => write!(f, "歌词内容为空"),
            ParseError::InvalidFormat(msg) => write!(f, "格式错误: {}", msg),
            ParseError::ParseFailed(msg) => write!(f, "解析失败: {}", msg),
            ParseError::UnsupportedFormat => write!(f, "不支持的歌词格式"),
        }
    }
}

impl std::error::Error for ParseError {}

/// 歌词元数据
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LyricMetadata {
    /// 歌曲标题
    pub title: Option<String>,
    /// 艺术家
    pub artist: Option<String>,
    /// 专辑
    pub album: Option<String>,
    /// 歌词作者
    pub by: Option<String>,
    /// 偏移量（毫秒）
    pub offset: i64,
    /// 其他元数据
    pub extra: HashMap<String, String>,
}

impl LyricMetadata {
    pub fn new() -> Self {
        Self::default()
    }
}

/// 歌词单词（逐字歌词用）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LyricWord {
    /// 开始时间（毫秒）
    pub start_time: u64,
    /// 持续时间（毫秒）
    pub duration: u64,
    /// 单词文本
    pub word: String,
}

impl LyricWord {
    pub fn new(start_time: u64, duration: u64, word: String) -> Self {
        Self {
            start_time,
            duration,
            word,
        }
    }
}

/// 歌词行
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LyricLine {
    /// 开始时间（毫秒）
    pub start_time: u64,
    /// 持续时间（毫秒）
    pub duration: u64,
    /// 单词列表（逐字歌词）
    pub words: Vec<LyricWord>,
}

impl LyricLine {
    pub fn new(start_time: u64, duration: u64) -> Self {
        Self {
            start_time,
            duration,
            words: Vec::new(),
        }
    }

    /// 获取整行文本
    pub fn text(&self) -> String {
        self.words.iter().map(|w| w.word.as_str()).collect::<String>()
    }

    /// 添加单词
    pub fn add_word(&mut self, word: LyricWord) {
        self.words.push(word);
    }
}

/// 解析后的歌词
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParsedLyric {
    /// 歌词行列表
    pub lines: Vec<LyricLine>,
    /// 歌词元数据
    pub metadata: LyricMetadata,
}

impl ParsedLyric {
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            metadata: LyricMetadata::new(),
        }
    }

    /// 根据时间查找当前歌词行
    pub fn find_line_by_time(&self, time_ms: u64) -> Option<&LyricLine> {
        if self.lines.is_empty() {
            return None;
        }

        // 使用二分查找优化
        let mut left = 0;
        let mut right = self.lines.len();

        while left < right {
            let mid = (left + right) / 2;
            let line = &self.lines[mid];

            if time_ms < line.start_time {
                right = mid;
            } else if time_ms >= line.start_time + line.duration {
                left = mid + 1;
            } else {
                return Some(line);
            }
        }

        // 如果找不到完全匹配的行，返回最后一行开始时间之前的行
        if left > 0 && left <= self.lines.len() {
            let prev_line = &self.lines[left - 1];
            if time_ms >= prev_line.start_time {
                return Some(prev_line);
            }
        }

        None
    }

    /// 根据时间查找当前歌词行索引
    pub fn find_current_line_index(&self, time_ms: u64) -> Option<usize> {
        if self.lines.is_empty() {
            return None;
        }

        // 使用二分查找优化
        let mut left = 0;
        let mut right = self.lines.len();

        while left < right {
            let mid = (left + right) / 2;
            let line = &self.lines[mid];

            if time_ms < line.start_time {
                right = mid;
            } else if time_ms >= line.start_time + line.duration {
                left = mid + 1;
            } else {
                return Some(mid);
            }
        }

        // 如果找不到完全匹配的行，返回最后一行开始时间之前的行索引
        if left > 0 && left <= self.lines.len() {
            let prev_idx = left - 1;
            let prev_line = &self.lines[prev_idx];
            if time_ms >= prev_line.start_time {
                return Some(prev_idx);
            }
        }

        None
    }

    /// 添加歌词行
    pub fn add_line(&mut self, line: LyricLine) {
        self.lines.push(line);
    }

    /// 按时间排序歌词行
    pub fn sort_lines(&mut self) {
        self.lines.sort_by_key(|line| line.start_time);
    }
}

impl Default for ParsedLyric {
    fn default() -> Self {
        Self::new()
    }
}

/// 歌词解析器
pub struct LyricParser;

impl LyricParser {
    /// 创建新的歌词解析器
    pub fn new() -> Self {
        Self
    }

    /// 解析歌词内容
    pub fn parse(&self, content: &str, format: LyricFormat) -> Result<ParsedLyric, ParseError> {
        if content.trim().is_empty() {
            return Err(ParseError::EmptyContent);
        }

        match format {
            LyricFormat::Lrc => self.parse_lrc(content),
            LyricFormat::Yrc => self.parse_yrc(content),
            LyricFormat::Qrc => self.parse_qrc(content),
            LyricFormat::Ttml => self.parse_ttml(content),
            LyricFormat::Unknown => Err(ParseError::UnsupportedFormat),
        }
    }

    /// 解析 LRC 格式
    fn parse_lrc(&self, content: &str) -> Result<ParsedLyric, ParseError> {
        let mut parsed = ParsedLyric::new();
        let mut temp_lines: Vec<(u64, String)> = Vec::new();

        let timestamp_regex = LRC_TIMESTAMP_REGEX.get_or_init(|| {
            Regex::new(r"\[(\d{2}):(\d{2})[\.:](\d{2,3})\]").expect("LRC timestamp regex should be valid")
        });
        let meta_regex = LRC_META_REGEX.get_or_init(|| {
            Regex::new(r"\[(\w+):([^\]]*)\]").expect("LRC meta regex should be valid")
        });

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // 解析元数据
            if let Some(caps) = meta_regex.captures(line) {
                let key = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                let value = caps.get(2).map(|m| m.as_str().trim()).unwrap_or("");

                match key {
                    "ti" => parsed.metadata.title = Some(value.to_string()),
                    "ar" => parsed.metadata.artist = Some(value.to_string()),
                    "al" => parsed.metadata.album = Some(value.to_string()),
                    "by" => parsed.metadata.by = Some(value.to_string()),
                    "offset" => {
                        if let Ok(offset) = value.parse::<i64>() {
                            parsed.metadata.offset = offset;
                        }
                    }
                    _ => {
                        parsed.metadata.extra.insert(key.to_string(), value.to_string());
                    }
                }
            }

            // 解析时间戳和歌词文本
            let timestamps: Vec<u64> = timestamp_regex
                .find_iter(line)
                .filter_map(|m| parse_lrc_timestamp(m.as_str()))
                .collect();

            if !timestamps.is_empty() {
                // 移除所有时间戳标签，获取纯歌词文本
                let text = timestamp_regex.replace_all(line, "").trim().to_string();

                for timestamp in timestamps {
                    temp_lines.push((timestamp, text.clone()));
                }
            }
        }

        // 按时间排序
        temp_lines.sort_by_key(|(time, _)| *time);

        // 转换为 LyricLine
        for i in 0..temp_lines.len() {
            let (start_time, text) = &temp_lines[i];
            let duration = if i + 1 < temp_lines.len() {
                temp_lines[i + 1].0 - start_time
            } else {
                DEFAULT_LINE_DURATION_MS // 默认最后一行持续5秒
            };

            let mut line = LyricLine::new(*start_time, duration);
            line.add_word(LyricWord::new(*start_time, duration, text.clone()));
            parsed.add_line(line);
        }

        Ok(parsed)
    }

    /// 解析 YRC 格式（网易云音乐逐字歌词）
    fn parse_yrc(&self, content: &str) -> Result<ParsedLyric, ParseError> {
        let mut parsed = ParsedLyric::new();

        // YRC 格式示例:
        // [ver:v1]
        // [ar:艺术家]
        // [ti:标题]
        // [by:网易云]
        // [offset:0]
        // [0,1000]<0,200>歌<200,300>词<500,500>内<800,200>容

        let meta_regex = LRC_META_REGEX.get_or_init(|| {
            Regex::new(r"\[(\w+):([^\]]*)\]").expect("Meta regex should be valid")
        });
        let line_regex = YRC_LINE_REGEX.get_or_init(|| {
            Regex::new(r"\[(\d+),(\d+)\](.*)").expect("YRC line regex should be valid")
        });
        let word_regex = YRC_WORD_REGEX.get_or_init(|| {
            Regex::new(r"<(\d+),(\d+)>([^<]*)").expect("YRC word regex should be valid")
        });

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // 解析元数据
            if let Some(caps) = meta_regex.captures(line) {
                let key = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                let value = caps.get(2).map(|m| m.as_str().trim()).unwrap_or("");

                match key {
                    "ar" => parsed.metadata.artist = Some(value.to_string()),
                    "ti" => parsed.metadata.title = Some(value.to_string()),
                    "al" => parsed.metadata.album = Some(value.to_string()),
                    "by" => parsed.metadata.by = Some(value.to_string()),
                    "offset" => {
                        if let Ok(offset) = value.parse::<i64>() {
                            parsed.metadata.offset = offset;
                        }
                    }
                    _ => {}
                }
                continue;
            }

            // 解析歌词行
            if let Some(caps) = line_regex.captures(line) {
                let start_time: u64 = caps
                    .get(1)
                    .and_then(|m| m.as_str().parse().ok())
                    .unwrap_or(0);
                let duration: u64 = caps
                    .get(2)
                    .and_then(|m| m.as_str().parse().ok())
                    .unwrap_or(0);
                let words_content = caps.get(3).map(|m| m.as_str()).unwrap_or("");

                let mut lyric_line = LyricLine::new(start_time, duration);

                // 解析逐字时间戳
                for word_caps in word_regex.captures_iter(words_content) {
                    let word_start: u64 = word_caps
                        .get(1)
                        .and_then(|m| m.as_str().parse().ok())
                        .unwrap_or(0);
                    let word_duration: u64 = word_caps
                        .get(2)
                        .and_then(|m| m.as_str().parse().ok())
                        .unwrap_or(0);
                    let word_text = word_caps.get(3).map(|m| m.as_str()).unwrap_or("");

                    lyric_line.add_word(LyricWord::new(
                        start_time + word_start,
                        word_duration,
                        word_text.to_string(),
                    ));
                }

                // 如果没有解析到逐字时间戳，将整个内容作为一个单词
                if lyric_line.words.is_empty() && !words_content.is_empty() {
                    lyric_line.add_word(LyricWord::new(start_time, duration, words_content.to_string()));
                }

                parsed.add_line(lyric_line);
            }
        }

        parsed.sort_lines();
        Ok(parsed)
    }

    /// 解析 QRC 格式（QQ 音乐歌词）
    fn parse_qrc(&self, content: &str) -> Result<ParsedLyric, ParseError> {
        let mut parsed = ParsedLyric::new();

        // QRC 格式示例:
        // [ver:qrc]
        // [ar:艺术家]
        // [ti:标题]
        // [offset:0]
        // [0,1000]歌词(0,200)内容(200,300)示例

        let meta_regex = LRC_META_REGEX.get_or_init(|| {
            Regex::new(r"\[(\w+):([^\]]*)\]").expect("Meta regex should be valid")
        });
        let line_regex = QRC_LINE_REGEX.get_or_init(|| {
            Regex::new(r"\[(\d+),(\d+)\](.*)").expect("QRC line regex should be valid")
        });
        let word_regex = QRC_WORD_REGEX.get_or_init(|| {
            Regex::new(r"\((\d+),(\d+)\)([^\(]*)").expect("QRC word regex should be valid")
        });

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // 解析元数据
            if let Some(caps) = meta_regex.captures(line) {
                let key = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                let value = caps.get(2).map(|m| m.as_str().trim()).unwrap_or("");

                match key {
                    "ar" => parsed.metadata.artist = Some(value.to_string()),
                    "ti" => parsed.metadata.title = Some(value.to_string()),
                    "al" => parsed.metadata.album = Some(value.to_string()),
                    "by" => parsed.metadata.by = Some(value.to_string()),
                    "offset" => {
                        if let Ok(offset) = value.parse::<i64>() {
                            parsed.metadata.offset = offset;
                        }
                    }
                    _ => {}
                }
                continue;
            }

            // 解析歌词行
            if let Some(caps) = line_regex.captures(line) {
                let start_time: u64 = caps
                    .get(1)
                    .and_then(|m| m.as_str().parse().ok())
                    .unwrap_or(0);
                let duration: u64 = caps
                    .get(2)
                    .and_then(|m| m.as_str().parse().ok())
                    .unwrap_or(0);
                let words_content = caps.get(3).map(|m| m.as_str()).unwrap_or("");

                let mut lyric_line = LyricLine::new(start_time, duration);

                // 解析逐字时间戳
                for word_caps in word_regex.captures_iter(words_content) {
                    let word_start: u64 = word_caps
                        .get(1)
                        .and_then(|m| m.as_str().parse().ok())
                        .unwrap_or(0);
                    let word_duration: u64 = word_caps
                        .get(2)
                        .and_then(|m| m.as_str().parse().ok())
                        .unwrap_or(0);
                    let word_text = word_caps.get(3).map(|m| m.as_str()).unwrap_or("");

                    lyric_line.add_word(LyricWord::new(
                        word_start,
                        word_duration,
                        word_text.to_string(),
                    ));
                }

                // 如果没有解析到逐字时间戳，将整个内容作为一个单词
                if lyric_line.words.is_empty() && !words_content.is_empty() {
                    lyric_line.add_word(LyricWord::new(start_time, duration, words_content.to_string()));
                }

                parsed.add_line(lyric_line);
            }
        }

        parsed.sort_lines();
        Ok(parsed)
    }

    /// 解析 TTML 格式
    fn parse_ttml(&self, content: &str) -> Result<ParsedLyric, ParseError> {
        let mut parsed = ParsedLyric::new();

        // TTML 格式示例:
        // <?xml version="1.0" encoding="UTF-8"?>
        // <tt xmlns="http://www.w3.org/ns/ttml">
        //   <body>
        //     <div>
        //       <p begin="00:00:01.000" end="00:00:05.000">歌词内容</p>
        //     </div>
        //   </body>
        // </tt>

        // 解析时间属性
        let time_full_regex = TTML_TIME_FULL_REGEX.get_or_init(|| {
            Regex::new(r"(\d{2}):(\d{2}):(\d{2})\.(\d{3})").expect("TTML time full regex should be valid")
        });
        let time_short_regex = TTML_TIME_SHORT_REGEX.get_or_init(|| {
            Regex::new(r"(\d{2}):(\d{2})\.(\d{3})").expect("TTML time short regex should be valid")
        });
        
        let parse_time = |time_str: &str| -> u64 {
            // 支持格式: 00:00:01.000 或 1.000s 或 1000ms
            if let Some(caps) = time_full_regex.captures(time_str) {
                let hours: u64 = caps.get(1).and_then(|m| m.as_str().parse().ok()).unwrap_or(0);
                let minutes: u64 = caps.get(2).and_then(|m| m.as_str().parse().ok()).unwrap_or(0);
                let seconds: u64 = caps.get(3).and_then(|m| m.as_str().parse().ok()).unwrap_or(0);
                let millis: u64 = caps.get(4).and_then(|m| m.as_str().parse().ok()).unwrap_or(0);
                return hours * 3600 * 1000 + minutes * 60 * 1000 + seconds * 1000 + millis;
            }

            if let Some(caps) = time_short_regex.captures(time_str) {
                let minutes: u64 = caps.get(1).and_then(|m| m.as_str().parse().ok()).unwrap_or(0);
                let seconds: u64 = caps.get(2).and_then(|m| m.as_str().parse().ok()).unwrap_or(0);
                let millis: u64 = caps.get(3).and_then(|m| m.as_str().parse().ok()).unwrap_or(0);
                return minutes * 60 * 1000 + seconds * 1000 + millis;
            }

            // 尝试解析纯毫秒或秒
            if let Ok(ms) = time_str.trim_end_matches("ms").parse::<u64>() {
                return ms;
            }

            if let Ok(s) = time_str.trim_end_matches('s').parse::<f64>() {
                return (s * 1000.0) as u64;
            }

            0
        };

        // 使用简单的正则表达式提取 p 标签
        let p_regex = TTML_P_REGEX.get_or_init(|| {
            Regex::new(r#"<p[^>]*begin=["']([^"']+)["'][^>]*end=["']([^"']+)["'][^>]*>(.*?)</p>"#)
                .expect("TTML p regex should be valid")
        });
        let html_tag_regex = TTML_HTML_TAG_REGEX.get_or_init(|| {
            Regex::new(r"<[^>]+>").expect("HTML tag regex should be valid")
        });

        for caps in p_regex.captures_iter(content) {
            let begin = caps.get(1).map(|m| m.as_str()).unwrap_or("0");
            let end = caps.get(2).map(|m| m.as_str()).unwrap_or("0");
            let text = caps.get(3).map(|m| m.as_str()).unwrap_or("");

            let start_time = parse_time(begin);
            let end_time = parse_time(end);
            let duration = if end_time > start_time {
                end_time - start_time
            } else {
                DEFAULT_LINE_DURATION_MS
            };

            let mut line = LyricLine::new(start_time, duration);

            // 移除 HTML 标签
            let clean_text = html_tag_regex.replace_all(text, "");
            line.add_word(LyricWord::new(start_time, duration, clean_text.to_string()));

            parsed.add_line(line);
        }

        // 如果没有解析到 p 标签，尝试解析 span 标签（逐字歌词）
        if parsed.lines.is_empty() {
            let span_regex = TTML_SPAN_REGEX.get_or_init(|| {
                Regex::new(r#"<span[^>]*begin=["']([^"']+)["'][^>]*end=["']([^"']+)["'][^>]*>(.*?)</span>"#)
                    .expect("TTML span regex should be valid")
            });

            let mut current_line_start: Option<u64> = None;
            let mut current_line_words: Vec<LyricWord> = Vec::new();

            for caps in span_regex.captures_iter(content) {
                let begin = caps.get(1).map(|m| m.as_str()).unwrap_or("0");
                let end = caps.get(2).map(|m| m.as_str()).unwrap_or("0");
                let text = caps.get(3).map(|m| m.as_str()).unwrap_or("");

                let word_start = parse_time(begin);
                let word_end = parse_time(end);
                let word_duration = if word_end > word_start {
                    word_end - word_start
                } else {
                    DEFAULT_WORD_DURATION_MS
                };

                // 移除 HTML 标签
                let clean_text = html_tag_regex.replace_all(text, "");

                // 检查是否是新行的开始（时间间隔较大）
                if let Some(line_start) = current_line_start {
                    if word_start > line_start + NEW_LINE_THRESHOLD_MS && !current_line_words.is_empty() {
                        // 超过10秒，认为是新行
                        let line_duration = current_line_words
                            .last()
                            .map(|w| w.start_time + w.duration - line_start)
                            .unwrap_or(DEFAULT_LINE_DURATION_MS);
                        let mut line = LyricLine::new(line_start, line_duration);
                        line.words = std::mem::take(&mut current_line_words);
                        parsed.add_line(line);
                        current_line_start = Some(word_start);
                    }
                } else {
                    current_line_start = Some(word_start);
                }

                current_line_words.push(LyricWord::new(
                    word_start,
                    word_duration,
                    clean_text.to_string(),
                ));
            }

            // 添加最后一行
            if !current_line_words.is_empty() {
                if let Some(line_start) = current_line_start {
                    let line_duration = current_line_words
                        .last()
                        .map(|w| w.start_time + w.duration - line_start)
                        .unwrap_or(DEFAULT_LINE_DURATION_MS);
                    let mut line = LyricLine::new(line_start, line_duration);
                    line.words = current_line_words;
                    parsed.add_line(line);
                }
            }
        }

        parsed.sort_lines();
        Ok(parsed)
    }
}

impl Default for LyricParser {
    fn default() -> Self {
        Self::new()
    }
}

/// 解析 LRC 时间戳 [mm:ss.xx]
fn parse_lrc_timestamp(timestamp: &str) -> Option<u64> {
    let clean = timestamp.trim_start_matches('[').trim_end_matches(']');
    let parts: Vec<&str> = clean.split(&[':', '.']).collect();

    if parts.len() >= 2 {
        let minutes = parts[0].parse::<u64>().ok()?;
        let seconds = parts[1].parse::<f64>().ok()?;
        let mut millis = minutes * 60 * 1000 + (seconds * 1000.0) as u64;

        if parts.len() >= 3 {
            let ms_part = parts[2].parse::<u64>().ok()?;
            millis += if parts[2].len() == 2 {
                ms_part * 10
            } else {
                ms_part
            };
        }

        return Some(millis);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lyric_format_detection() {
        let lrc_content = "[ti:Test Song]\n[ar:Test Artist]\n[00:00.00]Lyric line";
        assert_eq!(LyricFormat::from_content(lrc_content), LyricFormat::Lrc);

        let yrc_content = "[ver:v1]\n[by:网易云]\n[0,1000]<0,200>Test";
        assert_eq!(LyricFormat::from_content(yrc_content), LyricFormat::Yrc);

        let ttml_content = r#"<?xml version="1.0"?><tt><p begin="00:00:01">Test</p></tt>"#;
        assert_eq!(LyricFormat::from_content(ttml_content), LyricFormat::Ttml);
    }

    #[test]
    fn test_lrc_parsing() {
        let content = r#"[ti:Test Song]
[ar:Test Artist]
[00:00.00]First line
[00:05.50]Second line"#;

        let parser = LyricParser::new();
        let result = parser.parse(content, LyricFormat::Lrc).unwrap();

        assert_eq!(result.metadata.title, Some("Test Song".to_string()));
        assert_eq!(result.metadata.artist, Some("Test Artist".to_string()));
        assert_eq!(result.lines.len(), 2);
        assert_eq!(result.lines[0].start_time, 0);
        assert_eq!(result.lines[1].start_time, 5500);
    }

    #[test]
    fn test_find_line_by_time() {
        let mut parsed = ParsedLyric::new();
        
        let mut line1 = LyricLine::new(0, 5000);
        line1.add_word(LyricWord::new(0, 5000, "First".to_string()));
        parsed.add_line(line1);
        
        let mut line2 = LyricLine::new(5000, 5000);
        line2.add_word(LyricWord::new(5000, 5000, "Second".to_string()));
        parsed.add_line(line2);

        assert!(parsed.find_line_by_time(0).is_some());
        assert_eq!(parsed.find_line_by_time(0).unwrap().text(), "First");
        assert_eq!(parsed.find_line_by_time(6000).unwrap().text(), "Second");
    }

    #[test]
    fn test_find_current_line_index() {
        let mut parsed = ParsedLyric::new();
        
        let mut line1 = LyricLine::new(0, 5000);
        line1.add_word(LyricWord::new(0, 5000, "First".to_string()));
        parsed.add_line(line1);
        
        let mut line2 = LyricLine::new(5000, 5000);
        line2.add_word(LyricWord::new(5000, 5000, "Second".to_string()));
        parsed.add_line(line2);

        assert_eq!(parsed.find_current_line_index(0), Some(0));
        assert_eq!(parsed.find_current_line_index(6000), Some(1));
    }
}
