use crate::models::*;
use crate::parsers::attributes_helper;

pub fn parse(input: &str) -> LyricsData {
    let input = input.replace("[type:LyricifyLines]", "");
    let mut lyrics_lines: Vec<String> = input.trim().lines().map(|s| s.to_string()).collect();
    let mut data = LyricsData {
        track_metadata: Some(TrackMetadata::new()),
        file: Some(FileInfo {
            lyrics_type: LyricsTypes::LyricifyLines,
            sync_types: SyncTypes::LineSynced,
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
    let offset = offset.unwrap_or(0);
    let mut lyrics_array = Vec::new();

    for line in lines {
        if !line.starts_with('[') || !line.contains(',') || !line.contains(']') {
            continue;
        }

        if let Some((begin, end, text)) = parse_line(line) {
            lyrics_array.push(LineInfo::new_line(
                text.trim().to_string(),
                Some(begin - offset),
                Some(end - offset),
            ));
        }
    }

    lyrics_array
}

fn parse_line(line: &str) -> Option<(i32, i32, String)> {
    let bracket_start = line.find('[')?;
    let comma = line.find(',')?;
    let bracket_end = line.find(']')?;

    if bracket_start >= comma || comma >= bracket_end {
        return None;
    }

    let begin: i32 = line[bracket_start + 1..comma].parse().ok()?;
    let end: i32 = line[comma + 1..bracket_end].parse().ok()?;
    let text = line[bracket_end + 1..].to_string();

    Some((begin, end, text))
}
