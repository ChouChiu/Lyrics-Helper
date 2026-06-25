use async_trait::async_trait;

use crate::providers::web::netease::api;
use super::search_result::SearchResult;
use super::searcher::Searcher;
use super::Searchers;

pub struct NeteaseSearcher;

#[async_trait]
impl Searcher for NeteaseSearcher {
    fn name(&self) -> &str {
        "Netease"
    }

    fn display_name(&self) -> &str {
        "Netease Cloud Music"
    }

    fn searcher_type(&self) -> Searchers {
        Searchers::Netease
    }

    async fn search_for_results_str(&self, search_string: &str) -> Option<Vec<SearchResult>> {
        let response = api::search(search_string).await?;
        let songs = response.result?.songs?;

        let search_results: Vec<SearchResult> = songs
            .into_iter()
            .map(|song| {
                let artists: Vec<String> = song.artists.iter().map(|a| a.name.clone()).collect();

                SearchResult {
                    searcher_type: Searchers::Netease,
                    title: song.name,
                    artists,
                    album: song.album.name,
                    album_artists: None,
                    duration_ms: Some(song.duration as i32),
                    match_type: None,
                    id: song.id.to_string(),
                }
            })
            .collect();

        if search_results.is_empty() {
            return None;
        }

        Some(search_results)
    }
}
