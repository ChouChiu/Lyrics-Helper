use crate::models::{LineInfo, SyllableInfo, SyllableItem};

/// 为所有歌词行添加时间偏移量（单位：毫秒）。
///
/// 正值 `offset` 会将时间戳前移，负值则后移。行时间、音节时间与子行一并处理。
pub fn add_offset(lines: &mut [LineInfo], offset: i32) {
    for line in lines {
        add_offset_to_line(line, offset);
    }
}

fn add_offset_to_line(line: &mut LineInfo, offset: i32) {
    let (start_time, end_time) = line.line_times_mut();
    for time in [start_time, end_time].into_iter().flatten() {
        *time -= offset;
    }

    if let Some(syllables) = line.syllables_mut() {
        add_offset_to_syllable_items(syllables, offset);
    }

    if let Some(sub) = line.sub_line_mut() {
        add_offset_to_line(sub, offset);
    }
}

/// 为音节项列表中的每个音节添加时间偏移量（单位：毫秒），合并音节的子音节一并处理。
pub fn add_offset_to_syllable_items(syllables: &mut [SyllableItem], offset: i32) {
    for syllable in syllables {
        add_offset_to_syllables(syllable.parts_mut(), offset);
    }
}

/// 为音节列表中的每个音节添加时间偏移量（单位：毫秒）。
pub fn add_offset_to_syllables(syllables: &mut [SyllableInfo], offset: i32) {
    for syllable in syllables {
        syllable.start_time -= offset;
        syllable.end_time -= offset;
    }
}
