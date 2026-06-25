use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum LyricsTypes {
    #[default]
    Unknown = 0,
    LyricifySyllable = 1,
    LyricifyLines = 2,
    Lrc = 3,
    Qrc = 4,
    Krc = 5,
    Yrc = 6,
    Ttml = 7,
    Spotify = 8,
    Musixmatch = 9,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum LyricsRawTypes {
    #[default]
    Unknown = 0,
    LyricifySyllable = 1,
    LyricifyLines = 2,
    Lrc = 3,
    Qrc = 4,
    QrcFull = 10,
    Krc = 5,
    Yrc = 6,
    YrcFull = 11,
    Ttml = 7,
    AppleJson = 12,
    Spotify = 8,
    Musixmatch = 9,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum SyncTypes {
    #[default]
    Unknown = 0,
    SyllableSynced = 1,
    LineSynced = 2,
    MixedSynced = 3,
    Unsynced = 4,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LyricsAlignment {
    #[default]
    Unspecified,
    Left,
    Right,
}


