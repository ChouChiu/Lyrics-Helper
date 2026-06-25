use super::compare_helper::MatchType;
use super::Searchers;

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub searcher_type: Searchers,
    pub title: String,
    pub artists: Vec<String>,
    pub album: String,
    pub album_artists: Option<Vec<String>>,
    pub duration_ms: Option<i32>,
    pub match_type: Option<MatchType>,
    pub id: String,
}

impl SearchResult {
    pub fn artist(&self) -> String {
        self.artists.join(", ")
    }

    pub fn album_artist(&self) -> Option<String> {
        self.album_artists.as_ref().map(|a| a.join(", "))
    }
}
