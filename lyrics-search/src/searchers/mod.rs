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
use crate::error::SearchError;
use lyrics_core::models::TrackMetadata;
use search_result::SearchResult;
use searcher::Searcher;

/// 支持的歌词搜索平台枚举。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Searchers {
    /// QQ 音乐
    QQMusic,
    /// 网易云音乐
    Netease,
    /// 酷狗音乐
    Kugou,
    /// Musixmatch
    Musixmatch,
    /// 汽水音乐
    SodaMusic,
    /// Apple Music
    AppleMusic,
    /// Spotify
    Spotify,
    /// LRCLIB
    LRCLIB,
}

/// 比较目标曲目元数据与搜索结果的匹配程度，返回匹配等级。
/// 曲名权重。
const WEIGHT_TITLE: f64 = 1.0;
/// 歌手权重。
const WEIGHT_ARTIST: f64 = 1.0;
/// 专辑名权重。
const WEIGHT_ALBUM: f64 = 0.4;
/// 专辑歌手权重。
const WEIGHT_ALBUM_ARTIST: f64 = 0.2;
/// 时长权重。
const WEIGHT_DURATION: f64 = 1.0;
/// 单个字段的满分，与 `compare_helper` 中各 `score()` 的 `Perfect` 取值一致。
const FIELD_MAX_SCORE: f64 = 7.0;

pub fn compare_track(
    track: &TrackMetadata,
    result_title: Option<&str>,
    result_artists: &[String],
    result_album: Option<&str>,
    result_album_artists: &[String],
    result_duration_ms: Option<i32>,
) -> MatchType {
    let track_match = compare_name(track.title.as_deref(), result_title);
    let artist_match = compare_artist(track.artists.as_deref().unwrap_or(&[]), result_artists);
    let album_match = compare_name(track.album.as_deref(), result_album);
    let album_artist_match = compare_artist(
        track.album_artists.as_deref().unwrap_or(&[]),
        result_album_artists,
    );
    let duration_match = compare_duration(track.duration_ms, result_duration_ms);

    let mut total_score = 0.0f64;
    total_score += name_score(track_match) * WEIGHT_TITLE;
    total_score += artist_score(artist_match) * WEIGHT_ARTIST;
    total_score += name_score(album_match) * WEIGHT_ALBUM;
    total_score += artist_score(album_artist_match) * WEIGHT_ALBUM_ARTIST;
    total_score += duration_score(duration_match) * WEIGHT_DURATION;

    // 缺失的可选信息不参与评分，按可比较的字段重新分配权重
    let full_score =
        (WEIGHT_TITLE + WEIGHT_ARTIST + WEIGHT_ALBUM + WEIGHT_ALBUM_ARTIST + WEIGHT_DURATION)
            * FIELD_MAX_SCORE;
    let mut available_score = (WEIGHT_TITLE + WEIGHT_ARTIST) * FIELD_MAX_SCORE;
    if album_match.is_some() {
        available_score += WEIGHT_ALBUM * FIELD_MAX_SCORE;
    }
    if album_artist_match.is_some() {
        available_score += WEIGHT_ALBUM_ARTIST * FIELD_MAX_SCORE;
    }
    if duration_match.is_some() {
        available_score += WEIGHT_DURATION * FIELD_MAX_SCORE;
    }
    total_score *= full_score / available_score;

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

/// 比较目标曲目元数据与单条搜索结果的匹配程度，返回匹配等级。
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

/// 根据曲目元数据构建搜索查询字符串（标题 + 艺术家 + 专辑）。
pub fn build_search_string(track: &TrackMetadata) -> String {
    let title = track.title.as_deref().unwrap_or("");
    let artist = track.artist.as_deref().unwrap_or("").replace(", ", " ");
    let album = track.album.as_deref().unwrap_or("");
    format!("{} {} {}", title, artist, album)
        .replace(" - ", " ")
        .trim()
        .to_string()
}

/// 移除标题中的 featuring 标记（如 `(feat. xxx)` 或 ` - feat. xxx`）。
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

/// 根据曲目元数据构建渐进式搜索查询列表（从精确到宽泛）。
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

/// 使用渐进式搜索策略搜索歌词，先尝试精确匹配，失败后逐步放宽搜索条件。
///
/// 返回按匹配度排序的搜索结果列表。任何一层拿到结果即返回 `Ok(结果)`；
/// 全部层都没有结果、且至少有一层请求失败时返回 `Err`（最后一次错误）；
/// 没有结果也没有失败时返回 `Ok(vec![])`。
pub async fn search_with_refinement(
    searcher: &dyn Searcher,
    track: &TrackMetadata,
    full_search: bool,
) -> Result<Vec<SearchResult>, SearchError> {
    let mut last_error: Option<SearchError> = None;

    match searcher.search_for_results(track).await {
        Ok(results) if !results.is_empty() => {
            let mut results = results;
            for result in &mut results {
                result.match_type = Some(compare_track_result(track, result));
            }
            results.sort_by(|a, b| {
                let a_val = a.match_type.map(|m| m as i32).unwrap_or(-1);
                let b_val = b.match_type.map(|m| m as i32).unwrap_or(-1);
                b_val.cmp(&a_val)
            });
            return Ok(results);
        }
        Ok(_) => {}
        Err(error) => last_error = Some(error),
    }

    let initial_query = build_search_string(track);
    let refinements = build_refinement_queries(track);

    let mut all_results: Vec<SearchResult> = Vec::new();
    let mut current_query = initial_query;

    for level in 0..=refinements.len() {
        match searcher.search_for_results_str(&current_query).await {
            Ok(results) => all_results.extend(results),
            Err(error) => last_error = Some(error),
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

    // 没有任何结果时才把失败当成失败；拿到结果说明这次搜索是成功的。
    if all_results.is_empty() {
        if let Some(error) = last_error {
            return Err(error);
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

    Ok(all_results)
}

/// 搜索并返回匹配度最高的单条搜索结果。
///
/// 先做一轮不完整搜索，再做一轮完整搜索（两段式与旧行为一致）。
/// 任何一轮拿到结果即返回 `Ok(Some(第一个))`；两轮都没有结果且至少有一轮失败时
/// 返回 `Err`（优先返回第一轮的错误）；两轮都成功但没有结果时返回 `Ok(None)`。
pub async fn search_for_best_result(
    searcher: &dyn Searcher,
    track: &TrackMetadata,
) -> Result<Option<SearchResult>, SearchError> {
    let first_error = match search_with_refinement(searcher, track, false).await {
        Ok(results) => {
            if let Some(first) = results.into_iter().next() {
                return Ok(Some(first));
            }
            None
        }
        Err(error) => Some(error),
    };

    match search_with_refinement(searcher, track, true).await {
        Ok(results) => Ok(results.into_iter().next()),
        Err(error) => Err(first_error.unwrap_or(error)),
    }
}

/// 搜索并返回匹配度不低于指定等级的单条搜索结果。
///
/// 与 [`search_for_best_result`] 相同的两段式搜索，但每轮只有第一条结果达到
/// `minimum_match` 时才返回该轮结果。两轮都被阈值挡下且至少有一轮失败时返回
/// `Err`（优先返回第一轮的错误）；两轮都成功但没有合格结果时返回 `Ok(None)`。
pub async fn search_for_best_result_with_match(
    searcher: &dyn Searcher,
    track: &TrackMetadata,
    minimum_match: MatchType,
) -> Result<Option<SearchResult>, SearchError> {
    let mut first_error: Option<SearchError> = None;

    for full_search in [false, true] {
        match search_with_refinement(searcher, track, full_search).await {
            Ok(results) => {
                let matched = results
                    .first()
                    .is_some_and(|first| first.match_type.is_some_and(|m| m >= minimum_match));
                if matched {
                    return Ok(results.into_iter().next());
                }
            }
            Err(error) => {
                if first_error.is_none() {
                    first_error = Some(error);
                }
            }
        }
    }

    match first_error {
        Some(error) => Err(error),
        None => Ok(None),
    }
}
