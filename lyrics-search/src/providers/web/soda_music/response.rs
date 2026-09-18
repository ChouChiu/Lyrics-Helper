use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SearchResponse {
    #[serde(rename = "status_code")]
    #[allow(dead_code)]
    pub(crate) status_code: Option<i32>,
    #[serde(rename = "result_groups")]
    pub(crate) result_groups: Option<Vec<ResultGroup>>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ResultGroup {
    pub(crate) data: Option<Vec<ResultGroupItem>>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ResultGroupItem {
    pub(crate) meta: Option<Meta>,
    pub(crate) entity: Option<Entity>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Meta {
    #[serde(rename = "item_type")]
    pub(crate) item_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Entity {
    pub(crate) track: Option<Track>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Track {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) duration: Option<i64>,
    pub(crate) artists: Option<Vec<Artist>>,
    pub(crate) album: Option<Album>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Artist {
    pub(crate) name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Album {
    pub(crate) name: String,
}

/// 汽水音乐 H5 `seo_track` 接口返回（获取歌曲详情/歌词）
#[derive(Debug, Clone, Deserialize)]
pub struct TrackDetailResponse {
    #[serde(rename = "status_code")]
    #[allow(dead_code)]
    pub(crate) status_code: Option<i32>,
    #[serde(default)]
    pub(crate) lyric: Option<LyricData>,
    /// 歌曲信息，H5 接口可能只返回 `seo_track.track`，由 API 层回填。
    #[serde(default)]
    pub(crate) track: Option<Track>,
    /// 播放信息，H5 接口可能只返回 `seo_track.track_player`，由 API 层回填。
    #[serde(rename = "track_player", default)]
    #[allow(dead_code)]
    pub(crate) track_player: Option<TrackPlayer>,
    #[serde(rename = "seo_track", default)]
    pub(crate) seo_track: Option<SeoTrackDetail>,
}

/// H5 `seo_track` 字段
#[derive(Debug, Clone, Deserialize)]
pub struct SeoTrackDetail {
    #[serde(default)]
    pub(crate) track: Option<Track>,
    #[serde(rename = "track_player", default)]
    pub(crate) track_player: Option<TrackPlayer>,
}

/// 播放信息
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct TrackPlayer {
    #[serde(rename = "expire_at", default)]
    pub(crate) expire_at: Option<i64>,
    #[serde(rename = "media_id", default)]
    pub(crate) media_id: Option<String>,
    #[serde(rename = "url_player_info", default)]
    pub(crate) url_player_info: Option<String>,
    #[serde(rename = "video_model", default)]
    pub(crate) video_model: Option<String>,
    #[serde(rename = "video_model_type", default)]
    pub(crate) video_model_type: Option<i32>,
}

/// 歌词数据
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct LyricData {
    /// 歌词正文（LRC 格式）
    #[serde(default)]
    pub(crate) content: Option<String>,
    /// 歌词语言
    #[serde(default)]
    pub(crate) lang: Option<String>,
    /// 歌词类型（如 "lrc"）
    #[serde(rename = "type", default)]
    pub(crate) lyric_type: Option<String>,
    /// 各语言翻译
    #[serde(rename = "lang_translations", default)]
    pub(crate) lang_translations: Option<std::collections::HashMap<String, LyricTranslation>>,
    /// 简体中文翻译（来自 `translations.cn`）
    #[serde(default)]
    pub(crate) translations: Option<LyricTranslationData>,
}

/// 翻译歌词数据
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct LyricTranslation {
    #[serde(default)]
    pub(crate) content: Option<String>,
    #[serde(default)]
    pub(crate) lang: Option<String>,
    #[serde(rename = "type", default)]
    pub(crate) translation_type: Option<String>,
}

/// 翻译数据包装（`translations` 字段）
#[derive(Debug, Clone, Deserialize)]
pub struct LyricTranslationData {
    /// 简体中文翻译
    #[serde(default)]
    pub(crate) cn: Option<String>,
}
