use crate::models::LineInfo;
use crate::models::SyllableInfo;

pub fn add_offset(lines: &mut [LineInfo], offset: i32) {
    for line in lines.iter_mut() {
        add_offset_to_line(line, offset);
    }
}

fn add_offset_to_line(line: &mut LineInfo, offset: i32) {
    match line {
        LineInfo::Line { start_time, end_time, sub_line, .. } => {
            if let Some(st) = start_time {
                *st -= offset;
            }
            if let Some(et) = end_time {
                *et -= offset;
            }
            if let Some(sub) = sub_line {
                add_offset_to_line(sub, offset);
            }
        }
        LineInfo::Syllable { syllables, sub_line, .. } => {
            for syllable in syllables.iter_mut() {
                syllable.start_time -= offset;
                syllable.end_time -= offset;
            }
            if let Some(sub) = sub_line {
                add_offset_to_line(sub, offset);
            }
        }
        LineInfo::FullLine { start_time, end_time, sub_line, .. } => {
            if let Some(st) = start_time {
                *st -= offset;
            }
            if let Some(et) = end_time {
                *et -= offset;
            }
            if let Some(sub) = sub_line {
                add_offset_to_line(sub, offset);
            }
        }
        LineInfo::FullSyllable { syllables, sub_line, .. } => {
            for syllable in syllables.iter_mut() {
                syllable.start_time -= offset;
                syllable.end_time -= offset;
            }
            if let Some(sub) = sub_line {
                add_offset_to_line(sub, offset);
            }
        }
    }
}

pub fn add_offset_to_syllables(syllables: &mut [SyllableInfo], offset: i32) {
    for syllable in syllables.iter_mut() {
        syllable.start_time -= offset;
        syllable.end_time -= offset;
    }
}
