use async_trait::async_trait;

use crate::providers::web::musixmatch::api;
use super::compare_helper::MatchType;
use super::search_result::SearchResult;
use super::searcher::Searcher;
use super::Searchers;

/// Musixmatch 歌词搜索器。
pub struct MusixmatchSearcher;

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

    async fn search_for_results_str(&self, search_string: &str) -> Option<Vec<SearchResult>> {
        let token = api::get_token().await?;

        let (q_track, q_artist) = parse_search_string(search_string);

        let resp = api::search_track(&q_track, &q_artist, &token).await?;
        let message = resp.message?;
        let confidence = message.header.confidence;
        let track = message.body?.track?;

        let match_type = match confidence as i32 {
            1000 => MatchType::Perfect,
            950..=999 => MatchType::VeryHigh,
            900..=949 => MatchType::High,
            750..=899 => MatchType::PrettyHigh,
            600..=749 => MatchType::Medium,
            400..=599 => MatchType::Low,
            200..=399 => MatchType::VeryLow,
            _ => MatchType::NoMatch,
        };

        let artists: Vec<String> = track
            .artist_name
            .split(" feat. ")
            .flat_map(|s| s.split(" & "))
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let result = SearchResult {
            searcher_type: Searchers::Musixmatch,
            title: track.track_name,
            artists,
            album: track.album_name.unwrap_or_default(),
            album_artists: None,
            duration_ms: Some(track.track_length * 1000),
            match_type: Some(match_type),
            id: track.track_id.to_string(),
            numeric_id: None,
        };

        Some(vec![result])
    }
}

fn parse_search_string(search_string: &str) -> (String, String) {
    if let Some(idx) = search_string.find(" - ") {
        let artist = search_string[..idx].trim().to_string();
        let track = search_string[idx + 3..].trim().to_string();
        if !artist.is_empty() && !track.is_empty() {
            return (track, artist);
        }
    }

    let parts: Vec<&str> = search_string.split_whitespace().collect();
    match parts.len() {
        0 => (String::new(), String::new()),
        1 => (parts[0].to_string(), String::new()),
        _ => {
            let last = parts.len() - 1;
            let track = parts[..last].join(" ");
            let artist = parts[last].to_string();
            (track, artist)
        }
    }
}
