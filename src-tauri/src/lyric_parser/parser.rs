use super::types::*;
use std::collections::HashMap;

pub struct LyricParser;

impl LyricParser {
    pub fn new() -> Self {
        Self
    }
    
    pub fn parse(&self, content: &str, format: LyricFormat) -> Result<ParsedLyric, ParseError> {
        match format {
            LyricFormat::Lrc => self.parse_lrc(content),
            LyricFormat::Yrc => self.parse_yrc(content),
            LyricFormat::Qrc => self.parse_qrc(content),
            LyricFormat::Ttml => self.parse_ttml(content),
            LyricFormat::Unknown => Err(ParseError::InvalidFormat("Unknown lyric format".to_string())),
        }
    }
    
    pub fn parse_auto(&self, content: &str) -> Result<ParsedLyric, ParseError> {
        let format = LyricFormat::from_content(content);
        self.parse(content, format)
    }
    
    fn parse_lrc(&self, content: &str) -> Result<ParsedLyric, ParseError> {
        let mut parsed = ParsedLyric::new();
        let mut metadata = LyricMetadata {
            title: None,
            artist: None,
            album: None,
            author: None,
            offset: None,
            length: None,
        };
        
        let mut lines = Vec::new();
        
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            
            // 解析元数据标签 [ti:标题], [ar:艺术家], [al:专辑], [au:作者], [offset:偏移], [length:时长]
            if line.starts_with("[") && line.contains(":") && !line.contains(",") {
                if let Some(end_idx) = line.find(']') {
                    let tag_content = &line[1..end_idx];
                    if let Some(colon_idx) = tag_content.find(':') {
                        let tag = &tag_content[..colon_idx];
                        let value = &tag_content[colon_idx + 1..];
                        
                        match tag {
                            "ti" => metadata.title = Some(value.to_string()),
                            "ar" => metadata.artist = Some(value.to_string()),
                            "al" => metadata.album = Some(value.to_string()),
                            "au" => metadata.author = Some(value.to_string()),
                            "offset" => {
                                if let Ok(offset) = value.parse::<i64>() {
                                    metadata.offset = Some(offset);
                                }
                            },
                            "length" => {
                                if let Ok(length_str) = value.parse::<String>() {
                                    // 解析时长格式 mm:ss
                                    if let Some(colon_pos) = length_str.find(':') {
                                        let mins = &length_str[..colon_pos];
                                        let secs = &length_str[colon_pos + 1..];
                                        if let (Ok(m), Ok(s)) = (mins.parse::<u64>(), secs.parse::<f64>()) {
                                            metadata.length = Some(m * 60 * 1000 + (s * 1000.0) as u64);
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                continue;
            }
            
            // 解析时间戳 [mm:ss.xx] 或 [mm:ss:xx]
            if let Some(time_end) = line.find(']') {
                let time_part = &line[1..time_end];
                let lyric_part = line[time_end + 1..].trim();
                
                if let Ok(timestamp) = self.parse_lrc_timestamp(time_part) {
                    // 创建歌词行
                    let word = LyricWord {
                        start_time: timestamp,
                        end_time: timestamp + 3000, // 默认显示3秒
                        word: lyric_part.to_string(),
                        roman_word: None,
                    };
                    
                    let lyric_line = LyricLine {
                        words: vec![word],
                        translated_lyric: None,
                        roman_lyric: None,
                        is_bg: false,
                        is_duet: false,
                        start_time: timestamp,
                        end_time: timestamp + 3000,
                    };
                    
                    lines.push(lyric_line);
                }
            }
        }
        
        // 更新每行的结束时间
        for i in 0..lines.len() {
            if i < lines.len() - 1 {
                let next_start_time = lines[i + 1].start_time;
                lines[i].end_time = next_start_time;
                if let Some(word) = lines[i].words.get_mut(0) {
                    word.end_time = next_start_time;
                }
            }
        }
        
        // 按时间排序
        lines.sort_by_key(|line| line.start_time);
        
        parsed.lines = lines;
        parsed.metadata = Some(metadata);
        
        Ok(parsed)
    }
    
    fn parse_yrc(&self, content: &str) -> Result<ParsedLyric, ParseError> {
        let mut parsed = ParsedLyric::new();
        let mut lines = Vec::new();
        
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            
            // YRC 格式: [1234,567]逐词歌词(1234,567,0)内容
            if let Some(bracket_end) = line.find(']') {
                let time_part = &line[1..bracket_end];
                let content_part = &line[bracket_end + 1..];
                
                if let Some(comma_idx) = time_part.find(',') {
                    let start_time = &time_part[..comma_idx];
                    let duration = &time_part[comma_idx + 1..];
                    
                    if let (Ok(start), Ok(dur)) = (start_time.parse::<u64>(), duration.parse::<u64>()) {
                        let mut words = Vec::new();
                        let mut current_pos = 0;
                        
                        // 解析逐词内容
                        let mut i = 0;
                        while i < content_part.len() {
                            if content_part.chars().nth(i) == Some('(') {
                                // 查找匹配的右括号
                                if let Some(close_idx) = content_part[i + 1..].find(')') {
                                    let close_idx = i + 1 + close_idx;
                                    let word_time_part = &content_part[i + 1..close_idx];
                                    
                                    if let Some(word_comma_idx) = word_time_part.find(',') {
                                        let word_start = &word_time_part[..word_comma_idx];
                                        let remaining = &word_time_part[word_comma_idx + 1..];
                                        
                                        if let Some(word_comma2_idx) = remaining.find(',') {
                                            let word_dur = &remaining[..word_comma2_idx];
                                            
                                            if let (Ok(w_start), Ok(w_dur)) = (word_start.parse::<u64>(), word_dur.parse::<u64>()) {
                                                // 提取歌词文字
                                                let mut word_text = String::new();
                                                let mut j = close_idx + 1;
                                                while j < content_part.len() && content_part.chars().nth(j) != Some('(') {
                                                    if let Some(ch) = content_part.chars().nth(j) {
                                                        word_text.push(ch);
                                                    }
                                                    j += 1;
                                                }
                                                
                                                let word = LyricWord {
                                                    start_time: start + w_start,
                                                    end_time: start + w_start + w_dur,
                                                    word: word_text.trim().to_string(),
                                                    roman_word: None,
                                                };
                                                
                                                words.push(word);
                                                i = j;
                                                continue;
                                            }
                                        }
                                    }
                                }
                            }
                            i += 1;
                        }
                        
                        // 如果没有解析到逐词，将整个内容作为一个词
                        if words.is_empty() {
                            let word = LyricWord {
                                start_time: start,
                                end_time: start + dur,
                                word: content_part.to_string(),
                                roman_word: None,
                            };
                            words.push(word);
                        }
                        
                        let lyric_line = LyricLine {
                            words,
                            translated_lyric: None,
                            roman_lyric: None,
                            is_bg: false,
                            is_duet: false,
                            start_time: start,
                            end_time: start + dur,
                        };
                        
                        lines.push(lyric_line);
                    }
                }
            }
        }
        
        // 按时间排序
        lines.sort_by_key(|line| line.start_time);
        
        parsed.lines = lines;
        Ok(parsed)
    }
    
    fn parse_qrc(&self, content: &str) -> Result<ParsedLyric, ParseError> {
        // QRC 格式与 YRC 类似但略有不同，这里先简化处理
        // 实际 QRC 格式: [1234,567]Test(1234,567)内容
        self.parse_yrc(content) // 简化处理，实际应该有不同的解析逻辑
    }
    
    fn parse_ttml(&self, content: &str) -> Result<ParsedLyric, ParseError> {
        // TTML 解析较为复杂，这里先返回空结果
        // 后续可以使用 XML 解析库实现
        Ok(ParsedLyric::new())
    }
    
    fn parse_lrc_timestamp(&self, timestamp: &str) -> Result<u64, ParseError> {
        // 解析 [mm:ss.xx] 或 [mm:ss:xx] 格式
        let parts: Vec<&str> = timestamp.split(&[':', '.']).collect();
        
        if parts.len() >= 2 {
            if let (Ok(minutes), Ok(seconds)) = (parts[0].parse::<u64>(), parts[1].parse::<f64>()) {
                let mut milliseconds = minutes * 60 * 1000 + (seconds * 1000.0) as u64;
                
                // 处理毫秒部分
                if parts.len() >= 3 {
                    if let Ok(ms_part) = parts[2].parse::<u64>() {
                        if parts[2].len() == 2 {
                            // 百分之一秒
                            milliseconds += ms_part * 10;
                        } else {
                            // 毫秒
                            milliseconds += ms_part;
                        }
                    }
                }
                
                return Ok(milliseconds);
            }
        }
        
        Err(ParseError::InvalidTimestamp(format!("Invalid timestamp: {}", timestamp)))
    }
}

impl Default for LyricParser {
    fn default() -> Self {
        Self::new()
    }
}