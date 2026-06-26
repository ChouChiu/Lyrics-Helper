use serde::Deserialize;
use serde_json::Value;
use lyrics_core::models::*;
use crate::parsers::lrc_parser;

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

/// 解析 Musixmatch JSON 格式歌词，按优先级尝试 richsync、subtitles 和 unsynced，返回 [`LyricsData`]。
pub fn parse(raw_json: &str) -> Option<LyricsData> {
    parse_inner(raw_json, false)
}

/// 解析 Musixmatch JSON 格式歌词，可选择忽略逐音节数据。
///
/// `ignore_syllable` 为 `true` 时跳过 richsync 解析，直接尝试 subtitles。
pub fn parse_inner(raw_json: &str, ignore_syllable: bool) -> Option<LyricsData> {
    let json_obj: Value = serde_json::from_str(raw_json).ok()?;
    let calls = json_obj.get("message")?.get("body")?.get("macro_calls")?;

    fn check_header_200(obj: Option<&Value>) -> bool {
        obj.and_then(|o| o.get("message"))
            .and_then(|m| m.get("header"))
            .and_then(|h| h.get("status_code"))
            .and_then(|s| s.as_i64())
            == Some(200)
    }

    // Try richsync first
    if !ignore_syllable {
        let track_get = calls.get("track.richsync.get");
        if check_header_200(track_get) {
            let richsync_body = track_get
                .and_then(|t| t.get("message"))
                .and_then(|m| m.get("body"))
                .and_then(|b| b.get("richsync"))
                .and_then(|r| r.get("richsync_body"))
                .and_then(|r| r.as_str());

            if let Some(lyrics_str) = richsync_body {
                if let Ok(list) = serde_json::from_str::<Vec<RichSyncedLine>>(lyrics_str) {
                    let mut lines = Vec::new();
                    for line in &list {
                        let mut syllables = Vec::new();
                        let start = (line.time_start * 1000.0) as i32;
                        for i in 0..line.words.len() {
                            let end_time = if i + 1 < line.words.len() {
                                start + (line.words[i + 1].position * 1000.0) as i32
                            } else {
                                (line.time_end * 1000.0) as i32
                            };
                            syllables.push(SyllableInfo::new(
                                line.words[i].chars.clone(),
                                start + (line.words[i].position * 1000.0) as i32,
                                end_time,
                            ));
                        }
                        lines.push(LineInfo::new_syllable(syllables));
                    }

                    let language = track_get
                        .and_then(|t| t.get("message"))
                        .and_then(|m| m.get("body"))
                        .and_then(|b| b.get("richsync"))
                        .and_then(|r| r.get("richssync_language").or(r.get("richsync_language")))
                        .and_then(|l| l.as_str())
                        .map(|s| s.to_string());

                    let mut metadata = TrackMetadata::new();
                    if let Some(lang) = language {
                        metadata.language = Some(vec![lang]);
                    }

                    return Some(LyricsData {
                        file: Some(FileInfo {
                            lyrics_type: LyricsTypes::Musixmatch,
                            sync_types: SyncTypes::SyllableSynced,
                            additional_info: None,
                        }),
                        lines: Some(lines),
                        track_metadata: Some(metadata),
                        writers: None,
                    });
                }
            }
        }
    }

    // Try subtitles
    let track_get = calls.get("track.subtitles.get");
    if check_header_200(track_get) {
        let subtitle_list = track_get
            .and_then(|t| t.get("message"))
            .and_then(|m| m.get("body"))
            .and_then(|b| b.get("subtitle_list"))
            .and_then(|s| s.as_array());

        if let Some(list) = subtitle_list {
            if !list.is_empty() {
                let subtitle_body = list[0]
                    .get("subtitle")
                    .and_then(|s| s.get("subtitle_body"))
                    .and_then(|s| s.as_str());

                if let Some(subtitle) = subtitle_body {
                    let lines = lrc_parser::parse_lyrics(subtitle);
                    let language = list[0]
                        .get("subtitle")
                        .and_then(|s| s.get("subtitle_language"))
                        .and_then(|s| s.as_str())
                        .map(|s| s.to_string());

                    let mut metadata = TrackMetadata::new();
                    if let Some(lang) = language {
                        metadata.language = Some(vec![lang]);
                    }

                    return Some(LyricsData {
                        file: Some(FileInfo {
                            lyrics_type: LyricsTypes::Musixmatch,
                            sync_types: SyncTypes::LineSynced,
                            additional_info: None,
                        }),
                        lines: Some(lines),
                        track_metadata: Some(metadata),
                        writers: None,
                    });
                }
            }
        }
    }

    // Try lyrics (unsynced)
    let track_get = calls.get("track.lyrics.get");
    if check_header_200(track_get) {
        let lyrics_body = track_get
            .and_then(|t| t.get("message"))
            .and_then(|m| m.get("body"))
            .and_then(|b| b.get("lyrics"))
            .and_then(|l| l.get("lyrics_body"))
            .and_then(|l| l.as_str());

        if let Some(lyrics) = lyrics_body {
            let lines: Vec<LineInfo> = lyrics
                .trim()
                .lines()
                .map(|line| LineInfo::new_line_simple(line.to_string()))
                .collect();

            return Some(LyricsData {
                file: Some(FileInfo {
                    lyrics_type: LyricsTypes::Musixmatch,
                    sync_types: SyncTypes::Unsynced,
                    additional_info: None,
                }),
                lines: Some(lines),
                track_metadata: Some(TrackMetadata::new()),
                writers: None,
            });
        }
    }

    None
}
