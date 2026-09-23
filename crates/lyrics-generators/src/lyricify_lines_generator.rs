use crate::SubLinesOutputType;
use lyrics_core::models::*;
use std::fmt::Write;

/// 将歌词数据生成为 Lyricify Lines 行级歌词格式字符串（使用默认选项）。
pub fn generate(lyrics_data: &LyricsData) -> String {
    generate_with_options(lyrics_data, SubLinesOutputType::InMainLine)
}

/// 将歌词数据生成为 Lyricify Lines 行级歌词格式字符串，可自定义子行输出方式。
pub fn generate_with_options(
    lyrics_data: &LyricsData,
    sub_lines_output: SubLinesOutputType,
) -> String {
    let mut result = String::new();

    let Some(lines) = lyrics_data.lines.as_ref() else {
        return result;
    };

    for line in lines {
        match sub_lines_output {
            SubLinesOutputType::InMainLine => append_line(&mut result, line, &line.full_text()),
            SubLinesOutputType::InDiffLine => {
                append_line(&mut result, line, &line.text_from_any());

                if let Some(sub) = line.sub_line() {
                    append_line(&mut result, sub, &sub.text_from_any());
                }
            }
        }
    }

    result
}

/// 追加 `[开始时间,结束时间]文本`；首尾时间不全的行整行跳过。
fn append_line(result: &mut String, line: &LineInfo, text: &str) {
    if let (Some(start), Some(end)) = (line.start_time(), line.end_time()) {
        let _ = writeln!(result, "[{},{}]{}", start, end, text);
    }
}
