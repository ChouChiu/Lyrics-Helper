use super::eapi;
use super::response::{LyricContent, LyricsResponse, SearchResponse, SyllableLyricsResponse};
use crate::providers::web::base_api;

const REFERER: &str = "https://music.163.com/";

/// 逐字歌词接口地址，对应 C# `Api.GetLyricNew`。
const SYLLABLE_LYRICS_URL: &str = "https://interface3.music.163.com/eapi/song/lyric/v1";

fn standard_headers() -> Vec<(&'static str, &'static str)> {
    vec![
        ("Referer", REFERER),
        ("Cookie", base_api::COOKIE),
        ("User-Agent", base_api::USER_AGENT),
    ]
}

pub(crate) async fn search(keyword: &str) -> Option<SearchResponse> {
    let url = format!(
        "https://music.163.com/api/search/get?s={}&type=1&limit=10&offset=0",
        urlencoding::encode(keyword)
    );
    base_api::get_json_with_headers(&url, &standard_headers()).await
}

/// 获取网易云音乐歌词，返回 `(原文歌词, 翻译歌词)` 元组。
///
/// 该接口只返回逐行歌词；逐字歌词请使用 [`get_syllable_lyrics`]。
pub async fn get_lyrics(song_id: i64) -> Option<(Option<String>, Option<String>)> {
    let url = format!(
        "https://music.163.com/api/song/lyric?id={}&lv=1&kv=1&tv=-1",
        song_id
    );
    let resp: LyricsResponse = base_api::get_json_with_headers(&url, &standard_headers()).await?;
    Some((
        resp.lrc.and_then(|l| l.lyric),
        resp.tlyric.and_then(|t| t.lyric),
    ))
}

/// 网易云音乐逐字歌词，字段对应 eapi 接口返回体中各 `*.lyric` 字符串。
#[derive(Debug, Clone, Default)]
pub struct SyllableLyrics {
    /// 逐字歌词（YRC，含 JSON 信息行）
    pub yrc: Option<String>,
    /// 逐字翻译歌词（YRC）
    pub ytlrc: Option<String>,
    /// 逐字罗马音歌词（YRC）
    pub yromalrc: Option<String>,
}

/// 获取网易云音乐逐字歌词，对应 C# `Api.GetLyricNew`。
///
/// 逐字歌词为带 JSON 信息行的 YRC 格式，可直接交给
/// [`LyricsRawTypes::Yrc`](lyrics_core::models::LyricsRawTypes::Yrc) 解析。
/// 曲目没有逐字歌词时返回 `None`，此时可回退到 [`get_lyrics`] 获取逐行歌词。
pub async fn get_syllable_lyrics(song_id: i64) -> Option<SyllableLyrics> {
    let response = eapi::post(
        SYLLABLE_LYRICS_URL,
        serde_json::json!({
            "id": song_id.to_string(),
            "cp": "false",
            "lv": "0",
            "kv": "0",
            "tv": "0",
            "rv": "0",
            "yv": "0",
            "ytv": "0",
            "yrv": "0",
            "csrf_token": "",
        }),
    )
    .await?;

    let response: SyllableLyricsResponse = serde_json::from_str(&response).ok()?;
    let lyrics = SyllableLyrics {
        yrc: lyric_text(response.yrc),
        ytlrc: lyric_text(response.ytlrc),
        yromalrc: lyric_text(response.yromalrc),
    };

    (lyrics.yrc.is_some() || lyrics.ytlrc.is_some() || lyrics.yromalrc.is_some()).then_some(lyrics)
}

/// 取出歌词文本，接口会以空字符串表示没有该类歌词。
fn lyric_text(content: Option<LyricContent>) -> Option<String> {
    content
        .and_then(|content| content.lyric)
        .filter(|lyric| !lyric.is_empty())
}
