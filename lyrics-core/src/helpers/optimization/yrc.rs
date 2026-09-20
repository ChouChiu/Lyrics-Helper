use crate::models::{LineInfo, SyllableItem};

/// 针对 YRC 歌词格式的优化。
///
/// 移除末尾空格与空音节，把独立的空格与标点合并到前一个音节。
pub fn standardize_yrc_lyrics(lines: &mut [LineInfo]) {
    for line in lines.iter_mut() {
        standardize_yrc_lyrics_line(line);
    }
}

/// 针对单行 YRC 歌词的优化。
///
/// 上游实现按 `SyllableInfo` 处理音节，合并后的音节（`FullSyllableInfo`）会抛出异常；
/// 本实现改为把文本追加到合并音节的最后一个子音节，行为保持等价且不会失败。
pub fn standardize_yrc_lyrics_line(line: &mut LineInfo) {
    let Some(syllables) = line.syllables_mut() else {
        return;
    };

    // 移除最后的空格
    while syllables.last().is_some_and(|item| item.text() == " ") {
        syllables.pop();
    }

    let mut i = 0;
    while i < syllables.len() {
        let text = syllables[i].text();

        // 移除空白格
        if text.is_empty() {
            syllables.remove(i);
            continue;
        }

        // 合并单独的空格
        if text == " " {
            if i > 0 {
                append_text(&mut syllables[i - 1], &text);
            }
            syllables.remove(i);
            continue;
        }

        // 合并标点符号
        if i > 0
            && text.chars().count() <= 2
            && matches!(text.chars().next(), Some(',' | '.' | '?' | '!' | '"'))
        {
            append_text(&mut syllables[i - 1], &text);
            syllables.remove(i);
            continue;
        }

        i += 1;
    }
}

/// 把文本追加到音节（合并音节追加到其最后一个子音节）。
fn append_text(item: &mut SyllableItem, text: &str) {
    match item {
        SyllableItem::Syllable(syllable) => syllable.text.push_str(text),
        SyllableItem::Full(full) => {
            if let Some(last) = full.sub_items_mut().last_mut() {
                last.text.push_str(text);
            }
        }
    }
}
