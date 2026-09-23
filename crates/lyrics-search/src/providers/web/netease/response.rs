use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SearchResponse {
    pub(crate) result: Option<SearchResultData>,
    #[serde(rename = "code")]
    pub(crate) code: Option<i64>,
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
    pub(crate) name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Album {
    pub(crate) name: String,
}

/// 网易云 eapi 单曲搜索响应，对应 C# `EapiSearchResult`。
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct EapiSearchResponse {
    pub(crate) result: Option<EapiSearchResultData>,
    pub(crate) code: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct EapiSearchResultData {
    pub(crate) songs: Option<Vec<EapiSong>>,
}

/// eapi 搜索返回的单曲，字段名与 web 接口不同，对应 C# `EapiSong`。
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct EapiSong {
    pub(crate) id: i64,
    pub(crate) name: String,
    #[serde(rename = "ar")]
    pub(crate) artists: Vec<Artist>,
    #[serde(rename = "al")]
    pub(crate) album: Album,
    #[serde(rename = "dt")]
    pub(crate) duration: i64,
}

impl From<EapiSong> for Song {
    fn from(song: EapiSong) -> Self {
        Self {
            id: song.id,
            name: song.name,
            artists: song.artists,
            album: song.album,
            duration: song.duration,
        }
    }
}

impl From<EapiSearchResponse> for SearchResponse {
    fn from(response: EapiSearchResponse) -> Self {
        Self {
            result: response.result.map(|result| SearchResultData {
                songs: result
                    .songs
                    .map(|songs| songs.into_iter().map(Song::from).collect()),
            }),
            code: response.code,
        }
    }
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

/// 网易云音乐 eapi 逐字歌词响应，对应 C# `Api.GetLyricNew` 的返回体。
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SyllableLyricsResponse {
    /// 逐字歌词（YRC）
    pub(crate) yrc: Option<LyricContent>,
    /// 逐字翻译歌词（YRC）
    pub(crate) ytlrc: Option<LyricContent>,
    /// 逐字罗马音歌词（YRC）
    pub(crate) yromalrc: Option<LyricContent>,
}
