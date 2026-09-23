//! 歌词生成器库，支持将解析后的歌词模型导出为多种格式字符串。
//!
//! 支持的格式包括：LRC、QRC、KRC、YRC、Lyricify Syllable、Lyricify Lines。

pub mod krc_generator;
pub mod lrc_generator;
pub mod lyricify_lines_generator;
pub mod lyricify_syllable_generator;
pub mod qrc_generator;
pub mod yrc_generator;

use lyrics_core::models::*;
use lyrics_core::traits::LyricsGenerator;

/// 取出音节行的音节序列（合并音节已展开）与行首尾时间，非音节行返回 `None`。
///
/// 行时间优先取行头写的时间（如 QRC/KRC/YRC 的 `[开始时间,时长]`），没有则取首尾音节，
/// 因此生成的行头能还原出解析时读到的行时长。
/// 返回借用的音节引用，避免生成时深拷贝整行音节。
pub(crate) fn syllable_info(
    line: &LineInfo,
) -> Option<(Vec<&SyllableInfo>, Option<i32>, Option<i32>)> {
    let syllables = line.syllables()?;
    Some((
        syllables.iter().flat_map(SyllableItem::parts).collect(),
        line.start_time(),
        line.end_time(),
    ))
}

/// 依次对主行与子行调用 `append`，这是各逐字格式生成器共有的结构。
///
/// `append` 返回该行是否输出了内容；主行没有输出（非音节行）时整行跳过，子行也不再输出。
pub(crate) fn for_each_line_and_sub_line(
    lyrics_data: &LyricsData,
    mut append: impl FnMut(&LineInfo) -> bool,
) {
    let Some(lines) = lyrics_data.lines.as_ref() else {
        return;
    };

    for line in lines {
        if !append(line) {
            continue;
        }

        if let Some(sub) = line.sub_line() {
            append(sub);
        }
    }
}

/// 子歌词行的输出方式。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SubLinesOutputType {
    /// 子行内容合并到主行输出。
    InMainLine,
    /// 子行作为独立行输出。
    InDiffLine,
}

/// 为每种可生成的格式定义一个零大小的生成器类型，实现 [`LyricsGenerator`]。
///
/// 生成结果只有空白时视为没有可输出的内容，返回 `None`。
macro_rules! define_generators {
    ($($(#[$doc:meta])* $name:ident => $lyrics_type:ident, $module:ident;)*) => {
        $(
            $(#[$doc])*
            #[derive(Debug, Clone, Copy, Default)]
            pub struct $name;

            impl LyricsGenerator for $name {
                fn generate(&self, data: &LyricsData) -> Option<String> {
                    Some($module::generate(data)).filter(|result| !result.trim().is_empty())
                }

                fn lyrics_type(&self) -> LyricsTypes {
                    LyricsTypes::$lyrics_type
                }
            }
        )*
    };
}

define_generators! {
    /// Lyricify Syllable 生成器。
    LyricifySyllableGenerator => LyricifySyllable, lyricify_syllable_generator;
    /// Lyricify Lines 生成器（默认选项）。
    LyricifyLinesGenerator => LyricifyLines, lyricify_lines_generator;
    /// LRC 生成器（默认选项）。
    LrcGenerator => Lrc, lrc_generator;
    /// QRC 生成器。
    QrcGenerator => Qrc, qrc_generator;
    /// KRC 生成器。
    KrcGenerator => Krc, krc_generator;
    /// YRC 生成器。
    YrcGenerator => Yrc, yrc_generator;
}

/// 返回指定歌词类型对应的生成器，不支持生成的类型返回 `None`。
pub fn generator_for(lyrics_type: LyricsTypes) -> Option<&'static dyn LyricsGenerator> {
    Some(match lyrics_type {
        LyricsTypes::LyricifySyllable => &LyricifySyllableGenerator,
        LyricsTypes::LyricifyLines => &LyricifyLinesGenerator,
        LyricsTypes::Lrc => &LrcGenerator,
        LyricsTypes::Qrc => &QrcGenerator,
        LyricsTypes::Krc => &KrcGenerator,
        LyricsTypes::Yrc => &YrcGenerator,
        LyricsTypes::Ttml
        | LyricsTypes::Spotify
        | LyricsTypes::Musixmatch
        | LyricsTypes::Unknown => return None,
    })
}

/// 根据指定的歌词类型，将歌词数据生成为对应格式的字符串。
///
/// 返回 `None` 表示不支持的类型或生成结果为空。
pub fn generate_string(lyrics_data: &LyricsData, lyrics_type: LyricsTypes) -> Option<String> {
    generator_for(lyrics_type)?.generate(lyrics_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_generator_reports_the_type_it_is_registered_for() {
        for lyrics_type in [
            LyricsTypes::Unknown,
            LyricsTypes::LyricifySyllable,
            LyricsTypes::LyricifyLines,
            LyricsTypes::Lrc,
            LyricsTypes::Qrc,
            LyricsTypes::Krc,
            LyricsTypes::Yrc,
            LyricsTypes::Ttml,
            LyricsTypes::Spotify,
            LyricsTypes::Musixmatch,
        ] {
            if let Some(generator) = generator_for(lyrics_type) {
                assert_eq!(generator.lyrics_type(), lyrics_type);
            }
        }
    }

    #[test]
    fn empty_output_is_none() {
        assert_eq!(
            generate_string(&LyricsData::default(), LyricsTypes::Qrc),
            None
        );
    }
}
