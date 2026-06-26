use super::response::SearchResponse;
use crate::providers::web::base_api;

const REFERER: &str = "https://music.163.com/";
const COOKIE: &str = "os=pc;appver=2.9.7;channel=netease;";
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

fn standard_headers() -> Vec<(&'static str, &'static str)> {
    vec![
        ("Referer", REFERER),
        ("Cookie", COOKIE),
        ("User-Agent", USER_AGENT),
    ]
}

pub(crate) async fn search(keyword: &str) -> Option<SearchResponse> {
    let url = format!(
        "https://music.163.com/api/search/get?s={}&type=1&limit=10&offset=0",
        urlencoding::encode(keyword)
    );
    base_api::get_json_with_headers(&url, &standard_headers()).await
}
