//! Apple Music 歌词优化，对应上游 `Helpers/Optimization/AppleMusic.cs`。
//!
//! 包含两部分：
//! - [`prepare_lyrics`]：音节级预处理（边界空白裁剪、音节拆分、同单词音节合并、冗余翻译清理）；
//! - [`capitalization_normalization`]：整篇歌词的大小写规范化。

use crate::helpers::string_helper::{
    collapse_whitespace, is_chinese_or_japanese_character, remove_front_back_brackets,
};
use crate::models::{LineInfo, SyllableInfo, SyllableItem};

use super::syllable_word_merger;
use super::utf16::{
    is_letter_unit, is_lower_unit, is_upper_unit, to_upper_invariant_unit, utf16_slice,
};

// =========================
// Prepare Lyrics
// =========================

/// 预处理歌词列表。
pub fn prepare_lyrics(lines: &mut [LineInfo]) {
    for line in lines.iter_mut() {
        prepare_lyrics_line(line);
    }
}

/// 预处理单行歌词（主行与子行）。
pub fn prepare_lyrics_line(line: &mut LineInfo) {
    // 主行
    prepare_syllable_line(line);

    // SubLine（背景人声）：Rust 的行模型没有共享引用，取出处理后再放回
    if let Some(mut sub) = line.take_sub_line() {
        prepare_syllable_line(&mut sub);

        // 清理冗余翻译：翻译与原文完全一致 -> 移除该翻译项
        remove_redundant_translations(line, false);
        remove_redundant_translations(&mut sub, true);

        line.set_sub_line(Some(sub));
    } else {
        remove_redundant_translations(line, false);
    }
}

/// 音节行的预处理：裁剪边界空白、拆分音节、合并同单词音节。
fn prepare_syllable_line(line: &mut LineInfo) {
    let Some(syllables) = line.syllables_mut() else {
        return;
    };

    if syllables.is_empty() {
        return;
    }

    trim_boundary_whitespaces(syllables);

    // Step 1: 在音节内部展开/拆分
    let mut expanded: Vec<SyllableItem> = Vec::with_capacity(syllables.len() * 2);

    for syllable in syllables.drain(..) {
        let text = normalize_text(&syllable.text());
        let start_time = syllable.start_time();
        let end_time = syllable.end_time();

        if text.is_empty() {
            expanded.push(SyllableItem::Syllable(SyllableInfo::new(
                String::new(),
                start_time,
                end_time,
            )));
            continue;
        }

        if should_split_token(&text) {
            expanded.extend(
                split_into_tokens(&text, start_time, end_time)
                    .into_iter()
                    .map(SyllableItem::Syllable),
            );
        } else {
            expanded.push(SyllableItem::Syllable(SyllableInfo::new(
                text, start_time, end_time,
            )));
        }
    }

    // Step 2: 通过 FullSyllableInfo.SubItems 把同一单词的连续音节合并为一项
    *syllables = expanded;

    syllable_word_merger::merge(line);
}

/// 清理与原文完全一致的翻译项。
fn remove_redundant_translations(line: &mut LineInfo, is_background: bool) {
    let original = line.text_from_any();
    let original_norm = normalize_for_compare(&original, is_background);

    // 只有 FullLine / FullSyllable 才有翻译
    let Some(translations) = line.translations_mut() else {
        return;
    };

    if translations.is_empty() {
        return;
    }

    // 遍历时不能直接修改字典，先收集要删的 key
    let mut to_remove: Vec<String> = Vec::new();

    for (key, value) in translations.iter() {
        let trans_norm = normalize_for_compare(value, is_background);

        // “完全一致”用序数比较
        if original_norm == trans_norm {
            to_remove.push(key.clone());
        }
    }

    for key in to_remove {
        translations.remove(&key);
    }
}

/// 统一比较用文本：
/// - 去掉换行、Trim + 折叠空白；
/// - 背景人声额外去掉外层括号（半角/全角），只处理“首尾括号包裹”的情况。
fn normalize_for_compare(s: &str, is_background: bool) -> String {
    let s = s.replace(['\r', '\n'], "");
    let s = collapse_whitespace(&s);

    if !is_background {
        return s;
    }

    // 背景：去掉外层括号（支持 () 与 （）），再做一次空白归一化
    collapse_whitespace(&remove_front_back_brackets(&s))
}

/// 裁剪首尾音节的空白。
fn trim_boundary_whitespaces(syllables: &mut [SyllableItem]) {
    if let Some(first) = syllables.first_mut() {
        trim_boundary_safe(first, Edge::Start);
    }
    if let Some(last) = syllables.last_mut() {
        trim_boundary_safe(last, Edge::End);
    }
}

/// 裁剪音节的哪一侧空白。
#[derive(Clone, Copy)]
enum Edge {
    Start,
    End,
}

/// 裁剪音节指定一侧的空白，合并音节则裁剪其对应端的子音节。
fn trim_boundary_safe(syllable: &mut SyllableItem, edge: Edge) {
    fn trim(text: &str, edge: Edge) -> &str {
        match edge {
            Edge::Start => text.trim_start(),
            Edge::End => text.trim_end(),
        }
    }

    match syllable {
        SyllableItem::Syllable(si) => {
            si.text = trim(&si.text, edge).to_string();
        }
        SyllableItem::Full(fi) => {
            let sub_items = fi.sub_items_mut();
            let edge_item = match edge {
                Edge::Start => sub_items.first_mut(),
                Edge::End => sub_items.last_mut(),
            };

            if let Some(item) = edge_item {
                item.text = trim(&item.text, edge).to_string();
            }
        }
    }
}

// =========================
// Split rules
// =========================

/// 判断音节文本是否需要进一步拆分。
///
/// 触发拆分：
/// - 中文/日文可拆字符 >= 2（逐字拆）
/// - 或 中文/日文 + 拉丁混合
/// - 或 拉丁文本里出现空格 / 连字符分段
fn should_split_token(text: &str) -> bool {
    let mut zhja = 0;
    let mut has_latin = false;
    let mut has_space = false;
    let mut has_hyphen = false;

    for ch in text.chars() {
        if ch == ' ' {
            has_space = true;
        }
        if ch == '-' {
            has_hyphen = true;
        }

        if is_chinese_or_japanese_character(ch) {
            zhja += 1;
        } else if is_latin_word_char(ch) {
            has_latin = true;
        }
    }

    zhja >= 2 || (zhja >= 1 && has_latin) || (has_latin && (has_space || has_hyphen))
}

/// 把音节文本拆分为多个音节，并按权重分配时间。
///
/// 上游此处为迭代器（`yield return`）；本实现一次性收集为 `Vec`，行为相同。
fn split_into_tokens(text: &str, start_time: i32, end_time: i32) -> Vec<SyllableInfo> {
    let tokens = tokenize_mixed(text);

    if tokens.len() <= 1 {
        return vec![SyllableInfo::new(text.to_string(), start_time, end_time)];
    }

    // 加权分配时间：含中文/日文 token 权重=1；拉丁 token 权重=字母数字数(>=1)
    // `token_weight` 恒 >= 1，因此 `w_sum >= tokens.len() >= 2`。
    let weights: Vec<i32> = tokens.iter().map(|t| token_weight(t)).collect();
    let w_sum: i32 = weights.iter().sum();

    let total = end_time - start_time;
    if total <= 0 {
        return tokens
            .into_iter()
            .map(|t| SyllableInfo::new(t, start_time, end_time))
            .collect();
    }

    let mut result = Vec::with_capacity(tokens.len());
    let mut allocated = 0;
    let mut cur_start = start_time;

    for (i, token) in tokens.into_iter().enumerate() {
        let mut dur;
        if i == weights.len() - 1 {
            dur = total - allocated;
        } else {
            dur = (total as f64 * (weights[i] as f64 / w_sum as f64)).round() as i32;
            if dur < 1 {
                dur = 1;
            }

            // 给剩余 token 留至少 1ms
            let min_remain = (weights.len() - 1 - i) as i32;
            if allocated + dur > total - min_remain {
                dur = total - allocated - min_remain;
            }
        }

        let cur_end = cur_start + dur;
        allocated += dur;

        result.push(SyllableInfo::new(token, cur_start, cur_end));
        cur_start = cur_end;
    }

    result
}

/// 按“分隔符归前一个 token”的规则做混合分词：
/// - 空格 `' '` 追加到前一个 token 并结束该 token => `"word "`
/// - 连字符 `'-'` 追加到前一个 token 并结束该 token => `"Ooh-"`
/// - 中日文字符 => 每个字符一个 token
/// - 拉丁单词 => 成组
/// - 其他字符（含韩文、emoji 等）=> 有当前 token 则追加，否则追加到前一个 token
///
/// 上游逐个 UTF-16 码元遍历；此处按 `char` 遍历。截断的代理项与完整 astral 字符
/// 走的是同一条“其他字符”分支，分词结果与上游一致。
fn tokenize_mixed(text: &str) -> Vec<String> {
    let mut tokens: Vec<String> = Vec::new();
    let mut sb = String::new();

    fn flush(sb: &mut String, tokens: &mut Vec<String>) {
        if !sb.is_empty() {
            tokens.push(std::mem::take(sb));
        }
    }

    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];

        if ch == '\r' || ch == '\n' {
            i += 1;
            continue;
        }

        // Zh/Ja: per char token
        if is_chinese_or_japanese_character(ch) {
            flush(&mut sb, &mut tokens);
            tokens.push(ch.to_string());
            i += 1;
            continue;
        }

        // space / hyphen -> belongs to previous token end, and is a boundary
        if ch == ' ' || ch == '-' {
            if !sb.is_empty() {
                sb.push(ch);
                flush(&mut sb, &mut tokens);
            } else if let Some(last) = tokens.last_mut() {
                last.push(ch);
            } else {
                sb.push(ch);
            }
            i += 1;
            continue;
        }

        // Latin word chunk
        if is_latin_word_char(ch) {
            sb.push(ch);
            i += 1;

            while i < chars.len() {
                let c2 = chars[i];
                if c2 == '\r' || c2 == '\n' {
                    i += 1;
                    continue;
                }
                if is_latin_word_char(c2) {
                    sb.push(c2);
                    i += 1;
                    continue;
                }
                break;
            }
            continue;
        }

        // other chars (punctuation / Hangul / emoji ...)
        if !sb.is_empty() {
            sb.push(ch);
        } else if let Some(last) = tokens.last_mut() {
            // prefer attach to previous token (separator-to-previous style)
            last.push(ch);
        } else {
            sb.push(ch);
        }

        i += 1;
    }

    flush(&mut sb, &mut tokens);

    tokens.retain(|t| !t.is_empty());
    tokens
}

/// token 的时间分配权重：含中日文字符为 1，否则为字母数字个数（至少 1）。
fn token_weight(token: &str) -> i32 {
    if token.chars().any(is_chinese_or_japanese_character) {
        return 1;
    }

    let mut weight = 0;
    for ch in token.chars() {
        if is_letter_or_digit(ch) {
            weight += 1;
        }
    }
    weight.max(1)
}

// =========================
// Char classification
// =========================

/// 对应 C# `char.IsLetterOrDigit`：按 UTF-16 码元判断，
/// astral 字符（代理对）在 .NET 中不是字母数字，因此返回 `false`。
fn is_letter_or_digit(ch: char) -> bool {
    ch.len_utf16() == 1 && ch.is_alphanumeric()
}

/// 判断是否为拉丁单词字符。
fn is_latin_word_char(ch: char) -> bool {
    if is_letter_or_digit(ch) {
        return true;
    }
    ch == '\'' || ch == '’'
}

/// 去掉文本中的换行。
fn normalize_text(text: &str) -> String {
    text.replace(['\r', '\n'], "")
}

// =========================
// Capitalization Normalization
// =========================

/// 行的大小写类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LineCaseKind {
    /// 无字母或大小写混用
    NoLettersOrMixed,
    /// 全小写
    AllLower,
    /// 全大写
    AllUpper,
}

/// 行的大小写统计。
#[derive(Default)]
struct CaseCounts {
    lower: usize,
    upper: usize,
    other: usize,
}

impl CaseCounts {
    fn count_line(&mut self, text: &str) {
        match get_line_case_kind(text) {
            LineCaseKind::AllLower => self.lower += 1,
            LineCaseKind::AllUpper => self.upper += 1,
            LineCaseKind::NoLettersOrMixed => self.other += 1,
        }
    }
}

/// 整篇歌词的大小写规范化（主行与子行）。
///
/// 全篇以全大写为主时先整体小写再按句首大写；以全小写为主时只做句首大写；
/// 两者都不占主导（阈值 0.9）时不做任何处理。
pub fn capitalization_normalization(lines: &mut [LineInfo]) {
    // 1) 逐行分类
    let mut counts = CaseCounts::default();

    for line in lines.iter() {
        counts.count_line(&line.text_from_any());

        if let Some(sub) = line.sub_line() {
            counts.count_line(&sub.text_from_any());
        }
    }

    let total_lines = counts.lower + counts.upper + counts.other;
    if total_lines == 0 {
        return;
    }

    let lower_ratio = counts.lower as f64 / total_lines as f64;
    let upper_ratio = counts.upper as f64 / total_lines as f64;

    const THRESHOLD: f64 = 0.9;

    // mostly ALL-UPPER -> lower first；mostly ALL-lower -> sentence-cap only
    let do_lower_case_first = if upper_ratio >= THRESHOLD {
        true
    } else if lower_ratio >= THRESHOLD {
        false
    } else {
        // 不是“基本统一”的大小写 -> 不做处理
        return;
    };

    // 2) 对所有行（主行 + 子行）应用规范化
    for line in lines.iter_mut() {
        apply_capitalization(line, do_lower_case_first);
    }
}

/// 判断一行的大小写类型。
fn get_line_case_kind(s: &str) -> LineCaseKind {
    let mut has_upper = false;
    let mut has_lower = false;

    for unit in s.encode_utf16() {
        if is_upper_unit(unit) {
            has_upper = true;
        } else if is_lower_unit(unit) {
            has_lower = true;
        } else {
            continue;
        }

        // 大小写并存即为混用，无需继续扫描。
        if has_upper && has_lower {
            return LineCaseKind::NoLettersOrMixed;
        }
    }

    match (has_upper, has_lower) {
        (true, false) => LineCaseKind::AllUpper,
        (false, true) => LineCaseKind::AllLower,
        // 没有字母，或只有无大小写之分的字母（如汉字、假名）。
        _ => LineCaseKind::NoLettersOrMixed,
    }
}

/// 对一行（含子行）应用大小写规范化。
///
/// 上游在这里可能是“替换整行对象”，Rust 的模型统一就地修改，
/// 因此子行取出处理后再放回，效果等价。
fn apply_capitalization(line: &mut LineInfo, lower_first: bool) {
    apply_capitalization_main_only(line, lower_first);

    if let Some(mut sub) = line.take_sub_line() {
        apply_capitalization_main_only(&mut sub, lower_first);
        line.set_sub_line(Some(sub));
    }
}

/// 只处理主行（不含子行）的大小写规范化。
fn apply_capitalization_main_only(line: &mut LineInfo, lower_first: bool) {
    // 音节行
    if line
        .syllables()
        .is_some_and(|syllables| !syllables.is_empty())
    {
        apply_capitalization_to_syllables(line, lower_first);
        return;
    }

    // 音节行为空时其文本也为空，上游同样直接返回
    if line.text().is_empty() {
        return;
    }

    let normalized = normalize_sentence_case_preserve_length(line.text(), lower_first);

    // 上游对 LineInfo / FullLineInfo 就地赋值 Text（保留翻译与拼音），
    // 只在“其他实现”时才新建对象——Rust 的四种变体已全部覆盖，故无替换分支。
    match line {
        LineInfo::Line { text, .. } | LineInfo::FullLine { text, .. } => *text = normalized,
        LineInfo::Syllable { .. } | LineInfo::FullSyllable { .. } => {}
    }
}

/// 对音节行逐音节重写文本（保持音节结构不变）。
fn apply_capitalization_to_syllables(line: &mut LineInfo, lower_first: bool) {
    let Some(syllables) = line.syllables_mut() else {
        return;
    };

    if syllables.is_empty() {
        return;
    }

    let full: String = syllables.iter().map(|s| s.text()).collect();
    if full.is_empty() {
        return;
    }

    let normalized = normalize_sentence_case_preserve_length(&full, lower_first);
    // 上游用 UTF-16 码元长度切分（String.Length / Substring）
    let normalized_units: Vec<u16> = normalized.encode_utf16().collect();

    let mut pos = 0;
    for syllable in syllables.iter_mut() {
        let old_text = syllable.text();
        let len = old_text.encode_utf16().count();

        if len == 0 {
            continue;
        }
        if pos + len > normalized_units.len() {
            break;
        }

        let part = utf16_slice(&normalized_units, pos, len);
        pos += len;

        rewrite_syllable_text_preserve_structure(syllable, &part);
    }

    // 上游在此调用 line.RefreshProperties()；
    // Rust 的音节行文本始终由音节列表即时拼接，无需刷新缓存。
}

/// 用新文本重写音节的文本，保持其子项结构（合并音节按子项长度切分）。
fn rewrite_syllable_text_preserve_structure(syllable: &mut SyllableItem, new_text: &str) {
    match syllable {
        SyllableItem::Syllable(si) => si.text = new_text.to_string(),
        SyllableItem::Full(fi) => {
            if fi.sub_items().is_empty() {
                // 上游对空 SubItems 的 FullSyllableInfo 会新建一个普通音节，
                // 但取 `syllable.StartTime`（等价于 `SubItems.First()`）时会抛异常；
                // 这里保留空子项时的默认时间 0，不抛异常。
                let (start_time, end_time) = (fi.start_time(), fi.end_time());
                *syllable = SyllableItem::Syllable(SyllableInfo::new(
                    new_text.to_string(),
                    start_time,
                    end_time,
                ));
                return;
            }

            let new_units: Vec<u16> = new_text.encode_utf16().collect();
            let mut pos = 0;
            for item in fi.sub_items_mut() {
                let len = item.text.encode_utf16().count();

                if len == 0 {
                    continue;
                }
                if pos + len > new_units.len() {
                    break;
                }

                item.text = utf16_slice(&new_units, pos, len);
                pos += len;
            }
        }
    }
}

/// 句子大小写规范化，并保持字符串长度不变（逐码元处理）。
///
/// 与上游一致：先是（可选的）整体小写，再把句首字母大写，
/// 并顺带把独立的小写 `i` 修正为 `I`。
fn normalize_sentence_case_preserve_length(s: &str, lower_first: bool) -> String {
    // 上游为字符串级的 ToLowerInvariant()（完整大小写映射）
    let src = if lower_first {
        s.to_lowercase()
    } else {
        s.to_string()
    };
    let units: Vec<u16> = src.encode_utf16().collect();

    let mut out: Vec<u16> = Vec::with_capacity(units.len());
    let mut cap_next = true;

    for (i, unit) in units.iter().copied().enumerate() {
        // 独立的小写 i -> I（安全、保持长度）
        if unit == 'i' as u16 && is_standalone_i(&units, i) {
            out.push('I' as u16);
            cap_next = false;
            continue;
        }

        if is_letter_unit(unit) {
            if cap_next {
                out.push(to_upper_invariant_unit(unit));
                cap_next = false;
            } else {
                out.push(unit);
            }
        } else {
            out.push(unit);
        }

        if unit == '.' as u16
            || unit == '?' as u16
            || unit == '!' as u16
            || unit == '。' as u16
            || unit == '？' as u16
            || unit == '！' as u16
        {
            cap_next = true;
        }
    }

    // 输出与输入等长，且逐码元替换不会产生落单代理项
    String::from_utf16_lossy(&out)
}

/// 判断位置 `index` 的小写 `i` 是否独立成词。
fn is_standalone_i(units: &[u16], index: usize) -> bool {
    let left_ok = index == 0 || !is_letter_unit(units[index - 1]);
    let right_ok = index == units.len() - 1 || !is_letter_unit(units[index + 1]);
    left_ok && right_ok
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::models::{FullSyllableInfo, LineInfo, SyllableInfo, SyllableItem};

    use super::*;

    fn syllable(text: &str, start_time: i32, end_time: i32) -> SyllableItem {
        SyllableItem::Syllable(SyllableInfo::new(text.to_string(), start_time, end_time))
    }

    #[test]
    fn merges_latin_syllables_but_not_cjk() {
        let mut line =
            LineInfo::new_syllable(vec![syllable("Hel", 0, 100), syllable("lo", 100, 200)]);
        prepare_lyrics_line(&mut line);

        let items = line.syllables().unwrap();
        assert_eq!(items.len(), 1);
        assert!(items[0].is_full());
        assert_eq!(items[0].text(), "Hello");
        // 合并保留各子音节的时间信息
        assert_eq!(items[0].parts().len(), 2);
        assert_eq!(items[0].parts()[0].text, "Hel");
        assert_eq!(items[0].parts()[0].start_time, 0);
        assert_eq!(items[0].parts()[1].text, "lo");
        assert_eq!(items[0].parts()[1].end_time, 200);

        let mut cjk =
            LineInfo::new_syllable(vec![syllable("你", 0, 100), syllable("好", 100, 200)]);
        prepare_lyrics_line(&mut cjk);
        assert_eq!(cjk.syllables().unwrap().len(), 2);
    }

    #[test]
    fn prepare_lyrics_trims_boundaries_and_drops_redundant_translations() {
        let sub = LineInfo::new_full_line(
            "(Hello)".to_string(),
            None,
            None,
            HashMap::from([("zh".to_string(), "Hello".to_string())]),
            None,
        );

        let mut line = LineInfo::new_full_syllable(
            vec![syllable("  Hel", 0, 100), syllable("lo  ", 100, 200)],
            HashMap::from([
                ("zh".to_string(), "Hello".to_string()),
                ("en".to_string(), "你好".to_string()),
            ]),
            None,
        );
        line.set_sub_line(Some(Box::new(sub)));

        let mut lines = vec![line];
        prepare_lyrics(&mut lines);

        let line = &lines[0];
        assert_eq!(line.text_from_any(), "Hello");
        // 与原文一致的 "zh" 被移除，"en" 保留
        assert_eq!(line.translations().unwrap().len(), 1);
        assert_eq!(line.translations().unwrap().get("en").unwrap(), "你好");

        // 背景人声：去掉外层括号后与原文一致 -> 翻译被移除
        let sub = line.sub_line().unwrap();
        assert!(sub.translations().unwrap().is_empty());
        assert_eq!(sub.text_from_any(), "(Hello)");
    }

    #[test]
    fn capitalization_normalization_lowercases_all_caps_and_keeps_standalone_i() {
        let mut lines = vec![
            LineInfo::new_line_simple("HELLO WORLD".to_string()),
            LineInfo::new_line_simple("I AM HERE".to_string()),
        ];
        capitalization_normalization(&mut lines);

        assert_eq!(lines[0].text(), "Hello world");
        assert_eq!(lines[1].text(), "I am here");
    }

    #[test]
    fn capitalization_normalization_rewrites_syllables_preserving_structure() {
        let mut lines = vec![LineInfo::new_syllable(vec![
            syllable("HE", 0, 100),
            syllable("LLO", 100, 200),
        ])];
        capitalization_normalization(&mut lines);

        let items = lines[0].syllables().unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].text(), "He");
        assert_eq!(items[1].text(), "llo");

        // 合并音节：文本按子项长度切分，且合并属性被刷新
        let mut lines = vec![LineInfo::new_syllable(vec![SyllableItem::Full(
            FullSyllableInfo::new(vec![
                SyllableInfo::new("HE".to_string(), 0, 100),
                SyllableInfo::new("LLO".to_string(), 100, 200),
            ]),
        )])];
        assert_eq!(lines[0].text_from_any(), "HELLO");
        capitalization_normalization(&mut lines);

        let items = lines[0].syllables().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].text(), "Hello");
        let sub_items = items[0].as_full().unwrap().sub_items();
        assert_eq!(sub_items[0].text, "He");
        assert_eq!(sub_items[1].text, "llo");
    }
}
