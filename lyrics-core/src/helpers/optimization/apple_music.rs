use crate::models::LineInfo;

/// 移除歌词开头的空行（无文本且无时间戳的行）。
pub fn remove_leading_newlines(lines: &mut Vec<LineInfo>) {
    while let Some(first) = lines.first() {
        if let LineInfo::Line { text, start_time, .. } = first {
            if text.is_empty() && start_time.is_none() {
                lines.remove(0);
            } else {
                break;
            }
        } else {
            break;
        }
    }
}
