use crate::syllable_info;
use lyrics_core::models::*;
use std::fmt::Write;

/// 将歌词数据生成为 QRC 逐字歌词格式字符串。
pub fn generate(lyrics_data: &LyricsData) -> String {
    let mut result = String::new();

    if let Some(ref lines) = lyrics_data.lines {
        for line in lines {
            if let Some((syllables, start_time, _)) = syllable_info(line) {
                if let Some(st) = start_time {
                    let duration = line.duration().unwrap_or(0);
                    let _ = write!(result, "[{},{}]", st, duration);
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

                if let Some(sub) = line.sub_line() {
                    if let Some((sub_syllables, sub_start, _)) = syllable_info(sub) {
                        if let Some(st) = sub_start {
                            let duration = sub.duration().unwrap_or(0);
                            let _ = write!(result, "[{},{}]", st, duration);
                        }

                        for syllable in sub_syllables {
                            let _ = write!(
                                result,
                                "{}({},{})",
                                syllable.text,
                                syllable.start_time,
                                syllable.duration()
                            );
                        }

                        result.push('\n');
                    }
                }
            }
        }
    }

    result
}
