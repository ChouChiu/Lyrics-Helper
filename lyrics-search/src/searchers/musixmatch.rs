use async_trait::async_trait;

use lyrics_core::models::TrackMetadata;

use crate::error::SearchError;
use crate::providers::web::musixmatch::api;
use crate::providers::web::musixmatch::response::Track;

use super::Searchers;
use super::compare_track_result;
use super::search_result::SearchResult;
use super::searcher::Searcher;

/// Musixmatch 歌词搜索器。
pub struct MusixmatchSearcher;

impl MusixmatchSearcher {
    /// 使用关键字搜索，并在提供曲目信息时按匹配度降序重排。
    ///
    /// `Ok(vec![])` 表示请求成功但没有相关结果；`Err` 表示请求失败。
    pub async fn search_for_results_async(
        keyword: Option<&str>,
        track: Option<&str>,
        artist: Option<&str>,
        duration_ms: Option<i32>,
    ) -> Result<Vec<SearchResult>, SearchError> {
        let duration_secs = duration_ms.filter(|d| *d > 0).map(|d| d / 1000);
        let tracks = api::search_tracks(keyword, track, artist, duration_secs).await?;

        let results = tracks.iter().map(to_search_result).collect();
        Ok(rank_results(results, track, artist, duration_ms))
    }
}

#[async_trait]
impl Searcher for MusixmatchSearcher {
    fn name(&self) -> &str {
        "Musixmatch"
    }

    fn display_name(&self) -> &str {
        "Musixmatch"
    }

    fn searcher_type(&self) -> Searchers {
        Searchers::Musixmatch
    }

    async fn search_for_results_str(
        &self,
        search_string: &str,
    ) -> Result<Vec<SearchResult>, SearchError> {
        Self::search_for_results_async(Some(search_string), None, None, None).await
    }
}

/// 将 Musixmatch 曲目映射为搜索结果。
fn to_search_result(track: &Track) -> SearchResult {
    let artists: Vec<String> = track
        .artist_name
        .split(" feat. ")
        .flat_map(|s| s.split(" & "))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    SearchResult {
        searcher_type: Searchers::Musixmatch,
        title: track.track_name.clone(),
        artists,
        album: track.album_name.clone().unwrap_or_default(),
        album_artists: None,
        duration_ms: Some(track.track_length * 1000),
        match_type: None,
        id: track.track_id.to_string(),
        numeric_id: None,
    }
}

/// 提供曲目信息时写入匹配等级并按匹配度降序排序。
fn rank_results(
    mut results: Vec<SearchResult>,
    track: Option<&str>,
    artist: Option<&str>,
    duration_ms: Option<i32>,
) -> Vec<SearchResult> {
    if track.is_none_or(|t| t.trim().is_empty()) {
        return results;
    }

    let metadata = TrackMetadata {
        title: track.map(str::to_string),
        artist: artist.map(str::to_string),
        artists: artist.map(|a| a.split(", ").map(str::to_string).collect()),
        duration_ms,
        ..Default::default()
    };
    for result in &mut results {
        result.match_type = Some(compare_track_result(&metadata, result));
    }
    results
        .sort_by_key(|result| std::cmp::Reverse(result.match_type.map(|m| m as i32).unwrap_or(-1)));
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::searchers::compare_helper::MatchType;

    fn result(title: &str, artist: &str) -> SearchResult {
        SearchResult {
            searcher_type: Searchers::Musixmatch,
            title: title.to_string(),
            artists: vec![artist.to_string()],
            album: String::new(),
            album_artists: None,
            duration_ms: None,
            match_type: None,
            id: String::new(),
            numeric_id: None,
        }
    }

    #[test]
    fn test_rank_results_sorts_by_match_type() {
        let results = vec![result("Unrelated", "Nobody"), result("Hello", "Adele")];

        let ranked = rank_results(results, Some("Hello"), Some("Adele"), None);

        assert_eq!(ranked[0].title, "Hello");
        assert_eq!(ranked[0].match_type, Some(MatchType::Perfect));
        assert_eq!(ranked[1].title, "Unrelated");
        assert_eq!(ranked[1].match_type, Some(MatchType::NoMatch));
    }

    #[test]
    fn test_rank_results_without_track_keeps_order() {
        let results = vec![result("Unrelated", "Nobody"), result("Hello", "Adele")];

        let ranked = rank_results(results, Some("  "), Some("Adele"), None);

        assert_eq!(ranked[0].title, "Unrelated");
        assert_eq!(ranked[1].title, "Hello");
        assert!(ranked.iter().all(|r| r.match_type.is_none()));
    }
}
