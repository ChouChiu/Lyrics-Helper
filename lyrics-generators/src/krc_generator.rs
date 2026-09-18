use crate::syllable_info;
use lyrics_core::models::*;
use std::fmt::Write;

/// 将歌词数据生成为 KRC 逐字歌词格式字符串。
pub fn generate(lyrics_data: &LyricsData) -> String {
    let mut result = String::new();

    if let Some(ref lines) = lyrics_data.lines {
        for line in lines {
            if let Some((syllables, start_time, _)) = syllable_info(line) {
                append_line(&mut result, &syllables, start_time);

                if let Some(sub) = line.sub_line() {
                    if let Some((sub_syllables, sub_start, _)) = syllable_info(sub) {
                        append_line(&mut result, &sub_syllables, sub_start);
                    }
                }
            }
        }
    }

    result
}

/// 追加一行 KRC 歌词：行头 `[开始时间,时长]` + 逐音节 `<相对开始时间,时长,0>文本`。
fn append_line(result: &mut String, syllables: &[&SyllableInfo], start_time: Option<i32>) {
    if let Some(start) = start_time {
        let duration = syllables
            .last()
            .map(|last| last.end_time - start)
            .unwrap_or(0);
        let _ = write!(result, "[{},{}]", start, duration);
    }

    for syllable in syllables {
        let offset = syllable.start_time - start_time.unwrap_or(0);
        let _ = write!(
            result,
            "<{},{},0>{}",
            offset,
            syllable.duration(),
            syllable.text
        );
    }

    result.push('\n');
}
