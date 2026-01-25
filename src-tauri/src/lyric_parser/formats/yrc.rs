use super::super::types::*;

pub struct YrcParser;

impl YrcParser {
    pub fn new() -> Self {
        Self
    }
    
    pub fn parse_line_timestamp(&self, time_part: &str) -> Result<(u64, u64), ParseError> {
        if let Some(comma_idx) = time_part.find(',') {
            let start_time = &time_part[..comma_idx];
            let duration = &time_part[comma_idx + 1..];
            
            if let (Ok(start), Ok(dur)) = (start_time.parse::<u64>(), duration.parse::<u64>()) {
                return Ok((start, dur));
            }
        }
        
        Err(ParseError::InvalidTimestamp(format!("Invalid YRC line timestamp: {}", time_part)))
    }
    
    pub fn parse_word_timestamp(&self, word_time_part: &str) -> Result<(u64, u64), ParseError> {
        // YRC 格式: (开始时间,持续时间,0)
        let parts: Vec<&str> = word_time_part.split(',').collect();
        
        if parts.len() >= 2 {
            if let (Ok(start), Ok(dur)) = (parts[0].parse::<u64>(), parts[1].parse::<u64>()) {
                return Ok((start, dur));
            }
        }
        
        Err(ParseError::InvalidTimestamp(format!("Invalid YRC word timestamp: {}", word_time_part)))
    }
}

impl Default for YrcParser {
    fn default() -> Self {
        Self::new()
    }
}