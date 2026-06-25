use async_trait::async_trait;

use crate::providers::web::qq_music::api;
use super::search_result::SearchResult;
use super::searcher::Searcher;
use super::Searchers;

pub struct QQMusicSearcher;

#[async_trait]
impl Searcher for QQMusicSearcher {
    fn name(&self) -> &str {
        "QQMusic"
    }

    fn display_name(&self) -> &str {
        "QQ Music"
    }

    fn searcher_type(&self) -> Searchers {
        Searchers::QQMusic
    }

    async fn search_for_results_str(&self, search_string: &str) -> Option<Vec<SearchResult>> {
        let response = api::search(search_string).await?;

        if let Some(code) = response.code {
            if code != 0 {
                eprintln!("  [QQMusic] API error code: {}", code);
                return None;
            }
        }

        let req = response.request?;
        if let Some(code) = req.code {
            if code != 0 {
                eprintln!("  [QQMusic] Request error code: {}", code);
                return None;
            }
        }

        let songs = req.data?.body?.item_song?;

        let search_results: Vec<SearchResult> = songs
            .into_iter()
            .map(|song| {
                let artists: Vec<String> = song
                    .singer
                    .unwrap_or_default()
                    .into_iter()
                    .map(|s| s.name)
                    .collect();

                SearchResult {
                    searcher_type: Searchers::QQMusic,
                    title: song.title,
                    artists,
                    album: song.album.map(|a| a.name).unwrap_or_default(),
                    album_artists: None,
                    duration_ms: song.interval.map(|i| i * 1000),
                    match_type: None,
                    id: song.mid,
                }
            })
            .collect();

        if search_results.is_empty() {
            return None;
        }

        Some(search_results)
    }
}
