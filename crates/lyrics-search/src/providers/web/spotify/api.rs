use reqwest::Method;

use super::response::SearchResponse;
use crate::error::SearchError;
use crate::providers::web::base_api;

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36";

/// 通过 Spotify Web API 搜索曲目。
///
/// 网络、HTTP 状态码或响应解码失败时返回 [`SearchError`]；
/// 「搜索成功但没有匹配曲目」由响应内部的空 `tracks.items` 表示，不是错误。
pub(crate) async fn search(
    keyword: &str,
    access_token: &str,
) -> Result<SearchResponse, SearchError> {
    let url = format!(
        "https://api.spotify.com/v1/search?q={}&type=track&limit=10&market=from_token",
        urlencoding::encode(keyword),
    );
    let authorization = format!("Bearer {access_token}");
    let headers = [
        ("Authorization", authorization.as_str()),
        ("User-Agent", USER_AGENT),
    ];
    let response = base_api::send(Method::GET, &url, &headers).await?;
    base_api::json(response).await
}
