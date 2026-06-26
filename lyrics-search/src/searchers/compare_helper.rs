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

impl NameMatchType {
    /// 返回该匹配等级对应的分值。
    pub fn score(self) -> f64 {
        match self {
            NameMatchType::Perfect => 7.0,
            NameMatchType::VeryHigh => 6.0,
            NameMatchType::High => 5.0,
            NameMatchType::Medium => 4.0,
            NameMatchType::Low => 2.0,
            NameMatchType::NoMatch => 0.0,
        }
    }
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

impl ArtistMatchType {
    /// 返回该匹配等级对应的分值。
    pub fn score(self) -> f64 {
        match self {
            ArtistMatchType::Perfect => 7.0,
            ArtistMatchType::VeryHigh => 6.0,
            ArtistMatchType::High => 5.0,
            ArtistMatchType::Medium => 4.0,
            ArtistMatchType::Low => 2.0,
            ArtistMatchType::NoMatch => 0.0,
        }
    }
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

impl DurationMatchType {
    /// 返回该匹配等级对应的分值。
    pub fn score(self) -> f64 {
        match self {
            DurationMatchType::Perfect => 7.0,
            DurationMatchType::VeryHigh => 6.0,
            DurationMatchType::High => 5.0,
            DurationMatchType::Medium => 4.0,
            DurationMatchType::Low => 2.0,
            DurationMatchType::NoMatch => 0.0,
        }
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
    if artist1.is_empty() || artist2.is_empty() {
        return None;
    }

    let list1: Vec<String> = artist1
        .iter()
        .map(|a| to_simplified(&a.to_lowercase()))
        .collect();
    let list2: Vec<String> = artist2
        .iter()
        .map(|a| to_simplified(&a.to_lowercase()))
        .collect();

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

    if list1.len() > 5
        && (list2[0].contains("Various") || list2[0].contains("群星"))
    {
        return Some(ArtistMatchType::VeryHigh);
    }

    if list1.len() > 7 && list2.len() > 7 && count as f64 / list1.len() as f64 > 0.66 {
        return Some(ArtistMatchType::High);
    }

    if list1.len() == 1 && list2.len() > 1 && list1[0].starts_with(list2[0].as_str()) {
        return Some(ArtistMatchType::High);
    }

    if list1.len() == 1 && list2.len() > 1 && list2[0].chars().count() > 3 && list1[0].contains(list2[0].as_str()) {
        return Some(ArtistMatchType::High);
    }

    if list1.len() == 1 && list2.len() > 1 && list2[0].chars().count() > 1 && list1[0].contains(list2[0].as_str()) {
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
    s1.chars()
        .zip(s2.chars())
        .filter(|(a, b)| a == b)
        .count()
}

/// 比较两个名称（标题或专辑）的匹配程度，支持繁简中文和特殊标记容错。
pub fn compare_name(name1: Option<&str>, name2: Option<&str>) -> Option<NameMatchType> {
    let name1 = name1?;
    let name2 = name2?;

    let mut n1 = to_simplified(&name1.to_lowercase()).trim().to_string();
    let mut n2 = to_simplified(&name2.to_lowercase()).trim().to_string();

    if n1 == n2 {
        return Some(NameMatchType::Perfect);
    }

    n1 = n1
        .replace('\u{2019}', "'")
        .replace('\u{ff0c}', ",")
        .replace("\u{ff08}", " (")
        .replace("\u{ff09}", " )")
        .replace('[', "(")
        .replace(']', ")");
    n1 = remove_duo_spaces(&n1);

    n2 = n2
        .replace('\u{2019}', "'")
        .replace('\u{ff0c}', ",")
        .replace("\u{ff08}", " (")
        .replace("\u{ff09}", " )")
        .replace('[', "(")
        .replace(']', ")");
    n2 = remove_duo_spaces(&n2);

    n1 = n1.replace("acoustic version", "acoustic");
    n2 = n2.replace("acoustic version", "acoustic");

    {
        let s1 = format!("{})", n1.replace(" - ", " (").trim());
        let s2 = format!("{})", n2.replace(" - ", " (").trim());
        if s1.replace(' ', "") == s2.replace(' ', "") {
            return Some(NameMatchType::VeryHigh);
        }
    }

    fn special_compare(str1: &str, str2: &str, special: &str) -> bool {
        let special = format!("({special}");
        let c1 = str1.contains(&special);
        let c2 = str2.contains(&special);
        if c1 && !c2 {
            let idx = str1.find(&special).unwrap();
            if str1[..idx].trim() == str2 {
                return true;
            }
        }
        if c2 && !c1 {
            let idx = str2.find(&special).unwrap();
            if str2[..idx].trim() == str1 {
                return true;
            }
        }
        false
    }

    fn single_special_compare(str1: &str, str2: &str, special: &str) -> bool {
        let special = format!("({special}");
        if str1.contains(&special) && str2.contains(&special) {
            let i1 = str1.find(&special).unwrap();
            let i2 = str2.find(&special).unwrap();
            if str1[..i1].trim() == str2[..i2].trim() {
                return true;
            }
        }
        false
    }

    fn duo_special_compare(str1: &str, str2: &str, special1: &str, special2: &str) -> bool {
        let s1 = format!("({special1}");
        let s2 = format!("({special2}");
        if str1.contains(&s1) && str2.contains(&s2) {
            let i1 = str1.find(&s1).unwrap();
            let i2 = str2.find(&s2).unwrap();
            if str1[..i1].trim() == str2[..i2].trim() {
                return true;
            }
        }
        if str1.contains(&s2) && str2.contains(&s1) {
            let i1 = str1.find(&s2).unwrap();
            let i2 = str2.find(&s1).unwrap();
            if str1[..i1].trim() == str2[..i2].trim() {
                return true;
            }
        }
        false
    }

    fn brackets_compare(str1: &str, str2: &str) -> bool {
        if str1.contains('(') && !str2.contains('(') {
            let idx = str1.find('(').unwrap();
            if str1[..idx].trim() == str2 {
                return true;
            }
        }
        if str2.contains('(') && !str1.contains('(') {
            let idx = str2.find('(').unwrap();
            if str2[..idx].trim() == str1 {
                return true;
            }
        }
        false
    }

    if special_compare(&n1, &n2, "deluxe") {
        return Some(NameMatchType::VeryHigh);
    }
    if special_compare(&n1, &n2, "explicit") {
        return Some(NameMatchType::VeryHigh);
    }
    if special_compare(&n1, &n2, "special edition") {
        return Some(NameMatchType::VeryHigh);
    }
    if special_compare(&n1, &n2, "bonus track") {
        return Some(NameMatchType::VeryHigh);
    }
    if special_compare(&n1, &n2, "feat") {
        return Some(NameMatchType::VeryHigh);
    }
    if special_compare(&n1, &n2, "with") {
        return Some(NameMatchType::VeryHigh);
    }

    if duo_special_compare(&n1, &n2, "feat", "explicit") {
        return Some(NameMatchType::High);
    }
    if duo_special_compare(&n1, &n2, "with", "explicit") {
        return Some(NameMatchType::High);
    }
    if single_special_compare(&n1, &n2, "feat") {
        return Some(NameMatchType::High);
    }
    if single_special_compare(&n1, &n2, "with") {
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
        assert_eq!(compare_duration(Some(180000), Some(180000)), Some(DurationMatchType::Perfect));
    }

    #[test]
    fn test_compare_duration_close() {
        assert_eq!(compare_duration(Some(180000), Some(180200)), Some(DurationMatchType::VeryHigh));
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
        assert_eq!(compare_name(Some("Hello"), Some("Hello")), Some(NameMatchType::Perfect));
    }

    #[test]
    fn test_compare_name_case_insensitive() {
        assert_eq!(compare_name(Some("Hello"), Some("hello")), Some(NameMatchType::Perfect));
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

