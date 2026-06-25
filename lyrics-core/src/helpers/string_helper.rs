pub fn is_number(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_digit())
}

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
