use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SearchResponse {
    pub(crate) tracks: Option<SearchTracks>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SearchTracks {
    pub(crate) items: Option<Vec<SearchTrackItem>>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SearchTrackItem {
    pub(crate) id: String,
    pub(crate) name: String,
    #[serde(rename = "duration_ms")]
    pub(crate) duration_ms: i32,
    pub(crate) artists: Option<Vec<SearchArtistItem>>,
    pub(crate) album: Option<SearchAlbumItem>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SearchArtistItem {
    pub(crate) name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SearchAlbumItem {
    pub(crate) name: String,
    pub(crate) artists: Option<Vec<SearchArtistItem>>,
}
