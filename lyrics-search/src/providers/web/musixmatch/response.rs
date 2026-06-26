use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct TrackResponse {
    pub(crate) message: Option<Message>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Message {
    pub(crate) header: Header,
    pub(crate) body: Option<Body>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Header {
    #[serde(rename = "status_code")]
    pub(crate) _status_code: i32,
    #[serde(default)]
    pub(crate) confidence: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Body {
    pub(crate) track: Option<Track>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Track {
    pub(crate) track_id: i64,
    pub(crate) track_name: String,
    pub(crate) artist_name: String,
    pub(crate) album_name: Option<String>,
    #[serde(default)]
    pub(crate) track_length: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct TokenResponse {
    pub(crate) message: Option<TokenMessage>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct TokenMessage {
    pub(crate) body: Option<TokenBody>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct TokenBody {
    pub(crate) user_token: Option<String>,
}
