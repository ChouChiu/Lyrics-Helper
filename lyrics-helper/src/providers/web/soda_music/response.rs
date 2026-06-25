use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct SearchResponse {
    #[serde(rename = "result_groups")]
    pub result_groups: Option<Vec<ResultGroup>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResultGroup {
    pub data: Option<Vec<ResultGroupItem>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResultGroupItem {
    pub entity: Option<Entity>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Entity {
    pub track: Option<Track>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Track {
    pub id: String,
    pub name: String,
    pub duration: Option<i64>,
    pub artists: Option<Vec<Artist>>,
    pub album: Option<Album>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Artist {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Album {
    pub name: String,
}
