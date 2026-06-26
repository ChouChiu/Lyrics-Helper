use crate::models::LineInfo;

/// 标准化 Musixmatch 格式歌词：移除首尾的空行。
pub fn standardize_musixmatch_lyrics(lines: &mut Vec<LineInfo>) {
    // Remove empty lines at the beginning
    while let Some(first) = lines.first() {
        if let LineInfo::Line { text, .. } = first {
            if text.is_empty() {
                lines.remove(0);
            } else {
                break;
            }
        } else {
            break;
        }
    }

    // Remove empty lines at the end
    while let Some(last) = lines.last() {
        if let LineInfo::Line { text, .. } = last {
            if text.is_empty() {
                lines.pop();
            } else {
                break;
            }
        } else {
            break;
        }
    }
}
