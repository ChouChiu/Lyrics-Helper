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

        ensure_ok(response.code, "搜索")?;

        // 缺少 request 只是「没有匹配」，不是错误。
        let Some(req) = response.request else {
            return Ok(Vec::new());
        };
        ensure_ok(req.code, "搜索请求")?;

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
            .map(|song| SearchResult {
                numeric_id: song.numeric_id,
                ..SearchResult::new(
                    Searchers::QQMusic,
                    song.title,
                    song.singer
                        .unwrap_or_default()
                        .into_iter()
                        .map(|singer| singer.name)
                        .collect(),
                    song.album.map(|album| album.name).unwrap_or_default(),
                    song.interval.map(|interval| interval * 1000),
                    song.mid,
                )
            })
            .collect();

        Ok(search_results)
    }
}

/// 业务码非 0 即平台侧失败；缺失时按上游 C# 的 `int` 默认值视作 0。
fn ensure_ok(code: Option<i32>, scope: &str) -> Result<(), SearchError> {
    match code {
        None | Some(0) => Ok(()),
        Some(code) => Err(SearchError::Api(format!("QQ 音乐{scope}返回错误码 {code}"))),
    }
}
