use lyrics_core::helpers::chinese_helper::to_simplified;
use lyrics_core::helpers::string_helper::{compute_text_same, remove_duo_spaces};

/// 曲目匹配等级，用于评估搜索结果与目标曲目的相似程度。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MatchType {
    /// 不匹配
    NoMatch = -1,
    /// 极低匹配度
    VeryLow = 10,
    /// 低匹配度
    Low = 30,
    /// 中等匹配度
    Medium = 70,
    /// 较高匹配度
    PrettyHigh = 90,
    /// 高匹配度
    High = 95,
    /// 极高匹配度
    VeryHigh = 99,
    /// 完全匹配
    Perfect = 100,
}

impl PartialOrd for MatchType {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MatchType {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (*self as i32).cmp(&(*other as i32))
    }
}

/// 名称匹配等级。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameMatchType {
    /// 不匹配
    NoMatch = -1,
    /// 低匹配度
    Low = 0,
    /// 中等匹配度
    Medium = 1,
    /// 高匹配度
    High = 2,
    /// 极高匹配度
    VeryHigh = 3,
    /// 完全匹配
    Perfect = 4,
}

/// 艺术家匹配等级。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtistMatchType {
    /// 不匹配
    NoMatch = -1,
    /// 低匹配度
    Low = 0,
    /// 中等匹配度
    Medium = 1,
    /// 高匹配度
    High = 2,
    /// 极高匹配度
    VeryHigh = 3,
    /// 完全匹配
    Perfect = 4,
}

/// 时长匹配等级。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DurationMatchType {
    /// 不匹配
    NoMatch = -1,
    /// 低匹配度
    Low = 0,
    /// 中等匹配度
    Medium = 1,
    /// 高匹配度
    High = 2,
    /// 极高匹配度
    VeryHigh = 3,
    /// 完全匹配
    Perfect = 4,
}

/// 三种匹配等级共用的分值表（判别值 -1..=4 映射到分值）。
fn level_score(level: i32) -> f64 {
    match level {
        4 => 7.0, // Perfect
        3 => 6.0, // VeryHigh
        2 => 5.0, // High
        1 => 4.0, // Medium
        0 => 2.0, // Low
        _ => 0.0, // NoMatch
    }
}

impl NameMatchType {
    /// 返回该匹配等级对应的分值。
    pub fn score(self) -> f64 {
        level_score(self as i32)
    }
}

impl ArtistMatchType {
    /// 返回该匹配等级对应的分值。
    pub fn score(self) -> f64 {
        level_score(self as i32)
    }
}

impl DurationMatchType {
    /// 返回该匹配等级对应的分值。
    pub fn score(self) -> f64 {
        level_score(self as i32)
    }
}

/// 计算名称匹配类型的分值，`None` 时返回 0。
pub fn name_score(m: Option<NameMatchType>) -> f64 {
    m.map_or(0.0, |m| m.score())
}

/// 计算艺术家匹配类型的分值，`None` 时返回 0。
pub fn artist_score(m: Option<ArtistMatchType>) -> f64 {
    m.map_or(0.0, |m| m.score())
}

/// 计算时长匹配类型的分值，`None` 时返回 0。
pub fn duration_score(m: Option<DurationMatchType>) -> f64 {
    m.map_or(0.0, |m| m.score())
}

/// 比较两个时长值（毫秒）的匹配程度。
pub fn compare_duration(d1: Option<i32>, d2: Option<i32>) -> Option<DurationMatchType> {
    let d1 = d1?;
    let d2 = d2?;
    if d1 == 0 || d2 == 0 {
        return None;
    }
    let diff = (d1 - d2).unsigned_abs();
    Some(match diff {
        0 => DurationMatchType::Perfect,
        1..300 => DurationMatchType::VeryHigh,
        300..700 => DurationMatchType::High,
        700..1500 => DurationMatchType::Medium,
        1500..3500 => DurationMatchType::Low,
        _ => DurationMatchType::NoMatch,
    })
}

/// 比较两组艺术家列表的匹配程度，支持繁简中文转换。
pub fn compare_artist(artist1: &[String], artist2: &[String]) -> Option<ArtistMatchType> {
    let list1: Vec<String> = artist1
        .iter()
        .filter(|a| !a.trim().is_empty())
        .map(|a| to_simplified(&a.to_lowercase()))
        .collect();
    let list2: Vec<String> = artist2
        .iter()
        .filter(|a| !a.trim().is_empty())
        .map(|a| to_simplified(&a.to_lowercase()))
        .collect();

    if list1.is_empty() || list2.is_empty() {
        return None;
    }

    let count = list2.iter().filter(|a| list1.contains(a)).count();

    if count == list1.len() && list1.len() == list2.len() {
        return Some(ArtistMatchType::Perfect);
    }

    if (count + 1 >= list1.len() && list1.len() >= 2)
        || (list1.len() > 6 && count as f64 / list1.len() as f64 > 0.8)
    {
        return Some(ArtistMatchType::VeryHigh);
    }

    if count == 1 && list1.len() == 1 && list2.len() == 2 {
        return Some(ArtistMatchType::High);
    }

    if list1.len() > 5 && (list2[0].contains("Various") || list2[0].contains("群星")) {
        return Some(ArtistMatchType::VeryHigh);
    }

    if list1.len() > 7 && list2.len() > 7 && count as f64 / list1.len() as f64 > 0.66 {
        return Some(ArtistMatchType::High);
    }

    if list1.len() == 1 && list2.len() > 1 && list1[0].starts_with(list2[0].as_str()) {
        return Some(ArtistMatchType::High);
    }

    if list1.len() == 1
        && list2.len() > 1
        && list2[0].chars().count() > 3
        && list1[0].contains(list2[0].as_str())
    {
        return Some(ArtistMatchType::High);
    }

    if list1.len() == 1
        && list2.len() > 1
        && list2[0].chars().count() > 1
        && list1[0].contains(list2[0].as_str())
    {
        return Some(ArtistMatchType::Medium);
    }

    if count == 1 && list1.len() == 1 && list2.len() >= 3 {
        return Some(ArtistMatchType::Medium);
    }

    if count >= 2 {
        return Some(ArtistMatchType::Low);
    }

    Some(ArtistMatchType::NoMatch)
}

/// 计算两个字符串在相同位置上字符相等的数量。
pub fn chars_eq_at(s1: &str, s2: &str) -> usize {
    s1.chars().zip(s2.chars()).filter(|(a, b)| a == b).count()
}

/// 比较两个名称（标题或专辑）的匹配程度，支持繁简中文和特殊标记容错。
pub fn compare_name(name1: Option<&str>, name2: Option<&str>) -> Option<NameMatchType> {
    let name1 = name1.filter(|s| !s.trim().is_empty())?;
    let name2 = name2.filter(|s| !s.trim().is_empty())?;

    fn normalize_name(name: &str) -> String {
        let normalized = to_simplified(name)
            .to_lowercase()
            .trim()
            .replace('\u{2019}', "'")
            .replace('\u{ff0c}', ",")
            .replace('\u{ff08}', "(")
            .replace('\u{ff09}', ")")
            .replace('[', "(")
            .replace(']', ")");
        remove_duo_spaces(&normalized)
            .replace(" (", "(")
            .replace("( ", "(")
            .replace(" )", ")")
    }

    let mut n1 = normalize_name(name1);
    let mut n2 = normalize_name(name2);

    if n1 == n2 {
        return Some(NameMatchType::Perfect);
    }

    n1 = n1.replace("acoustic version", "acoustic");
    n2 = n2.replace("acoustic version", "acoustic");

    {
        let s1 = format!("{})", n1.replace(" - ", " (").trim());
        let s2 = format!("{})", n2.replace(" - ", " (").trim());
        if s1.replace(' ', "") == s2.replace(' ', "") {
            return Some(NameMatchType::VeryHigh);
        }
    }

    /// 取 `marker` 首次出现之前、去掉首尾空白的部分。
    fn prefix_before<'a>(text: &'a str, marker: &str) -> Option<&'a str> {
        text.find(marker).map(|index| text[..index].trim())
    }

    /// 一侧带 `(special…` 标记、另一侧没有：去掉标记段后相等即视为同一首。
    fn special_compare(str1: &str, str2: &str, special: &str) -> bool {
        let marker = format!("({special}");
        match (prefix_before(str1, &marker), prefix_before(str2, &marker)) {
            (Some(prefix), None) => prefix == str2,
            (None, Some(prefix)) => prefix == str1,
            _ => false,
        }
    }

    /// 两侧都带同一个 `(special…` 标记：比较标记之前的部分。
    fn single_special_compare(str1: &str, str2: &str, special: &str) -> bool {
        let marker = format!("({special}");
        match (prefix_before(str1, &marker), prefix_before(str2, &marker)) {
            (Some(prefix1), Some(prefix2)) => prefix1 == prefix2,
            _ => false,
        }
    }

    /// 两侧分别带两个不同的 `(special…` 标记：比较各自标记之前的部分。
    fn duo_special_compare(str1: &str, str2: &str, special1: &str, special2: &str) -> bool {
        let marker1 = format!("({special1}");
        let marker2 = format!("({special2}");
        matches!(
            (prefix_before(str1, &marker1), prefix_before(str2, &marker2)),
            (Some(prefix1), Some(prefix2)) if prefix1 == prefix2
        ) || matches!(
            (prefix_before(str1, &marker2), prefix_before(str2, &marker1)),
            (Some(prefix1), Some(prefix2)) if prefix1 == prefix2
        )
    }

    /// 一侧带括号、另一侧没有：去掉括号段后相等即视为同一首。
    fn brackets_compare(str1: &str, str2: &str) -> bool {
        special_compare(str1, str2, "")
    }

    /// 只出现在其中一侧、可以忽略的版本标记。
    const OPTIONAL_MARKERS: [&str; 6] = [
        "deluxe",
        "explicit",
        "special edition",
        "bonus track",
        "feat",
        "with",
    ];

    if OPTIONAL_MARKERS
        .iter()
        .any(|marker| special_compare(&n1, &n2, marker))
    {
        return Some(NameMatchType::VeryHigh);
    }

    if duo_special_compare(&n1, &n2, "feat", "explicit")
        || duo_special_compare(&n1, &n2, "with", "explicit")
        || single_special_compare(&n1, &n2, "feat")
        || single_special_compare(&n1, &n2, "with")
    {
        return Some(NameMatchType::High);
    }

    if brackets_compare(&n1, &n2) {
        return Some(NameMatchType::Medium);
    }

    let n1_len = n1.chars().count();
    let n2_len = n2.chars().count();
    if n1_len == n2_len && n1_len > 0 {
        let count = chars_eq_at(&n1, &n2);
        if (count as f64 / n1_len as f64 >= 0.8 && n1_len >= 4)
            || (count as f64 / n1_len as f64 >= 0.5 && (2..=3).contains(&n1_len))
        {
            return Some(NameMatchType::High);
        }
    }

    let text_same = compute_text_same(&n1, &n2, true);
    if text_same > 90.0 {
        return Some(NameMatchType::VeryHigh);
    }
    if text_same > 80.0 {
        return Some(NameMatchType::High);
    }
    if text_same > 68.0 {
        return Some(NameMatchType::Medium);
    }
    if text_same > 55.0 {
        return Some(NameMatchType::Low);
    }

    Some(NameMatchType::NoMatch)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_duration_exact() {
        assert_eq!(
            compare_duration(Some(180000), Some(180000)),
            Some(DurationMatchType::Perfect)
        );
    }

    #[test]
    fn test_compare_duration_close() {
        assert_eq!(
            compare_duration(Some(180000), Some(180200)),
            Some(DurationMatchType::VeryHigh)
        );
    }

    #[test]
    fn test_compare_duration_none() {
        assert_eq!(compare_duration(None, Some(180000)), None);
    }

    #[test]
    fn test_compare_duration_zero() {
        assert_eq!(compare_duration(Some(0), Some(180000)), None);
    }

    #[test]
    fn test_compare_name_exact() {
        assert_eq!(
            compare_name(Some("Hello"), Some("Hello")),
            Some(NameMatchType::Perfect)
        );
    }

    #[test]
    fn test_compare_name_case_insensitive() {
        assert_eq!(
            compare_name(Some("Hello"), Some("hello")),
            Some(NameMatchType::Perfect)
        );
    }

    #[test]
    fn test_compare_name_different() {
        let result = compare_name(Some("Hello World"), Some("Something Completely Different"));
        assert_eq!(result, Some(NameMatchType::NoMatch));
    }

    #[test]
    fn test_compare_name_none() {
        assert_eq!(compare_name(None, Some("Hello")), None);
    }

    #[test]
    fn test_compare_name_blank() {
        assert_eq!(compare_name(Some("Hello"), Some(" \t ")), None);
    }

    #[test]
    fn test_compare_name_fullwidth_brackets() {
        assert_eq!(
            compare_name(Some("Song（Deluxe）"), Some("Song (Deluxe)")),
            Some(NameMatchType::Perfect)
        );
    }

    #[test]
    fn test_compare_artist_exact() {
        let a = vec!["Taylor Swift".to_string()];
        let b = vec!["Taylor Swift".to_string()];
        assert_eq!(compare_artist(&a, &b), Some(ArtistMatchType::Perfect));
    }

    #[test]
    fn test_compare_artist_empty() {
        let a: Vec<String> = vec![];
        let b = vec!["Taylor Swift".to_string()];
        assert_eq!(compare_artist(&a, &b), None);
    }

    #[test]
    fn test_compare_artist_subset() {
        let a = vec!["Taylor Swift".to_string(), "Ed Sheeran".to_string()];
        let b = vec!["Taylor Swift".to_string()];
        let result = compare_artist(&a, &b);
        // 1 out of 2 match
        assert!(result.is_some());
    }
}
