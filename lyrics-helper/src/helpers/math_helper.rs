pub fn min_opt(a: Option<i32>, b: Option<i32>) -> Option<i32> {
    match (a, b) {
        (Some(x), Some(y)) => Some(x.min(y)),
        (Some(x), None) => Some(x),
        (None, Some(y)) => Some(y),
        (None, None) => None,
    }
}

pub fn max_opt(a: Option<i32>, b: Option<i32>) -> Option<i32> {
    match (a, b) {
        (Some(x), Some(y)) => Some(x.max(y)),
        (Some(x), None) => Some(x),
        (None, Some(y)) => Some(y),
        (None, None) => None,
    }
}

pub fn greater_than_zero_i32(x: i32) -> i32 {
    x.max(0)
}

pub fn greater_than_zero_f64(x: f64) -> f64 {
    x.max(0.0)
}

pub fn greater_than_zero_f32(x: f32) -> f32 {
    x.max(0.0)
}

pub fn greater_than_i32(x: i32, min: i32) -> i32 {
    x.max(min)
}

pub fn greater_than_f64(x: f64, min: f64) -> f64 {
    x.max(min)
}

pub fn greater_than_f32(x: f32, min: f32) -> f32 {
    x.max(min)
}

pub fn is_between_i32(x: i32, a: i32, b: i32, contain_edge: bool) -> bool {
    if contain_edge {
        x >= a && x <= b
    } else {
        x > a && x < b
    }
}

pub fn is_between_f64(x: f64, a: f64, b: f64, contain_edge: bool) -> bool {
    if contain_edge {
        x >= a && x <= b
    } else {
        x > a && x < b
    }
}

pub fn is_between_f32(x: f32, a: f32, b: f32, contain_edge: bool) -> bool {
    if contain_edge {
        x >= a && x <= b
    } else {
        x > a && x < b
    }
}

pub fn swap_i32(a: i32, b: i32) -> (i32, i32) {
    (b, a)
}

pub fn swap_f64(a: f64, b: f64) -> (f64, f64) {
    (b, a)
}

pub fn swap_f32(a: f32, b: f32) -> (f32, f32) {
    (b, a)
}
