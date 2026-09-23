use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct MusicuResponse {
    pub(crate) code: Option<i32>,
    pub(crate) request: Option<RequestData>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct RequestData {
    pub(crate) code: Option<i32>,
    pub(crate) data: Option<RequestDataInner>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct RequestDataInner {
    pub(crate) body: Option<Body>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Body {
    pub(crate) item_song: Option<Vec<SongItem>>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SongItem {
    #[serde(rename = "id")]
    pub(crate) numeric_id: Option<i64>,
    pub(crate) mid: String,
    pub(crate) title: String,
    pub(crate) singer: Option<Vec<Singer>>,
    pub(crate) album: Option<Album>,
    pub(crate) interval: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Singer {
    pub(crate) name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Album {
    pub(crate) name: String,
}

/// QQ 音乐歌词响应。
#[derive(Debug, Clone, Deserialize)]
pub struct LyricsResponse {
    /// 响应状态码
    pub code: Option<i32>,
    /// 请求数据
    pub request: Option<LyricsReqData>,
}

/// QQ 音乐歌词请求数据。
#[derive(Debug, Clone, Deserialize)]
pub struct LyricsReqData {
    /// 歌词数据
    pub data: Option<LyricsReqInner>,
}

/// QQ 音乐歌词详细数据。
#[derive(Debug, Clone, Deserialize)]
pub struct LyricsReqInner {
    /// 原文歌词
    pub lyric: Option<String>,
    /// 翻译歌词
    pub trans: Option<String>,
    /// 罗马音歌词
    pub roma: Option<String>,
    /// LRC 格式类型标记
    pub lrc_t: Option<i32>,
    /// QRC 格式类型标记
    pub qrc_t: Option<i32>,
    /// 翻译格式类型标记
    pub trans_t: Option<i32>,
    /// 罗马音格式类型标记
    pub roma_t: Option<i32>,
}
