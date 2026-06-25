use base64::Engine;
use serde::Deserialize;
use crate::models::*;
use crate::parsers::attributes_helper;

#[derive(Debug, Deserialize)]
pub struct KugouTranslation {
    pub content: Option<Vec<KugouTranslationContent>>,
}

#[derive(Debug, Deserialize)]
pub struct KugouTranslationContent {
    #[serde(rename = "type")]
    pub content_type: i32,
    #[serde(rename = "lyricContent")]
    pub lyric_content: Option<Vec<Option<Vec<String>>>>,
}

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

    let offset = attributes_helper::parse_general_attributes_to_lyrics_data_from_lines(&mut data, &mut lyrics_lines);
    let mut lyrics = parse_lyrics_from_lines(&lyrics_lines, offset);

    if check_krc_translation(input) {
        if let Some(lyrics_trans) = get_translation_from_krc(input) {
            for i in 0..lyrics.len().min(lyrics_trans.len()) {
                let t = if lyrics_trans[i] != "//" {
                    lyrics_trans[i].clone()
                } else {
                    String::new()
                };
                if let LineInfo::Syllable { syllables, alignment, sub_line } = &lyrics[i] {
                    let syllables = syllables.clone();
                    let alignment = *alignment;
                    let sub_line = sub_line.clone();
                    let mut translations = std::collections::HashMap::new();
                    if !t.is_empty() {
                        translations.insert("zh".to_string(), t);
                    }
                    lyrics[i] = LineInfo::FullSyllable {
                        syllables,
                        alignment,
                        sub_line,
                        translations,
                        pronunciation: None,
                    };
                }
            }
        }
    }

    data.lines = Some(lyrics);
    data
}

pub fn parse_lyrics(input: &str) -> Vec<LineInfo> {
    let lyrics_lines = get_splited_krc_without_info_line(input);
    let mut lyrics = parse_lyrics_from_lines(&lyrics_lines, None);

    if check_krc_translation(input) {
        if let Some(lyrics_trans) = get_translation_from_krc(input) {
            for i in 0..lyrics.len().min(lyrics_trans.len()) {
                let t = if lyrics_trans[i] != "//" {
                    lyrics_trans[i].clone()
                } else {
                    String::new()
                };
                if let LineInfo::Syllable { syllables, alignment, sub_line } = &lyrics[i] {
                    let syllables = syllables.clone();
                    let alignment = *alignment;
                    let sub_line = sub_line.clone();
                    let mut translations = std::collections::HashMap::new();
                    if !t.is_empty() {
                        translations.insert("zh".to_string(), t);
                    }
                    lyrics[i] = LineInfo::FullSyllable {
                        syllables,
                        alignment,
                        sub_line,
                        translations,
                        pronunciation: None,
                    };
                }
            }
        }
    }

    lyrics
}

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
            crate::helpers::offset_helper::add_offset(&mut lyrics, offset_val);
        }
    }

    lyrics
}

pub fn get_splited_krc(krc: &str) -> Vec<String> {
    let binding = krc.replace("\r\n", "\n").replace('\r', "");
    let lines: Vec<&str> = binding.split('\n').collect();
    let mut result = String::new();
    for line in lines {
        if line.starts_with('[') {
            result.push_str(line);
            result.push('\n');
        }
    }
    result.replace("\r\n", "\n").replace('\r', "").split('\n').map(|s| s.to_string()).collect()
}

pub fn get_splited_krc_without_info_line(krc: &str) -> Vec<String> {
    let binding = krc.replace("\r\n", "\n").replace('\r', "");
    let lines: Vec<&str> = binding.split('\n').collect();
    let mut result = String::new();
    for line in lines {
        if line.starts_with('[') && line.len() >= 5 {
            let second_char = line.chars().nth(1).unwrap_or(' ');
            if second_char.is_ascii_digit() {
                result.push_str(line);
                result.push('\n');
            }
        }
    }
    result.replace("\r\n", "\n").replace('\r', "").split('\n').map(|s| s.to_string()).collect()
}

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

    Some(LineInfo::new_syllable(syllables))
}

pub fn check_krc_translation(krc: &str) -> bool {
    if !krc.contains("[language:") {
        return false;
    }

    let Some(start_pos) = krc.find("[language:") else {
        return false;
    };
    let start = start_pos + "[language:".len();
    let Some(end_pos) = krc[start..].find(']') else {
        return false;
    };
    let end = end_pos + start;
    let language = &krc[start..end];

    let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(language) else {
        return false;
    };
    let Ok(decode) = String::from_utf8(decoded) else {
        return false;
    };
    let Ok(translation) = serde_json::from_str::<KugouTranslation>(&decode) else {
        return false;
    };

    translation.content.as_ref().is_some_and(|c| !c.is_empty())
}

pub fn get_translation_from_krc(krc: &str) -> Option<Vec<String>> {
    if !krc.contains("[language:") {
        return None;
    }

    let start = krc.find("[language:")? + "[language:".len();
    let end = krc[start..].find(']')? + start;
    let language = &krc[start..end];

    let decoded = base64::engine::general_purpose::STANDARD.decode(language).ok()?;
    let decode = String::from_utf8(decoded).ok()?;
    let translation: KugouTranslation = serde_json::from_str(&decode).ok()?;

    let content = translation.content?;
    if content.is_empty() {
        return None;
    }

    let content_item = content.iter().find(|c| c.content_type == 1)?;
    let lyric_content = content_item.lyric_content.as_ref()?;

    let mut result = Vec::new();
    for lines in lyric_content.iter().flatten() {
        if let Some(first) = lines.first() {
            result.push(first.clone());
        }
    }

    Some(result)
}

pub fn get_translation_raw_from_krc(krc: &str) -> Option<KugouTranslation> {
    if !krc.contains("[language:") {
        return None;
    }

    let start = krc.find("[language:")? + "[language:".len();
    let end = krc[start..].find(']')? + start;
    let language = &krc[start..end];

    let decoded = base64::engine::general_purpose::STANDARD.decode(language).ok()?;
    let decode = String::from_utf8(decoded).ok()?;
    serde_json::from_str(&decode).ok()
}
