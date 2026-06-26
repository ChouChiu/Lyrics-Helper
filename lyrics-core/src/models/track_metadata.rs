use serde::{Deserialize, Serialize};

/// 歌曲元数据，包含标题、艺术家、专辑等信息。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TrackMetadata {
    /// 歌曲标题
    pub title: Option<String>,
    /// 艺术家名称（多个艺术家以逗号分隔）
    pub artist: Option<String>,
    /// 专辑名称
    pub album: Option<String>,
    /// 专辑艺术家名称
    pub album_artist: Option<String>,
    /// 歌曲时长（毫秒）
    pub duration_ms: Option<i32>,
    /// 国际标准录音编码
    pub isrc: Option<String>,
    /// 歌词语言列表
    pub language: Option<Vec<String>>,
    /// 艺术家列表（拆分后的独立条目）
    pub artists: Option<Vec<String>>,
    /// 专辑艺术家列表（拆分后的独立条目）
    pub album_artists: Option<Vec<String>>,
    /// Spotify 曲目 ID
    pub spotify_id: Option<String>,
}

impl TrackMetadata {
    /// 创建默认的空元数据。
    pub fn new() -> Self {
        Self::default()
    }

    /// 返回 Spotify URI（格式为 `spotify:track:{id}`），无 ID 时返回 `None`。
    pub fn spotify_uri(&self) -> Option<String> {
        self.spotify_id.as_ref().map(|id| format!("spotify:track:{}", id))
    }

    /// 若 `artists` 列表为空，则从逗号分隔的 `artist` 字段拆分填充。
    pub fn ensure_artists(&mut self) {
        if self.artists.is_none() {
            if let Some(ref artist) = self.artist {
                self.artists = Some(artist.split(", ").map(|s| s.to_string()).collect());
            }
        }
    }

    /// 若 `album_artists` 列表为空，则从逗号分隔的 `album_artist` 字段拆分填充。
    pub fn ensure_album_artists(&mut self) {
        if self.album_artists.is_none() {
            if let Some(ref album_artist) = self.album_artist {
                self.album_artists = Some(album_artist.split(", ").map(|s| s.to_string()).collect());
            }
        }
    }

    /// 从艺术家列表同时设置 `artist`（逗号拼接）和 `artists` 字段。
    pub fn set_artist_from_list(&mut self, artists: Vec<String>) {
        self.artist = Some(artists.join(", "));
        self.artists = Some(artists);
    }

    /// 从专辑艺术家列表同时设置 `album_artist`（逗号拼接）和 `album_artists` 字段。
    pub fn set_album_artist_from_list(&mut self, album_artists: Vec<String>) {
        self.album_artist = Some(album_artists.join(", "));
        self.album_artists = Some(album_artists);
    }
}
