use serde::Deserialize;
use crate::models::*;

#[derive(Debug, Deserialize)]
pub struct CreditsInfo {
    #[serde(rename = "t")]
    pub timestamp: i32,
    #[serde(rename = "c")]
    pub credits: Vec<Credit>,
}

#[derive(Debug, Deserialize)]
pub struct Credit {
    #[serde(rename = "tx")]
    pub text: String,
    #[serde(rename = "li", default)]
    pub image: String,
    #[serde(rename = "or", default)]
    pub orpheus: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum CurrentState {
    None,
    LyricTimestamp,
    WordTimestamp,
    LyricDuration,
    WordDuration,
    WordUnknownItem,
    PossiblyLyricDuration,
    PossiblyWordDuration,
    PossiblyLyricTimestamp,
    PossiblyWordTimestamp,
    Lyric,
}

pub fn parse(input: &str) -> LyricsData {
    let mut lyrics_data = LyricsData {
        file: Some(FileInfo {
            lyrics_type: LyricsTypes::Yrc,
            sync_types: SyncTypes::SyllableSynced,
            additional_info: None,
        }),
        lines: Some(Vec::new()),
        writers: None,
        track_metadata: None,
    };

    let mut lines: Vec<LineInfo> = Vec::new();
    let chars: Vec<char> = input.chars().collect();

    // 处理信息行 (开头)
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '{' {
            let end_index = input[i..].find('\n').map(|idx| idx + i).unwrap_or(chars.len());
            let json_line: String = chars[i..end_index].iter().collect();
            if let Ok(credits) = serde_json::from_str::<CreditsInfo>(&json_line) {
                let text: String = credits.credits.iter().map(|c| c.text.as_str()).collect();
                lines.push(LineInfo::new_line_with_time(text, credits.timestamp));

                if let Some(ref mut file) = lyrics_data.file {
                    file.sync_types = SyncTypes::MixedSynced;
                }

                if !credits.credits.is_empty() && credits.credits[0].text.starts_with("作词") {
                    lyrics_data.writers = Some(
                        credits.credits[1..]
                            .iter()
                            .map(|c| c.text.clone())
                            .filter(|t| t != "/")
                            .collect(),
                    );
                }
            }
            i = end_index;
        } else if chars[i] == '\n' || chars[i] == '\r' {
            i += 1;
        } else {
            break;
        }
    }

    // 处理信息行 (末尾)
    let mut j = chars.len() as i32 - 1;
    while j >= 0 && (chars[j as usize] == '\n' || chars[j as usize] == '\r') {
        j -= 1;
    }

    let mut end_credits: Vec<LineInfo> = Vec::new();
    while j >= 0 {
        if chars[j as usize] == '}' {
            let start_index = {
                let pos = input[..j as usize + 1].rfind('\n');
                match pos {
                    Some(p) => p + 1,
                    None => 0,
                }
            };
            let json_line: String = chars[start_index..j as usize + 1].iter().collect();
            if let Ok(credits) = serde_json::from_str::<CreditsInfo>(&json_line) {
                let text: String = credits.credits.iter().map(|c| c.text.as_str()).collect();
                end_credits.push(LineInfo::new_line_with_time(text, credits.timestamp));

                if let Some(ref mut file) = lyrics_data.file {
                    file.sync_types = SyncTypes::MixedSynced;
                }

                if !credits.credits.is_empty() && credits.credits[0].text.starts_with("作词") {
                    lyrics_data.writers = Some(
                        credits.credits[1..]
                            .iter()
                            .map(|c| c.text.clone())
                            .filter(|t| t != "/")
                            .collect(),
                    );
                }
            }
            j = start_index as i32 - 1;
        } else if chars[j as usize] == '\n' || chars[j as usize] == '\r' {
            j -= 1;
        } else {
            break;
        }
    }

    // 处理歌词行
    let lyrics_end = (j + 1) as usize;
    if i < lyrics_end {
        let lyrics_span: String = chars[i..lyrics_end].iter().collect();
        let lyrics_list = parse_only_lyrics(&lyrics_span);
        lines.extend(lyrics_list);
    }

    end_credits.reverse();
    lines.extend(end_credits);

    lyrics_data.lines = Some(lines);
    lyrics_data
}

pub fn parse_lyrics(input: &str) -> Vec<LineInfo> {
    let mut lines: Vec<LineInfo> = Vec::new();
    let chars: Vec<char> = input.chars().collect();

    // 处理信息行
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '{' {
            let end_index = input[i..].find('\n').map(|idx| idx + i).unwrap_or(chars.len());
            let json_line: String = chars[i..end_index].iter().collect();
            if let Ok(credits) = serde_json::from_str::<CreditsInfo>(&json_line) {
                let text: String = credits.credits.iter().map(|c| c.text.as_str()).collect();
                lines.push(LineInfo::new_line_with_time(text, credits.timestamp));
            }
            i = end_index;
        } else if chars[i] == '\n' || chars[i] == '\r' {
            i += 1;
        } else {
            break;
        }
    }

    let mut j = chars.len() as i32 - 1;
    while j >= 0 && (chars[j as usize] == '\n' || chars[j as usize] == '\r') {
        j -= 1;
    }
    while j >= 0 {
        if chars[j as usize] == '}' {
            let start_index = {
                let pos = input[..j as usize + 1].rfind('\n');
                match pos {
                    Some(p) => p + 1,
                    None => 0,
                }
            };
            j = start_index as i32 - 1;
        } else if chars[j as usize] == '\n' || chars[j as usize] == '\r' {
            j -= 1;
        } else {
            break;
        }
    }
    if j < i as i32 {
        j = i as i32 - 1;
    }

    let lyrics_end = (j + 1) as usize;
    if i < lyrics_end {
        let lyrics_span: String = chars[i..lyrics_end].iter().collect();
        let lyrics_list = parse_only_lyrics(&lyrics_span);
        lines.extend(lyrics_list);
    }

    lines
}

pub fn parse_only_lyrics(input: &str) -> Vec<LineInfo> {
    let mut lines: Vec<LineInfo> = Vec::new();
    let mut karaoke_word_infos: Vec<SyllableInfo> = Vec::new();
    let mut time_span_builder = 0i32;
    let mut lyric_string_builder = String::new();
    let mut word_timespan = 0i32;
    let mut word_duration = 0i32;
    let mut state = CurrentState::None;
    let mut reaches_end = false;

    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let cur_char = chars[i];

        if cur_char == '\n' || cur_char == '\r' || i + 1 == chars.len() {
            if i + 1 < chars.len() {
                if i + 1 < chars.len() && (chars[i + 1] == '\n' || chars[i + 1] == '\r') {
                    i += 1;
                }
                karaoke_word_infos.push(SyllableInfo::new(
                    lyric_string_builder.clone(),
                    word_timespan,
                    word_timespan + word_duration,
                ));
                lines.push(LineInfo::new_syllable(karaoke_word_infos.clone()));
                karaoke_word_infos.clear();
                lyric_string_builder.clear();
                state = CurrentState::None;
                i += 1;
                continue;
            }
            if i + 1 == chars.len() {
                reaches_end = true;
            }
        }

        match cur_char {
            '[' => {
                if state == CurrentState::Lyric {
                    if i + 1 < chars.len() && !chars[i + 1].is_numeric() {
                        // Not a timestamp, treat as text
                    } else {
                        state = CurrentState::PossiblyLyricTimestamp;
                        i += 1;
                        continue;
                    }
                }
                state = CurrentState::PossiblyLyricTimestamp;
                i += 1;
                continue;
            }
            ',' => {
                if state == CurrentState::Lyric {
                    if i + 1 < chars.len() && !chars[i + 1].is_numeric() {
                        // Not a timestamp
                    } else {
                        state = CurrentState::PossiblyLyricDuration;
                        time_span_builder = 0;
                        i += 1;
                        continue;
                    }
                }
                if state == CurrentState::LyricTimestamp {
                    state = CurrentState::PossiblyLyricDuration;
                    time_span_builder = 0;
                } else if state == CurrentState::WordTimestamp {
                    state = CurrentState::PossiblyWordDuration;
                    word_timespan = time_span_builder;
                    time_span_builder = 0;
                } else {
                    state = CurrentState::WordUnknownItem;
                    word_duration = time_span_builder;
                    time_span_builder = 0;
                }
                i += 1;
                continue;
            }
            ']' => {
                if state == CurrentState::Lyric {
                    if i + 1 < chars.len() && !chars[i + 1].is_numeric() {
                        // Not a timestamp
                    } else {
                        state = CurrentState::None;
                        time_span_builder = 0;
                        i += 1;
                        continue;
                    }
                }
                state = CurrentState::None;
                time_span_builder = 0;
                i += 1;
                continue;
            }
            '(' => {
                if state == CurrentState::Lyric {
                    if i + 1 < chars.len() && !chars[i + 1].is_numeric() {
                        // Not a timestamp
                    } else {
                        karaoke_word_infos.push(SyllableInfo::new(
                            lyric_string_builder.clone(),
                            word_timespan,
                            word_timespan + word_duration,
                        ));
                        lyric_string_builder.clear();
                        state = CurrentState::PossiblyWordTimestamp;
                        i += 1;
                        continue;
                    }
                }
                state = CurrentState::PossiblyWordTimestamp;
                i += 1;
                continue;
            }
            ')' => {
                if state == CurrentState::Lyric {
                    if i + 1 < chars.len() && !chars[i + 1].is_numeric() {
                        // Not a timestamp
                    } else {
                        state = CurrentState::Lyric;
                        i += 1;
                        continue;
                    }
                }
                state = CurrentState::Lyric;
                i += 1;
                continue;
            }
            _ => {}
        }

        match state {
            CurrentState::PossiblyLyricTimestamp => {
                if cur_char.is_numeric() {
                    state = CurrentState::LyricTimestamp;
                }
                time_span_builder = time_span_builder.wrapping_mul(10).wrapping_add(cur_char as i32 - '0' as i32);
            }
            CurrentState::LyricTimestamp => {
                time_span_builder = time_span_builder.wrapping_mul(10).wrapping_add(cur_char as i32 - '0' as i32);
            }
            CurrentState::PossiblyWordTimestamp => {
                if cur_char.is_numeric() {
                    state = CurrentState::WordTimestamp;
                }
                time_span_builder = time_span_builder.wrapping_mul(10).wrapping_add(cur_char as i32 - '0' as i32);
            }
            CurrentState::WordTimestamp => {
                time_span_builder = time_span_builder.wrapping_mul(10).wrapping_add(cur_char as i32 - '0' as i32);
            }
            CurrentState::PossiblyLyricDuration => {
                if cur_char.is_numeric() {
                    state = CurrentState::LyricDuration;
                }
                time_span_builder = time_span_builder.wrapping_mul(10).wrapping_add(cur_char as i32 - '0' as i32);
            }
            CurrentState::LyricDuration => {
                time_span_builder = time_span_builder.wrapping_mul(10).wrapping_add(cur_char as i32 - '0' as i32);
            }
            CurrentState::PossiblyWordDuration => {
                if cur_char.is_numeric() {
                    state = CurrentState::WordDuration;
                }
                time_span_builder = time_span_builder.wrapping_mul(10).wrapping_add(cur_char as i32 - '0' as i32);
            }
            CurrentState::WordDuration => {
                time_span_builder = time_span_builder.wrapping_mul(10).wrapping_add(cur_char as i32 - '0' as i32);
            }
            CurrentState::Lyric => {
                if reaches_end && (cur_char == '\n' || cur_char == '\r') {
                    // Skip
                } else {
                    lyric_string_builder.push(cur_char);
                }
            }
            _ => {}
        }

        if reaches_end {
            karaoke_word_infos.push(SyllableInfo::new(
                lyric_string_builder.clone(),
                word_timespan,
                word_timespan + word_duration,
            ));
            lines.push(LineInfo::new_syllable(karaoke_word_infos.clone()));
            karaoke_word_infos.clear();
            lyric_string_builder.clear();
        }

        i += 1;
    }

    lines
}
