use crate::models::*;

pub fn generate(lyrics_data: &LyricsData) -> String {
    let mut result = String::new();

    if let Some(ref lines) = lyrics_data.lines {
        for line in lines {
            if let Some((syllables, start_time, end_time)) = get_syllable_info(line) {
                if let (Some(st), Some(et)) = (start_time, end_time) {
                    let duration = et - st;
                    result.push_str(&format!(
                        "[{},{}]",
                        st,
                        duration
                    ));
                }

                // First syllable uses <offset,duration,0> format
                if let Some(first) = syllables.first() {
                    let offset = first.start_time - start_time.unwrap_or(0);
                    result.push_str(&format!(
                        "<{},{},0>{}",
                        offset,
                        first.duration(),
                        first.text
                    ));
                }

                // Subsequent syllables use ,0><offset,duration,0> format
                for syllable in syllables.iter().skip(1) {
                    let offset = syllable.start_time - start_time.unwrap_or(0);
                    result.push_str(&format!(
                        ",0><{},{},0>{}",
                        offset,
                        syllable.duration(),
                        syllable.text
                    ));
                }

                result.push('\n');

                if let Some(sub) = line.sub_line() {
                    if let Some((sub_syllables, sub_start, sub_end)) = get_syllable_info(sub) {
                        if let (Some(st), Some(et)) = (sub_start, sub_end) {
                            let duration = et - st;
                            result.push_str(&format!(
                                "[{},{}]",
                                st,
                                duration
                            ));
                        }

                        if let Some(first) = sub_syllables.first() {
                            let offset = first.start_time - sub_start.unwrap_or(0);
                            result.push_str(&format!(
                                "<{},{},0>{}",
                                offset,
                                first.duration(),
                                first.text
                            ));
                        }

                        for syllable in sub_syllables.iter().skip(1) {
                            let offset = syllable.start_time - sub_start.unwrap_or(0);
                            result.push_str(&format!(
                                ",0><{},{},0>{}",
                                offset,
                                syllable.duration(),
                                syllable.text
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
