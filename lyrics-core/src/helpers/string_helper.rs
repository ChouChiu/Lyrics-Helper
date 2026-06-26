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
    if s.starts_with('(') && s.ends_with(')') {
        s[1..s.len() - 1].to_string()
    } else if s.starts_with('（') && s.ends_with('）') {
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

/// 计算两段文本的相似度（百分比），基于最长公共子序列（LCS）算法。
///
/// `is_case_sensitive` 为 `false` 时忽略大小写。返回值范围 0.0~100.0。
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

/// 计算两个字符串的最长公共子序列（LCS）长度。
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
