use crate::parsers::attributes_helper;
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
    let mut lyrics_lines = get_splited_krc(input);
    let mut data = LyricsData {
        file: Some(FileInfo {
            lyrics_type: LyricsTypes::Krc,
            sync_types: SyncTypes::SyllableSynced,
            additional_info: Some(AdditionalFileInfo::new_krc()),
        }),
        track_metadata: Some(TrackMetadata::new()),
        lines: None,
        writers: None,
    };

    let offset = attributes_helper::parse_general_attributes_to_lyrics_data_from_lines(
        &mut data,
        &mut lyrics_lines,
    );
    let mut lyrics = parse_lyrics_from_lines(&lyrics_lines, offset);
    apply_translations(&mut lyrics, input);

    data.lines = Some(lyrics);
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
pub fn parse_lyrics_from_lines(lyrics_lines: &[String], offset: Option<i32>) -> Vec<LineInfo> {
    let mut lyrics: Vec<LineInfo> = Vec::new();

    for line in lyrics_lines {
        if line.starts_with('[') {
            if let Some(l) = parse_lyrics_line(line) {
                lyrics.push(l);
            }
        }
    }

    if let Some(offset_val) = offset {
        if offset_val != 0 {
            lyrics_core::helpers::offset_helper::add_offset(&mut lyrics, offset_val);
        }
    }

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
pub fn parse_lyrics_line(line: &str) -> Option<LineInfo> {
    let bracket_end = line.find(']')?;
    let after_bracket = &line[bracket_end + 1..];
    let words: Vec<&str> = after_bracket.split(",0>").collect();

    if words.is_empty() {
        return None;
    }

    let line_time_str = &line[1..bracket_end];
    let line_time: Vec<&str> = line_time_str.split(',').collect();
    let line_start: i32 = line_time.first()?.parse().ok()?;

    let mut syllables: Vec<SyllableInfo> = Vec::new();

    // First word
    let first_word = words[0];
    let first_time_str = &first_word[1..]; // Skip the '<'
    let first_time: Vec<&str> = first_time_str.split(',').collect();
    let mut start: i32 = first_time.first()?.parse().ok()?;
    let mut duration: i32 = first_time.get(1)?.parse().ok()?;

    for &word in words.iter().skip(1) {
        let (text, next_start, next_duration) = if word.contains('<') {
            let last_lt = word.rfind('<').unwrap_or(word.len());
            let text = &word[..last_lt];
            let time_str = &word[last_lt + 1..];
            let time: Vec<&str> = time_str.split(',').collect();
            let ns: i32 = time.first().and_then(|s| s.parse().ok()).unwrap_or(start);
            let nd: i32 = time.get(1).and_then(|s| s.parse().ok()).unwrap_or(duration);
            (text, Some(ns), Some(nd))
        } else {
            (word, None, None)
        };

        syllables.push(SyllableInfo::new(
            text.to_string(),
            line_start + start,
            line_start + start + duration,
        ));

        if let (Some(ns), Some(nd)) = (next_start, next_duration) {
            start = ns;
            duration = nd;
        }
    }

    Some(LineInfo::new_syllable(to_syllable_items(syllables)))
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
