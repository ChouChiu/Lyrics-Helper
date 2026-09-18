use crate::syllable_info;
use lyrics_core::models::*;
use std::fmt::Write;

/// 将歌词数据生成为 Lyricify Syllable 逐字歌词格式字符串。
pub fn generate(lyrics_data: &LyricsData) -> String {
    let mut result = String::new();

    if let Some(ref lines) = lyrics_data.lines {
        for line in lines {
            let alignment_code = get_alignment_code(line);

            if let Some((syllables, _, _)) = syllable_info(line) {
                let _ = write!(result, "[{}]", alignment_code);

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

                // Sub line
                if let Some(sub) = line.sub_line() {
                    let sub_alignment = get_alignment_code(sub) + 3; // Background vocals offset
                    if let Some((sub_syllables, _, _)) = syllable_info(sub) {
                        let _ = write!(result, "[{}]", sub_alignment);

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

fn get_alignment_code(line: &LineInfo) -> i32 {
    match line.alignment() {
        LyricsAlignment::Unspecified => 3,
        LyricsAlignment::Left => 4,
        LyricsAlignment::Right => 5,
    }
}
