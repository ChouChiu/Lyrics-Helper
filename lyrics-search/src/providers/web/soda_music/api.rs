use super::response::{SearchResponse, TrackDetailResponse};
use crate::providers::web::base_api;
use rand::Rng;
use std::sync::LazyLock;

/// PC/H5 接口使用的默认 User-Agent（对应 C# `Api.UserAgent`）。
pub const USER_AGENT: &str = "LunaPC/2.1.0(12292405)";

/// 搜索接口使用的 Android User-Agent。
const SEARCH_USER_AGENT: &str = "com.luna.music/100198030 (Linux; U; Android 15; zh_CN_#Hans; ABR-AL80; Build/V417IR;tt-ok/3.12.13.19)";

/// H5 接口使用的浏览器 User-Agent。
const WEB_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36";

static SEARCH_DEVICE_ID: LazyLock<String> = LazyLock::new(generate_client_id);
static SEARCH_INSTALL_ID: LazyLock<String> = LazyLock::new(generate_client_id);

fn generate_client_id() -> String {
    let mut rng = rand::rng();
    let a = rng.random_range(10_000_000..99_999_999);
    let b = rng.random_range(10_000_000..99_999_999);
    format!("{}{}", a, b)
}

/// 汽水音乐开放接口地址前缀。
const API_BASE: &str = "https://api.qishui.com/luna/";

/// 汽水音乐 H5 接口地址前缀。
const H5_BASE: &str = "https://beta-luna.douyin.com/luna/h5/";

/// 以 `base` 为前缀拼接路径与查询串。
fn build_url(base: &str, path: &str, query: &[(&str, String)]) -> String {
    let qs: Vec<String> = query
        .iter()
        .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
        .collect();
    format!("{}{}?{}", base, path, qs.join("&"))
}

/// 获取汽水音乐歌曲详情（H5 `seo_track` 接口）。
pub async fn get_detail(track_id: &str) -> Option<TrackDetailResponse> {
    let query = [
        ("track_id", track_id.to_string()),
        ("device_platform", "web".to_string()),
    ];
    let url = build_url(H5_BASE, "seo_track", &query);
    let headers = [
        ("Accept", "application/json"),
        ("User-Agent", WEB_USER_AGENT),
    ];

    let mut result: TrackDetailResponse = base_api::get_json_with_headers(&url, &headers).await?;

    if let Some(seo_track) = result.seo_track.as_ref() {
        if result.track.is_none() {
            result.track = seo_track.track.clone();
        }
        if result.track_player.is_none() {
            result.track_player = seo_track.track_player.clone();
        }
    }

    Some(result)
}

/// 获取汽水音乐歌词，返回 `(原文歌词, 翻译歌词)`。
pub async fn get_lyrics(track_id: &str) -> Option<(Option<String>, Option<String>)> {
    let detail = get_detail(track_id).await?;
    let lyric = detail.lyric?;
    let original = lyric.content.filter(|c| !c.is_empty());
    let translation = lyric
        .translations
        .and_then(|t| t.cn)
        .filter(|c| !c.is_empty());
    Some((original, translation))
}

pub(crate) async fn search(keyword: &str) -> Option<SearchResponse> {
    let query = [
        ("device_platform", "android".to_string()),
        ("os", "android".to_string()),
        ("ssmix", "a".to_string()),
        ("cdid", "46556f98-1720-4248-83da-62b74b60b46a".to_string()),
        ("channel", "xiaomi_8478_64".to_string()),
        ("aid", "386088".to_string()),
        ("app_name", "luna".to_string()),
        ("version_code", "100198030".to_string()),
        ("version_name", "19.8.0".to_string()),
        ("manifest_version_code", "100198030".to_string()),
        ("update_version_code", "100198030".to_string()),
        ("resolution", "1080*1920".to_string()),
        ("dpi", "480".to_string()),
        ("device_type", "ABR-AL80".to_string()),
        ("device_brand", "HUAWEI".to_string()),
        ("language", "zh".to_string()),
        ("os_api", "35".to_string()),
        ("os_version", "15".to_string()),
        ("ac", "wifi".to_string()),
        ("device_model", "ABR-AL80".to_string()),
        ("tz_name", "Asia/Shanghai".to_string()),
        ("tz_offset", "28800".to_string()),
        ("package", "com.luna.music".to_string()),
        ("sim_region", "cn".to_string()),
        ("iid", SEARCH_INSTALL_ID.clone()),
        ("device_id", SEARCH_DEVICE_ID.clone()),
        ("_rticket", base_api::unix_millis().to_string()),
        ("q", keyword.to_string()),
        ("cursor", "0".to_string()),
        ("count", "20".to_string()),
    ];

    let url = build_url(API_BASE, "search/track", &query);
    let headers = [("Accept", "*/*"), ("User-Agent", SEARCH_USER_AGENT)];
    base_api::get_json_with_headers(&url, &headers).await
}
