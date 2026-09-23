//! Explicit 内容处理，对应上游 `Helpers/Optimization/Explicit.cs`。
//!
//! 提供两级屏蔽：
//! - [`clean`] 的 `strong = false`：保留首尾字符（`fuck` -> `f**k`），并按单词边界屏蔽 `ass` / `hoe`；
//! - [`clean`] 的 `strong = true`：先用 [`fix_explicit`] 把已屏蔽的星号形式还原成原词，
//!   再把整词屏蔽为星号。
//!
//! C# 的 `string` 以 UTF-16 码元索引，`Clean` 中的下标运算与 `Remove` / `Insert` 都基于码元；
//! Rust 的 `String` 是 UTF-8，按字节/字符索引都会与上游错位，因此这里的边界循环同样在
//! `Vec<u16>`（UTF-16 码元）上运算，保证边界判断与替换位置和上游逐码元一致。

use std::borrow::Cow;
use std::sync::LazyLock;

use regex::{Captures, Regex, RegexBuilder};

use super::utf16::is_upper_unit;

/// 处理字符串中的 Explicit 内容。
///
/// `strong` 为 `true` 时整词完全屏蔽为星号，否则只屏蔽词内部的字母。
pub fn clean(str: &str, strong: bool) -> String {
    if strong {
        // 上游顺序：先还原已屏蔽形式，再整词屏蔽，最后处理三处边界词。
        let str = apply_replacements(&fix_explicit(str), STRONG_MASKS);
        apply_boundary_masks(
            &str,
            [(*b"ass", *b"***"), (*b"Ass", *b"***"), (*b"hoe", *b"***")],
        )
    } else {
        // 上游顺序：先做一批固定替换，再处理三处边界词。
        let str = apply_replacements(str, PARTIAL_MASKS);
        apply_boundary_masks(
            &str,
            [(*b"ass", *b"a*s"), (*b"Ass", *b"A*s"), (*b"hoe", *b"h*e")],
        )
    }
}

/// 按表顺序逐项替换，对应上游 `Clean` 中链式的 `string.Replace`。
///
/// 替换必须保持顺序：后一项作用在前一项的结果上，与上游一致。
fn apply_replacements(str: &str, masks: &[(&str, &str)]) -> String {
    masks.iter().fold(str.to_string(), |text, (word, mask)| {
        text.replace(word, mask)
    })
}

/// `strong = true` 时的整词屏蔽表，对应上游 `Clean` 中的替换链。
///
/// 两种首字母大小写共用同一串星号，仍分开列出以保持与上游逐项一致的替换顺序。
const STRONG_MASKS: &[(&str, &str)] = &[
    ("bitches", "*****"),
    ("Bitches", "*****"),
    ("bitch", "*****"),
    ("Bitch", "*****"),
    ("damn", "****"),
    ("Damn", "****"),
    ("dammit", "******"),
    ("Dammit", "******"),
    ("dick", "****"),
    ("Dick", "****"),
    ("dope", "****"),
    ("Dope", "****"),
    ("fuck", "****"),
    ("Fuck", "****"),
    ("nigga", "*****"),
    ("Nigga", "*****"),
    ("nigras", "******"),
    ("Nigras", "******"),
    ("pussy", "*****"),
    ("Pussy", "*****"),
    ("sex", "***"),
    ("Sex", "***"),
    ("shit", "****"),
    ("Shit", "****"),
    ("weed", "****"),
    ("Weed", "****"),
    ("whore", "*****"),
    ("Whore", "*****"),
    ("cocaine", "*******"),
    ("Cocaine", "*******"),
    ("drug", "****"),
    ("Drug", "****"),
];

/// `strong = false` 时的词内屏蔽表，对应上游 `Clean` 中的替换链。
///
/// 上游的 `"dammit" -> "D**mit"` 首字母是大写，这里原样保留。
const PARTIAL_MASKS: &[(&str, &str)] = &[
    ("bitch", "b***h"),
    ("Bitch", "B***h"),
    ("damn", "d**n"),
    ("Damn", "D**n"),
    ("dammit", "D**mit"),
    ("Dammit", "D**mit"),
    ("dick", "d**k"),
    ("Dick", "D**k"),
    ("dope", "d**e"),
    ("Dope", "D**e"),
    ("fuck", "f**k"),
    ("Fuck", "F**k"),
    ("nigga", "n***a"),
    ("Nigga", "N***a"),
    ("nigras", "n***as"),
    ("Nigras", "N***as"),
    ("pussy", "p***y"),
    ("Pussy", "P***y"),
    ("sex", "s*x"),
    ("Sex", "S*x"),
    ("shit", "s**t"),
    ("Shit", "S**t"),
    ("weed", "w**d"),
    ("Weed", "W**d"),
    ("whore", "w***e"),
    ("Whore", "W***e"),
];

/// 修复字符串，把已屏蔽的星号形式还原为原词（对应上游 `FixExplicit`）。
///
/// 逐个应用 `REPLACEMENTS` 中的正则（忽略大小写）；若匹配到的首字符为大写，
/// 则替换词的首字母也大写（对应上游 `MatchEvaluator`）。
///
/// 与上游唯一的差异在“忽略大小写”的实现上：上游 `RegexOptions.IgnoreCase` 按
/// `char.ToLower` 逐码元比较，Rust `regex` 用的是 Unicode 简单大小写折叠表，两者对个别
/// 字符的判断不一致 —— U+017F `ſ` 在折叠表里等价于 `s`（.NET 不认为等价），
/// 反之 .NET 把 U+0130 `İ` 折成 `i`（折叠表不折）。ASCII 及常规歌词文本下两者结果完全一致。
pub fn fix_explicit(str: &str) -> String {
    let mut result = str.to_string();

    for (regex, replacement) in REPLACEMENT_REGEXES.iter() {
        // `replace_all` 未命中时返回 `Cow::Borrowed`，只有命中才需要接管新串。
        let replaced = regex.replace_all(&result, |caps: &Captures<'_>| -> Cow<'static, str> {
            // 上游 `char.IsUpper(original[0])` 按 UTF-16 码元判断首字符大小写。
            let first_unit = caps[0].encode_utf16().next().unwrap_or_default();

            if is_upper_unit(first_unit) {
                // 替换词首字符均为 ASCII 小写字母（`char.ToUpper` 对它们就等于 ASCII 大写）。
                let mut capitalized = String::with_capacity(replacement.len());
                capitalized.push(replacement.as_bytes()[0].to_ascii_uppercase() as char);
                capitalized.push_str(&replacement[1..]);
                Cow::Owned(capitalized)
            } else {
                Cow::Borrowed(replacement)
            }
        });

        if let Cow::Owned(replaced) = replaced {
            result = replaced;
        }
    }

    result
}

/// 已屏蔽形式到原词的还原表，对应上游 `Replacements`。
///
/// 顺序即上游顺序：先命中的条目先生效（例如 `n\*{3}a` 排在 `n\*{3}as` 之前，
/// 因此 `n***as` 会被还原为 `niggas` 而不是 `nigras`）。
/// 上游 C# 字面量（如 `"a\\*s"`）实际是正则 `a\*s`，即“转义星号 + 量词”，此处写成同样的正则。
static REPLACEMENTS: &[(&str, &str)] = &[
    (r"a\*s", "ass"),
    (r"a\*\*", "ass"),
    (r"b\*{3}h", "bitch"),
    (r"b\*{2}ch", "bitch"),
    (r"b\*{5}s", "bitches"),
    (r"d\*{2}n", "damn"),
    (r"d\*{2}mit", "dammit"),
    (r"d\*{2}k", "dick"),
    (r"d\*{2}e", "dope"),
    (r"f\*{2}k", "fuck"),
    (r"f\*ck", "fuck"),
    (r"fu\*k", "fuck"),
    (r"h\*e", "hoe"),
    (r"h\*{2}s", "hoes"),
    (r"mother\*{4}", "motherfuck"),
    (r"n\*{3}a", "nigga"),
    (r"n\*{2}ga", "nigga"),
    (r"ni\*{2}a", "nigga"),
    (r"n\*{3}as", "nigras"),
    (r"p\*{3}y", "pussy"),
    (r"p\*{2}sy", "pussy"),
    (r"sh\*t", "shit"),
    (r"s\*x", "sex"),
    (r"s\*{2}t", "shit"),
    (r"w\*{2}d", "weed"),
    (r"w\*{3}e", "whore"),
    (r"c\*{4}e", "cocaine"),
    (r"d\*{2}g", "drug"),
];

/// 与 [`REPLACEMENTS`] 一一对应、忽略大小写的已编译正则（对应上游 `RegexOptions.IgnoreCase`）。
static REPLACEMENT_REGEXES: LazyLock<Vec<(Regex, &'static str)>> = LazyLock::new(|| {
    REPLACEMENTS
        .iter()
        .map(|(pattern, replacement)| {
            let regex = RegexBuilder::new(pattern)
                .case_insensitive(true)
                .build()
                .expect("上游 Replacements 中的正则均为合法表达式");

            (regex, *replacement)
        })
        .collect()
});

/// 依次执行上游 `ass` / `Ass` / `hoe` 三段结构相同的边界屏蔽循环。
///
/// 三段循环在原串上顺序执行，且替换前后长度不变，因此在同一份码元序列上连续执行与上游等价。
fn apply_boundary_masks(str: &str, masks: [([u8; 3], [u8; 3]); 3]) -> String {
    let mut units: Vec<u16> = str.encode_utf16().collect();

    for (word, replacement) in masks {
        fix_boundary_word(&mut units, word, replacement);
    }

    // 入参来自合法的 `&str`，且只把 ASCII 码元替换为 ASCII 码元，结果必然合法。
    String::from_utf16_lossy(&units)
}

/// 按单词边界屏蔽一个 3 字符词，对应上游 `Fix ass` / `Fix Ass` / `Fix hoe` 的 `for` 循环。
///
/// `word` 与 `replacement` 都是 3 个 ASCII 字符（每个字符恰好一个 UTF-16 码元），
/// 因此替换前后长度不变，上游的 `Remove(i, 3)` + `Insert(i, ...)` 等价于就地覆写 3 个码元。
///
/// 仅当命中词处于单词边界（前后为空格 / 连字符，或已到串首 / 串尾）时才替换；
/// 否则按上游的 `i += 2; continue;` 跳过，再由 `for` 的自增前进 1，即一次前进 3 个码元。
fn fix_boundary_word(units: &mut [u16], word: [u8; 3], replacement: [u8; 3]) {
    const SPACE: u16 = b' ' as u16;
    const HYPHEN: u16 = b'-' as u16;

    let word = [u16::from(word[0]), u16::from(word[1]), u16::from(word[2])];
    let replacement = [
        u16::from(replacement[0]),
        u16::from(replacement[1]),
        u16::from(replacement[2]),
    ];

    // 上游循环条件 `i < str.Length - 2`：长度不足 3 时一次都不进入。
    let mut i = 0;

    while i + 2 < units.len() {
        if units[i..i + 3] == word {
            let at_left_boundary = i == 0 || units[i - 1] == SPACE || units[i - 1] == HYPHEN;
            // 上游这里只判断 `i + 3 < str.Length`，串尾没有对应分支，因此紧跟串尾视为可屏蔽。
            let at_right_boundary =
                i + 3 >= units.len() || units[i + 3] == SPACE || units[i + 3] == HYPHEN;

            if at_left_boundary && at_right_boundary {
                units[i..i + 3].copy_from_slice(&replacement);
            }

            i += 3;
        } else {
            i += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weak_clean_keeps_first_and_last_letter() {
        assert_eq!(clean("damn", false), "d**n");
        assert_eq!(clean("Fuck", false), "F**k");
        assert_eq!(
            clean("shut up and fucking listen", false),
            "shut up and f**king listen"
        );
        // 长词先替换，因此 `bitches` 只屏蔽 `bitch` 部分
        assert_eq!(clean("bitches", false), "b***hes");
        // 上游把小写 `dammit` 也映射为大写开头的 `D**mit`
        assert_eq!(clean("dammit", false), "D**mit");
        // 弱屏蔽的替换表不含 cocaine / drug（只有强屏蔽有）
        assert_eq!(clean("cocaine and drug", false), "cocaine and drug");
    }

    #[test]
    fn strong_clean_masks_whole_words() {
        assert_eq!(clean("damn", true), "****");
        assert_eq!(clean("Fuck", true), "****");
        assert_eq!(clean("Bitches be crazy", true), "***** be crazy");
        assert_eq!(clean("cocaine and drug", true), "******* and ****");
        assert_eq!(clean("ass Damn", true), "*** ****");
    }

    #[test]
    fn strong_clean_restores_masked_forms_before_masking() {
        assert_eq!(clean("F**k this s**t", true), "**** this ****");
    }

    #[test]
    fn standalone_ass_and_hoe_are_masked_at_word_boundaries() {
        assert_eq!(clean("ass", false), "a*s");
        assert_eq!(clean("Ass man", false), "A*s man");
        assert_eq!(clean("ass ass", false), "a*s a*s");
        assert_eq!(clean("hoe", false), "h*e");
        assert_eq!(clean("shoe hoe", false), "shoe h*e");
        assert_eq!(clean("ass", true), "***");
        assert_eq!(clean("ass-Ass", false), "a*s-A*s");
    }

    #[test]
    fn ass_and_hoe_inside_words_are_left_alone() {
        assert_eq!(clean("class", false), "class");
        assert_eq!(clean("pass", false), "pass");
        assert_eq!(clean("bass-", false), "bass-");
        assert_eq!(clean("badass", false), "badass");
        assert_eq!(clean("classic", false), "classic");
        assert_eq!(
            clean("what a badass bass player", false),
            "what a badass bass player"
        );
        // `ASS` / `Hoes` 不在上游循环的大小写范围内，原样保留
        assert_eq!(clean("ASS", false), "ASS");
        assert_eq!(clean("Hoes", false), "Hoes");
    }

    #[test]
    fn boundary_rule_requires_space_or_hyphen_next_to_the_match() {
        // 后接字母 / 标点 -> 跳过（上游 `i + 3 < Length && ...` 分支）
        assert_eq!(clean("asset", false), "asset");
        assert_eq!(clean("asshole", false), "asshole");
        assert_eq!(clean("ass.", false), "ass.");
        assert_eq!(clean("(ass)", false), "(ass)");
        assert_eq!(clean("Ass!", false), "Ass!");
        // 连字符视作边界：前接或后接连字符都会被屏蔽（上游行为）
        assert_eq!(clean("ass-", false), "a*s-");
        assert_eq!(clean("-ass", false), "-a*s");
        assert_eq!(clean("hoe-down", false), "h*e-down");
    }

    #[test]
    fn boundary_scan_indexes_utf16_code_units() {
        // 空格边界在 emoji 旁同样生效（上游按 UTF-16 码元扫描整串）
        assert_eq!(clean("🎵 ass 🎵", false), "🎵 a*s 🎵");
        // 相邻字符不是空格 / 连字符时不屏蔽，非 ASCII 邻居同样如此
        assert_eq!(clean("你ass好", false), "你ass好");
        assert_eq!(clean("🙂ass", false), "🙂ass");
    }

    #[test]
    fn fix_explicit_restores_words_and_preserves_capitalization() {
        assert_eq!(fix_explicit("F**k"), "Fuck");
        assert_eq!(fix_explicit("s**t"), "shit");
        assert_eq!(fix_explicit("F**K"), "Fuck");
        assert_eq!(fix_explicit("A*s"), "Ass");
        assert_eq!(fix_explicit("B***h"), "Bitch");
        assert_eq!(fix_explicit("b*****s"), "bitches");
        assert_eq!(fix_explicit("h**s"), "hoes");
        assert_eq!(fix_explicit("mother****"), "motherfuck");
        assert_eq!(fix_explicit("c****e"), "cocaine");
        // 表内顺序生效：`n\*{3}a` 先于 `n\*{3}as`，故 `n***as` 还原为 `niggas`
        assert_eq!(fix_explicit("n***as"), "niggas");
        // 未屏蔽的原文不受影响
        assert_eq!(fix_explicit("Fuck this shit"), "Fuck this shit");
    }
}
