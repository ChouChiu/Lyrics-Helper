use async_trait::async_trait;

use super::search_result::SearchResult;
use super::Searchers;
use crate::models::TrackMetadata;

#[async_trait]
pub trait Searcher: Sync {
    fn name(&self) -> &str;
    fn display_name(&self) -> &str;
    fn searcher_type(&self) -> Searchers;
    async fn search_for_results_str(&self, search_string: &str) -> Option<Vec<SearchResult>>;

    async fn search_for_results(&self, track: &TrackMetadata) -> Option<Vec<SearchResult>> {
        let title = track.title.as_deref().unwrap_or("");
        let artist = track.artist.as_deref().unwrap_or("").replace(", ", " ");
        let album = track.album.as_deref().unwrap_or("");
        let search_string = format!("{} {} {}", title, artist, album)
            .replace(" - ", " ")
            .trim()
            .to_string();
        self.search_for_results_str(&search_string).await
    }
}
