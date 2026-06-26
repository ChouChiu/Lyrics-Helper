use async_trait::async_trait;

use crate::providers::web::soda_music::api;
use super::search_result::SearchResult;
use super::searcher::Searcher;
use super::Searchers;

/// 汽水音乐歌词搜索器。
pub struct SodaMusicSearcher;

#[async_trait]
impl Searcher for SodaMusicSearcher {
    fn name(&self) -> &str {
        "SodaMusic"
    }

    fn display_name(&self) -> &str {
        "Soda Music"
    }

    fn searcher_type(&self) -> Searchers {
        Searchers::SodaMusic
    }

    async fn search_for_results_str(&self, search_string: &str) -> Option<Vec<SearchResult>> {
        let response = api::search(search_string).await?;
        let groups = response.result_groups?;
        let items = groups.first()?.data.as_ref()?;

        let search_results: Vec<SearchResult> = items
            .iter()
            .filter_map(|item| {
                let track = item.entity.as_ref()?.track.as_ref()?;

                let artists: Vec<String> = track
                    .artists
                    .as_ref()
                    .map(|a| a.iter().map(|ar| ar.name.clone()).collect())
                    .unwrap_or_default();

                Some(SearchResult {
                    searcher_type: Searchers::SodaMusic,
                    title: track.name.clone(),
                    artists,
                    album: track.album.as_ref().map(|a| a.name.clone()).unwrap_or_default(),
                    album_artists: None,
                    duration_ms: track.duration.map(|d| d as i32),
                    match_type: None,
                    id: track.id.clone(),
                    numeric_id: None,
                })
            })
            .collect();

        if search_results.is_empty() {
            return None;
        }

        Some(search_results)
    }
}
