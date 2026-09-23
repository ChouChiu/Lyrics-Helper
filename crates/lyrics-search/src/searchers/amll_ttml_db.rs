use async_trait::async_trait;

use super::Searchers;
use super::search_result::SearchResult;
use super::searcher::Searcher;
use crate::error::SearchError;
use crate::providers::web::amll_ttml_db::api;
use crate::providers::web::amll_ttml_db::response::LyricsEntry;
use lyrics_core::models::TrackMetadata;

/// AMLL TTML DB 歌词搜索器。
///
/// 结果的 `id` 是 `raw-lyrics/` 下的文件名，用
/// [`api::get_raw_lyrics`]
/// 取 TTML 歌词；`numeric_id` 是网易云歌曲 ID（有的话）。歌词库不记录时长。
pub struct AmllTtmlDbSearcher;

#[async_trait]
impl Searcher for AmllTtmlDbSearcher {
    fn name(&self) -> &str {
        "AmllTtmlDb"
    }

    fn display_name(&self) -> &str {
        "AMLL TTML DB"
    }

    fn searcher_type(&self) -> Searchers {
        Searchers::AmllTtmlDb
    }

    async fn search_for_results_str(
        &self,
        search_string: &str,
    ) -> Result<Vec<SearchResult>, SearchError> {
        let entries = api::search(search_string).await?;
        Ok(entries.into_iter().map(to_search_result).collect())
    }

    /// 只用曲名与艺术家搜索：歌词库的关键词须全部命中，各平台的专辑名写法不一，
    /// 带上专辑反而会漏掉结果。
    async fn search_for_results(
        &self,
        track: &TrackMetadata,
    ) -> Result<Vec<SearchResult>, SearchError> {
        let search_string = format!(
            "{} {}",
            track.title.as_deref().unwrap_or(""),
            track.artist.as_deref().unwrap_or("").replace(", ", " ")
        );
        self.search_for_results_str(&search_string).await
    }
}

fn to_search_result(entry: LyricsEntry) -> SearchResult {
    SearchResult {
        numeric_id: entry.ncm_music_ids.first().and_then(|id| id.parse().ok()),
        ..SearchResult::new(
            Searchers::AmllTtmlDb,
            entry.title().to_string(),
            entry.artists.clone(),
            entry.album().to_string(),
            None,
            entry.raw_lyric_file,
        )
    }
}
