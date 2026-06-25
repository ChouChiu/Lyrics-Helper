use crate::models::*;

pub fn generate(lyrics_data: &LyricsData) -> String {
    let mut result = String::new();

    if let Some(ref lines) = lyrics_data.lines {
        for line in lines {
            if let Some((syllables, start_time, _)) = get_syllable_info(line) {
                if let Some(st) = start_time {
                    let duration = line.duration().unwrap_or(0);
                    result.push_str(&format!(
                        "[{},{}]",
                        st,
                        duration
                    ));
                }

                for syllable in syllables {
                    result.push_str(&format!(
                        "{}({},{})",
                        syllable.text,
                        syllable.start_time,
                        syllable.duration()
                    ));
                }

                result.push('\n');

                if let Some(sub) = line.sub_line() {
                    if let Some((sub_syllables, sub_start, _)) = get_syllable_info(sub) {
                        if let Some(st) = sub_start {
                            let duration = sub.duration().unwrap_or(0);
                            result.push_str(&format!(
                                "[{},{}]",
                                st,
                                duration
                            ));
                        }

                        for syllable in sub_syllables {
                            result.push_str(&format!(
                                "{}({},{})",
                                syllable.text,
                                syllable.start_time,
                                syllable.duration()
                            ));
                        }

                        result.push('\n');
                    }
                }
            }
        }
    }

    result
}

fn get_syllable_info(line: &LineInfo) -> Option<(Vec<SyllableInfo>, Option<i32>, Option<i32>)> {
    match line {
        LineInfo::Syllable { syllables, .. } | LineInfo::FullSyllable { syllables, .. } => {
            let start_time = syllables.first().map(|s| s.start_time);
            let end_time = syllables.last().map(|s| s.end_time);
            Some((syllables.clone(), start_time, end_time))
        }
        _ => None,
    }
}
