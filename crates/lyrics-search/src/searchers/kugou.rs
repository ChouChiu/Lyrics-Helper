use async_trait::async_trait;

use super::Searchers;
use super::search_result::SearchResult;
use super::searcher::Searcher;
use crate::error::SearchError;
use crate::providers::web::kugou::api;

use super::search_result::split_artists;

/// 酷狗音乐歌词搜索器。
pub struct KugouSearcher;

#[async_trait]
impl Searcher for KugouSearcher {
    fn name(&self) -> &str {
        "Kugou"
    }

    fn display_name(&self) -> &str {
        "Kugou Music"
    }

    fn searcher_type(&self) -> Searchers {
        Searchers::Kugou
    }

    async fn search_for_results_str(
        &self,
        search_string: &str,
    ) -> Result<Vec<SearchResult>, SearchError> {
        let response = api::search(search_string).await?;

        // 缺少 data / info 都只是「没有匹配」，不是错误。
        let Some(songs) = response.data.and_then(|data| data.info) else {
            return Ok(Vec::new());
        };

        let search_results: Vec<SearchResult> = songs
            .into_iter()
            .map(|song| {
                SearchResult::new(
                    Searchers::Kugou,
                    song.songname,
                    split_artists(&song.singername),
                    song.album_name.unwrap_or_default(),
                    song.duration.map(|duration| duration * 1000),
                    song.hash,
                )
            })
            .collect();

        Ok(search_results)
    }
}
