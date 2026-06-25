use serde::Deserialize;
use lyrics_core::models::*;

#[derive(Debug, Deserialize)]
pub struct SpotifyColorLyrics {
    pub lyrics: Option<SpotifyLyrics>,
}

#[derive(Debug, Deserialize)]
pub struct SpotifyLyrics {
    #[serde(rename = "syncType")]
    pub sync_type: String,
    pub lines: Vec<SpotifyLyricsLine>,
    pub provider: Option<String>,
    #[serde(rename = "providerLyricsId")]
    pub provider_lyrics_id: Option<String>,
    #[serde(rename = "providerDisplayName")]
    pub provider_display_name: Option<String>,
    pub language: Option<String>,
    #[serde(rename = "isRtlLanguage")]
    pub is_rtl_language: Option<bool>,
    #[serde(rename = "alternatives")]
    pub alternatives: Option<Vec<AlternativeItem>>,
}

#[derive(Debug, Deserialize)]
pub struct SpotifyLyricsLine {
    #[serde(rename = "startTimeMs", default)]
    pub start_time_ms: String,
    #[serde(rename = "endTimeMs", default)]
    pub end_time_ms: String,
    pub words: String,
    pub syllables: Option<Vec<SyllableItem>>,
}

#[derive(Debug, Deserialize)]
pub struct SyllableItem {
    #[serde(rename = "startTimeMs", default)]
    pub start_time_ms: String,
    #[serde(rename = "endTimeMs", default)]
    pub end_time_ms: String,
    #[serde(rename = "numChars", alias = "charsCount", default)]
    pub chars_count: String,
}

#[derive(Debug, Deserialize)]
pub struct AlternativeItem {
    pub language: Option<String>,
    pub lines: Option<Vec<String>>,
    #[serde(rename = "isRtlLanguage")]
    pub is_rtl_language: Option<bool>,
}

pub fn parse(raw_json: &str) -> Option<LyricsData> {
    let color_lyrics: SpotifyColorLyrics = serde_json::from_str(raw_json).ok()?;
    let lyrics = color_lyrics.lyrics?;

    let parsed_lines = parse_lyrics_from_spotify(&lyrics);
    let sync_type = match lyrics.sync_type.as_str() {
        "UNSYNCED" => SyncTypes::Unsynced,
        "LINE_SYNCED" => SyncTypes::LineSynced,
        "SYLLABLE_SYNCED" => SyncTypes::SyllableSynced,
        _ => SyncTypes::Unknown,
    };

    Some(LyricsData {
        file: Some(FileInfo {
            lyrics_type: LyricsTypes::Spotify,
            sync_types: sync_type,
            additional_info: Some(AdditionalFileInfo::new_spotify(
                lyrics.provider.clone(),
                lyrics.provider_lyrics_id.clone(),
                lyrics.provider_display_name.clone(),
                lyrics.language.clone(),
            )),
        }),
        lines: Some(parsed_lines),
        writers: None,
        track_metadata: None,
    })
}

fn parse_lyrics_from_spotify(lyrics: &SpotifyLyrics) -> Vec<LineInfo> {
    if lyrics.sync_type == "UNSYNCED" {
        parse_unsynced_lyrics(&lyrics.lines)
    } else {
        parse_synced_lyrics(&lyrics.lines)
    }
}

fn parse_unsynced_lyrics(lyrics: &[SpotifyLyricsLine]) -> Vec<LineInfo> {
    lyrics
        .iter()
        .map(|line| LineInfo::new_line_simple(line.words.clone()))
        .collect()
}

fn parse_synced_lyrics(lyrics: &[SpotifyLyricsLine]) -> Vec<LineInfo> {
    let mut list = Vec::new();

    for line in lyrics {
        if let Some(ref syllables) = line.syllables {
            if !syllables.is_empty() {
                let mut syllable_list = Vec::new();
                let mut char_idx = 0;
                for syllable in syllables {
                    let chars_count: usize = syllable.chars_count.parse().unwrap_or(0);
                    let start_time: i32 = syllable.start_time_ms.parse().unwrap_or(0);
                    let end_time: i32 = syllable.end_time_ms.parse().unwrap_or(0);
                    let text = line.words.chars().skip(char_idx).take(chars_count).collect();
                    syllable_list.push(SyllableInfo::new(text, start_time, end_time));
                    char_idx += chars_count;
                }
                list.push(LineInfo::new_syllable(syllable_list));
            } else {
                let start_time: i32 = line.start_time_ms.parse().unwrap_or(0);
                let end_time: i32 = line.end_time_ms.parse().unwrap_or(0);
                if end_time != 0 {
                    list.push(LineInfo::new_line(line.words.clone(), Some(start_time), Some(end_time)));
                } else {
                    list.push(LineInfo::new_line_with_time(line.words.clone(), start_time));
                }
            }
        } else {
            let start_time: i32 = line.start_time_ms.parse().unwrap_or(0);
            let end_time: i32 = line.end_time_ms.parse().unwrap_or(0);
            if end_time != 0 {
                list.push(LineInfo::new_line(line.words.clone(), Some(start_time), Some(end_time)));
            } else {
                list.push(LineInfo::new_line_with_time(line.words.clone(), start_time));
            }
        }
    }

    list
}
