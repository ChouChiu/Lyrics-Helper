use crate::models::LineInfo;

/// 移除歌词行开头的 explicit 标记（如 `🅴` 或 `[E]`）。
pub fn remove_explicit_markers(lines: &mut [LineInfo]) {
    for line in lines.iter_mut() {
        let text = match line {
            LineInfo::Line { text, .. } | LineInfo::FullLine { text, .. } => text,
            _ => continue,
        };

        // Remove common explicit markers
        if text.starts_with("🅴") || text.starts_with("🅴 ") {
            *text = text.trim_start_matches("🅴").trim_start().to_string();
        }
        if text.starts_with("[E]") || text.starts_with("[E] ") {
            *text = text.trim_start_matches("[E]").trim_start().to_string();
        }
    }
}

/// 判断文本是否包含 explicit 标记（`🅴` 或 `[E]`）。
pub fn has_explicit_marker(text: &str) -> bool {
    text.contains("🅴") || text.contains("[E]")
}
