use crate::models::*;

pub fn parse_general_attributes_to_lyrics_data_from_lines(
    data: &mut LyricsData,
    lines: &mut Vec<String>,
) -> Option<i32> {
    let mut offset: Option<i32> = None;
    if data.track_metadata.is_none() {
        data.track_metadata = Some(TrackMetadata::new());
    }

    let i = 0;
    while i < lines.len() {
        if is_attribute_line(&lines[i]) {
            let (key, value) = get_attribute(&lines[i]);
            if let Some(ref mut meta) = data.track_metadata {
                match key.as_str() {
                    "ar" => meta.artist = Some(value.clone()),
                    "al" => meta.album = Some(value.clone()),
                    "ti" => meta.title = Some(value.clone()),
                    "length" => {
                        if let Ok(result) = value.parse::<i32>() {
                            meta.duration_ms = Some(result);
                        }
                    }
                    "offset" => {
                        if let Ok(result) = value.parse::<i32>() {
                            offset = Some(result);
                        }
                    }
                    _ => {}
                }
            }

            // Add to attributes
            if key == "hash" {
                if let Some(ref mut file) = data.file {
                    if let Some(AdditionalFileInfo::Krc { hash, .. }) = &mut file.additional_info {
                        *hash = Some(value.clone());
                    }
                }
            } else {
                if let Some(ref mut file) = data.file {
                    if let Some(attrs) = file.additional_info.as_mut().and_then(|ai| ai.attributes_mut()) {
                        attrs.push((key, value));
                    }
                }
            }

            lines.remove(i);
        } else {
            break;
        }
    }

    offset
}

pub fn parse_general_attributes_to_lyrics_data(
    data: &mut LyricsData,
    input: &str,
) -> (Option<i32>, usize) {
    let mut offset: Option<i32> = None;
    let mut index = 0;

    if data.track_metadata.is_none() {
        data.track_metadata = Some(TrackMetadata::new());
    }

    let chars: Vec<char> = input.chars().collect();
    while index < chars.len() {
        if chars[index] == '[' {
            let end_index = input[index..].find('\n').map(|i| i + index).unwrap_or(chars.len());
            let info_line: String = chars[index..end_index].iter().collect();

            if is_attribute_line(&info_line) {
                let (key, value) = get_attribute(&info_line);
                if let Some(ref mut meta) = data.track_metadata {
                    match key.as_str() {
                        "ar" => meta.artist = Some(value.clone()),
                        "al" => meta.album = Some(value.clone()),
                        "ti" => meta.title = Some(value.clone()),
                        "length" => {
                            if let Ok(result) = value.parse::<i32>() {
                                meta.duration_ms = Some(result);
                            }
                        }
                        "offset" => {
                            if let Ok(result) = value.parse::<i32>() {
                                offset = Some(result);
                            }
                        }
                        _ => {}
                    }
                }

                if let Some(ref mut file) = data.file {
                    if let Some(attrs) = file.additional_info.as_mut().and_then(|ai| ai.attributes_mut()) {
                        attrs.push((key, value));
                    }
                }

                index = end_index;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    (offset, index)
}

pub fn is_attribute_line(line: &str) -> bool {
    let line = line.trim();
    line.starts_with('[') && line.ends_with(']') && line.contains(':')
}

pub fn get_attribute(line: &str) -> (String, String) {
    let line = line.trim();
    let key = between(line, "[", ":");
    let colon_pos = line.find(':').unwrap_or(0);
    let value = if line.len() > colon_pos + 1 {
        line[colon_pos + 1..line.len() - 1].to_string()
    } else {
        String::new()
    };
    (key, value)
}

fn between(s: &str, start: &str, end: &str) -> String {
    let start_idx = s.find(start).map(|i| i + start.len());
    let end_idx = s.find(end);
    match (start_idx, end_idx) {
        (Some(si), Some(ei)) if si < ei => s[si..ei].to_string(),
        _ => String::new(),
    }
}
