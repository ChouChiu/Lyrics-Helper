use serde::{Deserialize, Serialize};

use super::lyrics_types::{LyricsTypes, SyncTypes};
use super::line_info::LineInfo;
use super::track_metadata::TrackMetadata;

/// 歌词数据的顶层容器，包含解析后的完整歌词信息。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LyricsData {
    /// 文件格式信息
    pub file: Option<FileInfo>,
    /// 歌词行列表
    pub lines: Option<Vec<LineInfo>>,
    /// 歌词作者列表
    pub writers: Option<Vec<String>>,
    /// 歌曲元数据（标题、艺术家等）
    pub track_metadata: Option<TrackMetadata>,
}

/// 歌词文件的格式和同步类型信息。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FileInfo {
    /// 歌词格式类型
    #[serde(rename = "type")]
    pub lyrics_type: LyricsTypes,
    /// 同步类型
    pub sync_types: SyncTypes,
    /// 格式相关的附加信息
    pub additional_info: Option<AdditionalFileInfo>,
}

/// 不同歌词格式的附加文件信息。
///
/// 使用 tagged union 序列化，JSON 中通过 `type` 字段区分变体。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AdditionalFileInfo {
    /// 通用格式附加信息（如 LRC 头部标签）
    General {
        /// 键值对属性列表
        attributes: Vec<(String, String)>,
    },
    /// KRC 格式附加信息
    Krc {
        /// 键值对属性列表
        attributes: Vec<(String, String)>,
        /// 文件哈希值
        hash: Option<String>,
    },
    /// Spotify 格式附加信息
    Spotify {
        /// 歌词提供商
        provider: Option<String>,
        /// 提供商歌词 ID
        provider_lyrics_id: Option<String>,
        /// 提供商显示名称
        provider_display_name: Option<String>,
        /// 歌词语言
        lyrics_language: Option<String>,
    },
}

impl AdditionalFileInfo {
    /// 创建空的通用格式附加信息。
    pub fn new_general() -> Self {
        Self::General {
            attributes: Vec::new(),
        }
    }

    /// 创建空的 KRC 格式附加信息。
    pub fn new_krc() -> Self {
        Self::Krc {
            attributes: Vec::new(),
            hash: None,
        }
    }

    /// 创建 Spotify 格式附加信息。
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

    /// 获取属性列表的只读引用（仅 General 和 Krc 变体支持）。
    pub fn attributes(&self) -> Option<&Vec<(String, String)>> {
        match self {
            Self::General { attributes } | Self::Krc { attributes, .. } => Some(attributes),
            _ => None,
        }
    }

    /// 获取属性列表的可变引用（仅 General 和 Krc 变体支持）。
    pub fn attributes_mut(&mut self) -> Option<&mut Vec<(String, String)>> {
        match self {
            Self::General { attributes } | Self::Krc { attributes, .. } => Some(attributes),
            _ => None,
        }
    }

    /// 获取 KRC 文件哈希值（仅 Krc 变体支持）。
    pub fn hash(&self) -> Option<&str> {
        match self {
            Self::Krc { hash, .. } => hash.as_deref(),
            _ => None,
        }
    }
}
