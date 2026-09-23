use crate::SubLinesOutputType;
use lyrics_core::helpers::string_helper::format_time_ms_to_timestamp_string;
use lyrics_core::models::*;
use std::fmt::Write;

/// LRC 格式中结束时间戳的输出策略。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EndTimeOutputType {
    /// 不输出结束时间戳。
    None,
    /// 仅在与下一行间隔超过 5 秒时输出结束时间戳。
    Huge,
    /// 对所有行输出结束时间戳。
    All,
}

/// `EndTimeOutputType::Huge` 判定「间隔很大」的阈值（毫秒）。
const HUGE_GAP_MS: i32 = 5000;

/// 将歌词数据生成为标准 LRC 格式字符串（使用默认选项）。
pub fn generate(lyrics_data: &LyricsData) -> String {
    generate_with_options(
        lyrics_data,
        EndTimeOutputType::Huge,
        SubLinesOutputType::InMainLine,
    )
}

/// 将歌词数据生成为标准 LRC 格式字符串，可自定义结束时间输出策略和子行输出方式。
pub fn generate_with_options(
    lyrics_data: &LyricsData,
    end_time_output: EndTimeOutputType,
    sub_lines_output: SubLinesOutputType,
) -> String {
    let mut result = String::new();

    if let Some(metadata) = lyrics_data.track_metadata.as_ref() {
        for (tag, value) in [
            ("ti", &metadata.title),
            ("ar", &metadata.artist),
            ("al", &metadata.album),
        ] {
            if let Some(value) = value {
                let _ = writeln!(result, "[{}:{}]", tag, value);
            }
        }
    }

    let Some(lines) = lyrics_data.lines.as_ref() else {
        return result;
    };

    for (index, line) in lines.iter().enumerate() {
        match sub_lines_output {
            SubLinesOutputType::InMainLine => {
                append_line(
                    &mut result,
                    line,
                    &line.full_text(),
                    index,
                    lines,
                    end_time_output,
                );
            }
            SubLinesOutputType::InDiffLine => {
                append_line(
                    &mut result,
                    line,
                    &line.text_from_any(),
                    index,
                    lines,
                    end_time_output,
                );

                if let Some(sub) = line.sub_line() {
                    append_line(
                        &mut result,
                        sub,
                        &sub.text_from_any(),
                        index,
                        lines,
                        end_time_output,
                    );
                }
            }
        }
    }

    result
}

/// 追加 `[mm:ss.SSS]文本`，并按策略补一条只有结束时间戳的空行。
///
/// 没有开始时间的行整行跳过（LRC 无法表达）。
fn append_line(
    result: &mut String,
    line: &LineInfo,
    text: &str,
    index: usize,
    lines: &[LineInfo],
    end_time_output: EndTimeOutputType,
) {
    let Some(start_time) = line.start_time() else {
        return;
    };

    let _ = writeln!(
        result,
        "[{}]{}",
        format_time_ms_to_timestamp_string(start_time as f32),
        text
    );

    let end_time = line.end_time();
    if should_add_end_time_line(end_time, index, lines, end_time_output)
        && let Some(end_time) = end_time
    {
        let _ = writeln!(
            result,
            "[{}]",
            format_time_ms_to_timestamp_string(end_time as f32)
        );
    }
}

fn should_add_end_time_line(
    end_time: Option<i32>,
    index: usize,
    lines: &[LineInfo],
    output_type: EndTimeOutputType,
) -> bool {
    let Some(end_time) = end_time.filter(|end_time| *end_time > 0) else {
        return false;
    };

    match output_type {
        EndTimeOutputType::None => false,
        EndTimeOutputType::All => true,
        // 最后一行没有「下一行」可比，总是输出结束时间戳。
        EndTimeOutputType::Huge => lines
            .get(index + 1)
            .and_then(LineInfo::start_time)
            .is_none_or(|next_start| next_start - end_time > HUGE_GAP_MS),
    }
}
