use super::response::{TokenResponse, TrackResponse};
use crate::providers::web::base_api;

const BASE_URL: &str = "https://apic-desktop.musixmatch.com/ws/1.1";

fn headers() -> Vec<(&'static str, &'static str)> {
    vec![
        ("Authority", "apic-desktop.musixmatch.com"),
        ("Cookie", "AWSELBCORS=0; AWSELB=0"),
    ]
}

pub(crate) async fn get_token() -> Option<String> {
    let url = format!("{}/token.get?app_id=web-desktop-app-v1.0", BASE_URL);
    let resp: TokenResponse = base_api::get_json_with_headers(&url, &headers()).await?;
    resp.message?.body?.user_token
}

pub(crate) async fn search_track(
    q_track: &str,
    q_artist: &str,
    user_token: &str,
) -> Option<TrackResponse> {
    let url = format!(
        "{}/matcher.track.get?app_id=web-desktop-app-v1.0&usertoken={}&q_track={}&q_artist={}",
        BASE_URL,
        user_token,
        urlencoding::encode(q_track),
        urlencoding::encode(q_artist),
    );
    base_api::get_json_with_headers(&url, &headers()).await
}
