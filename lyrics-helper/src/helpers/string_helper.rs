use once_cell::sync::Lazy;
use regex::Regex;

pub fn format_time_ms_to_timestamp_string(time_ms: f32) -> String {
    let total_ms = time_ms.max(0.0) as i32;
    let minute = total_ms / 60000;
    let second = (total_ms % 60000) / 1000;
    let ms = total_ms % 1000;
    format!("{:02}:{:02}.{:03}", minute, second, ms)
}

pub fn format_time_ms_to_timestamp_string_short(time_ms: f32) -> String {
    let total_ms = time_ms.max(0.0) as i32;
    let minute = total_ms / 60000;
    let second = (total_ms % 60000) / 1000;
    format!("{}:{:02}", minute, second)
}

pub fn get_milliseconds_from_string(time: &str) -> Option<i32> {
    let time = time.trim();
    if time.is_empty() {
        return None;
    }

    let parts: Vec<&str> = time.split(':').collect();
    match parts.len() {
        2 => {
            let minutes: i32 = parts[0].parse().ok()?;
            let sec_parts: Vec<&str> = parts[1].split('.').collect();
            let seconds: i32 = sec_parts[0].parse().ok()?;
            let ms: i32 = if sec_parts.len() > 1 {
                let ms_str = sec_parts[1];
                let ms_val: i32 = ms_str.parse().ok()?;
                match ms_str.len() {
                    1 => ms_val * 100,
                    2 => ms_val * 10,
                    3 => ms_val,
                    _ => ms_val / 10i32.pow((ms_str.len() as u32).saturating_sub(3)),
                }
            } else {
                0
            };
            Some(minutes * 60000 + seconds * 1000 + ms)
        }
        3 => {
            let hours: i32 = parts[0].parse().ok()?;
            let minutes: i32 = parts[1].parse().ok()?;
            let sec_parts: Vec<&str> = parts[2].split('.').collect();
            let seconds: i32 = sec_parts[0].parse().ok()?;
            let ms: i32 = if sec_parts.len() > 1 {
                let ms_str = sec_parts[1];
                let ms_val: i32 = ms_str.parse().ok()?;
                match ms_str.len() {
                    1 => ms_val * 100,
                    2 => ms_val * 10,
                    3 => ms_val,
                    _ => ms_val / 10i32.pow((ms_str.len() as u32).saturating_sub(3)),
                }
            } else {
                0
            };
            Some(hours * 3600000 + minutes * 60000 + seconds * 1000 + ms)
        }
        _ => None,
    }
}

pub fn compute_text_same(text_x: &str, text_y: &str, is_case_sensitive: bool) -> f64 {
    let (text_x, text_y) = if is_case_sensitive {
        (text_x.to_string(), text_y.to_string())
    } else {
        (text_x.to_lowercase(), text_y.to_lowercase())
    };

    if text_x == text_y {
        return 100.0;
    }

    let len_x = text_x.chars().count();
    let len_y = text_y.chars().count();

    if len_x == 0 || len_y == 0 {
        return 0.0;
    }

    let lcs_len = lcs_length(&text_x, &text_y);
    let max_len = len_x.max(len_y) as f64;
    (lcs_len as f64 / max_len) * 100.0
}

pub fn lcs_length(x: &str, y: &str) -> usize {
    let x_chars: Vec<char> = x.chars().collect();
    let y_chars: Vec<char> = y.chars().collect();
    let m = x_chars.len();
    let n = y_chars.len();

    if m == 0 || n == 0 {
        return 0;
    }

    let mut prev = vec![0usize; n + 1];
    let mut curr = vec![0usize; n + 1];

    for i in 1..=m {
        for j in 1..=n {
            if x_chars[i - 1] == y_chars[j - 1] {
                curr[j] = prev[j - 1] + 1;
            } else {
                curr[j] = prev[j].max(curr[j - 1]);
            }
        }
        std::mem::swap(&mut prev, &mut curr);
        curr.fill(0);
    }
    prev[n]
}

pub fn remove_duo_spaces(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut last_was_space = false;
    for c in s.chars() {
        if c == ' ' {
            if !last_was_space {
                result.push(c);
            }
            last_was_space = true;
        } else {
            result.push(c);
            last_was_space = false;
        }
    }
    result
}

pub fn remove_triple_spaces(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut space_count = 0;
    for c in s.chars() {
        if c == ' ' {
            space_count += 1;
            if space_count <= 2 {
                result.push(c);
            }
        } else {
            result.push(c);
            space_count = 0;
        }
    }
    result
}

pub fn fix_comma_after_space(s: &str) -> String {
    remove_duo_spaces(&s.replace(",", ", "))
}

pub fn remove_duo_newlines(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut newline_count = 0;
    for c in s.chars() {
        if c == '\n' {
            newline_count += 1;
            if newline_count <= 1 {
                result.push(c);
            }
        } else {
            result.push(c);
            newline_count = 0;
        }
    }
    result
}

pub fn remove_cr(s: &str) -> String {
    s.replace('\r', "")
}

pub fn to_upper_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => {
            let upper: String = c.to_uppercase().collect();
            upper + chars.as_str()
        }
    }
}

pub fn fix_i_words(s: &str) -> String {
    s.replace(" i ", " I ")
        .replace(" i'", " I'")
        .replace("i'd ", "I'd ")
        .replace("i'm", "I'm")
        .replace("i'll", "I'll")
        .replace("i've", "I've")
}

static CJK_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[\u{4e00}-\u{9fff}\u{3400}-\u{4dbf}\u{f900}-\u{faff}\u{20000}-\u{2a6df}\u{2a700}-\u{2b73f}\u{2b740}-\u{2b81f}\u{2b820}-\u{2ceaf}\u{2ceb0}-\u{2ebef}\u{30000}-\u{3134f}]").unwrap()
});

static CHINESE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[\u{4e00}-\u{9fff}\u{3400}-\u{4dbf}\u{f900}-\u{faff}\u{20000}-\u{2a6df}\u{2a700}-\u{2b73f}\u{2b740}-\u{2b81f}\u{2b820}-\u{2ceaf}\u{2ceb0}-\u{2ebef}\u{30000}-\u{3134f}]").unwrap()
});

static EMOJI_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[\u{1F600}-\u{1F64F}\u{1F300}-\u{1F5FF}\u{1F680}-\u{1F6FF}\u{1F1E0}-\u{1F1FF}\u{2702}-\u{27B0}\u{24C2}-\u{1F251}\u{1F926}-\u{1F937}\u{10000}-\u{10FFFF}\u{2640}\u{2642}\u{2600}-\u{2B55}\u{200d}\u{fe0f}\u{20e3}\u{2934}\u{2935}]").unwrap()
});

pub fn has_cjk(s: &str) -> bool {
    CJK_REGEX.is_match(s)
}

pub fn is_cjk(s: &str) -> bool {
    !s.is_empty() && CJK_REGEX.find(s).is_some_and(|m| m.len() == s.len())
}

pub fn optimize_cjk(s: &str) -> String {
    // Remove CJK characters that are standalone
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        if !is_cjk_char(c) || has_non_cjk_around(s, c) {
            result.push(c);
        }
    }
    result
}

pub fn is_cjk_char(c: char) -> bool {
    matches!(c,
        '\u{4e00}'..='\u{9fff}' |
        '\u{3400}'..='\u{4dbf}' |
        '\u{f900}'..='\u{faff}' |
        '\u{20000}'..='\u{2a6df}' |
        '\u{2a700}'..='\u{2b73f}' |
        '\u{2b740}'..='\u{2b81f}' |
        '\u{2b820}'..='\u{2ceaf}' |
        '\u{2ceb0}'..='\u{2ebef}' |
        '\u{30000}'..='\u{3134f}'
    )
}

fn has_non_cjk_around(_s: &str, _c: char) -> bool {
    // Simplified version - always return true for now
    true
}

pub fn has_chinese(s: &str) -> bool {
    CHINESE_REGEX.is_match(s)
}

pub fn is_chinese(c: char) -> bool {
    matches!(c,
        '\u{4e00}'..='\u{9fff}' |
        '\u{3400}'..='\u{4dbf}' |
        '\u{f900}'..='\u{faff}' |
        '\u{20000}'..='\u{2a6df}' |
        '\u{2a700}'..='\u{2b73f}' |
        '\u{2b740}'..='\u{2b81f}' |
        '\u{2b820}'..='\u{2ceaf}' |
        '\u{2ceb0}'..='\u{2ebef}' |
        '\u{30000}'..='\u{3134f}'
    )
}

pub fn chinese_percentage(s: &str) -> f64 {
    if s.is_empty() {
        return 0.0;
    }
    let chinese_count = s.chars().filter(|&c| is_chinese(c)).count();
    chinese_count as f64 / s.chars().count() as f64
}

pub fn traditional_chinese_confidence(s: &str) -> f64 {
    // Simplified implementation
    chinese_percentage(s)
}

pub fn is_emoji(s: &str) -> bool {
    EMOJI_REGEX.is_match(s)
}

pub fn contains_emoji(s: &str) -> bool {
    EMOJI_REGEX.is_match(s)
}

pub fn between(s: &str, start: &str, end: &str) -> String {
    let start_idx = s.find(start).map(|i| i + start.len());
    let end_idx = s.find(end);
    match (start_idx, end_idx) {
        (Some(si), Some(ei)) if si < ei => s[si..ei].to_string(),
        _ => String::new(),
    }
}

pub fn reverse(s: &str) -> String {
    s.chars().rev().collect()
}

pub fn remove(s: &str, c: char) -> String {
    s.chars().filter(|&x| x != c).collect()
}

pub fn remove_control_chars(s: &str) -> String {
    s.chars().filter(|c| !c.is_control() || *c == '\n' || *c == '\r').collect()
}

pub fn remove_front_back_brackets(s: &str) -> String {
    let s = s.trim();
    if s.starts_with('(') && s.ends_with(')') {
        s[1..s.len() - 1].to_string()
    } else if s.starts_with('（') && s.ends_with('）') {
        // Handle multi-byte Chinese brackets
        let chars: Vec<char> = s.chars().collect();
        if chars.len() >= 2 {
            chars[1..chars.len() - 1].iter().collect()
        } else {
            s.to_string()
        }
    } else {
        s.to_string()
    }
}

pub fn can_start_new_line(s: &str) -> bool {
    s.ends_with(' ') || s.ends_with(',') || s.ends_with('/')
}

pub fn contains_any(s: &str, list: &[&str]) -> bool {
    list.iter().any(|item| s.contains(item))
}

pub fn is_number(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_digit())
}

pub fn is_same(a: &str, b: &str) -> bool {
    a == b
}

pub fn is_same_whitespace(a: &str, b: &str) -> bool {
    remove_duo_spaces(a) == remove_duo_spaces(b)
}

pub fn is_same_trim(a: &str, b: &str) -> bool {
    a.trim() == b.trim()
}

pub fn trim(s: &str) -> &str {
    s.trim()
}

pub fn pad_left(s: &str, total_width: usize, padding_char: char) -> String {
    if s.len() >= total_width {
        s.to_string()
    } else {
        let padding: String = std::iter::repeat_n(padding_char, total_width - s.len()).collect();
        format!("{}{}", padding, s)
    }
}

pub fn pad_right(s: &str, total_width: usize, padding_char: char) -> String {
    if s.len() >= total_width {
        s.to_string()
    } else {
        let padding: String = std::iter::repeat_n(padding_char, total_width - s.len()).collect();
        format!("{}{}", s, padding)
    }
}
