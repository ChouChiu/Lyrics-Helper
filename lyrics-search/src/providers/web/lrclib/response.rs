use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SearchResultItem {
    pub(crate) id: i32,
    #[serde(rename = "name")]
    pub(crate) _name: String,
    #[serde(rename = "trackName")]
    pub(crate) track_name: String,
    #[serde(rename = "artistName")]
    pub(crate) artist_name: String,
    #[serde(rename = "albumName")]
    pub(crate) album_name: String,
    pub(crate) duration: f64,
    #[serde(rename = "instrumental")]
    pub(crate) _instrumental: bool,
    #[serde(rename = "plainLyrics")]
    pub(crate) _plain_lyrics: Option<String>,
    #[serde(rename = "syncedLyrics")]
    pub(crate) _synced_lyrics: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GetLyricResult {
    pub(crate) id: i32,
    #[serde(rename = "name")]
    pub(crate) _name: String,
    #[serde(rename = "trackName")]
    pub(crate) track_name: String,
    #[serde(rename = "artistName")]
    pub(crate) artist_name: String,
    #[serde(rename = "albumName")]
    pub(crate) album_name: String,
    pub(crate) duration: f64,
    #[serde(rename = "instrumental")]
    pub(crate) _instrumental: bool,
    #[serde(rename = "plainLyrics")]
    pub(crate) _plain_lyrics: Option<String>,
    #[serde(rename = "syncedLyrics")]
    pub(crate) _synced_lyrics: Option<String>,
}
