pub mod attributes_helper;
pub mod krc_parser;
pub mod lrc_parser;
pub mod lyricify_lines_parser;
pub mod lyricify_syllable_parser;
pub mod musixmatch_parser;
pub mod qrc_parser;
pub mod spotify_parser;
pub mod ttml_parser;
pub mod yrc_parser;

use lyrics_core::models::*;

/// 根据指定的歌词原始类型解析歌词内容，返回解析后的 [`LyricsData`]。
///
/// 与上游 `ParseHelper.ParseLyrics(string, LyricsRawTypes)` 一致：`Unknown` 以及只能由
/// 上层展开的原始信封类型 [`LyricsRawTypes::QrcFull`]、[`LyricsRawTypes::YrcFull`]、
/// [`LyricsRawTypes::AppleJson`] 返回 `None`，调用方需先从 XML/JSON 信封中取出歌词字符串
/// 再交给本函数。
pub fn parse_lyrics(input: &str, raw_type: LyricsRawTypes) -> Option<LyricsData> {
    match raw_type {
        LyricsRawTypes::LyricifySyllable => Some(lyricify_syllable_parser::parse(input)),
        LyricsRawTypes::LyricifyLines => Some(lyricify_lines_parser::parse(input)),
        LyricsRawTypes::Lrc => Some(lrc_parser::parse(input)),
        LyricsRawTypes::Qrc => Some(qrc_parser::parse(input)),
        LyricsRawTypes::Krc => Some(krc_parser::parse(input)),
        LyricsRawTypes::Yrc => Some(yrc_parser::parse(input)),
        LyricsRawTypes::Ttml => Some(ttml_parser::parse(input)),
        LyricsRawTypes::Spotify => spotify_parser::parse(input),
        LyricsRawTypes::Musixmatch => musixmatch_parser::parse(input),
        LyricsRawTypes::QrcFull | LyricsRawTypes::YrcFull | LyricsRawTypes::AppleJson => None,
        LyricsRawTypes::Unknown => None,
    }
}

/// 自动检测歌词格式并解析歌词内容，返回解析后的 [`LyricsData`]。
///
/// 对应上游 `ParseHelper.ParseLyrics(string)`：先检测原始类型再分发，检测失败或检测到
/// 无法直接解析的原始信封类型（`QrcFull`/`YrcFull`/`AppleJson`）时返回 `None`。
pub fn parse_lyrics_auto(input: &str) -> Option<LyricsData> {
    let raw_type = lyrics_core::helpers::type_helper::get_lyrics_types(input);
    parse_lyrics(input, raw_type)
}
