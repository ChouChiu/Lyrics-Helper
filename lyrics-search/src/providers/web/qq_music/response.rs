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
    #[serde(rename = "item_song")]
    pub(crate) item_song: Option<Vec<SongItem>>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SongItem {
    #[serde(rename = "id")]
    pub(crate) _id: Option<i64>,
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
