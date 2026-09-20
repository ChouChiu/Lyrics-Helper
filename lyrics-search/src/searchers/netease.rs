use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;

use super::Searchers;
use super::search_result::SearchResult;
use super::searcher::Searcher;
use crate::error::SearchError;
use crate::providers::web::netease::api;
use crate::providers::web::netease::response::SearchResponse;

/// 是否优先使用 eapi 搜索接口，对应 C# `NeteaseSearcher.useNewSearchFirst`。
///
/// 上游把该状态记在 `NeteaseApi` 单例上，因此跨请求生效；这里用进程级静态量保持相同语义：
/// 某个接口在当前网络环境下不可用时，后续请求直接改用另一个接口，避免每次都要先失败一次。
static USE_NEW_SEARCH_FIRST: AtomicBool = AtomicBool::new(false);

/// web 搜索接口被风控时的返回码，对应 C# 中的 `result?.Code == -460` 判断。
const RISK_CONTROL_CODE: i64 = -460;

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
        let response = search_response(search_string).await?;

        // 缺少 result / songs 都只是「没有匹配」，不是错误。
        let Some(songs) = response.result.and_then(|result| result.songs) else {
            return Ok(Vec::new());
        };

        let search_results: Vec<SearchResult> = songs
            .into_iter()
            .map(|song| {
                SearchResult::new(
                    Searchers::Netease,
                    song.name,
                    song.artists.into_iter().map(|artist| artist.name).collect(),
                    song.album.name,
                    Some(song.duration as i32),
                    song.id.to_string(),
                )
            })
            .collect();

        Ok(search_results)
    }
}

/// 依次尝试两个单曲搜索接口，对应 C# `NeteaseSearcher.SearchForResults` 的接口选择逻辑。
///
/// web 接口在海外 IP 下会返回加密结果、在风控网络环境下会返回 `-460`，两者都改用 eapi 接口兜底；
/// eapi 接口不可用时再改回 web 接口。两个接口都失败时返回第一次尝试的错误——真正的失败仍是 `Err`，
/// 「查无此歌」依旧由空列表表达。
async fn search_response(keyword: &str) -> Result<SearchResponse, SearchError> {
    let prefer_new = USE_NEW_SEARCH_FIRST.load(Ordering::Relaxed);

    let first_error = match search_once(keyword, prefer_new).await {
        Ok(response) => return Ok(response),
        Err(error) => error,
    };

    // 首选接口在当前网络环境下不可用，记住这件事并改用另一个接口。
    USE_NEW_SEARCH_FIRST.store(!prefer_new, Ordering::Relaxed);

    match search_once(keyword, !prefer_new).await {
        Ok(response) => Ok(response),
        Err(_) => {
            // 两个接口都不可用，恢复原来的偏好并返回首选接口的错误。
            USE_NEW_SEARCH_FIRST.store(prefer_new, Ordering::Relaxed);
            Err(first_error)
        }
    }
}

/// 按 `use_new` 选择 eapi 或 web 接口发起一次搜索。
async fn search_once(keyword: &str, use_new: bool) -> Result<SearchResponse, SearchError> {
    if use_new {
        api::search_new(keyword).await
    } else {
        search_web(keyword).await
    }
}

/// web 接口的风控返回码视为失败，以便切换到 eapi 接口（对应上游的 `throw new Exception()`）。
async fn search_web(keyword: &str) -> Result<SearchResponse, SearchError> {
    let response = api::search(keyword).await?;
    if response.code == Some(RISK_CONTROL_CODE) {
        return Err(SearchError::Api(format!(
            "网易云 web 搜索接口返回 {RISK_CONTROL_CODE}（网络环境被风控）"
        )));
    }
    Ok(response)
}
