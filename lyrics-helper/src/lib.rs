//! Lyricify 歌词助手 —— 统一的歌词解析、生成、解密和搜索门面库。
//!
//! 本 crate 聚合了底层各子 crate 的功能，提供简洁的顶层 API。
//!
//! # 示例
//!
//! ```rust,no_run
//! use lyrics_helper::{parse_auto, generate_string, LyricsTypes};
//!
//! let lrc_text = "[00:01.00]Hello World";
//! let lyrics = parse_auto(lrc_text).unwrap();
//! let output = generate_string(&lyrics, LyricsTypes::Lrc).unwrap();
//! assert!(output.contains("Hello World"));
//! ```

pub use lyrics_core::*;
pub use lyrics_parsers as parsers;
pub use lyrics_generators as generators;
pub use lyrics_crypto as decrypter;

#[cfg(feature = "search")]
pub use lyrics_search as search;
#[cfg(feature = "search")]
pub use lyrics_search::searchers;
#[cfg(feature = "search")]
pub use lyrics_search::providers;

pub use lyrics_parsers::parsers::parse_lyrics as parse;
pub use lyrics_parsers::parsers::parse_lyrics_auto as parse_auto;
pub use lyrics_generators::generate_string;

pub use lyrics_crypto::decrypter::qrc::decrypter::decrypt_lyrics as decrypt_qrc;
pub use lyrics_crypto::decrypter::krc::decrypter::decrypt_lyrics as decrypt_krc;
