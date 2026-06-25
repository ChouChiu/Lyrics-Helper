use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::lyrics_types::LyricsAlignment;
use super::syllable_info::SyllableInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LineInfo {
    Line {
        text: String,
        start_time: Option<i32>,
        end_time: Option<i32>,
        alignment: LyricsAlignment,
        sub_line: Option<Box<LineInfo>>,
    },
    Syllable {
        syllables: Vec<SyllableInfo>,
        alignment: LyricsAlignment,
        sub_line: Option<Box<LineInfo>>,
    },
    FullLine {
        text: String,
        start_time: Option<i32>,
        end_time: Option<i32>,
        alignment: LyricsAlignment,
        sub_line: Option<Box<LineInfo>>,
        translations: HashMap<String, String>,
        pronunciation: Option<String>,
    },
    FullSyllable {
        syllables: Vec<SyllableInfo>,
        alignment: LyricsAlignment,
        sub_line: Option<Box<LineInfo>>,
        translations: HashMap<String, String>,
        pronunciation: Option<String>,
    },
}

impl LineInfo {
    pub fn new_line(text: String, start_time: Option<i32>, end_time: Option<i32>) -> Self {
        Self::Line {
            text,
            start_time,
            end_time,
            alignment: LyricsAlignment::Unspecified,
            sub_line: None,
        }
    }

    pub fn new_line_simple(text: String) -> Self {
        Self::Line {
            text,
            start_time: None,
            end_time: None,
            alignment: LyricsAlignment::Unspecified,
            sub_line: None,
        }
    }

    pub fn new_line_with_time(text: String, start_time: i32) -> Self {
        Self::Line {
            text,
            start_time: Some(start_time),
            end_time: None,
            alignment: LyricsAlignment::Unspecified,
            sub_line: None,
        }
    }

    pub fn new_syllable(syllables: Vec<SyllableInfo>) -> Self {
        Self::Syllable {
            syllables,
            alignment: LyricsAlignment::Unspecified,
            sub_line: None,
        }
    }

    pub fn new_full_line(
        text: String,
        start_time: Option<i32>,
        end_time: Option<i32>,
        translations: HashMap<String, String>,
        pronunciation: Option<String>,
    ) -> Self {
        Self::FullLine {
            text,
            start_time,
            end_time,
            alignment: LyricsAlignment::Unspecified,
            sub_line: None,
            translations,
            pronunciation,
        }
    }

    pub fn new_full_syllable(
        syllables: Vec<SyllableInfo>,
        translations: HashMap<String, String>,
        pronunciation: Option<String>,
    ) -> Self {
        Self::FullSyllable {
            syllables,
            alignment: LyricsAlignment::Unspecified,
            sub_line: None,
            translations,
            pronunciation,
        }
    }

    pub fn text(&self) -> &str {
        match self {
            Self::Line { text, .. } | Self::FullLine { text, .. } => text,
            Self::Syllable { .. } | Self::FullSyllable { .. } => "",
        }
    }

    pub fn text_from_syllables(syllables: &[SyllableInfo]) -> String {
        syllables.iter().map(|s| s.text.as_str()).collect()
    }

    pub fn start_time(&self) -> Option<i32> {
        match self {
            Self::Line { start_time, .. } | Self::FullLine { start_time, .. } => *start_time,
            Self::Syllable { syllables, .. } | Self::FullSyllable { syllables, .. } => {
                syllables.first().map(|s| s.start_time)
            }
        }
    }

    pub fn end_time(&self) -> Option<i32> {
        match self {
            Self::Line { end_time, .. } | Self::FullLine { end_time, .. } => *end_time,
            Self::Syllable { syllables, .. } | Self::FullSyllable { syllables, .. } => {
                syllables.last().map(|s| s.end_time)
            }
        }
    }

    pub fn duration(&self) -> Option<i32> {
        match (self.start_time(), self.end_time()) {
            (Some(s), Some(e)) => Some(e - s),
            _ => None,
        }
    }

    pub fn alignment(&self) -> LyricsAlignment {
        match self {
            Self::Line { alignment, .. }
            | Self::Syllable { alignment, .. }
            | Self::FullLine { alignment, .. }
            | Self::FullSyllable { alignment, .. } => *alignment,
        }
    }

    pub fn set_alignment(&mut self, new_alignment: LyricsAlignment) {
        match self {
            Self::Line { alignment, .. }
            | Self::Syllable { alignment, .. }
            | Self::FullLine { alignment, .. }
            | Self::FullSyllable { alignment, .. } => *alignment = new_alignment,
        }
    }

    pub fn sub_line(&self) -> Option<&LineInfo> {
        match self {
            Self::Line { sub_line, .. }
            | Self::Syllable { sub_line, .. }
            | Self::FullLine { sub_line, .. }
            | Self::FullSyllable { sub_line, .. } => sub_line.as_deref(),
        }
    }

    pub fn set_sub_line(&mut self, new_sub_line: Option<Box<LineInfo>>) {
        match self {
            Self::Line { sub_line, .. }
            | Self::Syllable { sub_line, .. }
            | Self::FullLine { sub_line, .. }
            | Self::FullSyllable { sub_line, .. } => *sub_line = new_sub_line,
        }
    }

    pub fn is_syllable(&self) -> bool {
        matches!(self, Self::Syllable { .. } | Self::FullSyllable { .. })
    }

    pub fn is_full(&self) -> bool {
        matches!(self, Self::FullLine { .. } | Self::FullSyllable { .. })
    }

    pub fn start_time_with_sub_line(&self) -> Option<i32> {
        let main = self.start_time();
        let sub = self.sub_line().and_then(|s| s.start_time());
        crate::helpers::math_helper::min_opt(main, sub)
    }

    pub fn end_time_with_sub_line(&self) -> Option<i32> {
        let main = self.end_time();
        let sub = self.sub_line().and_then(|s| s.end_time());
        crate::helpers::math_helper::max_opt(main, sub)
    }

    pub fn duration_with_sub_line(&self) -> Option<i32> {
        match (self.start_time_with_sub_line(), self.end_time_with_sub_line()) {
            (Some(s), Some(e)) => Some(e - s),
            _ => None,
        }
    }

    pub fn full_text(&self) -> String {
        match self {
            Self::Line { text, sub_line, .. } | Self::FullLine { text, sub_line, .. } => {
                if let Some(sub) = sub_line {
                    let sub_text = crate::helpers::string_helper::remove_front_back_brackets(&sub.text_from_any());
                    if sub.start_time() < self.start_time() {
                        format!("({}) {}", sub_text, text.trim())
                    } else {
                        format!("{} ({})", text.trim(), sub_text)
                    }
                } else {
                    text.clone()
                }
            }
            Self::Syllable { syllables, sub_line, .. }
            | Self::FullSyllable { syllables, sub_line, .. } => {
                let text = Self::text_from_syllables(syllables);
                if let Some(sub) = sub_line {
                    let sub_text = crate::helpers::string_helper::remove_front_back_brackets(&sub.text_from_any());
                    if sub.start_time() < self.start_time() {
                        format!("({}) {}", sub_text, text.trim())
                    } else {
                        format!("{} ({})", text.trim(), sub_text)
                    }
                } else {
                    text
                }
            }
        }
    }

    pub fn text_from_any(&self) -> String {
        match self {
            Self::Line { text, .. } | Self::FullLine { text, .. } => text.clone(),
            Self::Syllable { syllables, .. } | Self::FullSyllable { syllables, .. } => {
                Self::text_from_syllables(syllables)
            }
        }
    }

    pub fn translations(&self) -> Option<&HashMap<String, String>> {
        match self {
            Self::FullLine { translations, .. } | Self::FullSyllable { translations, .. } => {
                Some(translations)
            }
            _ => None,
        }
    }

    pub fn translations_mut(&mut self) -> Option<&mut HashMap<String, String>> {
        match self {
            Self::FullLine { translations, .. } | Self::FullSyllable { translations, .. } => {
                Some(translations)
            }
            _ => None,
        }
    }

    pub fn chinese_translation(&self) -> Option<&str> {
        self.translations()?.get("zh").map(|s| s.as_str())
    }

    pub fn set_chinese_translation(&mut self, value: Option<String>) {
        if let Some(translations) = self.translations_mut() {
            match value {
                Some(v) if !v.is_empty() => {
                    translations.insert("zh".to_string(), v);
                }
                _ => {
                    translations.remove("zh");
                }
            }
        }
    }

    pub fn to_full_line(self, translations: HashMap<String, String>, pronunciation: Option<String>) -> Self {
        match self {
            Self::Line { text, start_time, end_time, alignment, sub_line } => {
                Self::FullLine { text, start_time, end_time, alignment, sub_line, translations, pronunciation }
            }
            Self::Syllable { syllables, alignment, sub_line } => {
                Self::FullSyllable { syllables, alignment, sub_line, translations, pronunciation }
            }
            other => other,
        }
    }

    pub fn to_full_syllable(self, translations: HashMap<String, String>, pronunciation: Option<String>) -> Self {
        match self {
            Self::Syllable { syllables, alignment, sub_line } => {
                Self::FullSyllable { syllables, alignment, sub_line, translations, pronunciation }
            }
            other => other,
        }
    }

    pub fn start_time_or_zero(&self) -> i32 {
        self.start_time().unwrap_or(0)
    }
}

impl PartialEq for LineInfo {
    fn eq(&self, other: &Self) -> bool {
        self.start_time() == other.start_time()
    }
}

impl Eq for LineInfo {}

impl PartialOrd for LineInfo {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LineInfo {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self.start_time(), other.start_time()) {
            (Some(a), Some(b)) => a.cmp(&b),
            _ => std::cmp::Ordering::Equal,
        }
    }
}
