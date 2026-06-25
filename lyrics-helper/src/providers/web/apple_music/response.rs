use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct SearchResponse {
    pub results: Option<SearchResults>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchResults {
    pub songs: Option<SongsContainer>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SongsContainer {
    pub data: Option<Vec<SongData>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SongData {
    pub id: String,
    pub attributes: Option<SongAttributes>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SongAttributes {
    pub name: String,
    #[serde(rename = "artistName")]
    pub artist_name: String,
    #[serde(rename = "albumName")]
    pub album_name: String,
    #[serde(rename = "durationInMillis")]
    pub duration_in_millis: Option<i32>,
}
