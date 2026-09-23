use async_trait::async_trait;

use super::Searchers;
use super::search_result::SearchResult;
use super::searcher::Searcher;
use crate::error::SearchError;
use crate::providers::web::lrclib::api;
use crate::providers::web::lrclib::response::SearchResultItem;
use lyrics_core::models::TrackMetadata;

use super::search_result::split_artists;

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
            return Ok(vec![to_search_result(result)]);
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

fn to_search_result(item: SearchResultItem) -> SearchResult {
    SearchResult::new(
        Searchers::LRCLIB,
        item.track_name,
        split_artists(&item.artist_name),
        item.album_name,
        Some((item.duration * 1000.0) as i32),
        item.id.to_string(),
    )
}

fn map_results(results: Vec<SearchResultItem>) -> Vec<SearchResult> {
    results.into_iter().map(to_search_result).collect()
}
