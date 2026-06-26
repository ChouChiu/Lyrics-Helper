use crate::models::LineInfo;

/// 标准化 YRC 格式歌词：移除末尾空格，将标点符号合并到前一个音节。
pub fn standardize_yrc_lyrics(lines: &mut [LineInfo]) {
    for line in lines.iter_mut() {
        if let LineInfo::Syllable { syllables, .. } | LineInfo::FullSyllable { syllables, .. } = line {
            // 移除末尾空格
            if let Some(last) = syllables.last_mut() {
                if last.text.ends_with(' ') {
                    last.text = last.text.trim_end().to_string();
                }
            }

            // 将标点符号合并到前一个音节
            let mut i = 1;
            while i < syllables.len() {
                if syllables[i].text.len() == 1 && is_punctuation(syllables[i].text.chars().next().unwrap()) {
                    let punct = syllables[i].text.clone();
                    let end_time = syllables[i].end_time;
                    syllables[i - 1].text.push_str(&punct);
                    syllables[i - 1].end_time = end_time;
                    syllables.remove(i);
                } else {
                    i += 1;
                }
            }
        }
    }
}

fn is_punctuation(c: char) -> bool {
    matches!(c,
        '.' | ',' | '!' | '?' | ';' | ':' | '-' | '(' | ')' |
        '。' | '，' | '！' | '？' | '；' | '：' | '—' | '（' | '）' |
        '\'' | '"' | '…' | '、' | '～' | '~'
    )
}
