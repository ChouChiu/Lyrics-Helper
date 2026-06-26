use std::cell::{Cell, RefCell};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyllableInfo {
    pub text: String,
    pub start_time: i32,
    pub end_time: i32,
}

impl SyllableInfo {
    pub fn new(text: String, start_time: i32, end_time: i32) -> Self {
        Self { text, start_time, end_time }
    }

    pub fn duration(&self) -> i32 {
        self.end_time - self.start_time
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullSyllableInfo {
    pub sub_items: Vec<SyllableInfo>,
    #[serde(skip)]
    cached_text: RefCell<Option<String>>,
    #[serde(skip)]
    cached_start_time: Cell<Option<i32>>,
    #[serde(skip)]
    cached_end_time: Cell<Option<i32>>,
}

impl FullSyllableInfo {
    pub fn new(sub_items: Vec<SyllableInfo>) -> Self {
        Self {
            sub_items,
            cached_text: RefCell::new(None),
            cached_start_time: Cell::new(None),
            cached_end_time: Cell::new(None),
        }
    }

    pub fn text(&self) -> String {
        if let Some(ref t) = *self.cached_text.borrow() {
            return t.clone();
        }
        let t: String = self.sub_items.iter().map(|s| s.text.as_str()).collect();
        *self.cached_text.borrow_mut() = Some(t.clone());
        t
    }

    pub fn start_time(&self) -> i32 {
        if let Some(t) = self.cached_start_time.get() {
            return t;
        }
        let t = self.sub_items.first().map(|s| s.start_time).unwrap_or(0);
        self.cached_start_time.set(Some(t));
        t
    }

    pub fn end_time(&self) -> i32 {
        if let Some(t) = self.cached_end_time.get() {
            return t;
        }
        let t = self.sub_items.last().map(|s| s.end_time).unwrap_or(0);
        self.cached_end_time.set(Some(t));
        t
    }

    pub fn duration(&self) -> i32 {
        self.end_time() - self.start_time()
    }

    pub fn refresh_properties(&self) {
        *self.cached_text.borrow_mut() = None;
        self.cached_start_time.set(None);
        self.cached_end_time.set(None);
    }
}

pub fn get_text_from_syllable_list(syllables: &[SyllableInfo]) -> String {
    syllables.iter().map(|s| s.text.as_str()).collect()
}
