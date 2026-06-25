use super::response::SearchResponse;
use crate::providers::web::base_api;

pub async fn search(
    keyword: &str,
    access_token: &str,
    storefront: &str,
    language: &str,
) -> Option<SearchResponse> {
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
    base_api::get_json_with_headers(&url, &header_refs).await
}
