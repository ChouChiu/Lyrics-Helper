use async_trait::async_trait;

use super::Searchers;
use super::search_result::SearchResult;
use super::searcher::Searcher;
use crate::error::SearchError;
use crate::providers::web::apple_music::api;

/// Apple Music 歌词搜索器。
pub struct AppleMusicSearcher {
    access_token: String,
    storefront: String,
    language: String,
}

impl AppleMusicSearcher {
    /// 创建新的 Apple Music 搜索器，默认使用美国 storefront 和英语。
    pub fn new(access_token: String) -> Self {
        Self {
            access_token,
            storefront: "us".to_string(),
            language: "en-US".to_string(),
        }
    }
}

#[async_trait]
impl Searcher for AppleMusicSearcher {
    fn name(&self) -> &str {
        "AppleMusic"
    }

    fn display_name(&self) -> &str {
        "Apple Music"
    }

    fn searcher_type(&self) -> Searchers {
        Searchers::AppleMusic
    }

    async fn search_for_results_str(
        &self,
        search_string: &str,
    ) -> Result<Vec<SearchResult>, SearchError> {
        let response = api::search(
            search_string,
            &self.access_token,
            &self.storefront,
            &self.language,
        )
        .await?;

        // 缺少 results / songs / data 都只是「没有匹配」，不是错误。
        let Some(songs) = response
            .results
            .and_then(|results| results.songs)
            .and_then(|songs| songs.data)
        else {
            return Ok(Vec::new());
        };

        let search_results: Vec<SearchResult> = songs
            .into_iter()
            .filter_map(|song| {
                let attrs = song.attributes?;
                Some(SearchResult::new(
                    Searchers::AppleMusic,
                    attrs.name,
                    vec![attrs.artist_name],
                    attrs.album_name,
                    attrs.duration_in_millis,
                    song.id,
                ))
            })
            .collect();

        Ok(search_results)
    }
}
