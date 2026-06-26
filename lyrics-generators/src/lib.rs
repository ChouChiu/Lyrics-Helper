//! 歌词生成器库，支持将解析后的歌词模型导出为多种格式字符串。
//!
//! 支持的格式包括：LRC、QRC、KRC、YRC、Lyricify Syllable、Lyricify Lines。

pub mod lrc_generator;
pub mod qrc_generator;
pub mod yrc_generator;
pub mod krc_generator;
pub mod lyricify_syllable_generator;
pub mod lyricify_lines_generator;

use lyrics_core::models::*;

/// 子歌词行的输出方式。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SubLinesOutputType {
    /// 子行内容合并到主行输出。
    InMainLine,
    /// 子行作为独立行输出。
    InDiffLine,
}

/// 根据指定的歌词类型，将歌词数据生成为对应格式的字符串。
///
/// 返回 `None` 表示不支持的类型或生成结果为空。
pub fn generate_string(lyrics_data: &LyricsData, lyrics_type: LyricsTypes) -> Option<String> {
    let result = match lyrics_type {
        LyricsTypes::LyricifySyllable => lyricify_syllable_generator::generate(lyrics_data),
        LyricsTypes::LyricifyLines => lyricify_lines_generator::generate(lyrics_data),
        LyricsTypes::Lrc => lrc_generator::generate(lyrics_data),
        LyricsTypes::Qrc => qrc_generator::generate(lyrics_data),
        LyricsTypes::Krc => krc_generator::generate(lyrics_data),
        LyricsTypes::Yrc => yrc_generator::generate(lyrics_data),
        _ => return None,
    };

    if result.trim().is_empty() {
        None
    } else {
        Some(result)
    }
}
