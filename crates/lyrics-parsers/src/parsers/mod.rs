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

use lyrics_core::helpers::offset_helper;
use lyrics_core::models::*;
use lyrics_core::traits::LyricsParser;

/// 为每种可直接解析的格式定义一个零大小的解析器类型，实现 [`LyricsParser`]。
macro_rules! define_parsers {
    ($($(#[$doc:meta])* $name:ident => $raw_type:ident, |$input:ident| $parse:expr;)*) => {
        $(
            $(#[$doc])*
            #[derive(Debug, Clone, Copy, Default)]
            pub struct $name;

            impl LyricsParser for $name {
                fn parse(&self, $input: &str) -> Option<LyricsData> {
                    $parse
                }

                fn raw_type(&self) -> LyricsRawTypes {
                    LyricsRawTypes::$raw_type
                }
            }
        )*
    };
}

define_parsers! {
    /// Lyricify Syllable 解析器。
    LyricifySyllableParser => LyricifySyllable, |input| Some(lyricify_syllable_parser::parse(input));
    /// Lyricify Lines 解析器。
    LyricifyLinesParser => LyricifyLines, |input| Some(lyricify_lines_parser::parse(input));
    /// LRC 解析器。
    LrcParser => Lrc, |input| Some(lrc_parser::parse(input));
    /// QRC 解析器（解密后的正文，不含 XML 信封）。
    QrcParser => Qrc, |input| Some(qrc_parser::parse(input));
    /// KRC 解析器（解密后的正文）。
    KrcParser => Krc, |input| Some(krc_parser::parse(input));
    /// YRC 解析器。
    YrcParser => Yrc, |input| Some(yrc_parser::parse(input));
    /// Apple Music TTML 解析器。
    TtmlParser => Ttml, |input| Some(ttml_parser::parse(input));
    /// Spotify JSON 解析器，JSON 无效时返回 `None`。
    SpotifyParser => Spotify, |input| spotify_parser::parse(input);
    /// Musixmatch JSON 解析器，没有任何可用歌词时返回 `None`。
    MusixmatchParser => Musixmatch, |input| musixmatch_parser::parse(input);
}

/// 返回指定原始类型对应的解析器。
///
/// `Unknown` 以及只能由上层展开的原始信封类型 [`LyricsRawTypes::QrcFull`]、
/// [`LyricsRawTypes::YrcFull`]、[`LyricsRawTypes::AppleJson`] 没有解析器。
pub fn parser_for(raw_type: LyricsRawTypes) -> Option<&'static dyn LyricsParser> {
    Some(match raw_type {
        LyricsRawTypes::LyricifySyllable => &LyricifySyllableParser,
        LyricsRawTypes::LyricifyLines => &LyricifyLinesParser,
        LyricsRawTypes::Lrc => &LrcParser,
        LyricsRawTypes::Qrc => &QrcParser,
        LyricsRawTypes::Krc => &KrcParser,
        LyricsRawTypes::Yrc => &YrcParser,
        LyricsRawTypes::Ttml => &TtmlParser,
        LyricsRawTypes::Spotify => &SpotifyParser,
        LyricsRawTypes::Musixmatch => &MusixmatchParser,
        LyricsRawTypes::QrcFull
        | LyricsRawTypes::YrcFull
        | LyricsRawTypes::AppleJson
        | LyricsRawTypes::Unknown => return None,
    })
}

/// 根据指定的歌词原始类型解析歌词内容，返回解析后的 [`LyricsData`]。
///
/// 与上游 `ParseHelper.ParseLyrics(string, LyricsRawTypes)` 一致：没有解析器的类型
/// （见 [`parser_for`]）返回 `None`，调用方需先从 XML/JSON 信封中取出歌词字符串
/// 再交给本函数。
pub fn parse_lyrics(input: &str, raw_type: LyricsRawTypes) -> Option<LyricsData> {
    parser_for(raw_type)?.parse(input)
}

/// 自动检测歌词格式并解析歌词内容，返回解析后的 [`LyricsData`]。
///
/// 对应上游 `ParseHelper.ParseLyrics(string)`：先检测原始类型再分发，检测失败或检测到
/// 无法直接解析的原始信封类型（`QrcFull`/`YrcFull`/`AppleJson`）时返回 `None`。
pub fn parse_lyrics_auto(input: &str) -> Option<LyricsData> {
    let raw_type = lyrics_core::helpers::type_helper::get_lyrics_types(input);
    parse_lyrics(input, raw_type)
}

/// 构造只填好文件信息的 [`LyricsData`]，歌词行与元数据由各解析器随后写入。
pub(crate) fn lyrics_data(
    lyrics_type: LyricsTypes,
    sync_types: SyncTypes,
    additional_info: Option<AdditionalFileInfo>,
) -> LyricsData {
    LyricsData {
        file: Some(FileInfo {
            lyrics_type,
            sync_types,
            additional_info,
        }),
        ..LyricsData::default()
    }
}

/// 按行格式共用的解析流程：先吃掉开头的 `[key:value]` 属性行，再把剩余行与属性里的
/// `offset` 交给 `parse_lines`。
pub(crate) fn parse_with_attributes(
    mut data: LyricsData,
    mut lines: Vec<String>,
    parse_lines: impl FnOnce(&[String], Option<i32>) -> Vec<LineInfo>,
) -> LyricsData {
    let offset = attributes_helper::parse_general_attributes_to_lyrics_data_from_lines(
        &mut data, &mut lines,
    );
    data.lines = Some(parse_lines(&lines, offset));
    data
}

/// 把属性里的 `offset` 应用到歌词行上。
pub(crate) fn apply_offset(lines: &mut [LineInfo], offset: Option<i32>) {
    if let Some(offset) = offset {
        offset_helper::add_offset(lines, offset);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_parser_reports_the_raw_type_it_is_registered_for() {
        let all = [
            LyricsRawTypes::Unknown,
            LyricsRawTypes::LyricifySyllable,
            LyricsRawTypes::LyricifyLines,
            LyricsRawTypes::Lrc,
            LyricsRawTypes::Qrc,
            LyricsRawTypes::QrcFull,
            LyricsRawTypes::Krc,
            LyricsRawTypes::Yrc,
            LyricsRawTypes::YrcFull,
            LyricsRawTypes::Ttml,
            LyricsRawTypes::AppleJson,
            LyricsRawTypes::Spotify,
            LyricsRawTypes::Musixmatch,
        ];

        for raw_type in all {
            if let Some(parser) = parser_for(raw_type) {
                assert_eq!(parser.raw_type(), raw_type);
            }
        }
        assert!(parser_for(LyricsRawTypes::QrcFull).is_none());
    }
}
