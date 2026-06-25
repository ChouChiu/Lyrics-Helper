use serde::{Deserialize, Serialize};

use super::lyrics_types::{LyricsTypes, SyncTypes};
use super::line_info::LineInfo;
use super::track_metadata::TrackMetadata;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LyricsData {
    pub file: Option<FileInfo>,
    pub lines: Option<Vec<LineInfo>>,
    pub writers: Option<Vec<String>>,
    pub track_metadata: Option<TrackMetadata>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FileInfo {
    #[serde(rename = "type")]
    pub lyrics_type: LyricsTypes,
    pub sync_types: SyncTypes,
    pub additional_info: Option<AdditionalFileInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AdditionalFileInfo {
    General {
        attributes: Vec<(String, String)>,
    },
    Krc {
        attributes: Vec<(String, String)>,
        hash: Option<String>,
    },
    Spotify {
        provider: Option<String>,
        provider_lyrics_id: Option<String>,
        provider_display_name: Option<String>,
        lyrics_language: Option<String>,
    },
}

impl AdditionalFileInfo {
    pub fn new_general() -> Self {
        Self::General {
            attributes: Vec::new(),
        }
    }

    pub fn new_krc() -> Self {
        Self::Krc {
            attributes: Vec::new(),
            hash: None,
        }
    }

    pub fn new_spotify(
        provider: Option<String>,
        provider_lyrics_id: Option<String>,
        provider_display_name: Option<String>,
        lyrics_language: Option<String>,
    ) -> Self {
        Self::Spotify {
            provider,
            provider_lyrics_id,
            provider_display_name,
            lyrics_language,
        }
    }

    pub fn attributes(&self) -> Option<&Vec<(String, String)>> {
        match self {
            Self::General { attributes } | Self::Krc { attributes, .. } => Some(attributes),
            _ => None,
        }
    }

    pub fn attributes_mut(&mut self) -> Option<&mut Vec<(String, String)>> {
        match self {
            Self::General { attributes } | Self::Krc { attributes, .. } => Some(attributes),
            _ => None,
        }
    }

    pub fn hash(&self) -> Option<&str> {
        match self {
            Self::Krc { hash, .. } => hash.as_deref(),
            _ => None,
        }
    }
}
