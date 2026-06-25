use async_trait::async_trait;

use crate::providers::web::kugou::api;
use super::search_result::SearchResult;
use super::searcher::Searcher;
use super::Searchers;

pub struct KugouSearcher;

fn split_artists(singername: &str) -> Vec<String> {
    singername
        .split(|c: char| c == ',' || c == '、' || c == '/')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

#[async_trait]
impl Searcher for KugouSearcher {
    fn name(&self) -> &str {
        "Kugou"
    }

    fn display_name(&self) -> &str {
        "Kugou Music"
    }

    fn searcher_type(&self) -> Searchers {
        Searchers::Kugou
    }

    async fn search_for_results_str(&self, search_string: &str) -> Option<Vec<SearchResult>> {
        let response = api::search(search_string).await?;
        let songs = response.data?.info?;

        let search_results: Vec<SearchResult> = songs
            .into_iter()
            .map(|song| {
                let artists = split_artists(&song.singername);

                SearchResult {
                    searcher_type: Searchers::Kugou,
                    title: song.songname,
                    artists,
                    album: song.album_name.unwrap_or_default(),
                    album_artists: None,
                    duration_ms: song.duration.map(|d| d * 1000),
                    match_type: None,
                    id: song.hash,
                }
            })
            .collect();

        if search_results.is_empty() {
            return None;
        }

        Some(search_results)
    }
}
