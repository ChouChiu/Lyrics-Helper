use crate::models::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EndTimeOutputType {
    None,
    Huge,
    All,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SubLinesOutputType {
    InMainLine,
    InDiffLine,
}

pub fn generate(lyrics_data: &LyricsData) -> String {
    generate_with_options(lyrics_data, EndTimeOutputType::Huge, SubLinesOutputType::InMainLine)
}

pub fn generate_with_options(
    lyrics_data: &LyricsData,
    end_time_output: EndTimeOutputType,
    sub_lines_output: SubLinesOutputType,
) -> String {
    let mut result = String::new();

    // Add metadata
    if let Some(ref metadata) = lyrics_data.track_metadata {
        if let Some(ref title) = metadata.title {
            result.push_str(&format!("[ti:{}]\n", title));
        }
        if let Some(ref artist) = metadata.artist {
            result.push_str(&format!("[ar:{}]\n", artist));
        }
        if let Some(ref album) = metadata.album {
            result.push_str(&format!("[al:{}]\n", album));
        }
    }

    if let Some(ref lines) = lyrics_data.lines {
        for (i, line) in lines.iter().enumerate() {
            match sub_lines_output {
                SubLinesOutputType::InDiffLine => {
                    // Main line
                    let start_time = line.start_time();
                    let end_time = line.end_time();
                    let text = line.text_from_any();

                    if let Some(st) = start_time {
                        result.push_str(&format!(
                            "[{}]{}\n",
                            crate::helpers::string_helper::format_time_ms_to_timestamp_string(st as f32),
                            text
                        ));

                        // End time line
                        if should_add_line(end_time, i, lines, end_time_output) {
                            if let Some(et) = end_time {
                                result.push_str(&format!(
                                    "[{}]\n",
                                    crate::helpers::string_helper::format_time_ms_to_timestamp_string(et as f32)
                                ));
                            }
                        }
                    }

                    // Sub line
                    if let Some(sub) = line.sub_line() {
                        let sub_start = sub.start_time();
                        let sub_end = sub.end_time();
                        let sub_text = sub.text_from_any();

                        if let Some(st) = sub_start {
                            result.push_str(&format!(
                                "[{}]{}\n",
                                crate::helpers::string_helper::format_time_ms_to_timestamp_string(st as f32),
                                sub_text
                            ));

                            if should_add_line(sub_end, i, lines, end_time_output) {
                                if let Some(et) = sub_end {
                                    result.push_str(&format!(
                                        "[{}]\n",
                                        crate::helpers::string_helper::format_time_ms_to_timestamp_string(et as f32)
                                    ));
                                }
                            }
                        }
                    }
                }
                SubLinesOutputType::InMainLine => {
                    let start_time = line.start_time();
                    let end_time = line.end_time();
                    let text = line.full_text();

                    if let Some(st) = start_time {
                        result.push_str(&format!(
                            "[{}]{}\n",
                            crate::helpers::string_helper::format_time_ms_to_timestamp_string(st as f32),
                            text
                        ));

                        if should_add_line(end_time, i, lines, end_time_output) {
                            if let Some(et) = end_time {
                                result.push_str(&format!(
                                    "[{}]\n",
                                    crate::helpers::string_helper::format_time_ms_to_timestamp_string(et as f32)
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    result
}

fn should_add_line(
    end_time: Option<i32>,
    index: usize,
    lines: &[LineInfo],
    output_type: EndTimeOutputType,
) -> bool {
    match output_type {
        EndTimeOutputType::None => false,
        EndTimeOutputType::All => end_time.is_some() && end_time.unwrap() > 0,
        EndTimeOutputType::Huge => {
            if let Some(et) = end_time {
                if et <= 0 {
                    return false;
                }
                // Check if gap to next line > 5000ms
                if index + 1 < lines.len() {
                    if let Some(next_start) = lines[index + 1].start_time() {
                        return next_start - et > 5000;
                    }
                }
                // Last line
                true
            } else {
                false
            }
        }
    }
}
