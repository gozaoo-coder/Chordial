//! 音乐元数据读取器测试模块

#[cfg(test)]
mod tests {
    use crate::{AudioFormat, AudioMetadata, Picture, PictureType};

    #[test]
    fn test_audio_format_from_extension() {
        assert_eq!(AudioFormat::from_extension("flac"), AudioFormat::Flac);
        assert_eq!(AudioFormat::from_extension("FLAC"), AudioFormat::Flac);
        assert_eq!(AudioFormat::from_extension("mp3"), AudioFormat::Mp3);
        assert_eq!(AudioFormat::from_extension("m4a"), AudioFormat::M4a);
        assert_eq!(AudioFormat::from_extension("ogg"), AudioFormat::Ogg);
        assert_eq!(AudioFormat::from_extension("wav"), AudioFormat::Wav);
        assert_eq!(AudioFormat::from_extension("unknown"), AudioFormat::Unknown);
    }

    #[test]
    fn test_audio_format_mime_type() {
        assert_eq!(AudioFormat::Flac.mime_type(), "audio/flac");
        assert_eq!(AudioFormat::Mp3.mime_type(), "audio/mpeg");
        assert_eq!(AudioFormat::M4a.mime_type(), "audio/mp4");
        assert_eq!(AudioFormat::Ogg.mime_type(), "audio/ogg");
        assert_eq!(AudioFormat::Wav.mime_type(), "audio/wav");
        assert_eq!(AudioFormat::Unknown.mime_type(), "application/octet-stream");
    }

    #[test]
    fn test_audio_metadata_creation() {
        let metadata = AudioMetadata::new(AudioFormat::Flac);
        assert_eq!(metadata.format, AudioFormat::Flac);
        assert!(!metadata.has_basic_tags());
    }

    #[test]
    fn test_audio_metadata_basic_tags() {
        let mut metadata = AudioMetadata::new(AudioFormat::Mp3);
        assert!(!metadata.has_basic_tags());

        metadata.title = Some("Test Song".to_string());
        assert!(metadata.has_basic_tags());

        metadata.artist = Some("Test Artist".to_string());
        metadata.album = Some("Test Album".to_string());
        assert!(metadata.has_basic_tags());
    }

    #[test]
    fn test_audio_metadata_artist_display() {
        let mut metadata = AudioMetadata::new(AudioFormat::Flac);
        
        // 只有艺术家
        metadata.artist = Some("Artist".to_string());
        assert_eq!(metadata.get_artist_display(), Some("Artist".to_string()));

        // 艺术家和专辑艺术家相同
        metadata.album_artist = Some("Artist".to_string());
        assert_eq!(metadata.get_artist_display(), Some("Artist".to_string()));

        // 艺术家和专辑艺术家不同
        metadata.album_artist = Some("Album Artist".to_string());
        assert_eq!(metadata.get_artist_display(), Some("Artist (Album Artist)".to_string()));

        // 只有专辑艺术家
        metadata.artist = None;
        assert_eq!(metadata.get_artist_display(), Some("Album Artist".to_string()));
    }

    #[test]
    fn test_audio_metadata_track_display() {
        let mut metadata = AudioMetadata::new(AudioFormat::Flac);
        
        assert_eq!(metadata.get_track_display(), None);

        metadata.track_number = Some(5);
        assert_eq!(metadata.get_track_display(), Some("5".to_string()));

        metadata.total_tracks = Some(12);
        assert_eq!(metadata.get_track_display(), Some("5/12".to_string()));
    }

    #[test]
    fn test_audio_metadata_disc_display() {
        let mut metadata = AudioMetadata::new(AudioFormat::Flac);
        
        assert_eq!(metadata.get_disc_display(), None);

        metadata.disc_number = Some(1);
        assert_eq!(metadata.get_disc_display(), Some("1".to_string()));

        metadata.total_discs = Some(2);
        assert_eq!(metadata.get_disc_display(), Some("1/2".to_string()));
    }

    #[test]
    fn test_picture_creation() {
        let data = vec![0xFF, 0xD8, 0xFF]; // JPEG文件头
        let picture = Picture::new(
            PictureType::CoverFront,
            "image/jpeg".to_string(),
            data.clone()
        );

        assert_eq!(picture.picture_type, PictureType::CoverFront);
        assert_eq!(picture.mime_type, "image/jpeg");
        assert_eq!(picture.data, data);
        assert!(picture.is_cover());
        assert_eq!(picture.size(), 3);
    }

    #[test]
    fn test_picture_type_conversion() {
        use PictureType;
        
        assert_eq!(PictureType::from_id3_code(0), PictureType::Other);
        assert_eq!(PictureType::from_id3_code(3), PictureType::CoverFront);
        assert_eq!(PictureType::from_id3_code(6), PictureType::Media);
        assert_eq!(PictureType::from_id3_code(255), PictureType::Other); // 未知代码

        assert_eq!(PictureType::Other.to_id3_code(), 0);
        assert_eq!(PictureType::CoverFront.to_id3_code(), 3);
        assert_eq!(PictureType::Media.to_id3_code(), 6);
    }
}