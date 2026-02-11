//! M4A/MP4 音频格式读取器
//!
//! 支持 iTunes 风格的元数据（moov/udta/meta 原子）

use crate::audio_metadata::{
    core::{AudioFormat, AudioMetadata, Picture, PictureType},
    utils::encoding::auto_decode_text,
};
use crate::audio_metadata::MetadataError;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::time::Duration;

/// M4A 元数据读取器
pub struct M4aReader;

impl M4aReader {
    /// 从任意实现了 Read + Seek 的读取器中读取 M4A 元数据
    pub fn read_from<R: Read + Seek>(mut reader: R) -> Result<AudioMetadata, MetadataError> {
        let mut metadata = AudioMetadata::new(AudioFormat::M4a);

        // 解析文件结构
        parse_moov_atom(&mut reader, &mut metadata)?;

        Ok(metadata)
    }
}

/// MP4 原子（Atom/Box）头部
#[derive(Debug)]
struct Mp4Atom {
    size: u64,
    atom_type: String,
    data_offset: u64,
}

/// 读取 M4A 文件元数据
pub fn read_m4a_metadata(path: &Path) -> Result<AudioMetadata, MetadataError> {
    let file = File::open(path)
        .map_err(|e| MetadataError::IoError(e.to_string()))?;
    M4aReader::read_from(file)
}

/// 读取原子头部
fn read_atom_header<R: Read>(reader: &mut R) -> Result<Option<Mp4Atom>, MetadataError> {
    let mut size_bytes = [0u8; 4];
    if reader.read_exact(&mut size_bytes).is_err() {
        return Ok(None);
    }

    let mut size = u32::from_be_bytes(size_bytes) as u64;

    let mut type_bytes = [0u8; 4];
    reader.read_exact(&mut type_bytes)
        .map_err(|e| MetadataError::IoError(e.to_string()))?;
    let atom_type = String::from_utf8_lossy(&type_bytes).to_string();

    // 处理扩展大小（64 位）
    if size == 1 {
        let mut ext_size = [0u8; 8];
        reader.read_exact(&mut ext_size)
            .map_err(|e| MetadataError::IoError(e.to_string()))?;
        size = u64::from_be_bytes(ext_size);
    } else if size == 0 {
        // 扩展到文件末尾
        return Err(MetadataError::InvalidFormat("不支持的原子大小".to_string()));
    }

    let header_size = if size == 1 { 16 } else { 8 };

    Ok(Some(Mp4Atom {
        size,
        atom_type,
        data_offset: header_size,
    }))
}

/// 解析 moov 原子
fn parse_moov_atom<R: Read + Seek>(
    reader: &mut R,
    metadata: &mut AudioMetadata,
) -> Result<(), MetadataError> {
    // 首先找到 moov 原子
    let moov_pos = find_atom(reader, "moov", 0)?;
    if moov_pos.is_none() {
        return Ok(());
    }

    // 解析 moov 的子原子
    reader.seek(SeekFrom::Start(moov_pos.unwrap()))
        .map_err(|e| MetadataError::IoError(e.to_string()))?;

    if let Some(header) = read_atom_header(reader)? {
        if header.atom_type != "moov" {
            return Ok(());
        }

        let moov_end = reader.stream_position()
            .map_err(|e| MetadataError::IoError(e.to_string()))?
            + header.size - header.data_offset;

        // 解析子原子
        while reader.stream_position()
            .map_err(|e| MetadataError::IoError(e.to_string()))? < moov_end {
            let pos = reader.stream_position()
                .map_err(|e| MetadataError::IoError(e.to_string()))?;

            if let Some(child) = read_atom_header(reader)? {
                match child.atom_type.as_str() {
                    "udta" => {
                        parse_udta_atom(reader, metadata, child.size - child.data_offset)?;
                    }
                    "trak" => {
                        // 解析音轨信息以获取技术参数
                        parse_trak_atom(reader, metadata, child.size - child.data_offset)?;
                    }
                    "mvhd" => {
                        // 解析电影头部以获取时长
                        parse_mvhd_atom(reader, metadata)?;
                    }
                    _ => {
                        // 跳过其他原子
                        reader.seek(SeekFrom::Current((child.size - child.data_offset) as i64))
                            .map_err(|e| MetadataError::IoError(e.to_string()))?;
                    }
                }
            } else {
                break;
            }
        }
    }

    Ok(())
}

/// 解析 udta 原子
fn parse_udta_atom<R: Read + Seek>(
    reader: &mut R,
    metadata: &mut AudioMetadata,
    size: u64,
) -> Result<(), MetadataError> {
    let end_pos = reader.stream_position()
        .map_err(|e| MetadataError::IoError(e.to_string()))?
        + size;

    while reader.stream_position()
        .map_err(|e| MetadataError::IoError(e.to_string()))? < end_pos {
        if let Some(child) = read_atom_header(reader)? {
            match child.atom_type.as_str() {
                "meta" => {
                    parse_meta_atom(reader, metadata, child.size - child.data_offset)?;
                }
                _ => {
                    reader.seek(SeekFrom::Current((child.size - child.data_offset) as i64))
                        .map_err(|e| MetadataError::IoError(e.to_string()))?;
                }
            }
        } else {
            break;
        }
    }

    Ok(())
}

/// 解析 meta 原子
fn parse_meta_atom<R: Read + Seek>(
    reader: &mut R,
    metadata: &mut AudioMetadata,
    size: u64,
) -> Result<(), MetadataError> {
    // meta 原子有 4 字节的版本/标志
    let mut version_flags = [0u8; 4];
    reader.read_exact(&mut version_flags)
        .map_err(|e| MetadataError::IoError(e.to_string()))?;

    let end_pos = reader.stream_position()
        .map_err(|e| MetadataError::IoError(e.to_string()))?
        + size - 4;

    while reader.stream_position()
        .map_err(|e| MetadataError::IoError(e.to_string()))? < end_pos {
        if let Some(child) = read_atom_header(reader)? {
            match child.atom_type.as_str() {
                "ilst" => {
                    parse_ilst_atom(reader, metadata, child.size - child.data_offset)?;
                }
                _ => {
                    reader.seek(SeekFrom::Current((child.size - child.data_offset) as i64))
                        .map_err(|e| MetadataError::IoError(e.to_string()))?;
                }
            }
        } else {
            break;
        }
    }

    Ok(())
}

/// 解析 ilst（iTunes 列表）原子
fn parse_ilst_atom<R: Read + Seek>(
    reader: &mut R,
    metadata: &mut AudioMetadata,
    size: u64,
) -> Result<(), MetadataError> {
    let end_pos = reader.stream_position()
        .map_err(|e| MetadataError::IoError(e.to_string()))?
        + size;

    while reader.stream_position()
        .map_err(|e| MetadataError::IoError(e.to_string()))? < end_pos {
        if let Some(child) = read_atom_header(reader)? {
            let atom_type = child.atom_type.clone();
            parse_itunes_tag(reader, metadata, &atom_type, child.size - child.data_offset)?;
        } else {
            break;
        }
    }

    Ok(())
}

/// 解析 iTunes 标签
fn parse_itunes_tag<R: Read + Seek>(
    reader: &mut R,
    metadata: &mut AudioMetadata,
    tag_type: &str,
    size: u64,
) -> Result<(), MetadataError> {
    let end_pos = reader.stream_position()
        .map_err(|e| MetadataError::IoError(e.to_string()))?
        + size;

    // 读取数据原子
    while reader.stream_position()
        .map_err(|e| MetadataError::IoError(e.to_string()))? < end_pos {
        if let Some(child) = read_atom_header(reader)? {
            match child.atom_type.as_str() {
                "data" => {
                    parse_data_atom(reader, metadata, tag_type, child.size - child.data_offset)?;
                }
                _ => {
                    reader.seek(SeekFrom::Current((child.size - child.data_offset) as i64))
                        .map_err(|e| MetadataError::IoError(e.to_string()))?;
                }
            }
        } else {
            break;
        }
    }

    Ok(())
}

/// 解析 data 原子
fn parse_data_atom<R: Read>(
    reader: &mut R,
    metadata: &mut AudioMetadata,
    tag_type: &str,
    size: u64,
) -> Result<(), MetadataError> {
    // data 原子格式: [类型(4)] [地区(4)] [数据...]
    let mut type_bytes = [0u8; 4];
    reader.read_exact(&mut type_bytes)
        .map_err(|e| MetadataError::IoError(e.to_string()))?;
    let data_type = u32::from_be_bytes(type_bytes);

    // 跳过地区指示器
    let mut locale = [0u8; 4];
    reader.read_exact(&mut locale)
        .map_err(|e| MetadataError::IoError(e.to_string()))?;

    // 读取实际数据
    let data_size = size - 8;
    let mut data = vec![0u8; data_size as usize];
    reader.read_exact(&mut data)
        .map_err(|e| MetadataError::IoError(e.to_string()))?;

    // 根据标签类型解析数据
    match tag_type {
        "\u{0A9}nam" => { // ©nam - 标题
            if let Ok(text) = auto_decode_text(&data) {
                metadata.title = Some(text);
            }
        }
        "\u{0A9}ART" => { // ©ART - 艺术家
            if let Ok(text) = auto_decode_text(&data) {
                metadata.artist = Some(text);
            }
        }
        "\u{0A9}alb" => { // ©alb - 专辑
            if let Ok(text) = auto_decode_text(&data) {
                metadata.album = Some(text);
            }
        }
        "aART" => { // aART - 专辑艺术家
            if let Ok(text) = auto_decode_text(&data) {
                metadata.album_artist = Some(text);
            }
        }
        "\u{0A9}gen" => { // ©gen - 流派
            if let Ok(text) = auto_decode_text(&data) {
                metadata.genre = Some(text);
            }
        }
        "\u{0A9}day" => { // ©day - 年份
            if let Ok(text) = auto_decode_text(&data) {
                metadata.year = text.chars().take(4).collect::<String>().parse().ok();
            }
        }
        "\u{0A9}wrt" => { // ©wrt - 作曲
            if let Ok(text) = auto_decode_text(&data) {
                metadata.composer = Some(text);
            }
        }
        "\u{0A9}cmt" => { // ©cmt - 评论
            if let Ok(text) = auto_decode_text(&data) {
                metadata.comment = Some(text);
            }
        }
        "\u{0A9}lyr" => { // ©lyr - 歌词
            if let Ok(text) = auto_decode_text(&data) {
                metadata.lyrics = Some(text);
            }
        }
        "trkn" => { // trkn - 音轨号
            if data.len() >= 4 {
                metadata.track_number = Some(u16::from_be_bytes([data[2], data[3]]) as u32);
                if data.len() >= 6 {
                    metadata.total_tracks = Some(u16::from_be_bytes([data[4], data[5]]) as u32);
                }
            }
        }
        "disk" => { // disk - 碟片号
            if data.len() >= 4 {
                metadata.disc_number = Some(u16::from_be_bytes([data[2], data[3]]) as u32);
                if data.len() >= 6 {
                    metadata.total_discs = Some(u16::from_be_bytes([data[4], data[5]]) as u32);
                }
            }
        }
        "covr" => { // covr - 封面图片
            // 检测 MIME 类型
            let mime_type = detect_image_mime(&data);
            let picture = Picture::new(
                PictureType::CoverFront,
                mime_type.to_string(),
                String::new(),
                data,
            );
            metadata.add_picture(picture);
        }
        "gnre" => { // gnre - 流派 ID
            if data.len() >= 2 {
                let genre_id = u16::from_be_bytes([data[0], data[1]]) as usize;
                metadata.genre = Some(get_itunes_genre(genre_id));
            }
        }
        "tmpo" => { // tmpo - BPM
            // 可选：存储 BPM
        }
        "cpil" => { // cpil - 合辑标志
            // 可选：标记是否为合辑
        }
        "pgap" => { // pgap - 播放间隙
            // 可选：标记播放间隙
        }
        _ => {}
    }

    Ok(())
}

/// 解析 trak 原子以获取技术信息
fn parse_trak_atom<R: Read + Seek>(
    reader: &mut R,
    metadata: &mut AudioMetadata,
    size: u64,
) -> Result<(), MetadataError> {
    let end_pos = reader.stream_position()
        .map_err(|e| MetadataError::IoError(e.to_string()))?
        + size;

    while reader.stream_position()
        .map_err(|e| MetadataError::IoError(e.to_string()))? < end_pos {
        if let Some(child) = read_atom_header(reader)? {
            match child.atom_type.as_str() {
                "mdia" => {
                    parse_mdia_atom(reader, metadata, child.size - child.data_offset)?;
                }
                _ => {
                    reader.seek(SeekFrom::Current((child.size - child.data_offset) as i64))
                        .map_err(|e| MetadataError::IoError(e.to_string()))?;
                }
            }
        } else {
            break;
        }
    }

    Ok(())
}

/// 解析 mdia 原子
fn parse_mdia_atom<R: Read + Seek>(
    reader: &mut R,
    metadata: &mut AudioMetadata,
    size: u64,
) -> Result<(), MetadataError> {
    let end_pos = reader.stream_position()
        .map_err(|e| MetadataError::IoError(e.to_string()))?
        + size;

    while reader.stream_position()
        .map_err(|e| MetadataError::IoError(e.to_string()))? < end_pos {
        if let Some(child) = read_atom_header(reader)? {
            match child.atom_type.as_str() {
                "minf" => {
                    parse_minf_atom(reader, metadata, child.size - child.data_offset)?;
                }
                _ => {
                    reader.seek(SeekFrom::Current((child.size - child.data_offset) as i64))
                        .map_err(|e| MetadataError::IoError(e.to_string()))?;
                }
            }
        } else {
            break;
        }
    }

    Ok(())
}

/// 解析 minf 原子
fn parse_minf_atom<R: Read + Seek>(
    reader: &mut R,
    metadata: &mut AudioMetadata,
    size: u64,
) -> Result<(), MetadataError> {
    let end_pos = reader.stream_position()
        .map_err(|e| MetadataError::IoError(e.to_string()))?
        + size;

    while reader.stream_position()
        .map_err(|e| MetadataError::IoError(e.to_string()))? < end_pos {
        if let Some(child) = read_atom_header(reader)? {
            match child.atom_type.as_str() {
                "stbl" => {
                    parse_stbl_atom(reader, metadata, child.size - child.data_offset)?;
                }
                _ => {
                    reader.seek(SeekFrom::Current((child.size - child.data_offset) as i64))
                        .map_err(|e| MetadataError::IoError(e.to_string()))?;
                }
            }
        } else {
            break;
        }
    }

    Ok(())
}

/// 解析 stbl 原子
fn parse_stbl_atom<R: Read + Seek>(
    reader: &mut R,
    _metadata: &mut AudioMetadata,
    size: u64,
) -> Result<(), MetadataError> {
    // 跳过 stbl 内容
    reader.seek(SeekFrom::Current(size as i64))
        .map_err(|e| MetadataError::IoError(e.to_string()))?;
    Ok(())
}

/// 解析 mvhd 原子以获取时长
fn parse_mvhd_atom<R: Read + Seek>(
    reader: &mut R,
    metadata: &mut AudioMetadata,
) -> Result<(), MetadataError> {
    // mvhd 格式: [版本(1)] [标志(3)] [创建时间(4/8)] [修改时间(4/8)]
    //           [时间刻度(4)] [时长(4/8)] [速率(4)] [音量(2)] ...

    let mut version_flags = [0u8; 4];
    reader.read_exact(&mut version_flags)
        .map_err(|e| MetadataError::IoError(e.to_string()))?;
    let version = version_flags[0];

    // 跳过创建和修改时间
    let time_size = if version == 1 { 8 } else { 4 };
    reader.seek(SeekFrom::Current((time_size * 2) as i64))
        .map_err(|e| MetadataError::IoError(e.to_string()))?;

    // 读取时间刻度
    let mut timescale_bytes = [0u8; 4];
    reader.read_exact(&mut timescale_bytes)
        .map_err(|e| MetadataError::IoError(e.to_string()))?;
    let timescale = u32::from_be_bytes(timescale_bytes);

    // 读取时长
    let duration: u64 = if version == 1 {
        let mut duration_bytes = [0u8; 8];
        reader.read_exact(&mut duration_bytes)
            .map_err(|e| MetadataError::IoError(e.to_string()))?;
        u64::from_be_bytes(duration_bytes)
    } else {
        let mut duration_bytes = [0u8; 4];
        reader.read_exact(&mut duration_bytes)
            .map_err(|e| MetadataError::IoError(e.to_string()))?;
        u32::from_be_bytes(duration_bytes) as u64
    };

    if timescale > 0 {
        let duration_secs = duration as f64 / timescale as f64;
        metadata.duration = Some(Duration::from_secs_f64(duration_secs));
    }

    Ok(())
}

/// 在文件中查找指定类型的原子
fn find_atom<R: Read + Seek>(
    reader: &mut R,
    atom_type: &str,
    start_pos: u64,
) -> Result<Option<u64>, MetadataError> {
    reader.seek(SeekFrom::Start(start_pos))
        .map_err(|e| MetadataError::IoError(e.to_string()))?;

    loop {
        let pos = reader.stream_position()
            .map_err(|e| MetadataError::IoError(e.to_string()))?;

        if let Some(header) = read_atom_header(reader)? {
            if header.atom_type == atom_type {
                return Ok(Some(pos));
            }

            // 跳过这个原子
            let skip_size = header.size - header.data_offset;
            reader.seek(SeekFrom::Current(skip_size as i64))
                .map_err(|e| MetadataError::IoError(e.to_string()))?;
        } else {
            break;
        }
    }

    Ok(None)
}

/// 检测图片 MIME 类型
fn detect_image_mime(data: &[u8]) -> &'static str {
    if data.len() < 4 {
        return "application/octet-stream";
    }

    // JPEG: FF D8 FF
    if data[0] == 0xFF && data[1] == 0xD8 && data[2] == 0xFF {
        return "image/jpeg";
    }

    // PNG: 89 50 4E 47
    if &data[0..4] == b"\x89PNG" {
        return "image/png";
    }

    // GIF: GIF87a 或 GIF89a
    if &data[0..4] == b"GIF8" {
        return "image/gif";
    }

    // BMP: BM
    if &data[0..2] == b"BM" {
        return "image/bmp";
    }

    "application/octet-stream"
}

/// 获取 iTunes 流派名称
fn get_itunes_genre(id: usize) -> String {
    let genres = [
        "Blues", "Classic Rock", "Country", "Dance", "Disco", "Funk", "Grunge",
        "Hip-Hop/Rap", "House", "Jazz", "Metal", "New Age", "Oldies", "Other", "Pop",
        "R&B/Soul", "Reggae", "Rock", "Techno", "Industrial", "Alternative", "Ska",
        "Death Metal/Black Metal", "Pranks", "Soundtrack", "Euro-Techno", "Ambient",
        "Trip-Hop", "Vocal", "Jazz+Funk", "Fusion", "Trance", "Classical", "Instrumental",
        "Acid", "House", "Game", "Sound Clip", "Gospel", "Noise", "Alternative Rock",
        "Bass", "Soul", "Punk", "Space", "Meditative", "Instrumental Pop",
        "Instrumental Rock", "Ethnic", "Gothic", "Darkwave", "Techno-Industrial",
        "Electronic", "Pop-Folk", "Eurodance", "Dream", "Southern Rock", "Comedy",
        "Cult", "Gangsta", "Top 40", "Christian Rap", "Pop/Funk", "Jungle",
        "Native US", "Cabaret", "New Wave", "Psychadelic", "Rave", "Showtunes",
        "Trailer", "Lo-Fi", "Tribal", "Acid Punk", "Acid Jazz", "Polka", "Retro",
        "Musical", "Rock & Roll", "Hard Rock", "Folk", "Folk-Rock", "National Folk",
        "Swing", "Fast Fusion", "Bebop", "Latin", "Revival", "Celtic", "Bluegrass",
        "Avantgarde", "Gothic Rock", "Progressive Rock", "Psychedelic Rock",
        "Symphonic Rock", "Slow Rock", "Big Band", "Chorus", "Easy Listening",
        "Acoustic", "Humour", "Speech", "Chanson", "Opera", "Chamber Music",
        "Sonata", "Symphony", "Booty Bass", "Primus", "Porn Groove", "Satire",
        "Slow Jam", "Club", "Tango", "Samba", "Folklore", "Ballad", "Power Ballad",
        "Rhythmic Soul", "Freestyle", "Duet", "Punk Rock", "Drum Solo", "A Cappella",
        "Euro-House", "Dance Hall", "Goa", "Drum & Bass", "Club-House", "Hardcore Techno",
        "Terror", "Indie", "BritPop", "Afro-Punk", "Polsk Punk", "Beat",
        "Christian Gangsta Rap", "Heavy Metal", "Black Metal", "Crossover",
        "Contemporary Christian", "Christian Rock", "Merengue", "Salsa", "Thrash Metal",
        "Anime", "JPop", "Synthpop", "Abstract", "Art Rock", "Baroque", "Bhangra",
        "Big Beat", "Breakbeat", "Chillout", "Downtempo", "Dub", "EBM", "Eclectic",
        "Electro", "Electroclash", "Emo", "Experimental", "Garage", "Global", "IDM",
        "Illbient", "Industro-Goth", "Jam Band", "Krautrock", "Leftfield", "Lounge",
        "Math Rock", "New Romantic", "Nu-Breakz", "Post-Punk", "Post-Rock", "Psytrance",
        "Shoegaze", "Space Rock", "Trop Rock", "World Music", "Neoclassical", "Audiobook",
        "Audio Theatre", "Neue Deutsche Welle", "Podcast", "Indie Rock", "G-Funk",
        "Dubstep", "Garage Rock", "Psybient",
    ];

    genres.get(id.saturating_sub(1))
        .map(|&s| s.to_string())
        .unwrap_or_else(|| "Unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_image_mime() {
        let jpeg = vec![0xFF, 0xD8, 0xFF, 0xE0];
        assert_eq!(detect_image_mime(&jpeg), "image/jpeg");

        let png = vec![0x89, 0x50, 0x4E, 0x47];
        assert_eq!(detect_image_mime(&png), "image/png");

        let gif = vec![0x47, 0x49, 0x46, 0x38];
        assert_eq!(detect_image_mime(&gif), "image/gif");
    }

    #[test]
    fn test_get_itunes_genre() {
        assert_eq!(get_itunes_genre(1), "Blues");    // ID 1 对应 Blues
        assert_eq!(get_itunes_genre(17), "Reggae");  // ID 17 对应 Reggae
        assert_eq!(get_itunes_genre(18), "Rock");    // ID 18 对应 Rock
        assert_eq!(get_itunes_genre(0), "Blues");    // ID 0 被映射到 0，返回数组第一个元素
        assert_eq!(get_itunes_genre(999), "Unknown"); // 超出范围的 ID 返回 Unknown
    }
}
