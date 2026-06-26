use async_trait::async_trait;

use crate::providers::web::apple_music::api;
use super::search_result::SearchResult;
use super::searcher::Searcher;
use super::Searchers;

pub struct AppleMusicSearcher {
    access_token: String,
    storefront: String,
    language: String,
}

impl AppleMusicSearcher {
    pub fn new(access_token: String) -> Self {
        Self {
            access_token,
            storefront: "us".to_string(),
            language: "en-US".to_string(),
        }
    }
}

#[async_trait]
impl Searcher for AppleMusicSearcher {
    fn name(&self) -> &str {
        "AppleMusic"
    }

    fn display_name(&self) -> &str {
        "Apple Music"
    }

    fn searcher_type(&self) -> Searchers {
        Searchers::AppleMusic
    }

    async fn search_for_results_str(&self, search_string: &str) -> Option<Vec<SearchResult>> {
        let response = api::search(
            search_string,
            &self.access_token,
            &self.storefront,
            &self.language,
        )
        .await?;

        let songs = response.results?.songs?.data?;

        let search_results: Vec<SearchResult> = songs
            .into_iter()
            .filter_map(|song| {
                let attrs = song.attributes?;
                Some(SearchResult {
                    searcher_type: Searchers::AppleMusic,
                    title: attrs.name,
                    artists: vec![attrs.artist_name],
                    album: attrs.album_name,
                    album_artists: None,
                    duration_ms: attrs.duration_in_millis,
                    match_type: None,
                    id: song.id,
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
