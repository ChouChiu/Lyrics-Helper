//! 把另一份逐词计时文档的时间读到已解析的行级转录上。
//!
//! 网易云的逐字歌词（YRC）是独立的第二份文档，它的文字是时间表而不是给人读的转录：
//! 词与词之间的分隔符被吃掉、括号当标签用、被打码的词与正文对不上。要显示的是转录的
//! 文字，要用的是逐词文档的时间，因此这里按行配对两份文档，把逐词文档的时间读到转录
//! 已经写好的文字上。
//!
//! 三条语义：
//!
//! 1. 行按开始时间顺序配对，相差不超过 1000 毫秒；
//! 2. 每个词吃掉转录里「到它最后一个词字符为止」的字符，分隔符写在前一个词的末尾而不是
//!    后一个词的开头，因此一行的词拼起来精确等于该行原文；
//! 3. 对不齐就整行放弃：保留原有行时间、不带词时间。全有或全无——贴错的词时间比没有词
//!    时间更糟。

use std::iter::Peekable;
use std::str::Chars;

use crate::models::{FullSyllableInfo, LineInfo, SyllableInfo, SyllableItem};

/// 逐词行与转录行允许相差的时间（毫秒）。
///
/// 网易云从「这一行开始唱」的时刻给词计时，行级转录又按行头写这一行自己的开始时间，
/// 同一行的两个时刻相差零点几秒；行按顺序配对，容差只需吸收这个漂移。
const ROW_TOLERANCE_MS: i32 = 1_000;

/// 把 `word_lines` 的逐词时间读到 `lines` 的文本上。
///
/// `lines` 是已解析的行级转录，`word_lines` 是同一首歌的逐词计时文档（如 YRC）。逐词
/// 文档给每个词计时，但它拼词用的文字是时间表而不是给人读的转录；因此保留转录的文字、
/// 只取逐词文档的时间：两份文档按开始时间顺序配对，相差不超过 1000 毫秒才算同一行，且
/// 每个逐词行只配一次。读不上的一行——逐词文档没有的那句旁白、它打码而转录写全的那个
/// 词——保持原样：留下原有的行时间、不带词时间。全有或全无，绝不部分贴合。
///
/// 配上词的行改成音节行（`Line` / `FullLine` 变为 `Syllable` / `FullSyllable`），文本由
/// 音节推导且精确等于该行原文；行时间、对齐方式、子行与翻译/拼音原样保留。合并音节
/// （[`SyllableItem::Full`]）按**一个词**处理：它整体吃掉转录里属于这个词的字符，再把这批
/// 字符按同样的规则分给它的子音节，合并结构不拆、子音节各自的时间保留。子音节之间同样
/// 按「分隔符归前一个」划分，因此词内的分隔符会跟着走到前一个子音节——子音节
/// `you` + `'re` 对齐后是 `you'` + `re`，拼起来仍是这个词。没有子音节的合并音节没有时间
/// 可用，会被跳过；没有词（音节）的逐词行给不出词时间，也不会占用某一行的配对。
///
/// 只配对顶层行，子行（如背景和声）不参与。
///
/// # 示例
///
/// ```
/// use lyrics_core::helpers::word_timing::apply_word_timings;
/// use lyrics_core::{LineInfo, SyllableInfo};
///
/// // 行级转录：这一行该显示的文字
/// let mut rows = vec![LineInfo::new_line(
///     "Ugh, you're a monster".to_string(),
///     Some(90),
///     Some(2160),
/// )];
/// // 逐词文档：同一行的词及其时间（词之间的分隔符被吃掉）
/// let words = vec![LineInfo::new_syllable(vec![
///     SyllableInfo::new("Ugh".to_string(), 90, 420).into(),
///     SyllableInfo::new("you're ".to_string(), 420, 690).into(),
///     SyllableInfo::new("a ".to_string(), 690, 960).into(),
///     SyllableInfo::new("monster".to_string(), 960, 2160).into(),
/// ])];
///
/// apply_word_timings(&mut rows, &words);
///
/// let timed: Vec<(i32, String)> = rows[0]
///     .syllables()
///     .expect("配上词后是音节行")
///     .iter()
///     .map(|syllable| (syllable.start_time(), syllable.text()))
///     .collect();
/// assert_eq!(timed[0], (90, "Ugh, ".to_string()));
/// assert_eq!(timed[1], (420, "you're ".to_string()));
/// ```
pub fn apply_word_timings(lines: &mut [LineInfo], word_lines: &[LineInfo]) {
    let mut next_word_line = 0;

    for line in lines.iter_mut() {
        let Some(line_start) = line.start_time() else {
            continue;
        };

        // 走掉配不上的逐词行，停在候选上。没有词（音节）的行给不出词时间；没有开始时间的
        // 行配不上任何行；开始时间早于本行超过一个容差的行也配不上——两份文档都按开始
        // 时间排序，跳过它们不会漏掉后面的行。
        let mut candidate = None;
        while let Some(word_line) = word_lines.get(next_word_line) {
            match (word_line.syllables(), word_line.start_time()) {
                (Some(words), Some(word_start))
                    if !words.is_empty() && word_start + ROW_TOLERANCE_MS >= line_start =>
                {
                    candidate = Some((words, word_start));
                    break;
                }
                _ => next_word_line += 1,
            }
        }
        let Some((word_words, word_start)) = candidate else {
            return;
        };

        // 离得太远：这一行不带词时间，逐词行留给后面的行——它可能配得上后面的某一行。
        if word_start.abs_diff(line_start) > ROW_TOLERANCE_MS as u32 {
            continue;
        }

        let Some(syllables) = read_words_onto(&line.text_from_any(), word_words) else {
            // 两份文档说的不是同一批词：整行放弃。
            continue;
        };

        // 原地换变体：先取出整行，避免深拷贝音节与子行。
        let owned = std::mem::replace(line, LineInfo::new_line_simple(String::new()));
        *line = with_words(owned, syllables);
        next_word_line += 1;
    }
}

/// 把对齐好的词写到行上，保留行时间、对齐方式、子行与翻译/拼音。
///
/// 音节是唯一能装词时间的地方，所以 `Line` / `FullLine` 在这里变成 `Syllable` /
/// `FullSyllable`；音节拼起来精确等于原来的行文本，文字没有丢。
fn with_words(line: LineInfo, syllables: Vec<SyllableItem>) -> LineInfo {
    match line {
        LineInfo::FullLine {
            start_time,
            end_time,
            alignment,
            sub_line,
            translations,
            pronunciation,
            ..
        }
        | LineInfo::FullSyllable {
            start_time,
            end_time,
            alignment,
            sub_line,
            translations,
            pronunciation,
            ..
        } => LineInfo::FullSyllable {
            syllables,
            start_time,
            end_time,
            alignment,
            sub_line,
            translations,
            pronunciation,
        },
        LineInfo::Line {
            start_time,
            end_time,
            alignment,
            sub_line,
            ..
        }
        | LineInfo::Syllable {
            start_time,
            end_time,
            alignment,
            sub_line,
            ..
        } => LineInfo::Syllable {
            syllables,
            start_time,
            end_time,
            alignment,
            sub_line,
        },
    }
}

/// 把 `words` 的词时间读到 `text` 的字符上，返回逐词对齐的结果。
///
/// 每个词保留自己的时间，并吃掉转录写到它身上的字符，即到它最后一个词字符为止的那些
/// 字符；它前面的分隔符写在前一个词的末尾，因此这些词拼起来精确等于 `text`。两者说的
/// 不是同一批词时返回 `None`，这是唯一能说明词时间不属于这批字符的情况。
fn read_words_onto(text: &str, words: &[SyllableItem]) -> Option<Vec<SyllableItem>> {
    let mut remaining = text.chars().peekable();
    let mut aligned: Vec<SyllableItem> = Vec::with_capacity(words.len());

    for word in words {
        // 没有子音节的合并音节没有时间可用（聚合时间由子项推导，空则是 0），跳过它。
        if word
            .as_full()
            .is_some_and(|full| full.sub_items().is_empty())
        {
            continue;
        }

        // 分隔符站在这个词与前一个词之间，转录把它写在前一个词的末尾而不是这个词的开头。
        let mut taken = String::new();
        while let Some(separator) = remaining.next_if(|character| !character.is_alphanumeric()) {
            match aligned.last_mut() {
                Some(previous) => push_character(previous, separator),
                None => taken.push(separator),
            }
        }
        take_word(characters_of(word), &mut remaining, &mut taken)?;

        // 吃不到字符的普通词没有词时间可用，丢掉它；合并音节的结构不拆，仍然保留。
        if taken.is_empty() && !word.is_full() {
            continue;
        }

        aligned.push(match word {
            // 合并音节整体按一个词处理，再把这批字符按同样的规则分给它的子音节。
            SyllableItem::Full(full) => SyllableItem::Full(FullSyllableInfo::new(
                split_onto_parts(&taken, full.sub_items())?,
            )),
            SyllableItem::Syllable(syllable) => SyllableItem::Syllable(SyllableInfo::new(
                taken,
                syllable.start_time,
                syllable.end_time,
            )),
        });
    }

    // 最后一个词之后剩下的字符跟着它走，转录写在那里就只能是分隔符。
    for character in remaining {
        if character.is_alphanumeric() {
            return None;
        }
        push_character(aligned.last_mut()?, character);
    }

    (!aligned.is_empty()).then_some(aligned)
}

/// 把 `text` 的字符分给合并音节的子音节。
///
/// 子音节沿用同一个规则：各自吃掉到自己的最后一个词字符为止的字符，前面的分隔符归前一个
/// 子音节。合并结构不拆也不丢——吃不到字符的子音节保留自己的时间，拼起来仍然精确等于
/// `text`。
fn split_onto_parts(text: &str, parts: &[SyllableInfo]) -> Option<Vec<SyllableInfo>> {
    let mut remaining = text.chars().peekable();
    let mut aligned: Vec<SyllableInfo> = Vec::with_capacity(parts.len());

    for part in parts {
        let mut taken = String::new();
        while let Some(separator) = remaining.next_if(|character| !character.is_alphanumeric()) {
            match aligned.last_mut() {
                Some(previous) => previous.text.push(separator),
                None => taken.push(separator),
            }
        }
        take_word(part.text.chars(), &mut remaining, &mut taken)?;

        aligned.push(SyllableInfo::new(taken, part.start_time, part.end_time));
    }

    // 最后一个子音节之后剩下的字符跟着它走，只有分隔符能剩下。
    for character in remaining {
        if character.is_alphanumeric() {
            return None;
        }
        aligned.last_mut()?.text.push(character);
    }

    (!aligned.is_empty()).then_some(aligned)
}

/// 按顺序列出词项各子音节的字符，不给合并音节额外分配聚合文本。
fn characters_of(word: &SyllableItem) -> impl Iterator<Item = char> + Clone + '_ {
    word.parts().iter().flat_map(|part| part.text.chars())
}

/// 从 `remaining` 里吃掉属于 `characters` 的字符，接到 `text` 上。
///
/// 一个词是「到它最后一个词字符为止」的一段，所以词里面的分隔符（`you're` 的撇号）跟着
/// 一起吃掉，不结束这个词；字母要逐位对得上（忽略大小写）。转录不够长或对不上时返回
/// `None`。
fn take_word(
    characters: impl Iterator<Item = char> + Clone,
    remaining: &mut Peekable<Chars>,
    text: &mut String,
) -> Option<()> {
    let mut expected = characters
        .clone()
        .filter(|character| character.is_alphanumeric());
    let mut word_characters = characters
        .filter(|character| character.is_alphanumeric())
        .count();
    while word_characters > 0 {
        let character = remaining.next()?;
        if character.is_alphanumeric() {
            if !character.eq_ignore_ascii_case(&expected.next()?) {
                return None;
            }
            word_characters -= 1;
        }
        text.push(character);
    }

    Some(())
}

/// 把字符追加到词项的末尾，合并音节写在它最后一个子音节上。
fn push_character(word: &mut SyllableItem, character: char) {
    match word {
        SyllableItem::Syllable(syllable) => syllable.text.push(character),
        SyllableItem::Full(full) => {
            if let Some(sub_item) = full.sub_items_mut().last_mut() {
                sub_item.text.push(character);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// 行级转录的一行。
    fn row(start_time: i32, end_time: i32, text: &str) -> LineInfo {
        LineInfo::new_line(text.to_string(), Some(start_time), Some(end_time))
    }

    /// 逐词文档的一行，每个词为 `(开始时间, 结束时间, 文本)`。
    fn word_row(start_time: i32, words: &[(i32, i32, &str)]) -> LineInfo {
        let syllables = words
            .iter()
            .map(|(start, end, text)| SyllableInfo::new(text.to_string(), *start, *end).into())
            .collect();
        LineInfo::new_syllable_with_time(syllables, Some(start_time), None)
    }

    /// 把一行的词读成 `(开始时间, 词文本)`。
    fn words_of(line: &LineInfo) -> Vec<(i32, String)> {
        line.syllables()
            .unwrap_or_default()
            .iter()
            .map(|syllable| (syllable.start_time(), syllable.text()))
            .collect()
    }

    /// 音节拼起来精确等于该行原文，行时间与翻译保留。
    #[test]
    fn reads_the_words_of_a_row_onto_the_text_of_the_transcription() {
        let mut lines = vec![row(90, 2_160, "Ugh, you're a monster")];
        let word_lines = vec![word_row(
            90,
            &[
                (90, 420, "Ugh"),
                (420, 690, "you're "),
                (690, 960, "a "),
                (960, 2_160, "monster"),
            ],
        )];

        apply_word_timings(&mut lines, &word_lines);

        // 逗号是转录写的：逐词文档把它吃掉了，这里写在 `Ugh` 的末尾。
        assert_eq!(
            words_of(&lines[0]),
            vec![
                (90, "Ugh, ".to_string()),
                (420, "you're ".to_string()),
                (690, "a ".to_string()),
                (960, "monster".to_string()),
            ]
        );
        assert_eq!(
            LineInfo::text_from_syllables(lines[0].syllables().unwrap()),
            "Ugh, you're a monster"
        );
        // 行时间保留
        assert_eq!(
            (lines[0].start_time(), lines[0].end_time()),
            (Some(90), Some(2_160))
        );
    }

    /// 大小写取转录写的那个。
    #[test]
    fn takes_the_characters_of_the_transcription() {
        let mut lines = vec![row(0, 1_000, "Ugh! You're a monster")];
        let word_lines = vec![word_row(0, &[(0, 1_000, "ughyou're a monster")])];

        apply_word_timings(&mut lines, &word_lines);

        assert_eq!(
            LineInfo::text_from_syllables(lines[0].syllables().unwrap()),
            "Ugh! You're a monster"
        );
    }

    /// 逐词文档打码的词与转录写全的词对不上：整行放弃，保留原有行时间。
    #[test]
    fn keeps_a_row_whose_words_do_not_spell_it() {
        let mut lines = vec![
            row(90, 2_160, "Shady's in this bitch, I'm posse'd up"),
            row(18_540, 20_900, "Consider it to cross me a costly mistake"),
        ];
        let word_lines = vec![
            word_row(
                90,
                &[
                    (90, 400, "Shady's "),
                    (400, 600, "in "),
                    (600, 800, "this "),
                    (800, 1_100, "*****"),
                    (1_100, 1_400, "I'm "),
                    (1_400, 2_160, "posse'd up"),
                ],
            ),
            word_row(
                18_540,
                &[
                    (18_540, 18_700, "Consider "),
                    (18_700, 19_900, "it to cross me a costly mistake"),
                ],
            ),
        ];

        apply_word_timings(&mut lines, &word_lines);

        // 打码那一行没有配上词，也还没有部分贴合。
        assert!(lines[0].syllables().is_none());
        assert_eq!(lines[0].text(), "Shady's in this bitch, I'm posse'd up");
        assert_eq!(
            (lines[0].start_time(), lines[0].end_time()),
            (Some(90), Some(2_160))
        );
        // 放弃一行不影响后面的行。
        assert_eq!(
            words_of(&lines[1]),
            vec![
                (18_540, "Consider ".to_string()),
                (18_700, "it to cross me a costly mistake".to_string()),
            ]
        );
    }

    /// 两份文档按顺序配对，逐词行早于本行超过一个容差就配不上后面的行。
    #[test]
    fn pairs_the_rows_in_order() {
        let mut lines = vec![
            row(396, 2_160, "Ugh, you're a monster"),
            row(2_345, 3_000, "Second row"),
        ];
        let word_lines = vec![
            word_row(90, &[(90, 2_160, "Ughyou're a monster")]),
            word_row(2_370, &[(2_370, 3_000, "Second row")]),
        ];

        apply_word_timings(&mut lines, &word_lines);

        assert_eq!(words_of(&lines[0])[0].0, 90);
        assert_eq!(words_of(&lines[1]), vec![(2_370, "Second row".to_string())]);
    }

    /// 相差正好 1000 毫秒仍算同一行。
    #[test]
    fn pairs_a_row_that_starts_within_the_tolerance() {
        let mut lines = vec![row(ROW_TOLERANCE_MS, 2_000, "Ugh, you're a monster")];
        let word_lines = vec![word_row(0, &[(0, 2_000, "Ughyou're a monster")])];

        apply_word_timings(&mut lines, &word_lines);

        assert_eq!(words_of(&lines[0])[0].0, 0);
    }

    /// 相差超过 1000 毫秒就不是同一行：这一行不带词时间。
    #[test]
    fn leaves_a_row_that_starts_beyond_the_tolerance() {
        let mut lines = vec![row(ROW_TOLERANCE_MS + 1, 2_000, "Ugh, you're a monster")];
        let word_lines = vec![word_row(0, &[(0, 2_000, "Ughyou're a monster")])];

        apply_word_timings(&mut lines, &word_lines);

        assert!(lines[0].syllables().is_none());
        assert_eq!(
            (lines[0].start_time(), lines[0].end_time()),
            (Some(ROW_TOLERANCE_MS + 1), Some(2_000))
        );
    }

    /// 没有词（音节）的逐词行给不出词时间，也不占用某一行的配对。
    #[test]
    fn skips_word_rows_without_words() {
        let mut lines = vec![row(1_050, 2_000, "Ugh, you're a monster")];
        let info_line = LineInfo::new_line("作词: 某人".to_string(), Some(1_000), None);
        let word_lines = vec![
            info_line,
            word_row(1_100, &[(1_100, 2_000, "Ughyou're a monster")]),
        ];

        apply_word_timings(&mut lines, &word_lines);

        assert_eq!(words_of(&lines[0])[0].0, 1_100);
    }

    /// 合并音节按一个词处理：整体吃词，子音节按同样的规则分字符，各自的时间保留。
    #[test]
    fn keeps_merged_syllables_as_one_word_with_their_own_times() {
        let mut lines = vec![row(0, 1_000, "Hello you're")];
        let word_lines = vec![LineInfo::new_syllable_with_time(
            vec![
                SyllableInfo::new("Hello ".to_string(), 0, 400).into(),
                FullSyllableInfo::new(vec![
                    SyllableInfo::new("you".to_string(), 400, 600),
                    SyllableInfo::new("'re".to_string(), 600, 900),
                ])
                .into(),
            ],
            Some(0),
            None,
        )];

        apply_word_timings(&mut lines, &word_lines);

        assert_eq!(
            words_of(&lines[0]),
            vec![(0, "Hello ".to_string()), (400, "you're".to_string())]
        );
        // 子音节沿用同一个规则：`you're` 的词内撇号是分隔符，归前一个子音节。
        let merged = &lines[0].syllables().unwrap()[1];
        assert_eq!(
            merged
                .as_full()
                .expect("合并音节不拆")
                .sub_items()
                .iter()
                .map(|sub_item| (
                    sub_item.start_time,
                    sub_item.end_time,
                    sub_item.text.as_str()
                ))
                .collect::<Vec<_>>(),
            vec![(400, 600, "you'"), (600, 900, "re")]
        );
        assert_eq!(
            LineInfo::text_from_syllables(lines[0].syllables().unwrap()),
            "Hello you're"
        );
    }

    /// 合并音节的子音节与转录对不上：整行放弃。
    #[test]
    fn keeps_a_row_whose_merged_syllables_do_not_spell_it() {
        let mut lines = vec![row(0, 1_000, "Hello world")];
        let word_lines = vec![LineInfo::new_syllable_with_time(
            vec![
                SyllableInfo::new("Hello ".to_string(), 0, 400).into(),
                FullSyllableInfo::new(vec![
                    SyllableInfo::new("wor".to_string(), 400, 700),
                    SyllableInfo::new("lds".to_string(), 700, 900),
                ])
                .into(),
            ],
            Some(0),
            None,
        )];

        apply_word_timings(&mut lines, &word_lines);

        assert!(lines[0].syllables().is_none());
        assert_eq!(lines[0].text(), "Hello world");
    }

    /// 翻译与拼音跟着行一起保留。
    #[test]
    fn keeps_the_translation_of_a_full_row() {
        let mut translations = HashMap::new();
        translations.insert("zh".to_string(), "呕，你真是只怪兽".to_string());
        let mut lines = vec![LineInfo::new_full_line(
            "Ugh, you're a monster".to_string(),
            Some(90),
            Some(2_160),
            translations,
            None,
        )];
        let word_lines = vec![word_row(90, &[(90, 2_160, "Ughyou're a monster")])];

        apply_word_timings(&mut lines, &word_lines);

        assert_eq!(lines[0].chinese_translation(), Some("呕，你真是只怪兽"));
        assert_eq!(
            LineInfo::text_from_syllables(lines[0].syllables().unwrap()),
            "Ugh, you're a monster"
        );
    }

    /// 逐词文档为空时不改动任何一行。
    #[test]
    fn leaves_the_lines_alone_without_word_lines() {
        let mut lines = vec![row(0, 1_000, "Ugh, you're a monster")];

        apply_word_timings(&mut lines, &[]);

        assert!(lines[0].syllables().is_none());
        assert_eq!(lines[0].text(), "Ugh, you're a monster");
    }
}
