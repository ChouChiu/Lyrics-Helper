use async_trait::async_trait;

use super::Searchers;
use super::search_result::SearchResult;
use super::searcher::Searcher;
use crate::error::SearchError;
use crate::providers::web::qq_music::api;

/// QQ 音乐歌词搜索器。
pub struct QQMusicSearcher;

#[async_trait]
impl Searcher for QQMusicSearcher {
    fn name(&self) -> &str {
        "QQMusic"
    }

    fn display_name(&self) -> &str {
        "QQ Music"
    }

    fn searcher_type(&self) -> Searchers {
        Searchers::QQMusic
    }

    async fn search_for_results_str(
        &self,
        search_string: &str,
    ) -> Result<Vec<SearchResult>, SearchError> {
        let response = api::search(search_string).await?;

        if let Some(code) = response.code {
            if code != 0 {
                return Err(SearchError::Api(format!("QQ 音乐搜索返回错误码 {code}")));
            }
        }

        let req = match response.request {
            Some(req) => req,
            // 缺少 request 只是「没有匹配」，不是错误。
            None => return Ok(Vec::new()),
        };

        if let Some(code) = req.code {
            if code != 0 {
                return Err(SearchError::Api(format!(
                    "QQ 音乐搜索请求返回错误码 {code}"
                )));
            }
        }

        // 缺少 data / body / item_song 都只是「没有匹配」，不是错误。
        let Some(songs) = req
            .data
            .and_then(|data| data.body)
            .and_then(|body| body.item_song)
        else {
            return Ok(Vec::new());
        };

        let search_results: Vec<SearchResult> = songs
            .into_iter()
            .map(|song| {
                let artists: Vec<String> = song
                    .singer
                    .unwrap_or_default()
                    .into_iter()
                    .map(|s| s.name)
                    .collect();

                SearchResult {
                    searcher_type: Searchers::QQMusic,
                    title: song.title,
                    artists,
                    album: song.album.map(|a| a.name).unwrap_or_default(),
                    album_artists: None,
                    duration_ms: song.interval.map(|i| i * 1000),
                    match_type: None,
                    id: song.mid,
                    numeric_id: song._id,
                }
            })
            .collect();

        Ok(search_results)
    }
}
