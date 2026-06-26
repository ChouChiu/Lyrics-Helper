use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub(crate) struct SearchResponse {
    pub(crate) result: Option<SearchResultData>,
    pub(crate) code: Option<i32>,
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
#[allow(dead_code)]
pub(crate) struct Artist {
    pub(crate) id: i64,
    pub(crate) name: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub(crate) struct Album {
    pub(crate) id: i64,
    pub(crate) name: String,
}
