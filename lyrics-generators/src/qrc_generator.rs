use crate::{for_each_line_and_sub_line, syllable_info};
use lyrics_core::models::*;
use std::fmt::Write;

/// 将歌词数据生成为 QRC 逐字歌词格式字符串。
pub fn generate(lyrics_data: &LyricsData) -> String {
    let mut result = String::new();
    for_each_line_and_sub_line(lyrics_data, |line| append_line(&mut result, line));
    result
}

/// 追加一行 QRC 歌词：行头 `[开始时间,时长]` + 逐音节 `文本(开始时间,时长)`。
///
/// 返回该行是否真的输出了内容。
fn append_line(result: &mut String, line: &LineInfo) -> bool {
    let Some((syllables, start_time, _)) = syllable_info(line) else {
        return false;
    };

    if let Some(start) = start_time {
        let _ = write!(result, "[{},{}]", start, line.duration().unwrap_or(0));
    }

    for syllable in syllables {
        let _ = write!(
            result,
            "{}({},{})",
            syllable.text,
            syllable.start_time,
            syllable.duration()
        );
    }

    result.push('\n');
    true
}
