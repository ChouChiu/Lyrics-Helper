use async_trait::async_trait;

use super::Searchers;
use super::search_result::SearchResult;
use super::searcher::Searcher;
use crate::error::SearchError;
use crate::providers::web::spotify::api;

/// Spotify 歌词搜索器。
pub struct SpotifySearcher {
    access_token: String,
}

impl SpotifySearcher {
    /// 创建新的 Spotify 搜索器。
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

    async fn search_for_results_str(
        &self,
        search_string: &str,
    ) -> Result<Vec<SearchResult>, SearchError> {
        let response = api::search(search_string, &self.access_token).await?;

        // 缺少 tracks / items 都只是「没有匹配」，不是错误。
        let Some(tracks) = response.tracks.and_then(|tracks| tracks.items) else {
            return Ok(Vec::new());
        };

        let search_results: Vec<SearchResult> = tracks
            .into_iter()
            .filter_map(|track| {
                let artists: Vec<String> = track
                    .artists
                    .unwrap_or_default()
                    .into_iter()
                    .map(|a| a.name)
                    .collect();

                // 没有专辑信息的条目直接跳过，不视为请求失败。
                let album = track.album.as_ref()?;
                let album_name = album.name.clone();
                let album_artists: Option<Vec<String>> = album
                    .artists
                    .as_ref()
                    .map(|artists| artists.iter().map(|a| a.name.clone()).collect());

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

        Ok(search_results)
    }
}
