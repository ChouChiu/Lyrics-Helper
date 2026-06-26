use super::response::{LyricDownloadResponse, LyricSearchResponse, SearchResponse};
use crate::providers::web::base_api;

pub(crate) async fn search(keyword: &str) -> Option<SearchResponse> {
    let url = format!(
        "http://mobilecdn.kugou.com/api/v3/search/song?format=json&keyword={}&page=1&pagesize=20&showtype=1",
        urlencoding::encode(keyword)
    );
    base_api::get_json(&url).await
}

/// 通过关键词、哈希值和时长搜索并获取酷狗歌词内容。
pub async fn get_lyrics(keyword: &str, hash: &str, duration_ms: i32) -> Option<String> {
    let search_url = format!(
        "http://lyrics.kugou.com/search?ver=1&man=yes&client=pc&keyword={}&hash={}&timelength={}",
        urlencoding::encode(keyword),
        hash,
        duration_ms
    );
    let search_resp: LyricSearchResponse = base_api::get_json(&search_url).await?;
    let candidate = search_resp.candidates?.into_iter().next()?;

    let download_url = format!(
        "http://lyrics.kugou.com/download?ver=1&client=pc&id={}&accesskey={}&fmt=lrc&charset=utf8",
        candidate.id, candidate.accesskey
    );
    let download_resp: LyricDownloadResponse = base_api::get_json(&download_url).await?;
    let content = download_resp.content?;

    use base64::Engine;
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(&content)
        .ok()?;
    String::from_utf8(decoded).ok()
}
