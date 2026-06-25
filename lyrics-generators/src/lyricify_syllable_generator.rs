use lyrics_core::models::*;

pub fn generate(lyrics_data: &LyricsData) -> String {
    let mut result = String::new();

    if let Some(ref lines) = lyrics_data.lines {
        for line in lines {
            let alignment_code = get_alignment_code(line);

            if let Some((syllables, _, _)) = get_syllable_info(line) {
                result.push_str(&format!("[{}]", alignment_code));

                for syllable in syllables {
                    result.push_str(&format!(
                        "{}({},{})",
                        syllable.text,
                        syllable.start_time,
                        syllable.duration()
                    ));
                }

                result.push('\n');

                // Sub line
                if let Some(sub) = line.sub_line() {
                    let sub_alignment = get_alignment_code(sub) + 3; // Background vocals offset
                    if let Some((sub_syllables, _, _)) = get_syllable_info(sub) {
                        result.push_str(&format!("[{}]", sub_alignment));

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

fn get_alignment_code(line: &LineInfo) -> i32 {
    match line.alignment() {
        LyricsAlignment::Unspecified => 3,
        LyricsAlignment::Left => 4,
        LyricsAlignment::Right => 5,
    }
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
