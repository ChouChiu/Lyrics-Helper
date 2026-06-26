use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SearchResponse {
    #[serde(rename = "result_groups")]
    pub(crate) result_groups: Option<Vec<ResultGroup>>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ResultGroup {
    pub(crate) data: Option<Vec<ResultGroupItem>>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ResultGroupItem {
    pub(crate) entity: Option<Entity>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Entity {
    pub(crate) track: Option<Track>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Track {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) duration: Option<i64>,
    pub(crate) artists: Option<Vec<Artist>>,
    pub(crate) album: Option<Album>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Artist {
    pub(crate) name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Album {
    pub(crate) name: String,
}
