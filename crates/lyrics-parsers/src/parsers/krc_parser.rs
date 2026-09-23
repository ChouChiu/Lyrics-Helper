use crate::parsers::{apply_offset, lyrics_data, parse_with_attributes};
use base64::Engine;
use lyrics_core::models::*;
use serde::Deserialize;
use std::collections::HashMap;

/// 酷狗 KRC 翻译数据的顶层结构。
#[derive(Debug, Deserialize)]
pub struct KugouTranslation {
    pub content: Option<Vec<KugouTranslationContent>>,
}

/// 酷狗 KRC 翻译内容条目，包含翻译类型和逐行歌词内容。
#[derive(Debug, Deserialize)]
pub struct KugouTranslationContent {
    #[serde(rename = "type")]
    pub content_type: i32,
    #[serde(rename = "lyricContent")]
    pub lyric_content: Option<Vec<Option<Vec<String>>>>,
}

/// 解析 KRC 格式歌词，自动提取属性、翻译和逐音节信息，返回 [`LyricsData`]。
pub fn parse(input: &str) -> LyricsData {
    let mut data = parse_with_attributes(
        lyrics_data(
            LyricsTypes::Krc,
            SyncTypes::SyllableSynced,
            Some(AdditionalFileInfo::new_krc()),
        ),
        get_splited_krc(input),
        parse_lyrics_from_lines,
    );
    if let Some(lines) = data.lines.as_mut() {
        apply_translations(lines, input);
    }
    data
}

/// 仅解析 KRC 歌词内容（不含属性行），返回包含翻译的歌词行列表。
pub fn parse_lyrics(input: &str) -> Vec<LineInfo> {
    let lyrics_lines = get_splited_krc_without_info_line(input);
    let mut lyrics = parse_lyrics_from_lines(&lyrics_lines, None);
    apply_translations(&mut lyrics, input);

    lyrics
}

/// 把 KRC 内嵌的逐行翻译合并进音节行，使其升级为 [`LineInfo::FullSyllable`]。
///
/// 占位符 `"//"` 与空字符串表示该行没有翻译，此时仍升级为 Full 变体但翻译为空，
/// 与上游 `KrcParser` 一致。
fn apply_translations(lyrics: &mut [LineInfo], input: &str) {
    let Some(translations) = get_translation_from_krc(input) else {
        return;
    };

    for (line, text) in lyrics.iter_mut().zip(translations) {
        if !matches!(line, LineInfo::Syllable { .. }) {
            continue;
        }

        let mut map = HashMap::new();
        if !text.is_empty() && text != "//" {
            map.insert("zh".to_string(), text);
        }

        // 原地换变体：先取出整行，避免深拷贝音节与子行。
        let owned = std::mem::replace(line, LineInfo::new_line_simple(String::new()));
        *line = owned.to_full_syllable(map, None);
    }
}

/// 从预分割的 KRC 歌词行列表解析歌词，可选地应用时间偏移。
///
/// 只有以 `[` 开头、行头与首个音节时间都能读出的行才会保留。
pub fn parse_lyrics_from_lines(lyrics_lines: &[String], offset: Option<i32>) -> Vec<LineInfo> {
    let mut lyrics: Vec<LineInfo> = lyrics_lines
        .iter()
        .filter_map(|line| parse_lyrics_line(line))
        .collect();
    apply_offset(&mut lyrics, offset);
    lyrics
}

/// 按 KRC 的换行约定切分文本：`\r\n` 视作换行，单独的 `\r` 直接丢弃。
fn split_lines(krc: &str) -> impl Iterator<Item = &str> {
    krc.split('\n').map(|line| line.trim_end_matches('\r'))
}

/// 将 KRC 原始文本按行分割，仅保留以 `[` 开头的有效歌词行。
pub fn get_splited_krc(krc: &str) -> Vec<String> {
    split_lines(krc)
        .filter(|line| line.starts_with('['))
        .map(str::to_string)
        .collect()
}

/// 将 KRC 原始文本按行分割，仅保留带时间戳的歌词行（不含属性信息行）。
pub fn get_splited_krc_without_info_line(krc: &str) -> Vec<String> {
    split_lines(krc)
        .filter(|line| {
            line.len() >= 5 && line.starts_with('[') && line.as_bytes()[1].is_ascii_digit()
        })
        .map(str::to_string)
        .collect()
}

/// 解析单行 KRC 歌词，提取逐音节时间信息，返回单个 [`LineInfo`]。
///
/// 行格式为 `[行开始时间,行时长]<相对开始,时长,0>文本<...>文本`。行头的开始时间或首个
/// 音节的时间读不出来时返回 `None`；后续音节的时间读不出来时沿用前一个音节的时间，
/// 与上游一致。
///
/// 与 QRC 一样，行头写的行时长会一并写进 [`LineInfo`]：行时间与音节时间相互独立，
/// 末个音节唱完不等于这行该消失。行时长读不出来时行结束时间回退到末个音节。
pub fn parse_lyrics_line(line: &str) -> Option<LineInfo> {
    let (header, body) = line.strip_prefix('[')?.split_once(']')?;
    let mut header = header.split(',');
    let line_start: i32 = header.next()?.parse().ok()?;
    let line_end = header
        .next()
        .and_then(|duration| duration.parse::<i32>().ok())
        .map(|duration| line_start + duration);

    let mut words = body.split(",0>");
    let mut first_time = words.next()?.strip_prefix('<')?.split(',');
    let mut start: i32 = first_time.next()?.parse().ok()?;
    let mut duration: i32 = first_time.next()?.parse().ok()?;

    let mut syllables: Vec<SyllableInfo> = Vec::new();
    for word in words {
        // 每段是「本音节文本<下一音节的时间」，最后一段没有 `<`。
        let (text, next_time) = match word.rsplit_once('<') {
            Some((text, time)) => (text, Some(time)),
            None => (word, None),
        };

        syllables.push(SyllableInfo::new(
            text.to_string(),
            line_start + start,
            line_start + start + duration,
        ));

        if let Some(time) = next_time {
            let mut time = time.split(',');
            start = time.next().and_then(|s| s.parse().ok()).unwrap_or(start);
            duration = time.next().and_then(|s| s.parse().ok()).unwrap_or(duration);
        }
    }

    Some(LineInfo::new_syllable_with_time(
        to_syllable_items(syllables),
        Some(line_start),
        line_end,
    ))
}

/// 检查 KRC 歌词是否包含翻译内容（通过 base64 编码的 `[language]` 标签）。
pub fn check_krc_translation(krc: &str) -> bool {
    get_translation_raw_from_krc(krc)
        .and_then(|translation| translation.content)
        .is_some_and(|content| !content.is_empty())
}

/// 从 KRC 歌词中提取原始翻译结构体 [`KugouTranslation`]。
pub fn get_translation_raw_from_krc(krc: &str) -> Option<KugouTranslation> {
    let start = krc.find("[language:")? + "[language:".len();
    let end = krc[start..].find(']')? + start;

    let decoded = base64::engine::general_purpose::STANDARD
        .decode(&krc[start..end])
        .ok()?;
    let decoded = String::from_utf8(decoded).ok()?;

    serde_json::from_str(&decoded).ok()
}

/// 从 KRC 歌词中提取翻译文本列表（逐行翻译）。
pub fn get_translation_from_krc(krc: &str) -> Option<Vec<String>> {
    let content = get_translation_raw_from_krc(krc)?.content?;
    let content_item = content.iter().find(|item| item.content_type == 1)?;

    Some(
        content_item
            .lyric_content
            .as_ref()?
            .iter()
            .flatten()
            .filter_map(|lines| lines.first().cloned())
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_relative_syllable_times() {
        let line = parse_lyrics_line("[1000,800]<0,300,0>Hel<300,500,0>lo").unwrap();
        let syllables = line.syllables().unwrap();
        assert_eq!(
            syllables
                .iter()
                .map(|s| (s.text(), s.start_time(), s.end_time()))
                .collect::<Vec<_>>(),
            vec![
                ("Hel".to_string(), 1000, 1300),
                ("lo".to_string(), 1300, 1800)
            ]
        );
    }

    #[test]
    fn keeps_the_line_duration_written_in_the_header() {
        // 末个音节 1800 唱完，但行头说这行显示到 3000。
        let line = parse_lyrics_line("[1000,2000]<0,300,0>Hel<300,500,0>lo").unwrap();

        assert_eq!(line.start_time(), Some(1000));
        assert_eq!(line.end_time(), Some(3000));
        assert_eq!(
            line.syllables().unwrap().last().unwrap().end_time(),
            1800,
            "音节时间不应被行时间改写"
        );
    }

    #[test]
    fn falls_back_to_syllables_without_a_line_duration() {
        let line = parse_lyrics_line("[1000]<0,300,0>Hel<300,500,0>lo").unwrap();
        assert_eq!(
            (line.start_time(), line.end_time()),
            (Some(1000), Some(1800))
        );
    }

    /// 畸形行返回 `None`，不能在切片时 panic。
    #[test]
    fn rejects_malformed_lines_without_panicking() {
        for line in [
            "",
            "]",
            "[",
            "[]",
            "[0,1]",
            "[x,1]<0,1,0>a",
            "[0,1]0,1,0>a",
            "[0,1]<é",
        ] {
            assert!(parse_lyrics_line(line).is_none(), "{line:?}");
        }
    }
}
