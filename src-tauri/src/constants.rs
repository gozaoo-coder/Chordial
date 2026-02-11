//! 常量定义

// ==================== 音频引擎常量 ====================

/// 默认采样率 (Hz)
pub const DEFAULT_SAMPLE_RATE: u32 = 48000;

/// 默认声道数
pub const DEFAULT_CHANNELS: u16 = 2;

/// 默认交叉淡化持续时间（秒）
pub const DEFAULT_CROSSFADE_DURATION: f32 = 10.0;

/// 音频线程休眠时间（微秒）
pub const AUDIO_THREAD_SLEEP_MICROS: u64 = 100;

/// 音频缓冲区大小（样本数）
pub const AUDIO_BUFFER_SIZE: usize = 1024;

/// BPM 检测窗口大小（基于 48kHz 采样率，5秒音频）
pub const BPM_DETECTION_WINDOW_SIZE: usize = (48000 * 5) / 1024;

/// 默认 BPM 值
pub const DEFAULT_BPM: f32 = 10.0;

// ==================== FFT/分析常量 ====================

/// FFT 大小
pub const FFT_SIZE: usize = 2048;

/// FFT hop 大小
pub const FFT_HOP_SIZE: usize = 512;

/// 频率带数量
pub const FREQUENCY_BANDS: usize = 4;

// ==================== 扫描器常量 ====================

/// 并行扫描任务数
pub const PARALLEL_SCAN_TASKS: usize = 4;

/// 扫描进度报告间隔（文件数）
pub const SCAN_PROGRESS_INTERVAL: usize = 10;

// ==================== 缓存常量 ====================

/// 默认歌词行持续时间（毫秒）
pub const DEFAULT_LYRIC_LINE_DURATION_MS: u64 = 5000;

/// 默认歌词词持续时间（毫秒）
pub const DEFAULT_LYRIC_WORD_DURATION_MS: u64 = 1000;

/// 最小歌词词持续时间（毫秒）
pub const MIN_LYRIC_WORD_DURATION_MS: u64 = 200;

// ==================== 时间拉伸常量 ====================

/// Sinc 插值窗口大小
pub const SINC_WINDOW_SIZE: usize = 64;

/// Sinc 插值过采样率
pub const SINC_OVERSAMPLING: usize = 128;

/// 时间拉伸最大拉伸因子
pub const MAX_STRETCH_FACTOR: f64 = 4.0;

/// 时间拉伸阈值（低于此值不处理）
pub const STRETCH_THRESHOLD: f64 = 0.03;

// ==================== 图片/媒体常量 ====================

/// 默认透明 PNG 图片数据（1x1 像素）
pub const DEFAULT_TRANSPARENT_PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
    0x00, 0x00, 0x00, 0x0D, // IHDR chunk length
    0x49, 0x48, 0x44, 0x52, // IHDR
    0x00, 0x00, 0x00, 0x01, // width: 1
    0x00, 0x00, 0x00, 0x01, // height: 1
    0x08, 0x06, 0x00, 0x00, 0x00, // 8-bit RGBA
    0x1F, 0x15, 0xC4, 0x89, // IHDR CRC
    0x00, 0x00, 0x00, 0x0A, // IDAT chunk length
    0x49, 0x44, 0x41, 0x54, // IDAT
    0x78, 0x9C, 0x63, 0x60, 0x00, 0x00, 0x00, 0x02, 0x00, 0x01,
    0xE2, 0x21, 0xBC, 0x33, // IDAT CRC
    0x00, 0x00, 0x00, 0x00, // IEND chunk length
    0x49, 0x45, 0x4E, 0x44, // IEND
    0xAE, 0x42, 0x60, 0x82, // IEND CRC
];

/// 默认艺术家图片路径
pub const DEFAULT_ARTIST_IMAGE_PATH: &str = "artist.jpg";

/// 支持的音频文件扩展名
pub const SUPPORTED_AUDIO_EXTENSIONS: &[&str] = &["mp3", "flac", "m4a", "ogg", "wav", "opus"];

/// 支持的图片文件扩展名
pub const SUPPORTED_IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "bmp", "webp"];
