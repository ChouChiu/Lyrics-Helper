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
        Self {
            text,
            start_time,
            end_time,
        }
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

    /// 返回子音节列表的只读切片。
    pub fn sub_items(&self) -> &[SyllableInfo] {
        &self.sub_items
    }

    /// 消耗自身并返回子音节列表。
    pub fn into_sub_items(self) -> Vec<SyllableInfo> {
        self.sub_items
    }

    /// 返回子音节列表的可变引用。
    ///
    /// 修改后必须调用 [`FullSyllableInfo::refresh_properties`]，否则缓存的聚合属性会失效。
    pub fn sub_items_mut(&mut self) -> &mut Vec<SyllableInfo> {
        &mut self.sub_items
    }

    /// 替换子音节列表并刷新缓存。
    pub fn set_sub_items(&mut self, sub_items: Vec<SyllableInfo>) {
        self.sub_items = sub_items;
        self.refresh_properties();
    }

    /// 追加子音节并刷新缓存。
    pub fn extend_sub_items(&mut self, items: impl IntoIterator<Item = SyllableInfo>) {
        self.sub_items.extend(items);
        self.refresh_properties();
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

/// 音节项，对应 C# 的 `ISyllableInfo`。
///
/// 普通音节为 [`SyllableInfo`]，同一单词内合并后的音节为 [`FullSyllableInfo`]
/// （保留各自的子音节时间信息，聚合文本与时间由子项推导）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyllableItem {
    /// 普通音节
    Syllable(SyllableInfo),
    /// 合并后的完整音节
    Full(FullSyllableInfo),
}

impl SyllableItem {
    /// 返回音节文本。
    pub fn text(&self) -> String {
        match self {
            Self::Syllable(s) => s.text.clone(),
            Self::Full(f) => f.text(),
        }
    }

    /// 返回开始时间（毫秒）。
    pub fn start_time(&self) -> i32 {
        match self {
            Self::Syllable(s) => s.start_time,
            Self::Full(f) => f.start_time(),
        }
    }

    /// 返回结束时间（毫秒）。
    pub fn end_time(&self) -> i32 {
        match self {
            Self::Syllable(s) => s.end_time,
            Self::Full(f) => f.end_time(),
        }
    }

    /// 返回持续时长（毫秒）。
    pub fn duration(&self) -> i32 {
        self.end_time() - self.start_time()
    }

    /// 是否为合并后的完整音节。
    pub fn is_full(&self) -> bool {
        matches!(self, Self::Full(_))
    }

    /// 返回生成时使用的普通音节序列：普通音节返回自身，合并音节返回全部子音节。
    pub fn parts(&self) -> &[SyllableInfo] {
        match self {
            Self::Syllable(syllable) => std::slice::from_ref(syllable),
            Self::Full(full) => full.sub_items(),
        }
    }

    /// 返回普通音节的只读引用，合并音节返回 `None`。
    pub fn as_syllable(&self) -> Option<&SyllableInfo> {
        match self {
            Self::Syllable(s) => Some(s),
            Self::Full(_) => None,
        }
    }

    /// 返回普通音节的可变引用，合并音节返回 `None`。
    pub fn as_syllable_mut(&mut self) -> Option<&mut SyllableInfo> {
        match self {
            Self::Syllable(s) => Some(s),
            Self::Full(_) => None,
        }
    }

    /// 返回合并音节的只读引用，普通音节返回 `None`。
    pub fn as_full(&self) -> Option<&FullSyllableInfo> {
        match self {
            Self::Syllable(_) => None,
            Self::Full(f) => Some(f),
        }
    }

    /// 返回合并音节的可变引用，普通音节返回 `None`。
    pub fn as_full_mut(&mut self) -> Option<&mut FullSyllableInfo> {
        match self {
            Self::Syllable(_) => None,
            Self::Full(f) => Some(f),
        }
    }
}

impl From<SyllableInfo> for SyllableItem {
    fn from(value: SyllableInfo) -> Self {
        Self::Syllable(value)
    }
}

impl From<FullSyllableInfo> for SyllableItem {
    fn from(value: FullSyllableInfo) -> Self {
        Self::Full(value)
    }
}

/// 将音节项列表拼接为完整文本字符串。
pub fn get_text_from_syllable_items(syllables: &[SyllableItem]) -> String {
    let mut text = String::new();
    for item in syllables {
        for part in item.parts() {
            text.push_str(&part.text);
        }
    }
    text
}

/// 将普通音节列表包装为音节项列表。
pub fn to_syllable_items(syllables: impl IntoIterator<Item = SyllableInfo>) -> Vec<SyllableItem> {
    syllables.into_iter().map(SyllableItem::from).collect()
}

/// 将音节项列表展开为普通音节列表（合并音节展开为其子音节）。
pub fn flatten_syllable_items(syllables: &[SyllableItem]) -> Vec<SyllableInfo> {
    let mut items = Vec::with_capacity(syllables.len());
    for syllable in syllables {
        items.extend_from_slice(syllable.parts());
    }
    items
}

/// 为音节项列表中的每个音节添加时间偏移量。
pub fn add_offset_to_syllable_items(syllables: &mut [SyllableItem], offset: i32) {
    for syllable in syllables.iter_mut() {
        match syllable {
            SyllableItem::Syllable(s) => {
                s.start_time -= offset;
                s.end_time -= offset;
            }
            SyllableItem::Full(f) => {
                for sub in f.sub_items_mut().iter_mut() {
                    sub.start_time -= offset;
                    sub.end_time -= offset;
                }
                f.refresh_properties();
            }
        }
    }
}
