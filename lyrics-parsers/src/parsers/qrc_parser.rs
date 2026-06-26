use regex::Regex;
use std::sync::LazyLock;
use lyrics_core::models::*;
use crate::parsers::attributes_helper;

static QRC_SYLLABLE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(.*?)\((\d+),(\d+)\)").unwrap());

pub fn parse(input: &str) -> LyricsData {
    let mut lyrics_lines: Vec<String> = input.trim().lines().map(|s| s.to_string()).collect();
    let mut data = LyricsData {
        track_metadata: Some(TrackMetadata::new()),
        file: Some(FileInfo {
            lyrics_type: LyricsTypes::Qrc,
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

pub fn parse_lyrics(lines: &[String], offset: Option<i32>) -> Vec<LineInfo> {
    let mut list: Vec<LineInfo> = Vec::new();

    for line in lines {
        if let Some(item) = parse_lyrics_line(line) {
            list.push(item);
        }
    }

    if let Some(offset_val) = offset {
        if offset_val != 0 {
            lyrics_core::helpers::offset_helper::add_offset(&mut list, offset_val);
        }
    }

    list
}

pub fn parse_lyrics_line(line: &str) -> Option<LineInfo> {
    let line = if let Some(bracket_pos) = line.find(']') {
        &line[bracket_pos + 1..]
    } else {
        line
    };

    let mut syllables: Vec<SyllableInfo> = Vec::new();

    for cap in QRC_SYLLABLE_RE.captures_iter(line) {
        if cap.len() == 4 {
            let text = cap[1].to_string();
            let start_time: i32 = cap[2].parse().ok()?;
            let duration: i32 = cap[3].parse().ok()?;
            let end_time = start_time + duration;

            syllables.push(SyllableInfo::new(text, start_time, end_time));
        }
    }

    Some(LineInfo::new_syllable(syllables))
}
