use super::super::types::*;

pub struct LrcParser;

impl LrcParser {
    pub fn new() -> Self {
        Self
    }
    
    pub fn parse_metadata(&self, line: &str) -> Option<(String, String)> {
        if line.starts_with('[') && line.contains(':') && !line.contains(",") {
            if let Some(end_idx) = line.find(']') {
                let tag_content = &line[1..end_idx];
                if let Some(colon_idx) = tag_content.find(':') {
                    let tag = tag_content[..colon_idx].to_string();
                    let value = tag_content[colon_idx + 1..].to_string();
                    return Some((tag, value));
                }
            }
        }
        None
    }
    
    pub fn parse_timestamp(&self, timestamp: &str) -> Result<u64, ParseError> {
        let parts: Vec<&str> = timestamp.split(&[':', '.']).collect();
        
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
                
                return Ok(milliseconds);
            }
        }
        
        Err(ParseError::InvalidTimestamp(format!("Invalid LRC timestamp: {}", timestamp)))
    }
}

impl Default for LrcParser {
    fn default() -> Self {
        Self::new()
    }
}