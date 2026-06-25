use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct TrackResponse {
    pub message: Option<Message>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Message {
    pub header: Header,
    pub body: Option<Body>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Header {
    pub status_code: i32,
    #[serde(default)]
    pub confidence: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Body {
    pub track: Option<Track>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Track {
    pub track_id: i64,
    pub track_name: String,
    pub artist_name: String,
    pub album_name: Option<String>,
    #[serde(default)]
    pub track_length: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TokenResponse {
    pub message: Option<TokenMessage>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TokenMessage {
    pub body: Option<TokenBody>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TokenBody {
    pub user_token: Option<String>,
}
