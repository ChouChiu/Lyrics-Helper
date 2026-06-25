use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct MusicuResponse {
    pub code: Option<i32>,
    pub request: Option<RequestData>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RequestData {
    pub code: Option<i32>,
    pub data: Option<RequestDataInner>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RequestDataInner {
    pub body: Option<Body>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Body {
    #[serde(rename = "item_song")]
    pub item_song: Option<Vec<SongItem>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SongItem {
    pub id: Option<i64>,
    pub mid: String,
    pub title: String,
    pub singer: Option<Vec<Singer>>,
    pub album: Option<Album>,
    pub interval: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Singer {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Album {
    pub name: String,
}
