//! 歌词优化工具模块，提供各格式歌词的标准化、清理和降级功能。

pub mod apple_music;
pub mod explicit;
pub mod info_lines;
pub mod musixmatch;
pub mod syllable_word_merger;
pub mod sync_downgrade;
mod utf16;
pub mod yrc;
