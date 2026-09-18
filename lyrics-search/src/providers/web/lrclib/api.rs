use reqwest::{Method, StatusCode};

use super::response::{GetLyricResult, SearchResultItem};
use crate::error::SearchError;
use crate::providers::web::base_api;

const BASE_URL: &str = "https://lrclib.net/api";
const USER_AGENT: &str = "Lyrics-Helper (https://github.com/WXRIW/Lyricify-Lyrics-Helper)";

/// 在 LRCLIB 中搜索歌词，可按艺术家、专辑和时长过滤。
///
/// 网络、HTTP 状态码或响应解码失败时返回 [`SearchError`]；
/// 「搜索成功但没有匹配结果」由空 `Vec` 表示，不是错误。
pub async fn search(
    track_name: &str,
    artist_name: Option<&str>,
    album_name: Option<&str>,
    duration: Option<f64>,
) -> Result<Vec<SearchResultItem>, SearchError> {
    let mut url = format!(
        "{}/search?track_name={}",
        BASE_URL,
        urlencoding::encode(track_name)
    );

    if let Some(artist) = artist_name {
        url.push_str(&format!("&artist_name={}", urlencoding::encode(artist)));
    }

    if let Some(album) = album_name {
        url.push_str(&format!("&album_name={}", urlencoding::encode(album)));
    }

    if let Some(dur) = duration {
        url.push_str(&format!("&duration={}", dur));
    }

    let response = base_api::send(Method::GET, &url, &[("User-Agent", USER_AGENT)]).await?;
    base_api::json(response).await
}

/// 精确获取 LRCLIB 歌词（按曲名、艺术家、专辑和时长匹配）。
///
/// 返回 `Ok(None)` 表示 LRCLIB 没有这首歌（HTTP 404）；
/// 网络、其他 HTTP 状态码或响应解码失败返回 [`SearchError`]。
pub async fn get(
    track_name: &str,
    artist_name: &str,
    album_name: Option<&str>,
    duration: Option<f64>,
) -> Result<Option<GetLyricResult>, SearchError> {
    let mut url = format!(
        "{}/get?track_name={}&artist_name={}",
        BASE_URL,
        urlencoding::encode(track_name),
        urlencoding::encode(artist_name)
    );

    if let Some(album) = album_name {
        url.push_str(&format!("&album_name={}", urlencoding::encode(album)));
    }

    if let Some(dur) = duration {
        url.push_str(&format!("&duration={}", dur));
    }

    let response = base_api::send(Method::GET, &url, &[("User-Agent", USER_AGENT)]).await?;
    if response.status() == StatusCode::NOT_FOUND {
        return Ok(None);
    }
    let lyric: GetLyricResult = base_api::json(response).await?;
    Ok(Some(lyric))
}

/// 通过 ID 直接获取 LRCLIB 歌词。
///
/// 返回 `Ok(None)` 表示该 ID 不存在（HTTP 404）；
/// 网络、其他 HTTP 状态码或响应解码失败返回 [`SearchError`]。
pub async fn get_by_id(id: i32) -> Result<Option<GetLyricResult>, SearchError> {
    let url = format!("{}/get/{}", BASE_URL, id);
    let response = base_api::send(Method::GET, &url, &[("User-Agent", USER_AGENT)]).await?;
    if response.status() == StatusCode::NOT_FOUND {
        return Ok(None);
    }
    let lyric: GetLyricResult = base_api::json(response).await?;
    Ok(Some(lyric))
}
