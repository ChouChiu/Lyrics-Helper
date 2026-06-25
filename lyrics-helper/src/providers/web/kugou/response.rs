use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct SearchResponse {
    pub data: Option<SearchData>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchData {
    pub info: Option<Vec<SongInfo>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SongInfo {
    pub hash: String,
    pub songname: String,
    pub singername: String,
    pub album_name: Option<String>,
    pub duration: Option<i32>,
}
