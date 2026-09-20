use crate::syllable_info;
use lyrics_core::models::*;
use std::fmt::Write;

/// 背景人声行的对齐编码偏移量。
const BACKGROUND_VOCALS_OFFSET: i32 = 3;

/// 将歌词数据生成为 Lyricify Syllable 逐字歌词格式字符串。
pub fn generate(lyrics_data: &LyricsData) -> String {
    let mut result = String::new();

    let Some(lines) = lyrics_data.lines.as_ref() else {
        return result;
    };

    for line in lines {
        // 主行没有音节时整行跳过（含其子行），与上游一致。
        if !append_line(&mut result, line, 0) {
            continue;
        }

        if let Some(sub) = line.sub_line() {
            append_line(&mut result, sub, BACKGROUND_VOCALS_OFFSET);
        }
    }

    result
}

/// 追加一行 Lyricify Syllable 歌词，返回该行是否真的输出了内容。
fn append_line(result: &mut String, line: &LineInfo, alignment_offset: i32) -> bool {
    let Some((syllables, _, _)) = syllable_info(line) else {
        return false;
    };

    let _ = write!(result, "[{}]", alignment_code(line) + alignment_offset);

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

fn alignment_code(line: &LineInfo) -> i32 {
    match line.alignment() {
        LyricsAlignment::Unspecified => 3,
        LyricsAlignment::Left => 4,
        LyricsAlignment::Right => 5,
    }
}
