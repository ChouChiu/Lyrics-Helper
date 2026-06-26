use super::response::SearchResponse;
use crate::providers::web::base_api;

pub(crate) async fn search(keyword: &str) -> Option<SearchResponse> {
    let encoded = urlencoding::encode(keyword);
    let url = format!(
        "https://api.qishui.com/luna/pc/search/track?aid=386088&app_name=&region=&geo_region=&os_region=&sim_region=&device_id=&cdid=&iid=&version_name=&version_code=&channel=&build_mode=&network_carrier=&ac=&tz_name=&resolution=&device_platform=&device_type=&os_version=&fp=&q={}&cursor=&search_id=&search_method=input&debug_params=&from_search_id=&search_scene=",
        encoded
    );
    let headers = [
        ("Referer", "https://api.qishui.com/"),
        ("User-Agent", "LunaPC/2.6.5(197449790)"),
    ];
    base_api::get_json_with_headers(&url, &headers).await
}
