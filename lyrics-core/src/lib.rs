//! Lyricify 歌词核心库
//!
//! 提供歌词的解析、生成、解密和辅助工具功能，支持多种歌词格式（LRC、QRC、KRC、YRC、TTML、Spotify 等）。

pub mod models;
pub mod traits;
pub mod helpers;

pub use models::*;
