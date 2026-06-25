use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct SearchResponse {
    pub result: Option<SearchResultData>,
    pub code: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchResultData {
    pub songs: Option<Vec<Song>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Song {
    pub id: i64,
    pub name: String,
    pub artists: Vec<Artist>,
    pub album: Album,
    pub duration: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Artist {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Album {
    pub id: i64,
    pub name: String,
}
