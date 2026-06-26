use regex::Regex;
use lyrics_core::models::*;
use crate::parsers::attributes_helper;

/// 解析 Lyricify Syllable 格式歌词，支持背景人声检测和对齐信息，返回 [`LyricsData`]。
pub fn parse(input: &str) -> LyricsData {
    let input = input.trim_start_matches('\u{feff}');
    let mut lyrics_lines: Vec<String> = input.trim().lines().map(|s| s.to_string()).collect();
    let mut data = LyricsData {
        track_metadata: Some(TrackMetadata::new()),
        file: Some(FileInfo {
            lyrics_type: LyricsTypes::LyricifySyllable,
            sync_types: SyncTypes::SyllableSynced,
            additional_info: Some(AdditionalFileInfo::new_general()),
        }),
        lines: None,
        writers: None,
    };

    let offset = attributes_helper::parse_general_attributes_to_lyrics_data_from_lines(&mut data, &mut lyrics_lines);
    let lines = parse_lyrics(&lyrics_lines, offset);
    data.lines = Some(lines);
    data
}

/// 解析 Lyricify Syllable 歌词行列表，处理背景人声和对齐信息，可选地应用时间偏移。
pub fn parse_lyrics(lines: &[String], offset: Option<i32>) -> Vec<LineInfo> {
    let mut list: Vec<(LineInfo, Option<bool>)> = Vec::new();

    for line in lines {
        if let Some((item, is_bg)) = parse_lyrics_line_with_state(line) {
            list.push((item, is_bg));
        }
    }

    // Set background vocals
    let mut new_list = set_background_vocals_info(list);

    if let Some(offset_val) = offset {
        if offset_val != 0 {
            lyrics_core::helpers::offset_helper::add_offset(&mut new_list, offset_val);
        }
    }

    new_list
}

fn parse_lyrics_line_with_state(line: &str) -> Option<(LineInfo, Option<bool>)> {
    let re = Regex::new(r"(.*?)\((\d+),(\d+)\)").ok()?;
    let mut syllables: Vec<SyllableInfo> = Vec::new();
    let mut is_background_vocals: Option<bool> = None;
    let mut alignment = LyricsAlignment::Unspecified;

    let line = line.trim_start_matches('\u{feff}');
    let line_to_parse = if let Some(bracket_pos) = line.find(']') {
        let properties = &line[..bracket_pos];
        let mut chars = properties.chars();
        if chars.next() == Some('[') {
            let prop_str: String = chars.collect();
            if lyrics_core::helpers::string_helper::is_number(&prop_str) {
                if let Ok(p) = prop_str.parse::<i32>() {
                    // Read preset background vocals
                    if p >= 6 {
                        is_background_vocals = Some(true);
                    } else if p >= 3 {
                        is_background_vocals = Some(false);
                    }

                    // Read preset duet view
                    alignment = match p % 3 {
                        0 => LyricsAlignment::Unspecified,
                        1 => LyricsAlignment::Left,
                        2 => LyricsAlignment::Right,
                        _ => LyricsAlignment::Unspecified,
                    };
                }
            }
        }
        &line[bracket_pos + 1..]
    } else {
        line
    };

    for cap in re.captures_iter(line_to_parse) {
        if cap.len() == 4 {
            let text = cap[1].to_string();
            let start_time: i32 = cap[2].parse().ok()?;
            let duration: i32 = cap[3].parse().ok()?;
            let end_time = start_time + duration;
            syllables.push(SyllableInfo::new(text, start_time, end_time));
        }
    }

    if syllables.is_empty() {
        return None;
    }

    let mut line_info = LineInfo::new_syllable(syllables);
    line_info.set_alignment(alignment);

    Some((line_info, is_background_vocals))
}

fn set_background_vocals_info(list: Vec<(LineInfo, Option<bool>)>) -> Vec<LineInfo> {
    let mut items: Vec<(LineInfo, Option<bool>)> = list;

    // Set already marked background vocals
    let mut i = 1;
    while i < items.len() {
        if items[i].1 == Some(true) {
            let sub_line = items[i].0.clone();
            items[i - 1].0.set_sub_line(Some(Box::new(sub_line)));
            items.remove(i);
        } else {
            i += 1;
        }
    }

    // Detect unmarked background vocals (bracketed lyrics)
    let is_not_bg = |item: &(LineInfo, Option<bool>)| -> bool {
        item.1.is_none() && !is_bracketed_lyrics(&item.0) || item.1 == Some(false)
    };

    let mut i = 1;
    while i < items.len() {
        if items[i].1.is_none() && is_bracketed_lyrics(&items[i].0)
            && is_not_bg(&items[i - 1]) && items[i].0.sub_line().is_none()
            && (i + 1 >= items.len() || is_not_bg(&items[i + 1]))
        {
            let sub_line = items[i].0.clone();
            items[i - 1].0.set_sub_line(Some(Box::new(sub_line)));
            items.remove(i);
        }
        i += 1;
    }

    items.into_iter().map(|(line, _)| line).collect()
}

fn is_bracketed_lyrics(line: &LineInfo) -> bool {
    let text = line.text_from_any();
    (text.starts_with('(') || text.starts_with('（')) && (text.ends_with(')') || text.ends_with('）'))
}
