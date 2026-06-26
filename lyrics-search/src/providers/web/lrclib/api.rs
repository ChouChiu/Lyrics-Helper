use super::response::{GetLyricResult, SearchResultItem};
use crate::providers::web::base_api;

const BASE_URL: &str = "https://lrclib.net/api";
const USER_AGENT: &str = "Lyrics-Helper (https://github.com/WXRIW/Lyricify-Lyrics-Helper)";

pub async fn search(
    track_name: &str,
    artist_name: Option<&str>,
    album_name: Option<&str>,
    duration: Option<f64>,
) -> Option<Vec<SearchResultItem>> {
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

    base_api::get_json_with_headers(&url, &[("User-Agent", USER_AGENT)]).await
}

pub async fn get(
    track_name: &str,
    artist_name: &str,
    album_name: Option<&str>,
    duration: Option<f64>,
) -> Option<GetLyricResult> {
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

    base_api::get_json_with_headers(&url, &[("User-Agent", USER_AGENT)]).await
}

pub async fn get_by_id(id: i32) -> Option<GetLyricResult> {
    let url = format!("{}/get/{}", BASE_URL, id);
    base_api::get_json_with_headers(&url, &[("User-Agent", USER_AGENT)]).await
}
