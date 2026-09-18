use crate::models::{LineInfo, SyllableItem};

use super::syllable_word_merger;

/// 针对 Musixmatch richsync 歌词格式的优化。
///
/// 将同一单词内的连续音节组合为合并音节，并把空白附加到前一个音节。
pub fn standardize_musixmatch_lyrics(lines: &mut [LineInfo]) {
    for line in lines.iter_mut() {
        standardize_musixmatch_line(line);
    }
}

/// 针对单行 Musixmatch richsync 歌词的优化。
///
/// 已合并过音节的歌词行会被跳过（与上游一致）。
pub fn standardize_musixmatch_line(line: &mut LineInfo) {
    let Some(syllables) = line.syllables_mut() else {
        return;
    };

    if syllables.is_empty() || syllables.iter().any(SyllableItem::is_full) {
        return;
    }

    // Musixmatch 把空白作为独立的时间片段，这里附加到前一个片段，
    // 使共享的合并逻辑可以把它当作单词边界。
    let mut index = 1;
    while index < syllables.len() {
        let is_whitespace = syllables[index]
            .parts()
            .iter()
            .flat_map(|part| part.text.chars())
            .all(char::is_whitespace);

        if is_whitespace {
            let whitespace = syllables.remove(index).text();
            if let Some(previous) = syllables[index - 1].as_syllable_mut() {
                previous.text.push_str(&whitespace);
            }
        } else {
            index += 1;
        }
    }

    syllable_word_merger::merge(line);
}
