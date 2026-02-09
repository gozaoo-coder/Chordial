//! WebDAV 客户端模块
//!
//! 提供 WebDAV 服务器连接、文件列表和元数据获取功能

use reqwest::blocking::{Client, Response};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error;

/// WebDAV 错误类型
#[derive(Error, Debug, Clone)]
pub enum WebDavError {
    #[error("网络请求失败: {0}")]
    RequestError(String),
    #[error("解析响应失败: {0}")]
    ParseError(String),
    #[error("认证失败: {0}")]
    AuthError(String),
    #[error("文件未找到: {0}")]
    NotFound(String),
    #[error("无效的 URL: {0}")]
    InvalidUrl(String),
    #[error("IO 错误: {0}")]
    IoError(String),
}

impl From<reqwest::Error> for WebDavError {
    fn from(e: reqwest::Error) -> Self {
        WebDavError::RequestError(e.to_string())
    }
}

impl From<std::io::Error> for WebDavError {
    fn from(e: std::io::Error) -> Self {
        WebDavError::IoError(e.to_string())
    }
}

impl From<quick_xml::Error> for WebDavError {
    fn from(e: quick_xml::Error) -> Self {
        WebDavError::ParseError(e.to_string())
    }
}

/// WebDAV 文件/目录项
#[derive(Debug, Clone)]
pub struct DavItem {
    pub path: String,
    pub is_collection: bool,
    pub size: u64,
    pub modified: Option<String>,
    pub content_type: Option<String>,
}

impl DavItem {
    pub fn is_audio_file(&self) -> bool {
        if self.is_collection {
            return false;
        }
        let path_lower = self.path.to_lowercase();
        path_lower.ends_with(".mp3")
            || path_lower.ends_with(".flac")
            || path_lower.ends_with(".m4a")
            || path_lower.ends_with(".ogg")
            || path_lower.ends_with(".wav")
            || path_lower.ends_with(".wma")
            || path_lower.ends_with(".aac")
    }

    pub fn file_name(&self) -> String {
        PathBuf::from(&self.path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| self.path.clone())
    }
}

/// WebDAV 客户端
pub struct WebDavClient {
    client: Client,
    base_url: String,
    username: Option<String>,
    password: Option<String>,
}

impl WebDavClient {
    /// 创建新的 WebDAV 客户端
    pub fn new(base_url: &str) -> Result<Self, WebDavError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        let normalized_url = if base_url.ends_with('/') {
            base_url.to_string()
        } else {
            format!("{}/", base_url)
        };

        Ok(Self {
            client,
            base_url: normalized_url,
            username: None,
            password: None,
        })
    }

    /// 设置认证信息
    pub fn with_auth(mut self, username: &str, password: &str) -> Self {
        self.username = Some(username.to_string());
        self.password = Some(password.to_string());
        self
    }

    /// 构建完整 URL
    fn build_url(&self, path: &str) -> String {
        let clean_path = if path.starts_with('/') {
            &path[1..]
        } else {
            path
        };
        format!("{}{}", self.base_url, clean_path)
    }

    /// 发送 PROPFIND 请求
    fn propfind(&self, path: &str, depth: &str) -> Result<Response, WebDavError> {
        let url = self.build_url(path);
        let body = r#"<?xml version="1.0" encoding="utf-8"?>
<D:propfind xmlns:D="DAV:">
    <D:prop>
        <D:displayname/>
        <D:getcontentlength/>
        <D:getlastmodified/>
        <D:getcontenttype/>
        <D:resourcetype/>
    </D:prop>
</D:propfind>"#;

        let mut request = self
            .client
            .request(reqwest::Method::from_bytes(b"PROPFIND").unwrap(), &url)
            .header("Content-Type", "application/xml")
            .header("Depth", depth)
            .body(body);

        if let (Some(user), Some(pass)) = (&self.username, &self.password) {
            request = request.basic_auth(user, Some(pass));
        }

        let response = request.send()?;

        match response.status().as_u16() {
            200 | 207 => Ok(response),
            401 => Err(WebDavError::AuthError("认证失败，请检查用户名和密码".to_string())),
            404 => Err(WebDavError::NotFound(format!("路径不存在: {}", path))),
            status => Err(WebDavError::RequestError(format!("HTTP {}", status))),
        }
    }

    /// 列出目录内容
    pub fn list_directory(&self, path: &str) -> Result<Vec<DavItem>, WebDavError> {
        let response = self.propfind(path, "1")?;
        let text = response.text()?;
        self.parse_propfind_response(&text)
    }

    /// 递归列出所有文件
    pub fn list_all_files(&self, path: &str) -> Result<Vec<DavItem>, WebDavError> {
        let response = self.propfind(path, "infinity")?;
        let text = response.text()?;
        let items = self.parse_propfind_response(&text)?;
        // 过滤出音频文件
        Ok(items.into_iter().filter(|i| i.is_audio_file()).collect())
    }

    /// 解析 PROPFIND 响应
    fn parse_propfind_response(&self, xml: &str) -> Result<Vec<DavItem>, WebDavError> {
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);

        let mut items = Vec::new();
        let mut current_item: Option<DavItem> = None;
        let mut current_tag = String::new();
        let mut in_prop = false;

        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    match name.as_str() {
                        "D:response" | "d:response" | "response" => {
                            current_item = Some(DavItem {
                                path: String::new(),
                                is_collection: false,
                                size: 0,
                                modified: None,
                                content_type: None,
                            });
                        }
                        "D:prop" | "d:prop" | "prop" => {
                            in_prop = true;
                        }
                        _ => {
                            if in_prop {
                                current_tag = name;
                            }
                        }
                    }
                }
                Ok(Event::Text(e)) => {
                    if let Some(ref mut item) = current_item {
                        let text = e.unescape().unwrap_or_default().to_string();
                        match current_tag.as_str() {
                            "D:href" | "d:href" | "href" => {
                                // 提取相对路径
                                let href = text;
                                let base_path = url::Url::parse(&self.base_url)
                                    .map(|u| u.path().to_string())
                                    .unwrap_or_default();
                                item.path = if href.starts_with(&base_path) {
                                    href[base_path.len()..].to_string()
                                } else {
                                    href
                                };
                            }
                            "D:getcontentlength" | "d:getcontentlength" | "getcontentlength" => {
                                item.size = text.parse().unwrap_or(0);
                            }
                            "D:getlastmodified" | "d:getlastmodified" | "getlastmodified" => {
                                item.modified = Some(text);
                            }
                            "D:getcontenttype" | "d:getcontenttype" | "getcontenttype" => {
                                item.content_type = Some(text);
                            }
                            _ => {}
                        }
                    }
                }
                Ok(Event::Empty(e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if let Some(ref mut item) = current_item {
                        if name == "D:collection" || name == "d:collection" || name == "collection" {
                            item.is_collection = true;
                        }
                    }
                }
                Ok(Event::End(e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    match name.as_str() {
                        "D:response" | "d:response" | "response" => {
                            if let Some(item) = current_item.take() {
                                // 跳过根目录本身
                                if !item.path.is_empty() && item.path != "/" {
                                    items.push(item);
                                }
                            }
                        }
                        "D:prop" | "d:prop" | "prop" => {
                            in_prop = false;
                            current_tag.clear();
                        }
                        _ => {
                            current_tag.clear();
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(WebDavError::ParseError(e.to_string())),
                _ => {}
            }
            buf.clear();
        }

        Ok(items)
    }

    /// 下载文件内容（用于获取元数据）
    pub fn download_file(&self, path: &str) -> Result<Vec<u8>, WebDavError> {
        let url = self.build_url(path);
        let mut request = self.client.get(&url);

        if let (Some(user), Some(pass)) = (&self.username, &self.password) {
            request = request.basic_auth(user, Some(pass));
        }

        let response = request.send()?;

        match response.status().as_u16() {
            200 => Ok(response.bytes()?.to_vec()),
            401 => Err(WebDavError::AuthError("认证失败".to_string())),
            404 => Err(WebDavError::NotFound(format!("文件不存在: {}", path))),
            status => Err(WebDavError::RequestError(format!("HTTP {}", status))),
        }
    }

    /// 测试连接
    pub fn test_connection(&self) -> Result<(), WebDavError> {
        self.propfind("", "0")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dav_item_is_audio() {
        let audio_item = DavItem {
            path: "/music/song.mp3".to_string(),
            is_collection: false,
            size: 1000,
            modified: None,
            content_type: None,
        };
        assert!(audio_item.is_audio_file());

        let dir_item = DavItem {
            path: "/music/".to_string(),
            is_collection: true,
            size: 0,
            modified: None,
            content_type: None,
        };
        assert!(!dir_item.is_audio_file());

        let txt_item = DavItem {
            path: "/music/readme.txt".to_string(),
            is_collection: false,
            size: 100,
            modified: None,
            content_type: None,
        };
        assert!(!txt_item.is_audio_file());
    }
}
