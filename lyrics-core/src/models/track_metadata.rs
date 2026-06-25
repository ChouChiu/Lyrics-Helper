use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TrackMetadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub duration_ms: Option<i32>,
    pub isrc: Option<String>,
    pub language: Option<Vec<String>>,
    pub artists: Option<Vec<String>>,
    pub album_artists: Option<Vec<String>>,
    pub spotify_id: Option<String>,
}

impl TrackMetadata {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spotify_uri(&self) -> Option<String> {
        self.spotify_id.as_ref().map(|id| format!("spotify:track:{}", id))
    }

    pub fn ensure_artists(&mut self) {
        if self.artists.is_none() {
            if let Some(ref artist) = self.artist {
                self.artists = Some(artist.split(", ").map(|s| s.to_string()).collect());
            }
        }
    }

    pub fn ensure_album_artists(&mut self) {
        if self.album_artists.is_none() {
            if let Some(ref album_artist) = self.album_artist {
                self.album_artists = Some(album_artist.split(", ").map(|s| s.to_string()).collect());
            }
        }
    }

    pub fn set_artist_from_list(&mut self, artists: Vec<String>) {
        self.artist = Some(artists.join(", "));
        self.artists = Some(artists);
    }

    pub fn set_album_artist_from_list(&mut self, album_artists: Vec<String>) {
        self.album_artist = Some(album_artists.join(", "));
        self.album_artists = Some(album_artists);
    }
}
