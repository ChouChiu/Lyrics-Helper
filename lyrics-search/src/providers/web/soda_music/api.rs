use reqwest::Method;

use super::response::{SearchResponse, TrackDetailResponse};
use crate::error::SearchError;
use crate::providers::web::base_api;
use rand::Rng;
use std::sync::LazyLock;

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

/// 获取汽水音乐歌词，返回 `(原文歌词, 翻译歌词)`。
///
/// 请求成功但该曲目没有歌词（缺少 `lyric` 字段，或正文/翻译为空）时对应元素为 `None`；
/// 网络、HTTP 状态码或响应解码失败返回 [`SearchError`]。
pub async fn get_lyrics(track_id: &str) -> Result<(Option<String>, Option<String>), SearchError> {
    let query = [
        ("track_id", track_id.to_string()),
        ("device_platform", "web".to_string()),
    ];
    let url = build_url(H5_BASE, "seo_track", &query);
    let headers = [
        ("Accept", "application/json"),
        ("User-Agent", WEB_USER_AGENT),
    ];

    let response = base_api::send(Method::GET, &url, &headers).await?;
    let detail: TrackDetailResponse = base_api::json(response).await?;

    // 歌词可能在顶层，也可能只在 `seo_track` 下。
    let Some(lyric) = detail
        .lyric
        .or_else(|| detail.seo_track.and_then(|seo_track| seo_track.lyric))
    else {
        return Ok((None, None));
    };

    Ok((
        lyric.content.filter(|content| !content.is_empty()),
        lyric
            .translations
            .and_then(|translations| translations.cn)
            .filter(|content| !content.is_empty()),
    ))
}

/// 搜索汽水音乐曲目。
///
/// 网络、HTTP 状态码或响应解码失败时返回 [`SearchError`]；
/// 「搜索成功但没有匹配曲目」由响应内部的空结果表示，不是错误。
pub(crate) async fn search(keyword: &str) -> Result<SearchResponse, SearchError> {
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
    let response = base_api::send(Method::GET, &url, &headers).await?;
    base_api::json(response).await
}
