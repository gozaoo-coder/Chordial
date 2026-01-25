use super::super::types::*;

pub struct TtmlParser;

impl TtmlParser {
    pub fn new() -> Self {
        Self
    }
    
    pub fn parse(&self, _content: &str) -> Result<ParsedLyric, ParseError> {
        // TTML 解析较为复杂，这里先返回空结果
        // 后续可以使用 XML 解析库实现
        Ok(ParsedLyric::new())
    }
}

impl Default for TtmlParser {
    fn default() -> Self {
        Self::new()
    }
}