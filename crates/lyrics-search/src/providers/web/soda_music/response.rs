use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SearchResponse {
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

/// 汽水音乐 H5 `seo_track` 接口返回（获取歌曲详情/歌词）。
///
/// 该接口把歌词放在顶层 `lyric`，也可能只放在 `seo_track` 下，由 API 层归一。
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct TrackDetailResponse {
    #[serde(default)]
    pub(crate) lyric: Option<LyricData>,
    #[serde(default)]
    pub(crate) seo_track: Option<SeoTrackDetail>,
}

/// H5 `seo_track` 字段。
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SeoTrackDetail {
    #[serde(default)]
    pub(crate) lyric: Option<LyricData>,
}

/// 歌词数据。
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct LyricData {
    /// 歌词正文（LRC 格式）
    #[serde(default)]
    pub(crate) content: Option<String>,
    /// 简体中文翻译（来自 `translations.cn`）
    #[serde(default)]
    pub(crate) translations: Option<LyricTranslationData>,
}

/// 翻译数据包装（`translations` 字段）。
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct LyricTranslationData {
    /// 简体中文翻译
    #[serde(default)]
    pub(crate) cn: Option<String>,
}
