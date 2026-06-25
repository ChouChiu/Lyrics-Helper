use super::response::SearchResponse;
use crate::providers::web::base_api;

pub async fn search(keyword: &str) -> Option<SearchResponse> {
    let url = format!(
        "http://mobilecdn.kugou.com/api/v3/search/song?format=json&keyword={}&page=1&pagesize=20&showtype=1",
        urlencoding::encode(keyword)
    );
    base_api::get_json(&url).await
}
