pub mod attributes_helper;
pub mod lrc_parser;
pub mod qrc_parser;
pub mod yrc_parser;
pub mod krc_parser;
pub mod ttml_parser;
pub mod spotify_parser;
pub mod musixmatch_parser;
pub mod lyricify_syllable_parser;
pub mod lyricify_lines_parser;

use lyrics_core::models::*;

pub fn parse_lyrics(input: &str, raw_type: LyricsRawTypes) -> Option<LyricsData> {
    match raw_type {
        LyricsRawTypes::LyricifySyllable => Some(lyricify_syllable_parser::parse(input)),
        LyricsRawTypes::LyricifyLines => Some(lyricify_lines_parser::parse(input)),
        LyricsRawTypes::Lrc => Some(lrc_parser::parse(input)),
        LyricsRawTypes::Qrc | LyricsRawTypes::QrcFull => Some(qrc_parser::parse(input)),
        LyricsRawTypes::Krc => Some(krc_parser::parse(input)),
        LyricsRawTypes::Yrc | LyricsRawTypes::YrcFull => Some(yrc_parser::parse(input)),
        LyricsRawTypes::Ttml => Some(ttml_parser::parse(input)),
        LyricsRawTypes::Spotify => spotify_parser::parse(input),
        LyricsRawTypes::Musixmatch => musixmatch_parser::parse(input),
        LyricsRawTypes::AppleJson => Some(ttml_parser::parse(input)), // Apple uses TTML
        LyricsRawTypes::Unknown => None,
    }
}

pub fn parse_lyrics_auto(input: &str) -> Option<LyricsData> {
    let raw_type = lyrics_core::helpers::type_helper::get_lyrics_types(input);
    parse_lyrics(input, raw_type)
}
