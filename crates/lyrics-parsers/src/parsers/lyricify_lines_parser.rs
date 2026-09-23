use crate::parsers::{lyrics_data, parse_with_attributes};
use lyrics_core::models::*;

/// 解析 Lyricify Lines 格式歌词（行同步），返回 [`LyricsData`]。
pub fn parse(input: &str) -> LyricsData {
    let input = input.replace("[type:LyricifyLines]", "");
    parse_with_attributes(
        lyrics_data(
            LyricsTypes::LyricifyLines,
            SyncTypes::LineSynced,
            Some(AdditionalFileInfo::new_general()),
        ),
        input.trim().lines().map(str::to_string).collect(),
        parse_lyrics,
    )
}

/// 解析 Lyricify Lines 歌词行列表，可选地应用时间偏移，返回行同步歌词列表。
///
/// 不是 `[开始时间,结束时间]文本` 形式的行会被跳过。
pub fn parse_lyrics(lines: &[String], offset: Option<i32>) -> Vec<LineInfo> {
    let offset = offset.unwrap_or(0);
    lines
        .iter()
        .filter_map(|line| parse_line(line))
        .map(|(begin, end, text)| {
            LineInfo::new_line(
                text.trim().to_string(),
                Some(begin - offset),
                Some(end - offset),
            )
        })
        .collect()
}

fn parse_line(line: &str) -> Option<(i32, i32, &str)> {
    let (header, text) = line.strip_prefix('[')?.split_once(']')?;
    let (begin, end) = header.split_once(',')?;
    Some((begin.parse().ok()?, end.parse().ok()?, text))
}
