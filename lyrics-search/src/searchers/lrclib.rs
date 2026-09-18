use async_trait::async_trait;

use super::Searchers;
use super::search_result::SearchResult;
use super::searcher::Searcher;
use crate::error::SearchError;
use crate::providers::web::lrclib::api;
use lyrics_core::models::TrackMetadata;

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

    async fn search_for_results_str(
        &self,
        search_string: &str,
    ) -> Result<Vec<SearchResult>, SearchError> {
        let results = api::search(search_string, None, None, None).await?;
        Ok(map_results(results))
    }

    /// 依次尝试精确获取、带过滤的搜索、不带过滤的搜索。
    ///
    /// 缺少曲名时无法构造任何查询，此时属于「没有结果」而非失败，返回 `Ok(vec![])`。
    async fn search_for_results(
        &self,
        track: &TrackMetadata,
    ) -> Result<Vec<SearchResult>, SearchError> {
        let Some(title) = track.title.as_deref() else {
            return Ok(Vec::new());
        };
        let artist = track.artist.as_deref().unwrap_or("");
        let album = track.album.as_deref();
        let duration = track.duration_ms.map(|ms| ms as f64 / 1000.0);

        if let Some(result) = api::get(title, artist, album, duration).await? {
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
            return Ok(vec![item]);
        }

        let results = api::search(title, Some(artist), album, duration).await?;
        if !results.is_empty() {
            return Ok(map_results(results));
        }

        let search_string = format!("{} {}", title, artist)
            .replace(" - ", " ")
            .trim()
            .to_string();
        let results = api::search(&search_string, None, None, None).await?;
        Ok(map_results(results))
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

fn map_results(
    results: Vec<super::super::providers::web::lrclib::response::SearchResultItem>,
) -> Vec<SearchResult> {
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
