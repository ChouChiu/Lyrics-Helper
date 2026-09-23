//! 核心 trait 定义，包括歌词解析器、生成器和解密器接口。

pub mod decrypter;
pub mod generator;
pub mod parser;

pub use decrypter::DecryptError;
pub use decrypter::LyricsDecrypter;
pub use generator::LyricsGenerator;
pub use parser::LyricsParser;
