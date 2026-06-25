pub mod apple_music;
pub mod compare_helper;
pub mod kugou;
pub mod lrclib;
pub mod musixmatch;
pub mod netease;
pub mod qq_music;
pub mod search_result;
pub mod searcher;
pub mod soda_music;
pub mod spotify;

use compare_helper::*;
use lyrics_core::models::TrackMetadata;
use search_result::SearchResult;
use searcher::Searcher;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Searchers {
    QQMusic,
    Netease,
    Kugou,
    Musixmatch,
    SodaMusic,
    AppleMusic,
    Spotify,
    LRCLIB,
}

pub fn compare_track(
    track: &TrackMetadata,
    result_title: Option<&str>,
    result_artists: &[String],
    result_album: Option<&str>,
    result_album_artists: &[String],
    result_duration_ms: Option<i32>,
) -> MatchType {
    let track_match = compare_name(track.title.as_deref(), result_title);
    let artist_match = compare_artist(
        track.artists.as_deref().unwrap_or(&[]),
        result_artists,
    );
    let album_match = compare_name(track.album.as_deref(), result_album);
    let album_artist_match = compare_artist(
        track.album_artists.as_deref().unwrap_or(&[]),
        result_album_artists,
    );
    let duration_match = compare_duration(track.duration_ms, result_duration_ms);

    let mut total_score = 0.0f64;
    total_score += name_score(track_match);
    total_score += artist_score(artist_match);
    total_score += name_score(album_match) * 0.4;
    total_score += artist_score(album_artist_match) * 0.2;
    total_score += duration_score(duration_match);

    let mut null_count = 0.0f64;
    if album_match.is_none() {
        null_count += 0.4;
    }
    if album_artist_match.is_none() {
        null_count += 0.2;
    }
    if duration_match.is_none() {
        null_count += 1.0;
    }
    let full_score = 30.0f64;
    if null_count > 0.0 {
        total_score = total_score * full_score / (full_score - null_count * 7.0);
    }

    if total_score > 21.0 {
        MatchType::Perfect
    } else if total_score > 19.0 {
        MatchType::VeryHigh
    } else if total_score > 17.0 {
        MatchType::High
    } else if total_score > 15.0 {
        MatchType::PrettyHigh
    } else if total_score > 11.0 {
        MatchType::Medium
    } else if total_score > 8.0 {
        MatchType::Low
    } else if total_score > 3.0 {
        MatchType::VeryLow
    } else {
        MatchType::NoMatch
    }
}

pub fn compare_track_result(track: &TrackMetadata, result: &SearchResult) -> MatchType {
    compare_track(
        track,
        Some(&result.title),
        &result.artists,
        Some(&result.album),
        result.album_artists.as_deref().unwrap_or(&[]),
        result.duration_ms,
    )
}

pub fn build_search_string(track: &TrackMetadata) -> String {
    let title = track.title.as_deref().unwrap_or("");
    let artist = track.artist.as_deref().unwrap_or("").replace(", ", " ");
    let album = track.album.as_deref().unwrap_or("");
    format!("{} {} {}", title, artist, album)
        .replace(" - ", " ")
        .trim()
        .to_string()
}

pub fn strip_feat(title: &str) -> String {
    let mut new_title = title.to_string();
    if let Some(idx) = new_title.find("(feat.") {
        new_title = new_title[..idx].trim().to_string();
    }
    if let Some(idx) = new_title.find(" - feat.") {
        new_title = new_title[..idx].trim().to_string();
    }
    new_title
}

pub fn build_refinement_queries(track: &TrackMetadata) -> Vec<String> {
    let title = track.title.as_deref().unwrap_or("");
    let new_title = strip_feat(title);
    let artist = track.artist.as_deref().unwrap_or("").replace(", ", " ");

    let level1 = format!("{} {}", new_title, artist)
        .replace(" - ", " ")
        .trim()
        .to_string();
    let level2 = new_title.replace(" - ", " ").trim().to_string();

    vec![level1, level2]
}

pub async fn search_with_refinement(
    searcher: &dyn Searcher,
    track: &TrackMetadata,
    full_search: bool,
) -> Vec<SearchResult> {
    if let Some(results) = searcher.search_for_results(track).await {
        if !results.is_empty() {
            let mut results = results;
            for result in &mut results {
                result.match_type = Some(compare_track_result(track, result));
            }
            results.sort_by(|a, b| {
                let a_val = a.match_type.map(|m| m as i32).unwrap_or(-1);
                let b_val = b.match_type.map(|m| m as i32).unwrap_or(-1);
                b_val.cmp(&a_val)
            });
            return results;
        }
    }

    let initial_query = build_search_string(track);
    let refinements = build_refinement_queries(track);

    let mut all_results: Vec<SearchResult> = Vec::new();
    let mut current_query = initial_query;

    for level in 0..=refinements.len() {
        if let Some(results) = searcher.search_for_results_str(&current_query).await {
            if !results.is_empty() {
                all_results.extend(results);
            }
        }

        if !full_search && !all_results.is_empty() {
            break;
        }

        if level < refinements.len() {
            let next_query = &refinements[level];
            if *next_query != current_query {
                current_query = next_query.clone();
            } else {
                break;
            }
        }
    }

    for result in &mut all_results {
        result.match_type = Some(compare_track_result(track, result));
    }

    all_results.sort_by(|a, b| {
        let a_val = a.match_type.map(|m| m as i32).unwrap_or(-1);
        let b_val = b.match_type.map(|m| m as i32).unwrap_or(-1);
        b_val.cmp(&a_val)
    });

    all_results
}

pub async fn search_for_best_result(
    searcher: &dyn Searcher,
    track: &TrackMetadata,
) -> Option<SearchResult> {
    let results = search_with_refinement(searcher, track, false).await;
    if !results.is_empty() {
        return Some(results.into_iter().next().unwrap());
    }

    let results = search_with_refinement(searcher, track, true).await;
    results.into_iter().next()
}

pub async fn search_for_best_result_with_match(
    searcher: &dyn Searcher,
    track: &TrackMetadata,
    minimum_match: MatchType,
) -> Option<SearchResult> {
    let results = search_with_refinement(searcher, track, false).await;
    if let Some(first) = results.first() {
        if first.match_type.map_or(false, |m| m >= minimum_match) {
            return Some(results.into_iter().next().unwrap());
        }
    }

    let results = search_with_refinement(searcher, track, true).await;
    if let Some(first) = results.first() {
        if first.match_type.map_or(false, |m| m >= minimum_match) {
            return Some(results.into_iter().next().unwrap());
        }
    }

    None
}
