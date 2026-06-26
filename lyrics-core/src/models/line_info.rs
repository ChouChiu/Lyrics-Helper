use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::lyrics_types::LyricsAlignment;
use super::syllable_info::SyllableInfo;

/// 歌词行信息，支持四种变体：简单行、音节行、完整行、完整音节行。
///
/// 所有变体均支持可选的子行（`sub_line`）和对齐方式（`alignment`）。
/// Full 系列变体额外支持翻译和拼音。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LineInfo {
    /// 简单歌词行，包含纯文本和可选的时间戳。
    Line {
        /// 行文本内容
        text: String,
        /// 开始时间（毫秒）
        start_time: Option<i32>,
        /// 结束时间（毫秒）
        end_time: Option<i32>,
        /// 文本对齐方式
        alignment: LyricsAlignment,
        /// 子行（如背景和声）
        sub_line: Option<Box<LineInfo>>,
    },
    /// 音节歌词行，由多个音节组成，无直接文本。
    Syllable {
        /// 音节列表
        syllables: Vec<SyllableInfo>,
        /// 文本对齐方式
        alignment: LyricsAlignment,
        /// 子行（如背景和声）
        sub_line: Option<Box<LineInfo>>,
    },
    /// 完整歌词行，在简单行基础上增加翻译和拼音信息。
    FullLine {
        /// 行文本内容
        text: String,
        /// 开始时间（毫秒）
        start_time: Option<i32>,
        /// 结束时间（毫秒）
        end_time: Option<i32>,
        /// 文本对齐方式
        alignment: LyricsAlignment,
        /// 子行（如背景和声）
        sub_line: Option<Box<LineInfo>>,
        /// 翻译映射（键为语言代码，如 `"zh"`）
        translations: HashMap<String, String>,
        /// 拼音/注音
        pronunciation: Option<String>,
    },
    /// 完整音节歌词行，在音节行基础上增加翻译和拼音信息。
    FullSyllable {
        /// 音节列表
        syllables: Vec<SyllableInfo>,
        /// 文本对齐方式
        alignment: LyricsAlignment,
        /// 子行（如背景和声）
        sub_line: Option<Box<LineInfo>>,
        /// 翻译映射（键为语言代码，如 `"zh"`）
        translations: HashMap<String, String>,
        /// 拼音/注音
        pronunciation: Option<String>,
    },
}

impl LineInfo {
    /// 创建带可选时间戳的简单歌词行。
    pub fn new_line(text: String, start_time: Option<i32>, end_time: Option<i32>) -> Self {
        Self::Line {
            text,
            start_time,
            end_time,
            alignment: LyricsAlignment::Unspecified,
            sub_line: None,
        }
    }

    /// 创建无时间信息的简单歌词行。
    pub fn new_line_simple(text: String) -> Self {
        Self::Line {
            text,
            start_time: None,
            end_time: None,
            alignment: LyricsAlignment::Unspecified,
            sub_line: None,
        }
    }

    /// 创建仅有开始时间的简单歌词行。
    pub fn new_line_with_time(text: String, start_time: i32) -> Self {
        Self::Line {
            text,
            start_time: Some(start_time),
            end_time: None,
            alignment: LyricsAlignment::Unspecified,
            sub_line: None,
        }
    }

    /// 创建音节歌词行。
    pub fn new_syllable(syllables: Vec<SyllableInfo>) -> Self {
        Self::Syllable {
            syllables,
            alignment: LyricsAlignment::Unspecified,
            sub_line: None,
        }
    }

    /// 创建带翻译和拼音的完整歌词行。
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

    /// 创建带翻译和拼音的完整音节歌词行。
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

    /// 返回行文本内容（音节行返回空字符串）。
    pub fn text(&self) -> &str {
        match self {
            Self::Line { text, .. } | Self::FullLine { text, .. } => text,
            Self::Syllable { .. } | Self::FullSyllable { .. } => "",
        }
    }

    /// 将音节列表拼接为完整文本字符串。
    pub fn text_from_syllables(syllables: &[SyllableInfo]) -> String {
        syllables.iter().map(|s| s.text.as_str()).collect()
    }

    /// 返回开始时间（毫秒）。音节行取第一个音节的开始时间。
    pub fn start_time(&self) -> Option<i32> {
        match self {
            Self::Line { start_time, .. } | Self::FullLine { start_time, .. } => *start_time,
            Self::Syllable { syllables, .. } | Self::FullSyllable { syllables, .. } => {
                syllables.first().map(|s| s.start_time)
            }
        }
    }

    /// 返回结束时间（毫秒）。音节行取最后一个音节的结束时间。
    pub fn end_time(&self) -> Option<i32> {
        match self {
            Self::Line { end_time, .. } | Self::FullLine { end_time, .. } => *end_time,
            Self::Syllable { syllables, .. } | Self::FullSyllable { syllables, .. } => {
                syllables.last().map(|s| s.end_time)
            }
        }
    }

    /// 返回持续时长（毫秒），缺少开始或结束时间时返回 `None`。
    pub fn duration(&self) -> Option<i32> {
        match (self.start_time(), self.end_time()) {
            (Some(s), Some(e)) => Some(e - s),
            _ => None,
        }
    }

    /// 返回文本对齐方式。
    pub fn alignment(&self) -> LyricsAlignment {
        match self {
            Self::Line { alignment, .. }
            | Self::Syllable { alignment, .. }
            | Self::FullLine { alignment, .. }
            | Self::FullSyllable { alignment, .. } => *alignment,
        }
    }

    /// 设置文本对齐方式。
    pub fn set_alignment(&mut self, new_alignment: LyricsAlignment) {
        match self {
            Self::Line { alignment, .. }
            | Self::Syllable { alignment, .. }
            | Self::FullLine { alignment, .. }
            | Self::FullSyllable { alignment, .. } => *alignment = new_alignment,
        }
    }

    /// 返回子行的只读引用（如背景和声）。
    pub fn sub_line(&self) -> Option<&LineInfo> {
        match self {
            Self::Line { sub_line, .. }
            | Self::Syllable { sub_line, .. }
            | Self::FullLine { sub_line, .. }
            | Self::FullSyllable { sub_line, .. } => sub_line.as_deref(),
        }
    }

    /// 设置子行。
    pub fn set_sub_line(&mut self, new_sub_line: Option<Box<LineInfo>>) {
        match self {
            Self::Line { sub_line, .. }
            | Self::Syllable { sub_line, .. }
            | Self::FullLine { sub_line, .. }
            | Self::FullSyllable { sub_line, .. } => *sub_line = new_sub_line,
        }
    }

    /// 判断是否为音节行（`Syllable` 或 `FullSyllable`）。
    pub fn is_syllable(&self) -> bool {
        matches!(self, Self::Syllable { .. } | Self::FullSyllable { .. })
    }

    /// 判断是否为完整行（`FullLine` 或 `FullSyllable`），即包含翻译和拼音。
    pub fn is_full(&self) -> bool {
        matches!(self, Self::FullLine { .. } | Self::FullSyllable { .. })
    }

    /// 返回主行与子行中较早的开始时间。
    pub fn start_time_with_sub_line(&self) -> Option<i32> {
        let main = self.start_time();
        let sub = self.sub_line().and_then(|s| s.start_time());
        crate::helpers::math_helper::min_opt(main, sub)
    }

    /// 返回主行与子行中较晚的结束时间。
    pub fn end_time_with_sub_line(&self) -> Option<i32> {
        let main = self.end_time();
        let sub = self.sub_line().and_then(|s| s.end_time());
        crate::helpers::math_helper::max_opt(main, sub)
    }

    /// 返回包含子行在内的总持续时长（毫秒）。
    pub fn duration_with_sub_line(&self) -> Option<i32> {
        match (self.start_time_with_sub_line(), self.end_time_with_sub_line()) {
            (Some(s), Some(e)) => Some(e - s),
            _ => None,
        }
    }

    /// 返回包含子行文本的完整显示字符串。
    ///
    /// 子行文本以括号附加，若子行开始时间早于主行则前置显示。
    pub fn full_text(&self) -> String {
        match self {
            Self::Line { text, sub_line, .. } | Self::FullLine { text, sub_line, .. } => {
                if let Some(sub) = sub_line {
                    let sub_text = crate::helpers::string_helper::remove_front_back_brackets(&sub.text_from_any());
                    match (sub.start_time(), self.start_time()) {
                        (Some(sub_t), Some(main_t)) if sub_t < main_t => {
                            format!("({}) {}", sub_text, text.trim())
                        }
                        _ => {
                            format!("{} ({})", text.trim(), sub_text)
                        }
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
                    match (sub.start_time(), self.start_time()) {
                        (Some(sub_t), Some(main_t)) if sub_t < main_t => {
                            format!("({}) {}", sub_text, text.trim())
                        }
                        _ => {
                            format!("{} ({})", text.trim(), sub_text)
                        }
                    }
                } else {
                    text
                }
            }
        }
    }

    /// 从任意变体获取文本内容（音节行从音节列表拼接）。
    pub fn text_from_any(&self) -> String {
        match self {
            Self::Line { text, .. } | Self::FullLine { text, .. } => text.clone(),
            Self::Syllable { syllables, .. } | Self::FullSyllable { syllables, .. } => {
                Self::text_from_syllables(syllables)
            }
        }
    }

    /// 返回翻译映射的只读引用（仅 Full 系列变体支持）。
    pub fn translations(&self) -> Option<&HashMap<String, String>> {
        match self {
            Self::FullLine { translations, .. } | Self::FullSyllable { translations, .. } => {
                Some(translations)
            }
            _ => None,
        }
    }

    /// 返回翻译映射的可变引用（仅 Full 系列变体支持）。
    pub fn translations_mut(&mut self) -> Option<&mut HashMap<String, String>> {
        match self {
            Self::FullLine { translations, .. } | Self::FullSyllable { translations, .. } => {
                Some(translations)
            }
            _ => None,
        }
    }

    /// 返回中文翻译内容（键为 `"zh"`）。
    pub fn chinese_translation(&self) -> Option<&str> {
        self.translations()?.get("zh").map(|s| s.as_str())
    }

    /// 设置中文翻译。传入 `None` 或空字符串时移除该翻译。
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

    /// 将 `Line` 转换为 `FullLine`，或 `Syllable` 转换为 `FullSyllable`。
    ///
    /// 已是 Full 系列或其他变体则原样返回。
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

    /// 将 `Syllable` 转换为 `FullSyllable`，其他变体原样返回。
    pub fn to_full_syllable(self, translations: HashMap<String, String>, pronunciation: Option<String>) -> Self {
        match self {
            Self::Syllable { syllables, alignment, sub_line } => {
                Self::FullSyllable { syllables, alignment, sub_line, translations, pronunciation }
            }
            other => other,
        }
    }

    /// 返回开始时间，无时间信息时返回 0。
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
