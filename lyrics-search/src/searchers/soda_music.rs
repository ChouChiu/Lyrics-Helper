use async_trait::async_trait;

use super::Searchers;
use super::search_result::SearchResult;
use super::searcher::Searcher;
use crate::error::SearchError;
use crate::providers::web::soda_music::api;
use crate::providers::web::soda_music::response::SearchResponse;

/// 汽水音乐歌词搜索器。
pub struct SodaMusicSearcher;

#[async_trait]
impl Searcher for SodaMusicSearcher {
    fn name(&self) -> &str {
        "SodaMusic"
    }

    fn display_name(&self) -> &str {
        "Soda Music"
    }

    fn searcher_type(&self) -> Searchers {
        Searchers::SodaMusic
    }

    async fn search_for_results_str(
        &self,
        search_string: &str,
    ) -> Result<Vec<SearchResult>, SearchError> {
        let response = api::search(search_string).await?;

        // 缺少 result_groups 只是「没有匹配」，由 `map_results` 映射为空列表。
        Ok(map_results(response))
    }
}

/// 将搜索响应中的全部结果分组映射为搜索结果（仅保留 track 类型条目）。
fn map_results(response: SearchResponse) -> Vec<SearchResult> {
    response
        .result_groups
        .unwrap_or_default()
        .into_iter()
        .filter_map(|group| group.data)
        .flatten()
        .filter_map(|item| {
            if item.meta.as_ref().and_then(|m| m.item_type.as_deref()) != Some("track") {
                return None;
            }
            let track = item.entity?.track?;

            Some(SearchResult::new(
                Searchers::SodaMusic,
                track.name,
                track
                    .artists
                    .unwrap_or_default()
                    .into_iter()
                    .map(|artist| artist.name)
                    .collect(),
                track.album.map(|album| album.name).unwrap_or_default(),
                track.duration.map(|duration| duration as i32),
                track.id,
            ))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_results_flattens_all_groups_and_skips_non_tracks() {
        let response: SearchResponse = serde_json::from_str(
            r#"{
                "status_code": 0,
                "result_groups": [
                    {
                        "data": [
                            {
                                "meta": { "item_type": "track" },
                                "entity": {
                                    "track": {
                                        "id": "1",
                                        "name": "First",
                                        "duration": 180000,
                                        "artists": [ { "name": "A" } ],
                                        "album": { "name": "Album" }
                                    }
                                }
                            },
                            { "meta": { "item_type": "album" }, "entity": { "track": { "id": "x", "name": "Ignored" } } }
                        ]
                    },
                    { "data": null },
                    {
                        "data": [
                            {
                                "meta": { "item_type": "track" },
                                "entity": {
                                    "track": { "id": "2", "name": "Second", "artists": [ { "name": "B" }, { "name": "C" } ] }
                                }
                            },
                            { "meta": { "item_type": "track" }, "entity": {} }
                        ]
                    }
                ]
            }"#,
        )
        .unwrap();

        let results = map_results(response);

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "First");
        assert_eq!(results[0].artists, vec!["A".to_string()]);
        assert_eq!(results[0].album, "Album");
        assert_eq!(results[0].duration_ms, Some(180000));
        assert_eq!(results[0].id, "1");
        assert_eq!(results[1].title, "Second");
        assert_eq!(results[1].artists, vec!["B".to_string(), "C".to_string()]);
        assert_eq!(results[1].id, "2");
    }
}
