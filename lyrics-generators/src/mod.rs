pub mod lrc_generator;
pub mod qrc_generator;
pub mod yrc_generator;
pub mod krc_generator;
pub mod lyricify_syllable_generator;
pub mod lyricify_lines_generator;

use lyrics_core::models::*;

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
