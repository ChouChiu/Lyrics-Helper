//! 搜索查询串的构造与渐进式搜索：先用完整元数据搜，没有结果再逐步放宽查询。

use crate::error::SearchError;
use lyrics_core::models::TrackMetadata;

use super::compare_helper::{MatchType, rank_by_match};
use super::search_result::SearchResult;
use super::searcher::Searcher;

/// 把若干片段拼成搜索查询串：去掉分隔用的 ` - `，压掉多余空白。
fn join_query(parts: [&str; 3]) -> String {
    parts
        .join(" ")
        .replace(" - ", " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// 根据曲目元数据构建搜索查询字符串（标题 + 艺术家 + 专辑）。
pub fn build_search_string(track: &TrackMetadata) -> String {
    join_query([
        track.title.as_deref().unwrap_or(""),
        &track.artist.as_deref().unwrap_or("").replace(", ", " "),
        track.album.as_deref().unwrap_or(""),
    ])
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
    let title = strip_feat(track.title.as_deref().unwrap_or(""));
    let artist = track.artist.as_deref().unwrap_or("").replace(", ", " ");

    vec![
        join_query([&title, &artist, ""]),
        join_query([&title, "", ""]),
    ]
}

/// 渐进式搜索依次发出的查询，从精确到宽泛。
///
/// 与前面某条相同的查询跳过而不是就此停下：没有专辑时完整查询与「曲名 + 歌手」相同，
/// 但更宽的「只用曲名」仍要试。
fn search_queries(track: &TrackMetadata) -> Vec<String> {
    let mut queries = vec![build_search_string(track)];
    for query in build_refinement_queries(track) {
        if !queries.contains(&query) {
            queries.push(query);
        }
    }
    queries
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
        Ok(mut results) if !results.is_empty() => {
            rank_by_match(&mut results, track);
            return Ok(results);
        }
        Ok(_) => {}
        Err(error) => last_error = Some(error),
    }

    let mut all_results: Vec<SearchResult> = Vec::new();
    for query in &search_queries(track) {
        match searcher.search_for_results_str(query).await {
            Ok(results) => all_results.extend(results),
            Err(error) => last_error = Some(error),
        }

        if !full_search && !all_results.is_empty() {
            break;
        }
    }

    // 没有任何结果时才把失败当成失败；拿到结果说明这次搜索是成功的。
    if all_results.is_empty()
        && let Some(error) = last_error
    {
        return Err(error);
    }

    rank_by_match(&mut all_results, track);

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

#[cfg(test)]
mod tests {
    use super::*;

    fn track(title: &str, artist: &str, album: Option<&str>) -> TrackMetadata {
        TrackMetadata {
            title: Some(title.to_string()),
            artist: Some(artist.to_string()),
            album: album.map(str::to_string),
            ..TrackMetadata::new()
        }
    }

    /// 没有专辑时完整查询与「曲名 + 歌手」相同，只用曲名的那一层仍要搜。
    #[test]
    fn falls_back_to_the_title_without_an_album() {
        assert_eq!(
            search_queries(&track("晴天", "周杰伦", None)),
            vec!["晴天 周杰伦".to_string(), "晴天".to_string()]
        );
    }

    #[test]
    fn goes_from_exact_to_broad() {
        assert_eq!(
            search_queries(&track("Idol (feat. X)", "YOASOBI", Some("Idol"))),
            vec![
                "Idol (feat. X) YOASOBI Idol".to_string(),
                "Idol YOASOBI".to_string(),
                "Idol".to_string(),
            ]
        );
    }
}
