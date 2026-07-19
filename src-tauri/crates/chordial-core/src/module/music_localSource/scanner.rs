//! 音频文件扫描 — 使用 symphonia 读取元数据。
//!
//! 对单个音频文件进行探测（probe），提取格式、标签等元信息，
//! 不进行完整解码，速度较快。
//!
//! ## 跨平台
//!
//! 通过 [`crate::module::platform`] 适配不同平台的文件访问：
//! - 桌面端：`std::fs::File` → symphonia
//! - Android：`Cursor<Vec<u8>>`（预读全部字节）→ symphonia

use crate::module::platform::{self, PlatformPath};
use crate::module::perf;
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
    /// 发行年份（来自 ID3 TYER/TDRC、Vorbis DATE/YEAR、MP4 ©day 等标签）
    pub year: Option<u32>,
}

/// 探测音频文件，提取元数据。
///
/// # 参数
/// - `path`: 音频文件的文件系统路径。
///
/// # 返回
/// 成功时返回 [`AudioMeta`]，失败时返回错误信息。
pub fn probe_file(path: &PlatformPath) -> Result<AudioMeta, String> {
    let _token = perf::start("scanner.probe_file");
    let src = platform::open_file(path)?;

    let mss = MediaSourceStream::new(Box::new(src), Default::default());

    // 创建 Hint 以加速格式探测
    let mut hint = Hint::new();
    if let Some(ext) = platform::path_extension(path) {
        hint.with_extension(&ext);
    }

    let mut format = symphonia::default::get_probe()
        .probe(
            &hint,
            mss,
            FormatOptions::default(),
            MetadataOptions::default(),
        )
        .map_err(|e| format!("无法识别音频格式 '{}': {}", platform::path_to_string(path), e))?;

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

            // 年份兜底解析：StandardTag 未覆盖或解析失败时，
            // 通过 raw tag key（不区分大小写）匹配常见 year/date 字段名。
            // 覆盖 ID3 (TYER/TDRC/TDRL)、Vorbis (DATE/YEAR)、MP4 (©day) 等。
            if meta.year.is_none() {
                let key_lower = tag.raw.key.to_lowercase();
                let is_year_key = matches!(
                    key_lower.as_str(),
                    "year"
                        | "date"
                        | "tdrc"
                        | "tdrl"
                        | "tory"
                        | "tyer"
                        | "release_date"
                        | "releasedate"
                        | "originaldate"
                        | "©day"
                );
                if is_year_key {
                    if let Some(y) = parse_year_from_value(&tag.raw.value) {
                        meta.year = Some(y);
                    }
                }
            }
        }
    }

    // 若标签中无标题，回退到文件名（不含扩展名）
    if meta.title.is_none() {
        meta.title = platform::path_file_stem(path);
    }

    // 仅在 perf 启用时构建 meta 字符串，避免 release 中无谓 format! 分配
    let meta_str = if perf::enabled() {
        Some(format!("path={}", platform::path_to_string(path)))
    } else {
        None
    };
    perf::end(&_token, meta_str.as_deref());
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
pub fn extract_cover_art(path: &PlatformPath) -> Result<Vec<u8>, String> {
    let _scope = perf::scope("scanner.extract_cover_art");
    let src = platform::open_file(path)?;

    let mss = MediaSourceStream::new(Box::new(src), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = platform::path_extension(path) {
        hint.with_extension(&ext);
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

/// 读取与音频文件同名的歌词文件（`.lrc` 优先，`.txt` 兜底）。
///
/// 给定音频文件路径 `foo.mp3`，依次尝试 `foo.lrc` 和 `foo.txt`。
/// 命中后以 UTF-8 解码；不存在或解码失败时返回 `None`。
///
/// 该函数仅做字节读取，不解析 LRC 时间戳。LRC 解析在前端 `lyricConverter.js` 完成。
pub fn read_lyric_file(path: &PlatformPath) -> Option<String> {
    for ext in &["lrc", "txt"] {
        let lyric_path = platform::path_with_extension(path, ext);
        if !platform::exists(&lyric_path) {
            continue;
        }
        match platform::read_bytes(&lyric_path) {
            Ok(bytes) => match String::from_utf8(bytes) {
                Ok(text) if !text.trim().is_empty() => return Some(text),
                _ => continue,
            },
            Err(_) => continue,
        }
    }
    None
}

/// 检查文件是否为 symphonia 支持的音频格式。
///
/// 通过扩展名快速过滤。
pub fn is_supported_audio(path: &PlatformPath) -> bool {
    if let Some(ext) = platform::path_extension(path) {
        matches!(
            ext.as_str(),
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

/// 从 symphonia `RawValue` 提取合法的年份（1900..=2100）。
///
/// 支持三种形式：
/// - 数字 `RawValue::UnsignedInt(2024)` / `RawValue::SignedInt(2024)` → 直接取值
/// - 字符串 `"2024"`、`"2024-01-01"`、`"2024-05"` → 取前 4 位数字解析
fn parse_year_from_value(value: &symphonia::core::meta::RawValue) -> Option<u32> {
    use symphonia::core::meta::RawValue;
    let raw_year: Option<u32> = match value {
        RawValue::UnsignedInt(n) => Some(*n as u32),
        RawValue::SignedInt(n) if *n > 0 => Some(*n as u32),
        RawValue::String(s) => s
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .take(4)
            .collect::<String>()
            .parse::<u32>()
            .ok(),
        _ => None,
    };
    raw_year.filter(|y| (1900..=2100).contains(y))
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::module::platform::PlatformPath;

    #[test]
    fn test_extension_filter() {
        assert!(is_supported_audio(&PlatformPath::from("song.mp3")));
        assert!(is_supported_audio(&PlatformPath::from("track.FLAC")));
        assert!(is_supported_audio(&PlatformPath::from("audio.ogg")));
        assert!(!is_supported_audio(&PlatformPath::from("cover.jpg")));
        assert!(!is_supported_audio(&PlatformPath::from("lyrics.lrc")));
        assert!(!is_supported_audio(&PlatformPath::from("readme.txt")));
    }
}
