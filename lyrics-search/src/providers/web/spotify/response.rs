use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct SearchResponse {
    pub tracks: Option<SearchTracks>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchTracks {
    pub items: Option<Vec<SearchTrackItem>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchTrackItem {
    pub id: String,
    pub name: String,
    #[serde(rename = "duration_ms")]
    pub duration_ms: i32,
    pub artists: Option<Vec<SearchArtistItem>>,
    pub album: Option<SearchAlbumItem>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchArtistItem {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchAlbumItem {
    pub name: String,
    pub artists: Option<Vec<SearchArtistItem>>,
}
