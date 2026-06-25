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
