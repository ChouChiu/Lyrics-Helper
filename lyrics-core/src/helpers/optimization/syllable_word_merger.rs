use crate::helpers::string_helper::is_chinese_or_japanese_character;
use crate::models::{FullSyllableInfo, LineInfo, SyllableInfo, SyllableItem};

/// 将同一单词内的连续音节合并为一个 [`FullSyllableInfo`]。
///
/// 合并保留各子音节的时间信息，聚合文本与时间由子项推导。
pub fn merge(line: &mut LineInfo) {
    let Some(syllables) = line.syllables_mut() else {
        return;
    };

    if syllables.len() < 2 {
        return;
    }

    let mut merged: Vec<SyllableItem> = Vec::with_capacity(syllables.len());
    for current in syllables.drain(..) {
        match merged.pop() {
            Some(previous) if should_merge(&previous, &current) => {
                merged.push(merge_items(previous, current));
            }
            Some(previous) => {
                merged.push(previous);
                merged.push(current);
            }
            None => merged.push(current),
        }
    }

    *syllables = merged;
}

/// 按顺序遍历音节项各子音节的字符，避免为聚合文本额外分配。
fn chars_of(item: &SyllableItem) -> impl DoubleEndedIterator<Item = char> + '_ {
    item.parts().iter().flat_map(|part| part.text.chars())
}

/// 判断相邻的两个音节是否应合并为同一单词。
fn should_merge(previous: &SyllableItem, current: &SyllableItem) -> bool {
    let (Some(previous_last), Some(current_first)) =
        (chars_of(previous).next_back(), chars_of(current).next())
    else {
        return false;
    };

    if previous_last.is_whitespace() || current_first.is_whitespace() {
        return false;
    }

    if chars_of(previous).any(is_chinese_or_japanese_character)
        || chars_of(current).any(is_chinese_or_japanese_character)
    {
        return false;
    }

    chars_of(previous).any(char::is_alphanumeric) && chars_of(current).any(char::is_alphanumeric)
}

/// 将当前音节合并进前一个音节，返回合并后的音节项。
fn merge_items(previous: SyllableItem, current: SyllableItem) -> SyllableItem {
    let current_items = into_parts(current);

    match previous {
        SyllableItem::Full(mut full_previous) => {
            full_previous.extend_sub_items(current_items);
            SyllableItem::Full(full_previous)
        }
        other => {
            let mut items = into_parts(other);
            items.extend(current_items);
            SyllableItem::Full(FullSyllableInfo::new(items))
        }
    }
}

/// 消耗音节项并取出其普通音节列表。
fn into_parts(item: SyllableItem) -> Vec<SyllableInfo> {
    match item {
        SyllableItem::Syllable(syllable) => vec![syllable],
        SyllableItem::Full(full) => full.into_sub_items(),
    }
}
