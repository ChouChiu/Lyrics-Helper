use crate::models::LineInfo;
use crate::models::SyllableInfo;

/// 为所有歌词行添加时间偏移量（单位：毫秒）。
///
/// 正值 `offset` 会将时间戳前移，负值则后移。递归处理子行。
pub fn add_offset(lines: &mut [LineInfo], offset: i32) {
    for line in lines.iter_mut() {
        add_offset_to_line(line, offset);
    }
}

fn add_offset_to_line(line: &mut LineInfo, offset: i32) {
    match line {
        LineInfo::Line {
            start_time,
            end_time,
            ..
        }
        | LineInfo::FullLine {
            start_time,
            end_time,
            ..
        } => {
            if let Some(start) = start_time {
                *start -= offset;
            }
            if let Some(end) = end_time {
                *end -= offset;
            }
        }
        LineInfo::Syllable { syllables, .. } | LineInfo::FullSyllable { syllables, .. } => {
            crate::models::add_offset_to_syllable_items(syllables, offset);
        }
    }

    if let Some(sub) = line.sub_line_mut() {
        add_offset_to_line(sub, offset);
    }
}

/// 为音节列表中的每个音节添加时间偏移量（单位：毫秒）。
pub fn add_offset_to_syllables(syllables: &mut [SyllableInfo], offset: i32) {
    for syllable in syllables.iter_mut() {
        syllable.start_time -= offset;
        syllable.end_time -= offset;
    }
}
