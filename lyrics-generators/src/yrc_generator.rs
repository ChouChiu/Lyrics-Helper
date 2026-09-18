use crate::syllable_info;
use lyrics_core::models::*;
use std::fmt::Write;

/// 将歌词数据生成为 YRC（网易云）逐字歌词格式字符串。
pub fn generate(lyrics_data: &LyricsData) -> String {
    let mut result = String::new();

    if let Some(ref lines) = lyrics_data.lines {
        for line in lines {
            if let Some((syllables, start_time, end_time)) = syllable_info(line) {
                if let (Some(st), Some(et)) = (start_time, end_time) {
                    let duration = et - st;
                    let _ = write!(result, "[{},{}]", st, duration);
                }

                for syllable in syllables {
                    let _ = write!(
                        result,
                        "({},{},0){}",
                        syllable.start_time,
                        syllable.duration(),
                        syllable.text
                    );
                }

                result.push('\n');

                if let Some(sub) = line.sub_line() {
                    if let Some((sub_syllables, sub_start, sub_end)) = syllable_info(sub) {
                        if let (Some(st), Some(et)) = (sub_start, sub_end) {
                            let duration = et - st;
                            let _ = write!(result, "[{},{}]", st, duration);
                        }

                        for syllable in sub_syllables {
                            let _ = write!(
                                result,
                                "({},{},0){}",
                                syllable.start_time,
                                syllable.duration(),
                                syllable.text
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
