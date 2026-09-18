use reqwest::Method;

use super::response::SearchResponse;
use crate::error::SearchError;
use crate::providers::web::base_api;

/// 通过 Apple Music `amp-api` 搜索歌曲。
///
/// 网络、HTTP 状态码或响应解码失败时返回 [`SearchError`]；
/// 「搜索成功但没有匹配歌曲」由响应内部的空 `results` 表示，不是错误。
pub(crate) async fn search(
    keyword: &str,
    access_token: &str,
    storefront: &str,
    language: &str,
) -> Result<SearchResponse, SearchError> {
    let url = format!(
        "https://amp-api.music.apple.com/v1/catalog/{}/search?term={}&types=songs&limit=10&l={}",
        storefront,
        urlencoding::encode(keyword),
        language,
    );
    let headers = [
        ("Authorization", format!("Bearer {}", access_token)),
        ("Origin", "https://music.apple.com".to_string()),
        ("Referer", "https://music.apple.com/".to_string()),
        ("Accept", "application/json".to_string()),
    ];
    let header_refs: Vec<(&str, &str)> = headers.iter().map(|(k, v)| (*k, v.as_str())).collect();
    let response = base_api::send(Method::GET, &url, &header_refs).await?;
    base_api::json(response).await
}
