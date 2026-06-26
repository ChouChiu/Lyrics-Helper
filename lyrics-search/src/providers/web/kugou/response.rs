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

#[derive(Debug, Clone, Deserialize)]
pub struct LyricSearchResponse {
    pub candidates: Option<Vec<LyricCandidate>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LyricCandidate {
    pub id: String,
    pub accesskey: String,
    pub song: Option<String>,
    pub singer: Option<String>,
    pub duration: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LyricDownloadResponse {
    pub content: Option<String>,
}
