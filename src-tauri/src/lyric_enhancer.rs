//! 高级歌词处理模块
//!
//! 提供从多种格式解析歌词并增强现有音频元数据的功能

use crate::lyric_parser::{
    LyricParser, ParsedLyric, LyricFormat, LyricLine as ParsedLyricLine,
    LyricMetadata as ParsedLyricMetadata, ParseError
};
use crate::audio_metadata::AudioMetadata;
use crate::audio_metadata::utils::encoding::auto_decode_text;
use std::time::Duration;

/// 增强音频元数据的歌词信息
pub struct EnhancedLyrics {
    /// 解析后的歌词数据
    pub parsed_lyric: Option<ParsedLyric>,
    /// 原始歌词内容
    pub raw_content: Option<String>,
    /// 歌词格式
    pub format: Option<LyricFormat>,
}

impl EnhancedLyrics {
    pub fn new() -> Self {
        Self {
            parsed_lyric: None,
            raw_content: None,
            format: None,
        }
    }

    /// 从字符串内容解析歌词
    pub fn parse_from_string(&mut self, content: String, format: Option<LyricFormat>) -> Result<(), ParseError> {
        self.raw_content = Some(content.clone());

        let parser = LyricParser::new();
        let detected_format = if let Some(fmt) = format {
            fmt
        } else {
            LyricFormat::from_content(&content)
        };

        self.format = Some(detected_format.clone());
        self.parsed_lyric = Some(parser.parse(&content, detected_format)?);

        Ok(())
    }

    /// 将解析的歌词转换为标准 LyricLine 格式
    pub fn to_standard_lyrics(&self) -> Option<Vec<crate::audio_metadata::LyricLine>> {
        self.parsed_lyric.as_ref().map(|parsed| {
            parsed.lines.iter().map(|line| {
                // 将整行文本合并
                let full_text = if line.words.is_empty() {
                    String::new()
                } else if line.words.len() == 1 {
                    line.words[0].word.clone()
                } else {
                    line.words.iter()
                        .map(|w| w.word.as_str())
                        .collect::<Vec<&str>>()
                        .join("")
                };

                crate::audio_metadata::LyricLine::new(
                    Duration::from_millis(line.start_time),
                    full_text
                )
            }).collect()
        })
    }

    /// 检查是否为同步歌词
    pub fn is_synced(&self) -> bool {
        self.parsed_lyric.as_ref()
            .map(|parsed| !parsed.lines.is_empty() && parsed.lines.iter().any(|line| line.words.len() > 0))
            .unwrap_or(false)
    }

    /// 获取歌词元数据
    pub fn get_metadata(&self) -> Option<ParsedLyricMetadata> {
        self.parsed_lyric.as_ref().map(|p| p.metadata.clone())
    }

    /// 获取当前时间的歌词行
    pub fn get_current_line(&self, time_ms: u64) -> Option<&ParsedLyricLine> {
        self.parsed_lyric.as_ref()?.find_line_by_time(time_ms)
    }

    /// 获取当前歌词行的索引
    pub fn get_current_line_index(&self, time_ms: u64) -> Option<usize> {
        self.parsed_lyric.as_ref()?.find_current_line_index(time_ms)
    }
}

impl Default for EnhancedLyrics {
    fn default() -> Self {
        Self::new()
    }
}

/// 增强音频元数据，添加高级歌词支持
pub fn enhance_metadata_with_lyrics(metadata: &mut AudioMetadata, lyric_content: Option<String>) {
    if let Some(content) = lyric_content {
        let mut enhanced = EnhancedLyrics::new();

        // 尝试解析歌词内容
        match enhanced.parse_from_string(content.clone(), None) {
            Ok(_) => {
                // 如果成功解析为同步歌词，更新元数据
                if enhanced.is_synced() {
                    if let Some(standard_lyrics) = enhanced.to_standard_lyrics() {
                        metadata.synced_lyrics = Some(standard_lyrics);
                    }
                }

                // 更新元数据中的歌词信息
                if let Some(lyric_metadata) = enhanced.get_metadata() {
                    // 如果音频元数据中没有这些信息，可以从歌词元数据中补充
                    if metadata.title.is_none() && lyric_metadata.title.is_some() {
                        metadata.title = lyric_metadata.title;
                    }
                    if metadata.artist.is_none() && lyric_metadata.artist.is_some() {
                        metadata.artist = lyric_metadata.artist;
                    }
                    if metadata.album.is_none() && lyric_metadata.album.is_some() {
                        metadata.album = lyric_metadata.album;
                    }
                }
            }
            Err(_) => {
                // 解析失败，保留原始歌词内容
                metadata.lyrics = Some(content);
            }
        }
    }
}

/// 从文件路径尝试读取歌词文件（支持自动编码检测）
pub fn find_lyric_file(audio_file_path: &std::path::Path) -> Option<String> {
    let file_stem = audio_file_path.file_stem()?;
    let parent_dir = audio_file_path.parent()?;

    // 尝试不同的歌词文件扩展名
    let lyric_extensions = ["lrc", "yrc", "qrc", "txt"];

    for ext in &lyric_extensions {
        let lyric_path = parent_dir.join(format!("{}.{}", file_stem.to_str()?, ext));
        if lyric_path.exists() {
            // 读取歌词文件并使用自动编码检测解码
            if let Ok(bytes) = std::fs::read(&lyric_path) {
                // 跳过BOM头（如果存在）
                let data_to_decode = if bytes.len() >= 3 && &bytes[0..3] == b"\xEF\xBB\xBF" {
                    &bytes[3..]
                } else {
                    &bytes[..]
                };

                // 使用自动编码检测解码
                match auto_decode_text(data_to_decode) {
                    Ok(content) => {
                        log::debug!("成功读取歌词文件: {:?}, 自动检测编码", lyric_path);
                        return Some(content);
                    }
                    Err(e) => {
                        log::warn!("歌词文件解码失败: {:?}, 错误: {}", lyric_path, e);
                        // 如果自动解码失败，使用有损解码作为后备
                        return Some(String::from_utf8_lossy(data_to_decode).into_owned());
                    }
                }
            }
        }
    }

    // 尝试同名的歌词文件（不区分大小写）
    if let Ok(entries) = std::fs::read_dir(parent_dir) {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_file() {
                    if let Some(file_name) = entry.file_name().to_str() {
                        let file_name_lower = file_name.to_lowercase();
                        let stem_lower = file_stem.to_str()?.to_lowercase();

                        if file_name_lower.starts_with(&stem_lower) {
                            for ext in &lyric_extensions {
                                if file_name_lower.ends_with(ext) {
                                    // 读取歌词文件并使用自动编码检测解码
                                    if let Ok(bytes) = std::fs::read(entry.path()) {
                                        // 跳过BOM头（如果存在）
                                        let data_to_decode = if bytes.len() >= 3 && &bytes[0..3] == b"\xEF\xBB\xBF" {
                                            &bytes[3..]
                                        } else {
                                            &bytes[..]
                                        };

                                        // 使用自动编码检测解码
                                        match auto_decode_text(data_to_decode) {
                                            Ok(content) => {
                                                log::debug!("成功读取歌词文件: {:?}, 自动检测编码", entry.path());
                                                return Some(content);
                                            }
                                            Err(e) => {
                                                log::warn!("歌词文件解码失败: {:?}, 错误: {}", entry.path(), e);
                                                // 如果自动解码失败，使用有损解码作为后备
                                                return Some(String::from_utf8_lossy(data_to_decode).into_owned());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

/// 歌词处理工具函数
pub mod utils {

    /// 格式化时间戳为 LRC 格式
    pub fn format_timestamp_lrc(milliseconds: u64) -> String {
        let seconds = milliseconds / 1000;
        let minutes = seconds / 60;
        let remaining_seconds = seconds % 60;
        let remaining_millis = milliseconds % 1000;

        format!("[{:02}:{:02}.{:03}]", minutes, remaining_seconds, remaining_millis)
    }

    /// 从 LRC 格式解析时间戳
    pub fn parse_lrc_timestamp(timestamp: &str) -> Option<u64> {
        // 移除方括号
        let clean_timestamp = timestamp.trim_start_matches('[').trim_end_matches(']');

        // 解析 mm:ss.xx 格式
        let parts: Vec<&str> = clean_timestamp.split(&[':', '.']).collect();

        if parts.len() >= 2 {
            if let (Ok(minutes), Ok(seconds)) = (parts[0].parse::<u64>(), parts[1].parse::<f64>()) {
                let mut milliseconds = minutes * 60 * 1000 + (seconds * 1000.0) as u64;

                if parts.len() >= 3 {
                    if let Ok(ms_part) = parts[2].parse::<u64>() {
                        milliseconds += if parts[2].len() == 2 {
                            ms_part * 10  // 百分之一秒
                        } else {
                            ms_part  // 毫秒
                        };
                    }
                }

                return Some(milliseconds);
            }
        }

        None
    }

    /// 清理歌词文本
    pub fn clean_lyric_text(text: &str) -> String {
        text.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .filter(|line| !line.starts_with('[') || !line.contains(':')) // 过滤元数据行
            .collect::<Vec<&str>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
