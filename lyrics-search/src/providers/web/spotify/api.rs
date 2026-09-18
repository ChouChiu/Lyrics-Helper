use super::response::SearchResponse;
use crate::providers::web::base_api;

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36";

pub(crate) async fn search(keyword: &str, access_token: &str) -> Option<SearchResponse> {
    let url = format!(
        "https://api.spotify.com/v1/search?q={}&type=track&limit=10&market=from_token",
        urlencoding::encode(keyword),
    );
    let headers = [
        ("Authorization", format!("Bearer {}", access_token)),
        ("User-Agent", USER_AGENT.to_string()),
    ];
    let header_refs: Vec<(&str, &str)> = headers.iter().map(|(k, v)| (*k, v.as_str())).collect();
    base_api::get_json_with_headers(&url, &header_refs).await
}
