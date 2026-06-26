use lyrics_core::models::*;
use crate::SubLinesOutputType;

/// 将歌词数据生成为 Lyricify Lines 行级歌词格式字符串（使用默认选项）。
pub fn generate(lyrics_data: &LyricsData) -> String {
    generate_with_options(lyrics_data, SubLinesOutputType::InMainLine)
}

/// 将歌词数据生成为 Lyricify Lines 行级歌词格式字符串，可自定义子行输出方式。
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
