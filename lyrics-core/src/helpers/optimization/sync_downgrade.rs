use crate::models::LineInfo;

pub fn downgrade_to_line_synced(lines: &mut [LineInfo]) {
    for line in lines.iter_mut() {
        match line {
            LineInfo::Syllable { syllables, alignment, sub_line } => {
                let text: String = syllables.iter().map(|s| s.text.as_str()).collect();
                let start_time = syllables.first().map(|s| s.start_time);
                let end_time = syllables.last().map(|s| s.end_time);
                let alignment = *alignment;
                let sub_line = sub_line.take();

                *line = LineInfo::Line {
                    text,
                    start_time,
                    end_time,
                    alignment,
                    sub_line,
                };
            }
            LineInfo::FullSyllable { syllables, alignment, sub_line, translations, pronunciation } => {
                let text: String = syllables.iter().map(|s| s.text.as_str()).collect();
                let start_time = syllables.first().map(|s| s.start_time);
                let end_time = syllables.last().map(|s| s.end_time);
                let alignment = *alignment;
                let sub_line = sub_line.take();
                let translations = translations.clone();
                let pronunciation = pronunciation.take();

                *line = LineInfo::FullLine {
                    text,
                    start_time,
                    end_time,
                    alignment,
                    sub_line,
                    translations,
                    pronunciation,
                };
            }
            _ => {}
        }
    }
}
