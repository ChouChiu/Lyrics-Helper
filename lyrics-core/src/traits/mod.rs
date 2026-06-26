//! 核心 trait 定义，包括歌词解析器、生成器和解密器接口。

pub mod parser;
pub mod generator;
pub mod decrypter;

pub use parser::LyricsParser;
pub use generator::LyricsGenerator;
pub use decrypter::LyricsDecrypter;
pub use decrypter::DecryptError;
