use async_trait::async_trait;

use crate::providers::web::spotify::api;
use super::search_result::SearchResult;
use super::searcher::Searcher;
use super::Searchers;

pub struct SpotifySearcher {
    access_token: String,
}

impl SpotifySearcher {
    pub fn new(access_token: String) -> Self {
        Self { access_token }
    }
}

#[async_trait]
impl Searcher for SpotifySearcher {
    fn name(&self) -> &str {
        "Spotify"
    }

    fn display_name(&self) -> &str {
        "Spotify"
    }

    fn searcher_type(&self) -> Searchers {
        Searchers::Spotify
    }

    async fn search_for_results_str(&self, search_string: &str) -> Option<Vec<SearchResult>> {
        let response = api::search(search_string, &self.access_token).await?;

        let tracks = response.tracks?.items?;

        let search_results: Vec<SearchResult> = tracks
            .into_iter()
            .filter_map(|track| {
                let artists: Vec<String> = track
                    .artists
                    .unwrap_or_default()
                    .into_iter()
                    .map(|a| a.name)
                    .collect();

                let album = track.album.as_ref()?;
                let album_name = album.name.clone();
                let album_artists: Option<Vec<String>> = album.artists.as_ref().map(|artists| {
                    artists.iter().map(|a| a.name.clone()).collect()
                });

                Some(SearchResult {
                    searcher_type: Searchers::Spotify,
                    title: track.name,
                    artists,
                    album: album_name,
                    album_artists,
                    duration_ms: Some(track.duration_ms),
                    match_type: None,
                    id: track.id,
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
