/// 返回两个 `Option<i32>` 中的较小值。
///
/// 任一为 `Some` 时返回有效值；两者均为 `None` 时返回 `None`。
pub fn min_opt(a: Option<i32>, b: Option<i32>) -> Option<i32> {
    match (a, b) {
        (Some(x), Some(y)) => Some(x.min(y)),
        _ => a.or(b),
    }
}

/// 返回两个 `Option<i32>` 中的较大值。
///
/// 任一为 `Some` 时返回有效值；两者均为 `None` 时返回 `None`。
pub fn max_opt(a: Option<i32>, b: Option<i32>) -> Option<i32> {
    match (a, b) {
        (Some(x), Some(y)) => Some(x.max(y)),
        _ => a.or(b),
    }
}
