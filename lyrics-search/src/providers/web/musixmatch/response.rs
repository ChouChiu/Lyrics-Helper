use serde::Deserialize;

/// Musixmatch 曲目信息（对应 C# `GetTrackResponse.Track`）。
#[derive(Debug, Clone, Deserialize)]
pub struct Track {
    /// 曲目 ID
    pub track_id: i64,
    /// 曲目名称
    #[serde(default)]
    pub track_name: String,
    /// 艺人名称
    #[serde(default)]
    pub artist_name: String,
    /// 专辑名称
    #[serde(default)]
    pub album_name: Option<String>,
    /// 时长（秒）
    #[serde(default)]
    pub track_length: i32,
    /// 曲目 vanity ID
    #[serde(default)]
    pub commontrack_vanity_id: Option<String>,
}

/// 获取曲目响应（对应 C# `GetTrackResponse`）。
#[derive(Debug, Clone, Deserialize)]
pub struct TrackResponse {
    pub message: Option<Message>,
}

impl TrackResponse {
    /// 用单条搜索结果构造响应（对应 C# `GetTrack` 中手动构造的 `GetTrackResponse`）。
    pub(crate) fn from_track(track: Track) -> Self {
        Self {
            message: Some(Message {
                header: Header {
                    status_code: 200,
                    confidence: 1000.0,
                },
                body: Some(Body { track: Some(track) }),
            }),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Message {
    pub header: Header,
    pub body: Option<Body>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Header {
    #[serde(rename = "status_code")]
    pub status_code: i32,
    #[serde(default)]
    pub confidence: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Body {
    pub track: Option<Track>,
}
