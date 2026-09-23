//! UTF-16 码元级字符操作，用于保持与上游 C# 实现（`System.String` 为 UTF-16）一致的行为。

/// 对应 C# `char.IsLetter`：按 UTF-16 码元判断（代理项返回 `false`）。
pub(super) fn is_letter_unit(unit: u16) -> bool {
    char::from_u32(unit as u32).is_some_and(|c| c.is_alphabetic())
}

/// 对应 C# `char.IsUpper`：按 UTF-16 码元判断（代理项返回 `false`）。
pub(super) fn is_upper_unit(unit: u16) -> bool {
    char::from_u32(unit as u32).is_some_and(|c| c.is_uppercase())
}

/// 对应 C# `char.IsLower`：按 UTF-16 码元判断（代理项返回 `false`）。
pub(super) fn is_lower_unit(unit: u16) -> bool {
    char::from_u32(unit as u32).is_some_and(|c| c.is_lowercase())
}

/// 对应 C# `char.ToUpperInvariant(char)`：单码元的简单大写映射。
///
/// Rust 的 `char::to_uppercase` 是完整映射（如 `ß` -> `SS`），与 .NET 的单字符映射
/// 不同；这里只在映射结果为单个字符时采用，否则保持原码元，与 .NET 的简单映射一致。
pub(super) fn to_upper_invariant_unit(unit: u16) -> u16 {
    let Some(ch) = char::from_u32(unit as u32) else {
        return unit;
    };

    let mut upper = ch.to_uppercase();
    match (upper.next(), upper.next()) {
        (Some(c), None) if c.len_utf16() == 1 => c as u16,
        _ => unit,
    }
}

/// 按 UTF-16 码元切片并转回字符串，对应 C# 的 `String.Substring`。
///
/// 极端情况下上游会切出落单代理项（Rust 字符串无法表示），这里以 U+FFFD 兜底。
pub(super) fn utf16_slice(units: &[u16], start: usize, len: usize) -> String {
    String::from_utf16_lossy(&units[start..start + len])
}
