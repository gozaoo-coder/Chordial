//! 音频文件扫描 — 使用 symphonia 读取元数据。
//!
//! 对单个音频文件进行探测（probe），提取格式、标签等元信息，
//! 不进行完整解码，速度较快。

use std::path::Path;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::{FormatOptions, TrackType};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::{MetadataOptions, StandardTag};

/// 从音频文件中提取的元数据。
#[derive(Debug, Clone, Default)]
pub struct AudioMeta {
    /// 歌曲标题（来自 ID3 / Vorbis comment / MP4 等标签）
    pub title: Option<String>,
    /// 艺术家名称
    pub artist: Option<String>,
    /// 专辑名称
    pub album: Option<String>,
    /// 时长（秒）
    pub duration_secs: Option<u64>,
    /// 采样率（Hz）
    pub sample_rate: Option<u32>,
    /// 声道数
    pub channels: Option<u8>,
    /// 容器格式名称（如 "FLAC", "MP3", "MP4"）
    pub format_name: Option<String>,
}

/// 探测音频文件，提取元数据。
///
/// # 参数
/// - `path`: 音频文件的文件系统路径。
///
/// # 返回
/// 成功时返回 [`AudioMeta`]，失败时返回错误信息。
pub fn probe_file(path: &Path) -> Result<AudioMeta, String> {
    let src = std::fs::File::open(path)
        .map_err(|e| format!("无法打开文件 '{}': {}", path.display(), e))?;

    let mss = MediaSourceStream::new(Box::new(src), Default::default());

    // 创建 Hint 以加速格式探测
    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let mut format = symphonia::default::get_probe()
        .probe(
            &hint,
            mss,
            FormatOptions::default(),
            MetadataOptions::default(),
        )
        .map_err(|e| format!("无法识别音频格式 '{}': {}", path.display(), e))?;

    let mut meta = AudioMeta::default();

    // 容器格式信息
    let fi = format.format_info();
    meta.format_name = Some(fi.short_name.to_string());

    // Track 信息（采样率、声道）
    if let Some(track) = format.default_track(TrackType::Audio) {
        if let Some(params) = &track.codec_params {
            if let Some(audio_params) = params.audio() {
                meta.sample_rate = audio_params.sample_rate;
                meta.channels = audio_params.channels.as_ref().map(|c| c.count() as u8);
            }
        }
    }

    // 标签（标题、艺术家、专辑）
    // 消费所有旧版本，只留最新
    loop {
        let is_latest = format.metadata().is_latest();
        if is_latest {
            break;
        }
        format.metadata().pop();
    }

    if let Some(revision) = format.metadata().current() {
        for tag in &revision.media.tags {
            match &tag.std {
                Some(StandardTag::TrackTitle(title)) => {
                    meta.title = Some(title.to_string());
                }
                Some(StandardTag::Artist(artist)) => {
                    meta.artist = Some(artist.to_string());
                }
                Some(StandardTag::Album(album)) => {
                    meta.album = Some(album.to_string());
                }
                _ => {}
            }
        }
    }

    // 若标签中无标题，回退到文件名（不含扩展名）
    if meta.title.is_none() {
        meta.title = path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string());
    }

    Ok(meta)
}

/// 从音频文件中提取嵌入封面图片。
///
/// 使用 symphonia 读取 FLAC/Vorbis comments 或 ID3v2 中的封面数据。
/// 优先返回 front cover，其次返回任何图片。
///
/// # 参数
/// - `path`: 音频文件的文件系统路径。
///
/// # 返回
/// 成功时返回图片字节数据（JPEG / PNG），失败时返回错误信息。
pub fn extract_cover_art(path: &Path) -> Result<Vec<u8>, String> {
    let src = std::fs::File::open(path)
        .map_err(|e| format!("无法打开文件 '{}': {}", path.display(), e))?;

    let mss = MediaSourceStream::new(Box::new(src), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let mut format = symphonia::default::get_probe()
        .probe(
            &hint,
            mss,
            FormatOptions::default(),
            MetadataOptions::default(),
        )
        .map_err(|e| format!("无法识别音频格式: {}", e))?;

    // 消费所有旧修订，留最新的
    loop {
        let is_latest = format.metadata().is_latest();
        if is_latest {
            break;
        }
        format.metadata().pop();
    }

    if let Some(revision) = format.metadata().current() {
        // 优先返回 front cover
        let mut first_image: Option<Vec<u8>> = None;

        for visual in &revision.media.visuals {
            // media_type 可能为 None，继续下一个
            let media_type = match &visual.media_type {
                Some(t) => t,
                None => continue,
            };
            if !media_type.starts_with("image/") {
                continue;
            }
            let data = visual.data.to_vec();
            // FrontCover 优先
            if let Some(ref usage) = visual.usage {
                if usage == &symphonia::core::meta::StandardVisualKey::FrontCover {
                    return Ok(data);
                }
            }
            if first_image.is_none() {
                first_image = Some(data);
            }
        }

        if let Some(data) = first_image {
            return Ok(data);
        }
    }

    Err("音频文件中无嵌入封面".to_string())
}

/// 检查文件是否为 symphonia 支持的音频格式。
///
/// 通过扩展名快速过滤。
pub fn is_supported_audio(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        matches!(
            ext.to_lowercase().as_str(),
            "mp3"
                | "flac"
                | "wav"
                | "ogg"
                | "oga"
                | "opus"
                | "m4a"
                | "aac"
                | "wma"
                | "aiff"
                | "aif"
                | "caf"
        )
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_filter() {
        assert!(is_supported_audio(Path::new("song.mp3")));
        assert!(is_supported_audio(Path::new("track.FLAC")));
        assert!(is_supported_audio(Path::new("audio.ogg")));
        assert!(!is_supported_audio(Path::new("cover.jpg")));
        assert!(!is_supported_audio(Path::new("lyrics.lrc")));
        assert!(!is_supported_audio(Path::new("readme.txt")));
    }
}
