use crate::parsers::{lrc_parser, lyrics_data};
use lyrics_core::models::*;
use serde::Deserialize;
use serde_json::Value;

/// Musixmatch 逐音节同步（richsync）的单行数据。
#[derive(Debug, Deserialize)]
pub struct RichSyncedLine {
    #[serde(rename = "ts")]
    pub time_start: f64,
    #[serde(rename = "te")]
    pub time_end: f64,
    #[serde(rename = "l")]
    pub words: Vec<RichSyncWord>,
    #[serde(rename = "x")]
    pub text: Option<String>,
}

/// Musixmatch richsync 中的单个词/字符及其时间偏移。
#[derive(Debug, Deserialize)]
pub struct RichSyncWord {
    #[serde(rename = "c")]
    pub chars: String,
    #[serde(rename = "o")]
    pub position: f64,
}

/// 按路径逐级取值，等价于一串 `get`。
fn dig<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    path.iter().try_fold(value, |value, key| value.get(key))
}

/// 宏调用是否返回了业务状态码 200。
fn is_ok(call: Option<&Value>) -> bool {
    call.and_then(|call| dig(call, &["message", "header", "status_code"]))
        .and_then(Value::as_i64)
        == Some(200)
}

/// 取宏调用响应体里指定路径上的字符串。
fn call_str<'a>(call: Option<&'a Value>, path: &[&str]) -> Option<&'a str> {
    let body = dig(call?, &["message", "body"])?;
    dig(body, path)?.as_str()
}

/// 构造一份只有歌词行的 [`LyricsData`]。
fn musixmatch_data(
    lines: Vec<LineInfo>,
    sync_types: SyncTypes,
    language: Option<&str>,
) -> LyricsData {
    LyricsData {
        lines: Some(lines),
        track_metadata: Some(TrackMetadata {
            language: language.map(|language| vec![language.to_string()]),
            ..TrackMetadata::default()
        }),
        ..lyrics_data(LyricsTypes::Musixmatch, sync_types, None)
    }
}

/// 解析 Musixmatch JSON 格式歌词，按优先级尝试 richsync、subtitles 和 unsynced，返回 [`LyricsData`]。
pub fn parse(raw_json: &str) -> Option<LyricsData> {
    parse_inner(raw_json, false)
}

/// 解析 Musixmatch JSON 格式歌词，可选择忽略逐音节数据。
///
/// `ignore_syllable` 为 `true` 时跳过 richsync 解析，直接尝试 subtitles。
pub fn parse_inner(raw_json: &str, ignore_syllable: bool) -> Option<LyricsData> {
    let json: Value = serde_json::from_str(raw_json).ok()?;
    let calls = dig(&json, &["message", "body", "macro_calls"])?;

    if !ignore_syllable && let Some(data) = parse_richsync(calls.get("track.richsync.get")) {
        return Some(data);
    }

    parse_subtitles(calls.get("track.subtitles.get"))
        .or_else(|| parse_unsynced(calls.get("track.lyrics.get")))
}

/// 逐音节同步（richsync）：每个词带一个相对行首的偏移量。
fn parse_richsync(call: Option<&Value>) -> Option<LyricsData> {
    if !is_ok(call) {
        return None;
    }

    let body = call_str(call, &["richsync", "richsync_body"])?;
    let list: Vec<RichSyncedLine> = serde_json::from_str(body).ok()?;

    let lines = list
        .iter()
        .map(|line| {
            let start = (line.time_start * 1000.0) as i32;
            let syllables: Vec<SyllableInfo> = line
                .words
                .iter()
                .enumerate()
                .map(|(index, word)| {
                    let end_time = match line.words.get(index + 1) {
                        Some(next) => start + (next.position * 1000.0) as i32,
                        None => (line.time_end * 1000.0) as i32,
                    };
                    SyllableInfo::new(
                        word.chars.clone(),
                        start + (word.position * 1000.0) as i32,
                        end_time,
                    )
                })
                .collect();
            LineInfo::new_syllable(to_syllable_items(syllables))
        })
        .collect();

    // 上游两种拼写都出现过，任取其一。
    let language = call_str(call, &["richsync", "richssync_language"])
        .or_else(|| call_str(call, &["richsync", "richsync_language"]));

    Some(musixmatch_data(lines, SyncTypes::SyllableSynced, language))
}

/// 行同步字幕：正文本身就是 LRC。
fn parse_subtitles(call: Option<&Value>) -> Option<LyricsData> {
    if !is_ok(call) {
        return None;
    }

    // `subtitle_list` 是数组，只取第一条字幕。
    let subtitle = dig(call?, &["message", "body", "subtitle_list"])?
        .get(0)?
        .get("subtitle")?;
    let body = subtitle.get("subtitle_body")?.as_str()?;

    Some(musixmatch_data(
        lrc_parser::parse_lyrics(body),
        SyncTypes::LineSynced,
        subtitle.get("subtitle_language").and_then(Value::as_str),
    ))
}

/// 无时间信息的纯文本歌词。
fn parse_unsynced(call: Option<&Value>) -> Option<LyricsData> {
    if !is_ok(call) {
        return None;
    }

    let body = call_str(call, &["lyrics", "lyrics_body"])?;
    let lines = body
        .trim()
        .lines()
        .map(|line| LineInfo::new_line_simple(line.to_string()))
        .collect();

    Some(musixmatch_data(lines, SyncTypes::Unsynced, None))
}
