//! 音乐元数据读取器测试模块
//!
//! 包含所有模块的单元测试和集成测试

#[cfg(test)]
mod tests {
    // ==================== audio_metadata 模块测试 ====================
    mod audio_metadata_tests {
        use crate::audio_metadata::{
            AudioFormat, AudioMetadata, Picture, PictureType, LyricLine,
            MetadataError, read_metadata, filter_supported_files,
        };
        use std::path::PathBuf;
        use std::time::Duration;

        #[test]
        fn test_audio_format_from_extension() {
            assert_eq!(AudioFormat::from_extension(PathBuf::from("test.flac").as_path()), AudioFormat::Flac);
            assert_eq!(AudioFormat::from_extension(PathBuf::from("test.FLAC").as_path()), AudioFormat::Flac);
            assert_eq!(AudioFormat::from_extension(PathBuf::from("test.mp3").as_path()), AudioFormat::Mp3);
            assert_eq!(AudioFormat::from_extension(PathBuf::from("test.m4a").as_path()), AudioFormat::M4a);
            assert_eq!(AudioFormat::from_extension(PathBuf::from("test.ogg").as_path()), AudioFormat::Ogg);
            assert_eq!(AudioFormat::from_extension(PathBuf::from("test.wav").as_path()), AudioFormat::Wav);
            assert_eq!(AudioFormat::from_extension(PathBuf::from("test.unknown").as_path()), AudioFormat::Unknown);
        }

        #[test]
        fn test_audio_format_from_magic_bytes() {
            // FLAC: "fLaC" + padding (需要至少12字节)
            let flac_magic = b"fLaCXXXXXXXX";
            assert_eq!(AudioFormat::from_magic_bytes(flac_magic), AudioFormat::Flac);

            // MP3: ID3 标签 + padding
            let mp3_magic = b"ID3XXXXXXXXX";
            assert_eq!(AudioFormat::from_magic_bytes(mp3_magic), AudioFormat::Mp3);

            // OGG: "OggS" + padding
            let ogg_magic = b"OggSXXXXXXXX";
            assert_eq!(AudioFormat::from_magic_bytes(ogg_magic), AudioFormat::Ogg);

            // WAV: "RIFF" + "WAVE"
            let wav_magic = b"RIFFXXXXWAVE";
            assert_eq!(AudioFormat::from_magic_bytes(wav_magic), AudioFormat::Wav);

            // 空数据
            assert_eq!(AudioFormat::from_magic_bytes(&[]), AudioFormat::Unknown);

            // 数据太短
            assert_eq!(AudioFormat::from_magic_bytes(b"fLaC"), AudioFormat::Unknown);
        }

        #[test]
        fn test_audio_format_display() {
            assert_eq!(format!("{}", AudioFormat::Flac), "FLAC");
            assert_eq!(format!("{}", AudioFormat::Mp3), "MP3");
            assert_eq!(format!("{}", AudioFormat::M4a), "M4A");
            assert_eq!(format!("{}", AudioFormat::Ogg), "OGG");
            assert_eq!(format!("{}", AudioFormat::Wav), "WAV");
            assert_eq!(format!("{}", AudioFormat::Unknown), "Unknown");
        }

        #[test]
        fn test_audio_metadata_creation() {
            let metadata = AudioMetadata::new(AudioFormat::Flac);
            assert_eq!(metadata.format, AudioFormat::Flac);
            assert!(metadata.title.is_none());
            assert!(metadata.artist.is_none());
            assert!(metadata.album.is_none());
        }

        #[test]
        fn test_audio_metadata_default() {
            let metadata: AudioMetadata = Default::default();
            assert_eq!(metadata.format, AudioFormat::Unknown);
        }

        #[test]
        fn test_audio_metadata_title_or_filename() {
            let mut metadata = AudioMetadata::new(AudioFormat::Flac);
            let path = PathBuf::from("/music/test_song.flac");

            // 没有标题时返回文件名
            assert_eq!(metadata.title_or_filename(&path), "test_song");

            // 有标题时返回标题
            metadata.title = Some("My Song".to_string());
            assert_eq!(metadata.title_or_filename(&path), "My Song");
        }

        #[test]
        fn test_audio_metadata_artist_or_unknown() {
            let mut metadata = AudioMetadata::new(AudioFormat::Flac);
            assert_eq!(metadata.artist_or_unknown(), "Unknown Artist");

            metadata.artist = Some("Test Artist".to_string());
            assert_eq!(metadata.artist_or_unknown(), "Test Artist");
        }

        #[test]
        fn test_audio_metadata_cover_picture() {
            let mut metadata = AudioMetadata::new(AudioFormat::Flac);
            assert!(metadata.cover_picture().is_none());

            // 添加非封面图片
            let picture1 = Picture::new(
                PictureType::Other,
                "image/jpeg".to_string(),
                "desc".to_string(),
                vec![0xFF, 0xD8],
            );
            metadata.add_picture(picture1);

            // 返回第一张图片
            assert!(metadata.cover_picture().is_some());

            // 添加封面图片
            let cover = Picture::new(
                PictureType::CoverFront,
                "image/png".to_string(),
                "cover".to_string(),
                vec![0x89, 0x50],
            );
            metadata.add_picture(cover);

            // 返回封面图片
            let found_cover = metadata.cover_picture().unwrap();
            assert_eq!(found_cover.picture_type, PictureType::CoverFront);
        }

        #[test]
        fn test_picture_creation() {
            let data = vec![0xFF, 0xD8, 0xFF];
            let picture = Picture::new(
                PictureType::CoverFront,
                "image/jpeg".to_string(),
                "Front Cover".to_string(),
                data.clone(),
            );

            assert_eq!(picture.picture_type, PictureType::CoverFront);
            assert_eq!(picture.mime_type, "image/jpeg");
            assert_eq!(picture.description, "Front Cover");
            assert_eq!(picture.data, data);
            assert!(picture.is_cover());
        }

        #[test]
        fn test_picture_format_extension() {
            let jpeg = Picture::new(PictureType::CoverFront, "image/jpeg".to_string(), "".to_string(), vec![]);
            assert_eq!(jpeg.format_extension(), "jpg");

            let jpg = Picture::new(PictureType::CoverFront, "image/jpg".to_string(), "".to_string(), vec![]);
            assert_eq!(jpg.format_extension(), "jpg");

            let png = Picture::new(PictureType::CoverFront, "image/png".to_string(), "".to_string(), vec![]);
            assert_eq!(png.format_extension(), "png");

            let gif = Picture::new(PictureType::CoverFront, "image/gif".to_string(), "".to_string(), vec![]);
            assert_eq!(gif.format_extension(), "gif");

            let unknown = Picture::new(PictureType::CoverFront, "image/webp".to_string(), "".to_string(), vec![]);
            assert_eq!(unknown.format_extension(), "webp");

            let bin = Picture::new(PictureType::CoverFront, "application/octet-stream".to_string(), "".to_string(), vec![]);
            assert_eq!(bin.format_extension(), "bin");
        }

        #[test]
        fn test_picture_type_from_id3v2_type() {
            assert_eq!(PictureType::from_id3v2_type(0), PictureType::Other);
            assert_eq!(PictureType::from_id3v2_type(1), PictureType::FileIcon);
            assert_eq!(PictureType::from_id3v2_type(2), PictureType::OtherFileIcon);
            assert_eq!(PictureType::from_id3v2_type(3), PictureType::CoverFront);
            assert_eq!(PictureType::from_id3v2_type(4), PictureType::CoverBack);
            assert_eq!(PictureType::from_id3v2_type(6), PictureType::Media);
            assert_eq!(PictureType::from_id3v2_type(255), PictureType::Other); // 未知代码
        }

        #[test]
        fn test_picture_type_to_vorbis_string() {
            assert_eq!(PictureType::Other.to_vorbis_string(), "Other");
            assert_eq!(PictureType::CoverFront.to_vorbis_string(), "Cover (front)");
            assert_eq!(PictureType::CoverBack.to_vorbis_string(), "Cover (back)");
            assert_eq!(PictureType::Media.to_vorbis_string(), "Media (e.g. label side of CD)");
        }

        #[test]
        fn test_lyric_line_creation() {
            let timestamp = Duration::from_millis(5000);
            let line = LyricLine::new(timestamp, "Test lyric".to_string());

            assert_eq!(line.timestamp_millis(), 5000);
            assert_eq!(line.text, "Test lyric");
        }

        #[test]
        fn test_metadata_error_display() {
            let error = MetadataError::IoError("test error".to_string());
            assert!(error.to_string().contains("IO 错误"));

            let error = MetadataError::InvalidFormat("invalid".to_string());
            assert!(error.to_string().contains("无效的格式"));

            let error = MetadataError::UnsupportedFormat("unsupported".to_string());
            assert!(error.to_string().contains("不支持的格式"));

            let error = MetadataError::ParseError("parse failed".to_string());
            assert!(error.to_string().contains("解析错误"));

            let error = MetadataError::FileTooLarge;
            assert!(error.to_string().contains("文件过大"));

            let error = MetadataError::Unknown;
            assert!(error.to_string().contains("未知错误"));
        }

        #[test]
        fn test_read_metadata_nonexistent() {
            let path = PathBuf::from("/nonexistent/file.mp3");
            assert!(read_metadata(&path).is_err());
        }

        #[test]
        fn test_filter_supported_files() {
            let paths = vec![
                PathBuf::from("test.mp3"),
                PathBuf::from("test.flac"),
                PathBuf::from("test.txt"),
                PathBuf::from("test.mp4"),
                PathBuf::from("test.doc"),
                PathBuf::from("test.m4a"),
                PathBuf::from("test.ogg"),
                PathBuf::from("test.wav"),
            ];

            let supported = filter_supported_files(&paths);
            assert_eq!(supported.len(), 6); // mp3, flac, m4a, ogg, wav
        }
    }

    // ==================== lyric_parser 模块测试 ====================
    mod lyric_parser_tests {
        use crate::lyric_parser::{LyricFormat, LyricParser, ParseError, ParsedLyric, LyricLine, LyricWord};

        #[test]
        fn test_lyric_format_detection_lrc() {
            let lrc_content = "[ti:Test Song]\n[ar:Test Artist]\n[00:00.00]Lyric line";
            assert_eq!(LyricFormat::from_content(lrc_content), LyricFormat::Lrc);

            let lrc_content2 = "[00:12.34]Test line";
            assert_eq!(LyricFormat::from_content(lrc_content2), LyricFormat::Lrc);
        }

        #[test]
        fn test_lyric_format_detection_yrc() {
            let yrc_content = "[ver:v1]\n[by:网易云]\n[0,1000]<0,200>Test";
            assert_eq!(LyricFormat::from_content(yrc_content), LyricFormat::Yrc);
        }

        #[test]
        fn test_lyric_format_detection_qrc() {
            let qrc_content = "[ver:qrc]\n[ar:Artist]\n[0,1000]Test(0,200)word";
            assert_eq!(LyricFormat::from_content(qrc_content), LyricFormat::Qrc);
        }

        #[test]
        fn test_lyric_format_detection_ttml() {
            let ttml_content = r#"<?xml version="1.0"?><tt><p begin="00:00:01">Test</p></tt>"#;
            assert_eq!(LyricFormat::from_content(ttml_content), LyricFormat::Ttml);

            let ttml_content2 = r#"<ttml><p begin="00:00:01.000" end="00:00:05.000">Test</p></ttml>"#;
            assert_eq!(LyricFormat::from_content(ttml_content2), LyricFormat::Ttml);
        }

        #[test]
        fn test_lyric_format_detection_unknown() {
            let unknown_content = "This is just plain text without any format";
            assert_eq!(LyricFormat::from_content(unknown_content), LyricFormat::Unknown);
        }

        #[test]
        fn test_lrc_parsing() {
            let content = r#"[ti:Test Song]
[ar:Test Artist]
[al:Test Album]
[by:Lyric Author]
[offset:100]
[00:00.00]First line
[00:05.50]Second line
[00:10.00]Third line"#;

            let parser = LyricParser::new();
            let result = parser.parse(content, LyricFormat::Lrc).unwrap();

            assert_eq!(result.metadata.title, Some("Test Song".to_string()));
            assert_eq!(result.metadata.artist, Some("Test Artist".to_string()));
            assert_eq!(result.metadata.album, Some("Test Album".to_string()));
            assert_eq!(result.metadata.by, Some("Lyric Author".to_string()));
            assert_eq!(result.metadata.offset, 100);
            assert_eq!(result.lines.len(), 3);
            assert_eq!(result.lines[0].start_time, 0);
            assert_eq!(result.lines[1].start_time, 5500);
            assert_eq!(result.lines[2].start_time, 10000);
        }

        #[test]
        fn test_lrc_parsing_multiple_timestamps() {
            let content = "[00:00.00][00:05.00]Same text";

            let parser = LyricParser::new();
            let result = parser.parse(content, LyricFormat::Lrc).unwrap();

            assert_eq!(result.lines.len(), 2);
            assert_eq!(result.lines[0].start_time, 0);
            assert_eq!(result.lines[1].start_time, 5000);
        }

        #[test]
        fn test_yrc_parsing() {
            let content = r#"[ver:v1]
[ar:Test Artist]
[ti:Test Song]
[0,1000]<0,200>歌<200,300>词<500,500>内<800,200>容"#;

            let parser = LyricParser::new();
            let result = parser.parse(content, LyricFormat::Yrc).unwrap();

            assert_eq!(result.metadata.artist, Some("Test Artist".to_string()));
            assert_eq!(result.metadata.title, Some("Test Song".to_string()));
            assert_eq!(result.lines.len(), 1);
            assert_eq!(result.lines[0].words.len(), 4);
            assert_eq!(result.lines[0].text(), "歌词内容");
        }

        #[test]
        fn test_qrc_parsing() {
            let content = r#"[ver:qrc]
[ar:Test Artist]
[0,1000]歌词(0,200)内(200,300)容(500,500)"#;

            let parser = LyricParser::new();
            let result = parser.parse(content, LyricFormat::Qrc).unwrap();

            assert_eq!(result.metadata.artist, Some("Test Artist".to_string()));
            assert_eq!(result.lines.len(), 1);
            assert_eq!(result.lines[0].words.len(), 3);
        }

        #[test]
        fn test_ttml_parsing() {
            let content = r#"<?xml version="1.0"?>
<tt>
  <body>
    <div>
      <p begin="00:00:01.000" end="00:00:05.000">First line</p>
      <p begin="00:00:06.000" end="00:00:10.000">Second line</p>
    </div>
  </body>
</tt>"#;

            let parser = LyricParser::new();
            let result = parser.parse(content, LyricFormat::Ttml).unwrap();

            assert_eq!(result.lines.len(), 2);
            assert_eq!(result.lines[0].start_time, 1000);
            assert_eq!(result.lines[1].start_time, 6000);
        }

        #[test]
        fn test_parse_empty_content() {
            let parser = LyricParser::new();
            let result = parser.parse("", LyricFormat::Lrc);
            assert!(matches!(result, Err(ParseError::EmptyContent)));
        }

        #[test]
        fn test_parse_unsupported_format() {
            let parser = LyricParser::new();
            let result = parser.parse("some content", LyricFormat::Unknown);
            assert!(matches!(result, Err(ParseError::UnsupportedFormat)));
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

            let mut line3 = LyricLine::new(10000, 5000);
            line3.add_word(LyricWord::new(10000, 5000, "Third".to_string()));
            parsed.add_line(line3);

            assert_eq!(parsed.find_line_by_time(0).unwrap().text(), "First");
            assert_eq!(parsed.find_line_by_time(2500).unwrap().text(), "First");
            assert_eq!(parsed.find_line_by_time(5000).unwrap().text(), "Second");
            assert_eq!(parsed.find_line_by_time(7500).unwrap().text(), "Second");
            assert_eq!(parsed.find_line_by_time(12000).unwrap().text(), "Third");
            // 超过最后一行的时间会返回最后一行（这是当前实现的行为）
            assert_eq!(parsed.find_line_by_time(20000).unwrap().text(), "Third");
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
            assert_eq!(parsed.find_current_line_index(2500), Some(0));
            assert_eq!(parsed.find_current_line_index(6000), Some(1));
        }

        #[test]
        fn test_lyric_line_text() {
            let mut line = LyricLine::new(0, 5000);
            line.add_word(LyricWord::new(0, 1000, "Hello".to_string()));
            line.add_word(LyricWord::new(1000, 1000, " ".to_string()));
            line.add_word(LyricWord::new(2000, 1000, "World".to_string()));

            assert_eq!(line.text(), "Hello World");
        }

        #[test]
        fn test_parsed_lyric_sort_lines() {
            let mut parsed = ParsedLyric::new();

            let mut line1 = LyricLine::new(5000, 1000);
            line1.add_word(LyricWord::new(5000, 1000, "Second".to_string()));
            parsed.add_line(line1);

            let mut line2 = LyricLine::new(0, 1000);
            line2.add_word(LyricWord::new(0, 1000, "First".to_string()));
            parsed.add_line(line2);

            parsed.sort_lines();

            assert_eq!(parsed.lines[0].start_time, 0);
            assert_eq!(parsed.lines[1].start_time, 5000);
        }

        #[test]
        fn test_parse_error_display() {
            let error = ParseError::EmptyContent;
            assert!(error.to_string().contains("为空"));

            let error = ParseError::InvalidFormat("test".to_string());
            assert!(error.to_string().contains("格式错误"));

            let error = ParseError::ParseFailed("test".to_string());
            assert!(error.to_string().contains("解析失败"));

            let error = ParseError::UnsupportedFormat;
            assert!(error.to_string().contains("不支持"));
        }
    }

    // ==================== cache_manager 模块测试 ====================
    mod cache_manager_tests {
        use crate::cache::{CacheManager, CacheError};
        use crate::music_source::{MusicLibrary, SourceManager, SourceType, TrackMetadata, Artist, Album};
        use std::path::PathBuf;
        use tempfile::TempDir;

        fn create_test_cache_manager() -> (CacheManager, TempDir) {
            let temp_dir = TempDir::new().unwrap();
            let cache_manager = CacheManager::with_directory(temp_dir.path().to_path_buf());
            (cache_manager, temp_dir)
        }

        fn create_test_library() -> MusicLibrary {
            let mut library = MusicLibrary::new();

            let artist = Artist::new("artist_1".to_string(), "测试歌手".to_string());
            library.artists.push(artist);

            let album = Album::new(
                "album_1".to_string(),
                "测试专辑".to_string(),
                "artist_1".to_string(),
                "测试歌手".to_string(),
            );
            library.albums.push(album);

            library
        }

        #[test]
        fn test_cache_manager_creation() {
            let (cache_manager, _temp_dir) = create_test_cache_manager();
            assert!(cache_manager.cache_dir().exists());
        }

        #[test]
        fn test_cache_manager_default() {
            let cache_manager: CacheManager = Default::default();
            assert!(!cache_manager.cache_dir().to_string_lossy().is_empty());
        }

        #[test]
        fn test_save_and_load_library() {
            let (cache_manager, _temp_dir) = create_test_cache_manager();
            let library = create_test_library();

            // 保存音乐库
            cache_manager.save_library(&library).unwrap();

            // 加载音乐库
            let loaded = cache_manager.load_library().unwrap();
            assert_eq!(loaded.artists.len(), library.artists.len());
            assert_eq!(loaded.albums.len(), library.albums.len());
            assert_eq!(loaded.artists[0].name, "测试歌手");
        }

        #[test]
        fn test_load_nonexistent_library() {
            let (cache_manager, _temp_dir) = create_test_cache_manager();
            let result = cache_manager.load_library();
            assert!(matches!(result, Err(CacheError::FileNotFound(_))));
        }

        #[test]
        fn test_save_and_load_sources() {
            let (cache_manager, temp_dir) = create_test_cache_manager();
            let mut source_manager = SourceManager::new();

            // 使用实际存在的临时目录
            let test_path = temp_dir.path().join("music");
            std::fs::create_dir(&test_path).unwrap();
            source_manager.add_local_folder(test_path, true).unwrap();

            // 保存源配置
            cache_manager.save_sources(&source_manager).unwrap();

            // 加载源配置
            let loaded = cache_manager.load_sources().unwrap();
            assert_eq!(loaded.len(), 1);
            assert_eq!(loaded[0].source_type, SourceType::LocalFolder);
        }

        #[test]
        fn test_load_nonexistent_sources() {
            let (cache_manager, _temp_dir) = create_test_cache_manager();
            let loaded = cache_manager.load_sources().unwrap();
            assert!(loaded.is_empty());
        }

        #[test]
        fn test_save_and_load_source_cache() {
            let (cache_manager, _temp_dir) = create_test_cache_manager();
            let source_id = "test_source";
            let tracks: Vec<TrackMetadata> = vec![];

            // 保存源缓存
            cache_manager.save_source_cache(source_id, &tracks).unwrap();

            // 加载源缓存
            let loaded = cache_manager.load_source_cache(source_id).unwrap();
            assert_eq!(loaded.len(), 0);
        }

        #[test]
        fn test_load_nonexistent_source_cache() {
            let (cache_manager, _temp_dir) = create_test_cache_manager();
            let result = cache_manager.load_source_cache("nonexistent");
            assert!(result.is_ok());
            assert!(result.unwrap().is_empty());
        }

        #[test]
        fn test_delete_source_cache() {
            let (cache_manager, _temp_dir) = create_test_cache_manager();
            let source_id = "test_source";
            let tracks: Vec<TrackMetadata> = vec![];

            // 保存源缓存
            cache_manager.save_source_cache(source_id, &tracks).unwrap();

            // 验证文件存在
            let cache_path = cache_manager.cache_dir().join("sources").join(format!("{}.json", source_id));
            assert!(cache_path.exists());

            // 删除源缓存
            cache_manager.delete_source_cache(source_id).unwrap();

            // 验证已删除
            assert!(!cache_path.exists());
        }

        #[test]
        fn test_clear_all_cache() {
            let (cache_manager, temp_dir) = create_test_cache_manager();
            let library = create_test_library();
            let mut source_manager = SourceManager::new();

            // 使用实际存在的临时目录
            let test_path = temp_dir.path().join("music");
            std::fs::create_dir(&test_path).unwrap();
            source_manager.add_local_folder(test_path, true).unwrap();

            // 保存数据
            cache_manager.save_library(&library).unwrap();
            cache_manager.save_sources(&source_manager).unwrap();
            cache_manager.save_source_cache("test", &[]).unwrap();

            // 清除所有缓存
            cache_manager.clear_all_cache().unwrap();

            // 验证文件已删除
            assert!(cache_manager.load_library().is_err());
            assert!(cache_manager.load_sources().unwrap().is_empty());
        }

        #[test]
        fn test_cache_size() {
            let (cache_manager, _temp_dir) = create_test_cache_manager();
            let library = create_test_library();

            // 初始大小应为 0
            let initial_size = cache_manager.cache_size().unwrap();
            assert_eq!(initial_size, 0);

            // 保存数据
            cache_manager.save_library(&library).unwrap();

            // 检查大小是否增加
            let size_after_save = cache_manager.cache_size().unwrap();
            assert!(size_after_save > 0);
        }

        #[test]
        fn test_cache_error_display() {
            let error = CacheError::IoError("test".to_string());
            assert!(error.to_string().contains("IO 错误"));

            let error = CacheError::SerializationError("test".to_string());
            assert!(error.to_string().contains("序列化错误"));

            let error = CacheError::DeserializationError("test".to_string());
            assert!(error.to_string().contains("反序列化错误"));

            let error = CacheError::DirectoryNotFound("/test".to_string());
            assert!(error.to_string().contains("缓存目录不存在"));

            let error = CacheError::FileNotFound("/test.json".to_string());
            assert!(error.to_string().contains("缓存文件不存在"));

            let error = CacheError::InvalidData("test".to_string());
            assert!(error.to_string().contains("无效的缓存数据"));
        }
    }

    // ==================== music_scanner 模块测试 ====================
    mod music_scanner_tests {
        use crate::scanner::{MusicScanner, ScanProgress};
        use crate::cache::CacheManager;
        use crate::music_source::SourceConfig;
        use std::path::PathBuf;
        use tempfile::TempDir;

        #[test]
        fn test_music_scanner_new() {
            let _scanner = MusicScanner::new();
        }

        #[test]
        fn test_music_scanner_default() {
            let _scanner: MusicScanner = Default::default();
        }

        #[test]
        fn test_scan_progress_default() {
            let progress = ScanProgress::default();
            assert!(progress.source_id.is_empty());
            assert!(progress.source_name.is_empty());
            assert_eq!(progress.scanned_count, 0);
            assert_eq!(progress.total_count, 0);
            assert_eq!(progress.found_count, 0);
            assert_eq!(progress.error_count, 0);
            assert!(!progress.is_complete);
            assert!(progress.error_message.is_none());
        }

        #[test]
        fn test_music_scanner_with_cache_manager() {
            let temp_dir = TempDir::new().unwrap();
            let cache_manager = CacheManager::with_directory(temp_dir.path().to_path_buf());
            let _scanner = MusicScanner::with_cache_manager(cache_manager);
        }

        #[test]
        fn test_scan_source_creation() {
            // 创建一个本地文件夹源
            let source = SourceConfig::new_local_folder(PathBuf::from("/nonexistent"), None, true);
            assert_eq!(source.source_type, crate::music_source::SourceType::LocalFolder);
            assert!(source.enabled);
        }
    }

    // ==================== artist/album 模块测试 ====================
    mod artist_album_tests {
        use crate::music_source::{Artist, ArtistSummary, Album, AlbumSummary, ArtistParser, AlbumIdGenerator};

        // Artist 测试
        #[test]
        fn test_artist_new() {
            let artist = Artist::new("artist_123".to_string(), "周杰伦".to_string());
            assert_eq!(artist.id, "artist_123");
            assert_eq!(artist.name, "周杰伦");
            assert!(artist.bio.is_none());
            assert!(artist.genres.is_empty());
            assert!(artist.cover_data.is_none());
            assert!(artist.album_ids.is_empty());
            assert!(artist.track_ids.is_empty());
        }

        #[test]
        fn test_artist_add_album() {
            let mut artist = Artist::new("artist_123".to_string(), "周杰伦".to_string());
            artist.add_album("album_1".to_string());
            artist.add_album("album_2".to_string());
            artist.add_album("album_1".to_string()); // 重复添加应该被忽略

            assert_eq!(artist.album_count(), 2);
            assert_eq!(artist.album_ids, vec!["album_1", "album_2"]);
        }

        #[test]
        fn test_artist_add_track() {
            let mut artist = Artist::new("artist_123".to_string(), "周杰伦".to_string());
            artist.add_track("track_1".to_string());
            artist.add_track("track_2".to_string());
            artist.add_track("track_1".to_string()); // 重复添加应该被忽略

            assert_eq!(artist.track_count(), 2);
            assert_eq!(artist.track_ids, vec!["track_1", "track_2"]);
        }

        #[test]
        fn test_artist_to_summary() {
            let mut artist = Artist::new("artist_123".to_string(), "周杰伦".to_string());
            artist.bio = Some("华语流行歌手".to_string());
            artist.add_album("album_1".to_string());
            artist.add_album("album_2".to_string());
            artist.add_track("track_1".to_string());
            artist.add_track("track_2".to_string());
            artist.add_track("track_3".to_string());

            let summary = artist.to_summary();
            assert_eq!(summary.id, "artist_123");
            assert_eq!(summary.name, "周杰伦");
            assert_eq!(summary.album_count, 2);
            assert_eq!(summary.track_count, 3);
        }

        #[test]
        fn test_artist_summary_new() {
            let summary = ArtistSummary::new("artist_123".to_string(), "周杰伦".to_string());
            assert_eq!(summary.id, "artist_123");
            assert_eq!(summary.name, "周杰伦");
            assert_eq!(summary.album_count, 0);
            assert_eq!(summary.track_count, 0);
        }

        // Album 测试
        #[test]
        fn test_album_new() {
            let album = Album::new(
                "album_123".to_string(),
                "范特西".to_string(),
                "artist_456".to_string(),
                "周杰伦".to_string(),
            );

            assert_eq!(album.id, "album_123");
            assert_eq!(album.title, "范特西");
            assert_eq!(album.artist_id, "artist_456");
            assert_eq!(album.artist_name, "周杰伦");
            assert!(album.year.is_none());
            assert!(album.genres.is_empty());
            assert!(album.cover_data.is_none());
            assert!(album.track_ids.is_empty());
            assert_eq!(album.total_duration, 0);
        }

        #[test]
        fn test_album_add_track() {
            let mut album = Album::new(
                "album_123".to_string(),
                "范特西".to_string(),
                "artist_456".to_string(),
                "周杰伦".to_string(),
            );

            album.add_track("track_1".to_string());
            album.add_track("track_2".to_string());
            album.add_track("track_1".to_string()); // 重复添加应该被忽略

            assert_eq!(album.track_count(), 2);
            assert_eq!(album.track_ids, vec!["track_1", "track_2"]);
        }

        #[test]
        fn test_album_to_summary() {
            let mut album = Album::new(
                "album_123".to_string(),
                "范特西".to_string(),
                "artist_456".to_string(),
                "周杰伦".to_string(),
            );
            album.year = Some(2001);
            album.add_track("track_1".to_string());
            album.add_track("track_2".to_string());

            let summary = album.to_summary();
            assert_eq!(summary.id, "album_123");
            assert_eq!(summary.title, "范特西");
            assert_eq!(summary.artist_id, "artist_456");
            assert_eq!(summary.artist_name, "周杰伦");
            assert_eq!(summary.year, Some(2001));
            assert_eq!(summary.track_count, 2);
        }

        #[test]
        fn test_album_summary_new() {
            let summary = AlbumSummary::new(
                "album_123".to_string(),
                "范特西".to_string(),
                "artist_456".to_string(),
                "周杰伦".to_string(),
            );

            assert_eq!(summary.id, "album_123");
            assert_eq!(summary.title, "范特西");
            assert_eq!(summary.artist_id, "artist_456");
            assert_eq!(summary.artist_name, "周杰伦");
            assert!(summary.year.is_none());
            assert_eq!(summary.track_count, 0);
        }

        // ArtistParser 测试
        #[test]
        fn test_artist_parser_single() {
            let artists = ArtistParser::parse("周杰伦");
            assert_eq!(artists, vec!["周杰伦"]);
        }

        #[test]
        fn test_artist_parser_slash_separator() {
            let artists = ArtistParser::parse("周杰伦/费玉清");
            assert_eq!(artists, vec!["周杰伦", "费玉清"]);
        }

        #[test]
        fn test_artist_parser_ampersand_separator() {
            let artists = ArtistParser::parse("Taylor Swift & Ed Sheeran");
            assert_eq!(artists, vec!["taylor swift", "ed sheeran"]);
        }

        #[test]
        fn test_artist_parser_mixed_separators() {
            // 优先使用 /
            let artists = ArtistParser::parse("A/B & C");
            assert_eq!(artists, vec!["a", "b & c"]);
        }

        #[test]
        fn test_artist_parser_whitespace() {
            let artists = ArtistParser::parse("  周杰伦  /  费玉清  ");
            assert_eq!(artists, vec!["周杰伦", "费玉清"]);
        }

        #[test]
        fn test_artist_parser_empty() {
            let artists = ArtistParser::parse("");
            assert!(artists.is_empty());
        }

        #[test]
        fn test_artist_parser_normalize_name() {
            assert_eq!(ArtistParser::normalize_name("  周杰伦  "), "周杰伦");
            assert_eq!(ArtistParser::normalize_name("Taylor   Swift"), "taylor swift");
            assert_eq!(ArtistParser::normalize_name("  John   Paul   "), "john paul");
        }

        #[test]
        fn test_artist_parser_generate_id() {
            let id1 = ArtistParser::generate_id("周杰伦");
            let id2 = ArtistParser::generate_id("周杰伦");
            let id3 = ArtistParser::generate_id("王力宏");

            assert_eq!(id1, id2); // 相同输入应该产生相同ID
            assert_ne!(id1, id3); // 不同输入应该产生不同ID
            assert!(id1.starts_with("artist_"));
        }

        #[test]
        fn test_artist_parser_generate_combined_id() {
            let id1 = ArtistParser::generate_combined_id(&["周杰伦".to_string()]);
            let id2 = ArtistParser::generate_combined_id(&["周杰伦".to_string(), "费玉清".to_string()]);

            assert!(id1.starts_with("artist_"));
            assert!(id2.starts_with("artists_"));
            assert_ne!(id1, id2);
        }

        // AlbumIdGenerator 测试
        #[test]
        fn test_album_id_generator_generate_id() {
            let id1 = AlbumIdGenerator::generate_id("范特西", "周杰伦");
            let id2 = AlbumIdGenerator::generate_id("范特西", "周杰伦");
            let id3 = AlbumIdGenerator::generate_id("叶惠美", "周杰伦");

            assert_eq!(id1, id2); // 相同输入应该产生相同ID
            assert_ne!(id1, id3); // 不同输入应该产生不同ID
            assert!(id1.starts_with("album_"));
        }

        #[test]
        fn test_album_id_generator_normalize_name() {
            assert_eq!(AlbumIdGenerator::normalize_name("  范特西  "), "范特西");
            assert_eq!(AlbumIdGenerator::normalize_name("The Album"), "The Album");
        }
    }

    // ==================== 集成测试 ====================
    mod integration_tests {
        use crate::cache::CacheManager;
        use crate::music_source::{MusicLibrary, SourceManager, Artist, Album};
        use crate::scanner::MusicScanner;
        use std::path::PathBuf;
        use tempfile::TempDir;

        #[test]
        fn test_cache_lifecycle() {
            let temp_dir = TempDir::new().unwrap();
            let cache_manager = CacheManager::with_directory(temp_dir.path().to_path_buf());

            // 1. 创建音乐库
            let mut library = MusicLibrary::new();
            let artist = Artist::new("artist_1".to_string(), "测试歌手".to_string());
            library.artists.push(artist);

            // 2. 保存音乐库
            cache_manager.save_library(&library).unwrap();

            // 3. 加载音乐库
            let loaded = cache_manager.load_library().unwrap();
            assert_eq!(loaded.artists.len(), 1);

            // 4. 更新音乐库
            let artist2 = Artist::new("artist_2".to_string(), "另一个歌手".to_string());
            let mut updated_library = loaded;
            updated_library.artists.push(artist2);

            // 5. 重新保存
            cache_manager.save_library(&updated_library).unwrap();

            // 6. 再次加载验证
            let reloaded = cache_manager.load_library().unwrap();
            assert_eq!(reloaded.artists.len(), 2);

            // 7. 清除缓存
            cache_manager.clear_all_cache().unwrap();

            // 8. 验证缓存已清除
            assert!(cache_manager.load_library().is_err());
        }

        #[test]
        fn test_source_cache_lifecycle() {
            let temp_dir = TempDir::new().unwrap();
            let cache_manager = CacheManager::with_directory(temp_dir.path().to_path_buf());
            let source_id = "test_source";

            // 初始加载应该返回空列表
            let tracks = cache_manager.load_source_cache(source_id).unwrap();
            assert!(tracks.is_empty());

            // 保存空列表
            cache_manager.save_source_cache(source_id, &[]).unwrap();

            // 加载空列表
            let tracks = cache_manager.load_source_cache(source_id).unwrap();
            assert!(tracks.is_empty());

            // 删除缓存
            cache_manager.delete_source_cache(source_id).unwrap();

            // 验证已删除 - 通过检查缓存目录下的文件
            let cache_path = cache_manager.cache_dir().join("sources").join(format!("{}.json", source_id));
            assert!(!cache_path.exists());
        }

        #[test]
        fn test_scanner_with_cache() {
            let temp_dir = TempDir::new().unwrap();
            let cache_manager = CacheManager::with_directory(temp_dir.path().to_path_buf());
            let scanner = MusicScanner::with_cache_manager(cache_manager.clone());

            // 测试从缓存加载（缓存不存在时返回 Some([]) 因为 load_source_cache 返回空列表）
            let cached = scanner.load_from_cache("nonexistent");
            // load_source_cache 在文件不存在时返回空列表，所以这里是 Some([])
            assert!(cached.is_some());
            assert!(cached.unwrap().is_empty());

            // 保存空列表到缓存
            scanner.save_to_cache("test_source", &[]);

            // 从缓存加载
            let cached = scanner.load_from_cache("test_source");
            assert!(cached.is_some());
            assert!(cached.unwrap().is_empty());
        }

        #[test]
        fn test_full_workflow_simulation() {
            let temp_dir = TempDir::new().unwrap();
            let cache_manager = CacheManager::with_directory(temp_dir.path().to_path_buf());

            // 1. 创建源管理器并添加源
            let mut source_manager = SourceManager::new();
            let test_path = temp_dir.path().join("music");
            std::fs::create_dir(&test_path).unwrap();
            source_manager.add_local_folder(test_path, true).unwrap();

            // 2. 保存源配置
            cache_manager.save_sources(&source_manager).unwrap();

            // 3. 创建音乐库
            let mut library = MusicLibrary::new();
            library.sources = source_manager.get_all_sources().to_vec();

            let artist = Artist::new("artist_1".to_string(), "测试歌手".to_string());
            let album = Album::new(
                "album_1".to_string(),
                "测试专辑".to_string(),
                "artist_1".to_string(),
                "测试歌手".to_string(),
            );

            library.artists.push(artist);
            library.albums.push(album);

            // 4. 保存音乐库
            cache_manager.save_library(&library).unwrap();

            // 5. 验证缓存大小
            let size = cache_manager.cache_size().unwrap();
            assert!(size > 0);

            // 6. 加载并验证
            let loaded_library = cache_manager.load_library().unwrap();
            assert_eq!(loaded_library.artists.len(), 1);
            assert_eq!(loaded_library.albums.len(), 1);

            let loaded_sources = cache_manager.load_sources().unwrap();
            assert_eq!(loaded_sources.len(), 1);
        }
    }
}
