use async_trait::async_trait;

use super::Searchers;
use super::search_result::SearchResult;
use super::searcher::Searcher;
use crate::error::SearchError;
use crate::providers::web::kugou::api;

/// 酷狗音乐歌词搜索器。
pub struct KugouSearcher;

/// 将艺术家字符串按逗号、顿号或斜杠拆分为列表。
fn split_artists(singername: &str) -> Vec<String> {
    singername
        .split([',', '、', '/'])
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

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
                let artists = split_artists(&song.singername);

                SearchResult {
                    searcher_type: Searchers::Kugou,
                    title: song.songname,
                    artists,
                    album: song.album_name.unwrap_or_default(),
                    album_artists: None,
                    duration_ms: song.duration.map(|d| d * 1000),
                    match_type: None,
                    id: song.hash,
                    numeric_id: None,
                }
            })
            .collect();

        Ok(search_results)
    }
}
