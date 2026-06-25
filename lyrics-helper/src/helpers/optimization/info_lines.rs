use crate::models::LineInfo;

pub fn remove_info_lines(lines: &mut Vec<LineInfo>) {
    lines.retain(|line| {
        if let LineInfo::Line { text, start_time, .. } = line {
            // Remove lines that look like info/credit lines
            if text.starts_with("作词") || text.starts_with("作曲") || text.starts_with("编曲") {
                return false;
            }
            if text.starts_with("词：") || text.starts_with("曲：") {
                return false;
            }
            if start_time.is_some() && text.is_empty() {
                return false;
            }
        }
        true
    });
}
