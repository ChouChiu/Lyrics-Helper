use std::cell::{Cell, RefCell};

use serde::{Deserialize, Serialize};

/// 单个音节信息，包含文本内容和时间范围。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyllableInfo {
    /// 音节文本内容
    pub text: String,
    /// 开始时间（毫秒）
    pub start_time: i32,
    /// 结束时间（毫秒）
    pub end_time: i32,
}

impl SyllableInfo {
    /// 创建新的音节信息。
    pub fn new(text: String, start_time: i32, end_time: i32) -> Self {
        Self { text, start_time, end_time }
    }

    /// 返回音节持续时长（毫秒）。
    pub fn duration(&self) -> i32 {
        self.end_time - self.start_time
    }
}

/// 完整音节信息，由多个 [`SyllableInfo`] 子项组成，并缓存聚合属性。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullSyllableInfo {
    /// 子音节列表
    pub sub_items: Vec<SyllableInfo>,
    #[serde(skip)]
    cached_text: RefCell<Option<String>>,
    #[serde(skip)]
    cached_start_time: Cell<Option<i32>>,
    #[serde(skip)]
    cached_end_time: Cell<Option<i32>>,
}

impl FullSyllableInfo {
    /// 创建新的完整音节信息，初始缓存为空。
    pub fn new(sub_items: Vec<SyllableInfo>) -> Self {
        Self {
            sub_items,
            cached_text: RefCell::new(None),
            cached_start_time: Cell::new(None),
            cached_end_time: Cell::new(None),
        }
    }

    /// 返回所有子音节拼接后的完整文本（带缓存）。
    pub fn text(&self) -> String {
        if let Some(ref t) = *self.cached_text.borrow() {
            return t.clone();
        }
        let t: String = self.sub_items.iter().map(|s| s.text.as_str()).collect();
        *self.cached_text.borrow_mut() = Some(t.clone());
        t
    }

    /// 返回第一个子音节的开始时间（带缓存），无子音节时返回 0。
    pub fn start_time(&self) -> i32 {
        if let Some(t) = self.cached_start_time.get() {
            return t;
        }
        let t = self.sub_items.first().map(|s| s.start_time).unwrap_or(0);
        self.cached_start_time.set(Some(t));
        t
    }

    /// 返回最后一个子音节的结束时间（带缓存），无子音节时返回 0。
    pub fn end_time(&self) -> i32 {
        if let Some(t) = self.cached_end_time.get() {
            return t;
        }
        let t = self.sub_items.last().map(|s| s.end_time).unwrap_or(0);
        self.cached_end_time.set(Some(t));
        t
    }

    /// 返回总持续时长（毫秒）。
    pub fn duration(&self) -> i32 {
        self.end_time() - self.start_time()
    }

    /// 清除所有缓存属性，使其在下次访问时重新计算。
    pub fn refresh_properties(&self) {
        *self.cached_text.borrow_mut() = None;
        self.cached_start_time.set(None);
        self.cached_end_time.set(None);
    }
}

/// 将音节列表拼接为完整文本字符串。
pub fn get_text_from_syllable_list(syllables: &[SyllableInfo]) -> String {
    syllables.iter().map(|s| s.text.as_str()).collect()
}
