//! 解析器模块

pub mod flac;
pub mod mp3;
pub mod m4a;
pub mod ogg;
pub mod wav;

pub use flac::FlacParser;
pub use mp3::Mp3Parser;
pub use m4a::M4aParser;
pub use ogg::OggParser;
pub use wav::WavParser;