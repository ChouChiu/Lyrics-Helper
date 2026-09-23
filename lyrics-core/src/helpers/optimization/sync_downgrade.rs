use crate::models::{LineInfo, LyricsAlignment};

/// 将音节级同步歌词降级为行级同步，递归处理子行。
pub fn downgrade_to_line_synced(lines: &mut [LineInfo]) {
    for line in lines.iter_mut() {
        downgrade_line(line);
    }
}

/// 将单行音节级同步歌词降级为行级同步。
///
/// 已是行级同步的歌词仅递归更新子行；音节为空的异常情况保持原样。
pub fn downgrade_line(line: &mut LineInfo) {
    // 先递归处理子行
    if let Some(sub) = line.sub_line_mut() {
        downgrade_line(sub);
    }

    if !line.is_syllable() {
        return;
    }

    // 取出音节行，按需重建为行级歌词
    let placeholder = LineInfo::Line {
        text: String::new(),
        start_time: None,
        end_time: None,
        alignment: LyricsAlignment::Unspecified,
        sub_line: None,
    };
    let owned = std::mem::replace(line, placeholder);

    let (syllables, start_time, end_time, alignment, sub_line, translations, pronunciation) =
        match owned {
            LineInfo::Syllable {
                syllables,
                start_time,
                end_time,
                alignment,
                sub_line,
            } => (
                syllables, start_time, end_time, alignment, sub_line, None, None,
            ),
            LineInfo::FullSyllable {
                syllables,
                start_time,
                end_time,
                alignment,
                sub_line,
                translations,
                pronunciation,
            } => (
                syllables,
                start_time,
                end_time,
                alignment,
                sub_line,
                Some(translations),
                pronunciation,
            ),
            other => {
                *line = other;
                return;
            }
        };

    if syllables.is_empty() {
        // 没有音节数据的异常情况：保持原样
        *line = match translations {
            Some(translations) => LineInfo::FullSyllable {
                syllables,
                start_time,
                end_time,
                alignment,
                sub_line,
                translations,
                pronunciation,
            },
            None => LineInfo::Syllable {
                syllables,
                start_time,
                end_time,
                alignment,
                sub_line,
            },
        };
        return;
    }

    let text = LineInfo::text_from_syllables(&syllables);
    let start_time = start_time.or_else(|| syllables.first().map(|s| s.start_time()));
    let end_time = end_time.or_else(|| syllables.last().map(|s| s.end_time()));

    *line = match translations {
        Some(translations) => LineInfo::FullLine {
            text,
            start_time,
            end_time,
            alignment,
            sub_line,
            translations,
            pronunciation,
        },
        None => LineInfo::Line {
            text,
            start_time,
            end_time,
            alignment,
            sub_line,
        },
    };
}
