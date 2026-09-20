use lyrics_core::models::*;
use serde::Deserialize;

/// YRC 信息行（如作词/作曲）的 JSON 结构。
#[derive(Debug, Deserialize)]
pub struct CreditsInfo {
    #[serde(rename = "t")]
    pub timestamp: i32,
    #[serde(rename = "c")]
    pub credits: Vec<Credit>,
}

/// YRC 信息行中的单个条目（如一位作词人）。
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

/// 从开头连续扫描信息行（`{...}` 的 JSON 行），返回解析到的信息行与歌词正文的起始位置。
fn scan_leading_credits(chars: &[char]) -> (Vec<CreditsInfo>, usize) {
    let mut credits = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        match chars[i] {
            '{' => {
                let end_index = chars[i..]
                    .iter()
                    .position(|&c| c == '\n')
                    .map_or(chars.len(), |index| index + i);
                let json_line: String = chars[i..end_index].iter().collect();
                if let Ok(parsed) = serde_json::from_str::<CreditsInfo>(&json_line) {
                    credits.push(parsed);
                }
                i = end_index;
            }
            '\n' | '\r' => i += 1,
            _ => break,
        }
    }

    (credits, i)
}

/// 从结尾反向连续扫描信息行，返回按原始顺序排列的信息行与歌词正文的结束位置（含）。
///
/// 整篇都是信息行时返回的下标为 `-1`。
fn scan_trailing_credits(chars: &[char]) -> (Vec<CreditsInfo>, i32) {
    let mut credits = Vec::new();
    let mut j = chars.len() as i32 - 1;

    while j >= 0 && (chars[j as usize] == '\n' || chars[j as usize] == '\r') {
        j -= 1;
    }

    while j >= 0 {
        match chars[j as usize] {
            '}' => {
                let start_index = chars[..=j as usize]
                    .iter()
                    .rposition(|&c| c == '\n')
                    .map_or(0, |position| position + 1);
                let json_line: String = chars[start_index..j as usize + 1].iter().collect();
                if let Ok(parsed) = serde_json::from_str::<CreditsInfo>(&json_line) {
                    credits.push(parsed);
                }
                j = start_index as i32 - 1;
            }
            '\n' | '\r' => j -= 1,
            _ => break,
        }
    }

    credits.reverse();
    (credits, j)
}

/// 把信息行拼接为一条带时间戳的歌词行。
fn credits_line(credits: &CreditsInfo) -> LineInfo {
    let text: String = credits.credits.iter().map(|c| c.text.as_str()).collect();
    LineInfo::new_line_with_time(text, credits.timestamp)
}

/// 「作词」信息行的其余条目即作者列表（`/` 是分隔符，不是作者）。
fn writers_from(credits: &CreditsInfo) -> Option<Vec<String>> {
    if !credits.credits.first()?.text.starts_with("作词") {
        return None;
    }

    Some(
        credits.credits[1..]
            .iter()
            .map(|c| c.text.clone())
            .filter(|text| text != "/")
            .collect(),
    )
}

/// 取歌词正文区间 `[start, end_exclusive)` 的逐音节歌词行。
fn parse_lyrics_span(chars: &[char], start: usize, end_exclusive: usize) -> Vec<LineInfo> {
    if start >= end_exclusive {
        return Vec::new();
    }

    let span: String = chars[start..end_exclusive].iter().collect();
    parse_only_lyrics(&span)
}

/// 解析 YRC 格式歌词，自动处理信息行（作词/作曲等）和逐音节歌词，返回 [`LyricsData`]。
pub fn parse(input: &str) -> LyricsData {
    let chars: Vec<char> = input.chars().collect();
    let (head_credits, lyrics_start) = scan_leading_credits(&chars);
    let (tail_credits, lyrics_end) = scan_trailing_credits(&chars);

    let mut sync_types = SyncTypes::SyllableSynced;
    let mut writers = None;

    for credits in head_credits.iter().chain(&tail_credits) {
        // 信息行是行同步的，与逐字正文混在一起即为混合同步。
        sync_types = SyncTypes::MixedSynced;
        if let Some(found) = writers_from(credits) {
            writers = Some(found);
        }
    }

    let mut lines: Vec<LineInfo> = head_credits.iter().map(credits_line).collect();
    lines.extend(parse_lyrics_span(
        &chars,
        lyrics_start,
        (lyrics_end + 1) as usize,
    ));
    lines.extend(tail_credits.iter().map(credits_line));

    LyricsData {
        file: Some(FileInfo {
            lyrics_type: LyricsTypes::Yrc,
            sync_types,
            additional_info: None,
        }),
        lines: Some(lines),
        writers,
        track_metadata: None,
    }
}

/// 解析 YRC 歌词内容，包含信息行和歌词行，返回歌词行列表。
///
/// 与 [`parse`] 不同：只保留开头的信息行，结尾的信息行仅用于定位正文范围。
pub fn parse_lyrics(input: &str) -> Vec<LineInfo> {
    let chars: Vec<char> = input.chars().collect();
    let (head_credits, lyrics_start) = scan_leading_credits(&chars);
    let (_, lyrics_end) = scan_trailing_credits(&chars);
    let lyrics_end = lyrics_end.max(lyrics_start as i32 - 1);

    let mut lines: Vec<LineInfo> = head_credits.iter().map(credits_line).collect();
    lines.extend(parse_lyrics_span(
        &chars,
        lyrics_start,
        (lyrics_end + 1) as usize,
    ));

    lines
}

/// 仅解析 YRC 歌词行部分（不含信息行），返回逐音节歌词行列表。
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
                lines.push(LineInfo::new_syllable(to_syllable_items(std::mem::take(
                    &mut karaoke_word_infos,
                ))));
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
                time_span_builder = time_span_builder
                    .wrapping_mul(10)
                    .wrapping_add(cur_char as i32 - '0' as i32);
            }
            CurrentState::LyricTimestamp => {
                time_span_builder = time_span_builder
                    .wrapping_mul(10)
                    .wrapping_add(cur_char as i32 - '0' as i32);
            }
            CurrentState::PossiblyWordTimestamp => {
                if cur_char.is_numeric() {
                    state = CurrentState::WordTimestamp;
                }
                time_span_builder = time_span_builder
                    .wrapping_mul(10)
                    .wrapping_add(cur_char as i32 - '0' as i32);
            }
            CurrentState::WordTimestamp => {
                time_span_builder = time_span_builder
                    .wrapping_mul(10)
                    .wrapping_add(cur_char as i32 - '0' as i32);
            }
            CurrentState::PossiblyLyricDuration => {
                if cur_char.is_numeric() {
                    state = CurrentState::LyricDuration;
                }
                time_span_builder = time_span_builder
                    .wrapping_mul(10)
                    .wrapping_add(cur_char as i32 - '0' as i32);
            }
            CurrentState::LyricDuration => {
                time_span_builder = time_span_builder
                    .wrapping_mul(10)
                    .wrapping_add(cur_char as i32 - '0' as i32);
            }
            CurrentState::PossiblyWordDuration => {
                if cur_char.is_numeric() {
                    state = CurrentState::WordDuration;
                }
                time_span_builder = time_span_builder
                    .wrapping_mul(10)
                    .wrapping_add(cur_char as i32 - '0' as i32);
            }
            CurrentState::WordDuration => {
                time_span_builder = time_span_builder
                    .wrapping_mul(10)
                    .wrapping_add(cur_char as i32 - '0' as i32);
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
            lines.push(LineInfo::new_syllable(to_syllable_items(std::mem::take(
                &mut karaoke_word_infos,
            ))));
            lyric_string_builder.clear();
        }

        i += 1;
    }

    lines
}
