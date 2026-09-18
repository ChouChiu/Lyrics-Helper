use async_trait::async_trait;

use super::Searchers;
use super::search_result::SearchResult;
use super::searcher::Searcher;
use crate::error::SearchError;
use crate::providers::web::netease::api;

/// 网易云音乐歌词搜索器。
pub struct NeteaseSearcher;

#[async_trait]
impl Searcher for NeteaseSearcher {
    fn name(&self) -> &str {
        "Netease"
    }

    fn display_name(&self) -> &str {
        "Netease Cloud Music"
    }

    fn searcher_type(&self) -> Searchers {
        Searchers::Netease
    }

    async fn search_for_results_str(
        &self,
        search_string: &str,
    ) -> Result<Vec<SearchResult>, SearchError> {
        let response = api::search(search_string).await?;

        // 缺少 result / songs 都只是「没有匹配」，不是错误。
        let Some(songs) = response.result.and_then(|result| result.songs) else {
            return Ok(Vec::new());
        };

        let search_results: Vec<SearchResult> = songs
            .into_iter()
            .map(|song| {
                let artists: Vec<String> = song.artists.iter().map(|a| a.name.clone()).collect();

                SearchResult {
                    searcher_type: Searchers::Netease,
                    title: song.name,
                    artists,
                    album: song.album.name,
                    album_artists: None,
                    duration_ms: Some(song.duration as i32),
                    match_type: None,
                    id: song.id.to_string(),
                    numeric_id: None,
                }
            })
            .collect();

        Ok(search_results)
    }
}
