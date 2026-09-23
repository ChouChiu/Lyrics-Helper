use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SearchResponse {
    pub(crate) results: Option<SearchResults>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SearchResults {
    pub(crate) songs: Option<SongsContainer>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SongsContainer {
    pub(crate) data: Option<Vec<SongData>>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SongData {
    pub(crate) id: String,
    pub(crate) attributes: Option<SongAttributes>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SongAttributes {
    pub(crate) name: String,
    #[serde(rename = "artistName")]
    pub(crate) artist_name: String,
    #[serde(rename = "albumName")]
    pub(crate) album_name: String,
    #[serde(rename = "durationInMillis")]
    pub(crate) duration_in_millis: Option<i32>,
}
