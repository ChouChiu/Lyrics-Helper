use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SearchResponse {
    pub(crate) data: Option<SearchData>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SearchData {
    pub(crate) info: Option<Vec<SongInfo>>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SongInfo {
    pub(crate) hash: String,
    pub(crate) songname: String,
    pub(crate) singername: String,
    pub(crate) album_name: Option<String>,
    pub(crate) duration: Option<i32>,
}
