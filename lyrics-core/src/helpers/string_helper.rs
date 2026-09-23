use std::borrow::Cow;

/// 将毫秒时间值格式化为 `mm:ss.SSS` 格式的时间戳字符串。
///
/// 负值会被当作 0 处理。
pub fn format_time_ms_to_timestamp_string(time_ms: f32) -> String {
    let total_ms = time_ms.max(0.0) as i32;
    let minute = total_ms / 60000;
    let second = (total_ms % 60000) / 1000;
    let ms = total_ms % 1000;
    format!("{:02}:{:02}.{:03}", minute, second, ms)
}

/// 判断字符串是否全部由 ASCII 数字组成（非空）。
pub fn is_number(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_digit())
}

/// 移除字符串首尾的半角括号 `()` 或全角括号 `（）`。
pub fn remove_front_back_brackets(s: &str) -> String {
    let s = s.trim();
    for (open, close) in [('(', ')'), ('（', '）')] {
        if let Some(inner) = s.strip_prefix(open).and_then(|s| s.strip_suffix(close)) {
            return inner.to_string();
        }
    }
    s.to_string()
}

/// 计算两段文本的相似度（百分比），基于最长公共子序列（LCS）算法。
///
/// `is_case_sensitive` 为 `false` 时忽略大小写。返回值范围 0.0~100.0。
pub fn compute_text_same(text_x: &str, text_y: &str, is_case_sensitive: bool) -> f64 {
    let (text_x, text_y): (Cow<str>, Cow<str>) = if is_case_sensitive {
        (text_x.into(), text_y.into())
    } else {
        (text_x.to_lowercase().into(), text_y.to_lowercase().into())
    };

    // 两段都为空时也在这里返回，下面的除数因此不会为 0。
    if text_x == text_y {
        return 100.0;
    }

    let max_len = text_x.chars().count().max(text_y.chars().count()) as f64;
    (lcs_length(&text_x, &text_y) as f64 / max_len) * 100.0
}

/// 计算两个字符串的最长公共子序列（LCS）长度。
pub fn lcs_length(x: &str, y: &str) -> usize {
    let x_chars: Vec<char> = x.chars().collect();
    let y_chars: Vec<char> = y.chars().collect();
    let m = x_chars.len();
    let n = y_chars.len();

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
        // `curr` 的 1..=n 会在下一轮被全部覆盖，`curr[0]` 恒为 0，无需清零。
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[n]
}

/// 判断字符是否为中日文字符（CJK 统一表意文字、扩展 A、平假名、片假名及其扩展）。
pub fn is_chinese_or_japanese_character(character: char) -> bool {
    matches!(character,
        '\u{4E00}'..='\u{9FFF}'
        | '\u{3400}'..='\u{4DBF}'
        | '\u{3040}'..='\u{309F}'
        | '\u{30A0}'..='\u{30FF}'
        | '\u{31F0}'..='\u{31FF}')
}

/// 判断文本是否包含中文（`\u{4e00}-\u{9fff}`），对应 C# `StringHelper.HasChinese`。
///
/// 只覆盖 CJK 统一表意文字区，不含假名，与 [`is_chinese_or_japanese_character`] 的范围不同。
pub fn has_chinese(s: &str) -> bool {
    s.chars().any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c))
}

/// 将字符串中所有连续空白折叠为单个空格，并去除首尾空白。
pub fn collapse_whitespace(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for part in s.split_whitespace() {
        if !result.is_empty() {
            result.push(' ');
        }
        result.push_str(part);
    }
    result
}

/// 将字符串中连续的空格压缩为单个空格。
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
