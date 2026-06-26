use async_trait::async_trait;

use lyrics_core::models::TrackMetadata;
use crate::providers::web::lrclib::api;
use super::search_result::SearchResult;
use super::searcher::Searcher;
use super::Searchers;

/// LRCLIB 歌词搜索器。
pub struct LRCLIBSearcher;

#[async_trait]
impl Searcher for LRCLIBSearcher {
    fn name(&self) -> &str {
        "LRCLIB"
    }

    fn display_name(&self) -> &str {
        "LRCLIB"
    }

    fn searcher_type(&self) -> Searchers {
        Searchers::LRCLIB
    }

    async fn search_for_results_str(&self, search_string: &str) -> Option<Vec<SearchResult>> {
        let results = api::search(search_string, None, None, None).await?;
        if results.is_empty() {
            return None;
        }
        Some(map_results(results))
    }

    async fn search_for_results(&self, track: &TrackMetadata) -> Option<Vec<SearchResult>> {
        let title = track.title.as_deref()?;
        let artist = track.artist.as_deref().unwrap_or("");
        let album = track.album.as_deref();
        let duration = track.duration_ms.map(|ms| ms as f64 / 1000.0);

        if let Some(result) = api::get(title, artist, album, duration).await {
            let item = super::search_result::SearchResult {
                searcher_type: Searchers::LRCLIB,
                title: result.track_name,
                artists: parse_artists(&result.artist_name),
                album: result.album_name,
                album_artists: None,
                duration_ms: Some((result.duration * 1000.0) as i32),
                match_type: None,
                id: result.id.to_string(),
                numeric_id: None,
            };
            return Some(vec![item]);
        }

        let results = api::search(title, Some(artist), album, duration).await;
        if let Some(r) = results.filter(|r| !r.is_empty()) {
            return Some(map_results(r));
        }

        let search_string = format!("{} {}", title, artist).replace(" - ", " ").trim().to_string();
        let results = api::search(&search_string, None, None, None).await?;
        if results.is_empty() {
            return None;
        }
        Some(map_results(results))
    }
}

fn parse_artists(artist_str: &str) -> Vec<String> {
    artist_str
        .split(", ")
        .flat_map(|s| s.split(" & "))
        .flat_map(|s| s.split(" feat. "))
        .flat_map(|s| s.split(" ft. "))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn map_results(results: Vec<super::super::providers::web::lrclib::response::SearchResultItem>) -> Vec<SearchResult> {
    results
        .into_iter()
        .map(|item| SearchResult {
            searcher_type: Searchers::LRCLIB,
            title: item.track_name,
            artists: parse_artists(&item.artist_name),
            album: item.album_name,
            album_artists: None,
            duration_ms: Some((item.duration * 1000.0) as i32),
            match_type: None,
            id: item.id.to_string(),
            numeric_id: None,
        })
        .collect()
}
