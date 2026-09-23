use crate::parsers::{apply_offset, lyrics_data, parse_with_attributes};
use lyrics_core::helpers::string_helper::is_number;
use lyrics_core::models::*;
use regex::Regex;
use std::sync::LazyLock;

/// 逐字时间片段：`文本(开始时间,时长)`，与 QRC 的片段格式相同。
static SYLLABLE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(.*?)\((\d+),(\d+)\)").unwrap());

/// 解析 Lyricify Syllable 格式歌词，支持背景人声检测和对齐信息，返回 [`LyricsData`]。
pub fn parse(input: &str) -> LyricsData {
    let input = input.trim_start_matches('\u{feff}');
    parse_with_attributes(
        lyrics_data(
            LyricsTypes::LyricifySyllable,
            SyncTypes::SyllableSynced,
            Some(AdditionalFileInfo::new_general()),
        ),
        input.trim().lines().map(str::to_string).collect(),
        parse_lyrics,
    )
}

/// 解析 Lyricify Syllable 歌词行列表，处理背景人声和对齐信息，可选地应用时间偏移。
pub fn parse_lyrics(lines: &[String], offset: Option<i32>) -> Vec<LineInfo> {
    let list = lines
        .iter()
        .filter_map(|line| parse_lyrics_line_with_state(line))
        .collect();

    let mut new_list = set_background_vocals_info(list);
    apply_offset(&mut new_list, offset);
    new_list
}

/// 解析一行，返回歌词行与行头预设里的背景人声标记；没有任何音节的行返回 `None`。
fn parse_lyrics_line_with_state(line: &str) -> Option<(LineInfo, Option<bool>)> {
    let line = line.trim_start_matches('\u{feff}');
    let (preset, body) = match line.split_once(']') {
        Some((properties, body)) => (properties.strip_prefix('[').and_then(parse_preset), body),
        None => (None, line),
    };
    let (is_background_vocals, alignment) = preset.unwrap_or((None, LyricsAlignment::Unspecified));

    let syllables = SYLLABLE_RE
        .captures_iter(body)
        .map(|cap| {
            let start_time: i32 = cap[2].parse().ok()?;
            let duration: i32 = cap[3].parse().ok()?;
            Some(SyllableInfo::new(
                cap[1].to_string(),
                start_time,
                start_time + duration,
            ))
        })
        .collect::<Option<Vec<_>>>()?;

    if syllables.is_empty() {
        return None;
    }

    let mut line_info = LineInfo::new_syllable(to_syllable_items(syllables));
    line_info.set_alignment(alignment);

    Some((line_info, is_background_vocals))
}

/// 读取行头预设：`0..=2` 为主行、`3..=5` 为非背景人声、`6..` 为背景人声，
/// 对 3 取余得到对齐方式（0 未指定、1 左、2 右）。
fn parse_preset(properties: &str) -> Option<(Option<bool>, LyricsAlignment)> {
    if !is_number(properties) {
        return None;
    }
    let preset: u32 = properties.parse().ok()?;

    let is_background_vocals = match preset {
        6.. => Some(true),
        3.. => Some(false),
        _ => None,
    };
    let alignment = match preset % 3 {
        1 => LyricsAlignment::Left,
        2 => LyricsAlignment::Right,
        _ => LyricsAlignment::Unspecified,
    };

    Some((is_background_vocals, alignment))
}

fn set_background_vocals_info(list: Vec<(LineInfo, Option<bool>)>) -> Vec<LineInfo> {
    let mut items: Vec<(LineInfo, Option<bool>)> = list;

    // Set already marked background vocals
    let mut i = 1;
    while i < items.len() {
        if items[i].1 == Some(true) {
            let (sub_line, _) = items.remove(i);
            items[i - 1].0.set_sub_line(Some(Box::new(sub_line)));
        } else {
            i += 1;
        }
    }

    // Detect unmarked background vocals (bracketed lyrics)
    let is_not_bg = |item: &(LineInfo, Option<bool>)| -> bool {
        item.1.is_none() && !is_bracketed_lyrics(&item.0) || item.1 == Some(false)
    };

    let mut i = 1;
    while i < items.len() {
        if items[i].1.is_none()
            && is_bracketed_lyrics(&items[i].0)
            && is_not_bg(&items[i - 1])
            && items[i].0.sub_line().is_none()
            && (i + 1 >= items.len() || is_not_bg(&items[i + 1]))
        {
            let (sub_line, _) = items.remove(i);
            items[i - 1].0.set_sub_line(Some(Box::new(sub_line)));
        }
        i += 1;
    }

    items.into_iter().map(|(line, _)| line).collect()
}

fn is_bracketed_lyrics(line: &LineInfo) -> bool {
    let text = line.text_from_any();
    (text.starts_with('(') || text.starts_with('（'))
        && (text.ends_with(')') || text.ends_with('）'))
}
