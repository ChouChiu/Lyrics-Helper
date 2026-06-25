use crate::models::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SubLinesOutputType {
    InMainLine,
    InDiffLine,
}

pub fn generate(lyrics_data: &LyricsData) -> String {
    generate_with_options(lyrics_data, SubLinesOutputType::InMainLine)
}

pub fn generate_with_options(lyrics_data: &LyricsData, sub_lines_output: SubLinesOutputType) -> String {
    let mut result = String::new();

    if let Some(ref lines) = lyrics_data.lines {
        for line in lines {
            let start_time = line.start_time();
            let end_time = line.end_time();

            match sub_lines_output {
                SubLinesOutputType::InDiffLine => {
                    if let (Some(st), Some(et)) = (start_time, end_time) {
                        let text = line.text_from_any();
                        result.push_str(&format!("[{},{}]{}\n", st, et, text));
                    }

                    if let Some(sub) = line.sub_line() {
                        if let (Some(st), Some(et)) = (sub.start_time(), sub.end_time()) {
                            let text = sub.text_from_any();
                            result.push_str(&format!("[{},{}]{}\n", st, et, text));
                        }
                    }
                }
                SubLinesOutputType::InMainLine => {
                    if let (Some(st), Some(et)) = (start_time, end_time) {
                        let text = line.full_text();
                        result.push_str(&format!("[{},{}]{}\n", st, et, text));
                    }
                }
            }
        }
    }

    result
}
