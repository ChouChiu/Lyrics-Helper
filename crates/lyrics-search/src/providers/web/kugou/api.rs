use reqwest::Method;

use super::response::{LyricDownloadResponse, LyricSearchResponse, SearchResponse};
use crate::error::SearchError;
use crate::providers::web::base_api;

/// 搜索酷狗歌曲。
///
/// 网络、HTTP 状态码或响应解码失败时返回 [`SearchError`]；
/// 「搜索成功但没有匹配歌曲」由响应内部的空 `data.info` 表示，不是错误。
pub(crate) async fn search(keyword: &str) -> Result<SearchResponse, SearchError> {
    let url = format!(
        "http://mobilecdn.kugou.com/api/v3/search/song?format=json&keyword={}&page=1&pagesize=20&showtype=1",
        urlencoding::encode(keyword)
    );
    let response = base_api::send(Method::GET, &url, &[]).await?;
    base_api::json(response).await
}

/// 通过关键词、哈希值和时长搜索并获取酷狗歌词内容。
///
/// 返回 `Ok(None)` 表示请求成功但没有匹配到歌词（没有候选、下载响应缺少 `content`
/// 字段）；base64 或 UTF-8 解码失败返回 [`SearchError::Payload`]。
pub async fn get_lyrics(
    keyword: &str,
    hash: &str,
    duration_ms: i32,
) -> Result<Option<String>, SearchError> {
    let search_url = format!(
        "http://lyrics.kugou.com/search?ver=1&man=yes&client=pc&keyword={}&hash={}&timelength={}",
        urlencoding::encode(keyword),
        hash,
        duration_ms
    );
    let response = base_api::send(Method::GET, &search_url, &[]).await?;
    let search_resp: LyricSearchResponse = base_api::json(response).await?;
    let Some(candidate) = search_resp
        .candidates
        .and_then(|candidates| candidates.into_iter().next())
    else {
        return Ok(None);
    };

    let download_url = format!(
        "http://lyrics.kugou.com/download?ver=1&client=pc&id={}&accesskey={}&fmt=lrc&charset=utf8",
        candidate.id, candidate.accesskey
    );
    let response = base_api::send(Method::GET, &download_url, &[]).await?;
    let download_resp: LyricDownloadResponse = base_api::json(response).await?;
    let Some(content) = download_resp.content else {
        return Ok(None);
    };

    use base64::Engine;
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(&content)
        .map_err(|error| SearchError::Payload(format!("酷狗歌词 base64 解码失败：{error}")))?;
    let lyric = String::from_utf8(decoded)
        .map_err(|error| SearchError::Payload(format!("酷狗歌词不是合法 UTF-8：{error}")))?;
    Ok(Some(lyric))
}
