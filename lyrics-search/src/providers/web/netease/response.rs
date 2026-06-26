use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SearchResponse {
    pub(crate) result: Option<SearchResultData>,
    #[serde(rename = "code")]
    pub(crate) _code: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SearchResultData {
    pub(crate) songs: Option<Vec<Song>>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Song {
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) artists: Vec<Artist>,
    pub(crate) album: Album,
    pub(crate) duration: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Artist {
    #[serde(rename = "id")]
    pub(crate) _id: i64,
    pub(crate) name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Album {
    #[serde(rename = "id")]
    pub(crate) _id: i64,
    pub(crate) name: String,
}

/// 网易云音乐歌词响应。
#[derive(Debug, Clone, Deserialize)]
pub struct LyricsResponse {
    /// 响应状态码
    pub code: Option<i32>,
    /// 原文歌词
    pub lrc: Option<LyricContent>,
    /// 翻译歌词
    pub tlyric: Option<LyricContent>,
}

/// 网易云音乐歌词内容。
#[derive(Debug, Clone, Deserialize)]
pub struct LyricContent {
    /// LRC 格式歌词文本
    pub lyric: Option<String>,
}
