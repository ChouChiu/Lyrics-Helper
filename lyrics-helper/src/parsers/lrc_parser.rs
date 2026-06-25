use crate::models::*;

#[derive(Debug, Clone, Copy, PartialEq)]
enum CurrentState {
    None,
    AwaitingState,
    AwaitingStateLyric,
    Attribute,
    AttributeContent,
    Timestamp,
    PossiblyLyric,
    Lyric,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum TimeStampType {
    Minutes,
    Seconds,
    Milliseconds,
    None,
}

pub fn parse(input: &str) -> LyricsData {
    let mut lines: Vec<LineInfo> = Vec::new();
    let mut attribute_name = String::new();
    let mut attributes: Vec<(String, String)> = Vec::new();
    let mut track_metadata = TrackMetadata::new();
    let mut cur_state_start_position = 0;
    let mut time_calculation_cache = 0i32;
    let mut cur_timestamps = vec![-1i32; 64];
    let mut cur_timestamp = 0i32;
    let mut current_timestamp_position = 0;
    let mut offset = 0i32;
    let mut reaches_end = false;
    let mut last_character_is_line_break = false;
    let mut state = CurrentState::None;
    let mut time_stamp_type = TimeStampType::None;

    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let cur_char = chars[i];

        // 剥离开, 方便分支预测
        if state == CurrentState::Lyric {
            if cur_char != '\n' && cur_char != '\r' && i + 1 < chars.len() {
                i += 1;
                continue;
            } else {
                if i + 1 < chars.len() {
                    let text: String = chars[cur_state_start_position + 1..i].iter().collect();
                    let trimmed = text.trim().to_string();
                    for ts in cur_timestamps.iter().take_while(|&&t| t != -1) {
                        lines.push(LineInfo::new_line_with_time(
                            trimmed.clone(),
                            *ts - offset,
                        ));
                    }
                    if i + 1 < chars.len() && (chars[i + 1] == '\n' || chars[i + 1] == '\r') {
                        i += 1;
                    }
                    current_timestamp_position = 0;
                    state = CurrentState::None;
                    i += 1;
                    continue;
                }
                if i + 1 == chars.len() {
                    reaches_end = true;
                    if cur_char == '\r' || cur_char == '\n' {
                        last_character_is_line_break = true;
                    }
                }
            }
        }

        if reaches_end && state == CurrentState::Lyric {
            let text: String = chars[cur_state_start_position + 1..i].iter().collect();
            let trim_end = if last_character_is_line_break { 1 } else { 0 };
            let trimmed = text[..text.len() - trim_end].trim().to_string();
            for ts in cur_timestamps.iter().take_while(|&&t| t != -1) {
                lines.push(LineInfo::new_line_with_time(
                    trimmed.clone(),
                    *ts - offset,
                ));
            }
            i += 1;
            continue;
        }

        match state {
            CurrentState::PossiblyLyric => {
                if cur_char == '[' {
                    state = CurrentState::AwaitingStateLyric;
                } else {
                    i -= 1;
                    state = CurrentState::Lyric;
                }
            }
            CurrentState::None => {
                cur_timestamps.fill(-1);
                if cur_char == '[' {
                    state = CurrentState::AwaitingState;
                }
            }
            CurrentState::AwaitingState => {
                if cur_char.is_ascii_digit() {
                    state = CurrentState::Timestamp;
                    time_stamp_type = TimeStampType::Minutes;
                    cur_state_start_position = i;
                    cur_timestamp = 0;
                    i -= 1;
                } else {
                    state = CurrentState::Attribute;
                    cur_state_start_position = i;
                }
            }
            CurrentState::AwaitingStateLyric => {
                if cur_char.is_ascii_digit() {
                    state = CurrentState::Timestamp;
                    time_stamp_type = TimeStampType::Minutes;
                    cur_state_start_position = i;
                    cur_timestamp = 0;
                    i -= 1;
                } else {
                    state = CurrentState::Lyric;
                    cur_state_start_position = i.saturating_sub(2);
                }
            }
            CurrentState::Attribute => {
                if cur_char == ':' {
                    attribute_name = chars[cur_state_start_position..i].iter().collect();
                    cur_state_start_position = i + 1;
                    state = CurrentState::AttributeContent;
                }
            }
            CurrentState::AttributeContent => {
                if cur_char == ']' {
                    let attribute_value = if attribute_name == "offset" {
                        offset = time_calculation_cache;
                        time_calculation_cache = 0;
                        offset.to_string()
                    } else {
                        chars[cur_state_start_position..i].iter().collect()
                    };
                    match attribute_name.as_str() {
                        "ar" => track_metadata.artist = Some(attribute_value.clone()),
                        "al" => track_metadata.album = Some(attribute_value.clone()),
                        "ti" => track_metadata.title = Some(attribute_value.clone()),
                        "length" => {
                            if let Ok(result) = attribute_value.parse::<i32>() {
                                track_metadata.duration_ms = Some(result);
                            }
                        }
                        _ => {}
                    }
                    attributes.push((attribute_name.clone(), attribute_value));
                    attribute_name.clear();
                    state = CurrentState::None;
                }
                if attribute_name == "offset" && cur_char != ']' {
                    time_calculation_cache = time_calculation_cache * 10 + (cur_char as i32 - '0' as i32);
                    i += 1;
                    continue;
                }
            }
            CurrentState::Timestamp => {
                if time_stamp_type == TimeStampType::Milliseconds {
                    if cur_char != ']' {
                        time_calculation_cache = time_calculation_cache * 10 + (cur_char as i32 - '0' as i32);
                        i += 1;
                        continue;
                    } else {
                        let pow = i - cur_state_start_position - 1;
                        cur_timestamp += time_calculation_cache * 10i32.pow(3 - pow as u32);
                        if current_timestamp_position + 1 >= cur_timestamps.len() {
                            cur_timestamps.extend(vec![-1i32; cur_timestamps.len()]);
                        }
                        cur_timestamps[current_timestamp_position] = cur_timestamp;
                        current_timestamp_position += 1;
                        time_stamp_type = TimeStampType::None;
                        time_calculation_cache = 0;
                        cur_state_start_position = i;
                        cur_timestamp = 0;
                        state = CurrentState::PossiblyLyric;
                        i += 1;
                        continue;
                    }
                }
                match cur_char {
                    ':' | '.' => {
                        if time_stamp_type == TimeStampType::Minutes {
                            cur_timestamp = (cur_timestamp + time_calculation_cache) * 60;
                            time_calculation_cache = 0;
                            time_stamp_type = TimeStampType::Seconds;
                            i += 1;
                            continue;
                        }
                        if time_stamp_type == TimeStampType::Seconds {
                            cur_timestamp = (cur_timestamp + time_calculation_cache) * 1000;
                            cur_state_start_position = i;
                            time_calculation_cache = 0;
                            time_stamp_type = TimeStampType::Milliseconds;
                            i += 1;
                            continue;
                        }
                    }
                    ']' => {
                        if current_timestamp_position + 1 >= cur_timestamps.len() {
                            cur_timestamps.extend(vec![-1i32; cur_timestamps.len()]);
                        }
                        cur_timestamps[current_timestamp_position] = (cur_timestamp + time_calculation_cache) * 1000;
                        current_timestamp_position += 1;
                        time_calculation_cache = 0;
                        cur_state_start_position = i;
                        cur_timestamp = 0;
                        state = CurrentState::PossiblyLyric;
                        time_stamp_type = TimeStampType::None;
                        i += 1;
                        continue;
                    }
                    _ => {
                        time_calculation_cache = time_calculation_cache * 10 + (cur_char as i32 - '0' as i32);
                    }
                }
            }
            _ => {} // Lyric state is handled at the top of the loop
        }

        i += 1;
    }

    lines.sort();

    LyricsData {
        track_metadata: Some(track_metadata),
        lines: Some(lines),
        file: Some(FileInfo {
            lyrics_type: LyricsTypes::Lrc,
            sync_types: SyncTypes::LineSynced,
            additional_info: Some(AdditionalFileInfo::General {
                attributes,
            }),
        }),
        writers: None,
    }
}

pub fn parse_lyrics(input: &str) -> Vec<LineInfo> {
    let mut lines: Vec<LineInfo> = Vec::new();
    let mut cur_state_start_position = 0;
    let mut time_calculation_cache = 0i32;
    let mut cur_timestamps = vec![-1i32; 64];
    let mut cur_timestamp = 0i32;
    let mut current_timestamp_position = 0;
    let mut reaches_end = false;
    let mut last_character_is_line_break = false;
    let mut state = CurrentState::None;
    let mut time_stamp_type = TimeStampType::None;

    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let cur_char = chars[i];

        if state == CurrentState::Lyric {
            if cur_char != '\n' && cur_char != '\r' && i + 1 < chars.len() {
                i += 1;
                continue;
            } else {
                if i + 1 < chars.len() {
                    let text: String = chars[cur_state_start_position + 1..i].iter().collect();
                    let trimmed = text.trim().to_string();
                    for ts in cur_timestamps.iter().take_while(|&&t| t != -1) {
                        lines.push(LineInfo::new_line_with_time(trimmed.clone(), *ts));
                    }
                    if i + 1 < chars.len() && (chars[i + 1] == '\n' || chars[i + 1] == '\r') {
                        i += 1;
                    }
                    current_timestamp_position = 0;
                    state = CurrentState::None;
                    i += 1;
                    continue;
                }
                if i + 1 == chars.len() {
                    reaches_end = true;
                    if cur_char == '\r' || cur_char == '\n' {
                        last_character_is_line_break = true;
                    }
                }
            }
        }

        if reaches_end {
            let text: String = chars[cur_state_start_position + 1..i].iter().collect();
            let trim_end = if last_character_is_line_break { 1 } else { 0 };
            let trimmed = text[..text.len() - trim_end].trim().to_string();
            for ts in cur_timestamps.iter().take_while(|&&t| t != -1) {
                lines.push(LineInfo::new_line_with_time(trimmed.clone(), *ts));
            }
            i += 1;
            continue;
        }

        match state {
            CurrentState::PossiblyLyric => {
                if cur_char == '[' {
                    state = CurrentState::AwaitingStateLyric;
                } else {
                    i -= 1;
                    state = CurrentState::Lyric;
                }
            }
            CurrentState::None => {
                cur_timestamps.fill(-1);
                if cur_char == '[' {
                    state = CurrentState::AwaitingState;
                }
            }
            CurrentState::AwaitingState => {
                if cur_char.is_ascii_digit() {
                    state = CurrentState::Timestamp;
                    time_stamp_type = TimeStampType::Minutes;
                    cur_state_start_position = i;
                    cur_timestamp = 0;
                    i -= 1;
                } else {
                    state = CurrentState::Attribute;
                    cur_state_start_position = i;
                }
            }
            CurrentState::AwaitingStateLyric => {
                if cur_char.is_ascii_digit() {
                    state = CurrentState::Timestamp;
                    time_stamp_type = TimeStampType::Minutes;
                    cur_state_start_position = i;
                    cur_timestamp = 0;
                    i -= 1;
                } else {
                    state = CurrentState::Lyric;
                    cur_state_start_position = i.saturating_sub(2);
                }
            }
            CurrentState::Attribute => {
                if cur_char == ':' {
                    state = CurrentState::AttributeContent;
                }
            }
            CurrentState::AttributeContent => {
                if cur_char == ']' {
                    state = CurrentState::None;
                }
            }
            CurrentState::Timestamp => {
                if time_stamp_type == TimeStampType::Milliseconds {
                    if cur_char != ']' {
                        time_calculation_cache = time_calculation_cache * 10 + (cur_char as i32 - '0' as i32);
                        i += 1;
                        continue;
                    } else {
                        let pow = i - cur_state_start_position - 1;
                        cur_timestamp += time_calculation_cache * 10i32.pow(3 - pow as u32);
                        if current_timestamp_position + 1 >= cur_timestamps.len() {
                            cur_timestamps.extend(vec![-1i32; cur_timestamps.len()]);
                        }
                        cur_timestamps[current_timestamp_position] = cur_timestamp;
                        current_timestamp_position += 1;
                        time_stamp_type = TimeStampType::None;
                        time_calculation_cache = 0;
                        cur_state_start_position = i;
                        cur_timestamp = 0;
                        state = CurrentState::PossiblyLyric;
                        i += 1;
                        continue;
                    }
                }
                match cur_char {
                    ':' | '.' => {
                        if time_stamp_type == TimeStampType::Minutes {
                            cur_timestamp = (cur_timestamp + time_calculation_cache) * 60;
                            time_calculation_cache = 0;
                            time_stamp_type = TimeStampType::Seconds;
                            i += 1;
                            continue;
                        }
                        if time_stamp_type == TimeStampType::Seconds {
                            cur_timestamp = (cur_timestamp + time_calculation_cache) * 1000;
                            cur_state_start_position = i;
                            time_calculation_cache = 0;
                            time_stamp_type = TimeStampType::Milliseconds;
                            i += 1;
                            continue;
                        }
                    }
                    ']' => {
                        if current_timestamp_position + 1 >= cur_timestamps.len() {
                            cur_timestamps.extend(vec![-1i32; cur_timestamps.len()]);
                        }
                        cur_timestamps[current_timestamp_position] = (cur_timestamp + time_calculation_cache) * 1000;
                        current_timestamp_position += 1;
                        time_calculation_cache = 0;
                        cur_state_start_position = i;
                        cur_timestamp = 0;
                        state = CurrentState::PossiblyLyric;
                        time_stamp_type = TimeStampType::None;
                        i += 1;
                        continue;
                    }
                    _ => {
                        time_calculation_cache = time_calculation_cache * 10 + (cur_char as i32 - '0' as i32);
                    }
                }
            }
            _ => {} // Lyric state is handled at the top of the loop
        }

        i += 1;
    }

    lines.sort();
    lines
}
