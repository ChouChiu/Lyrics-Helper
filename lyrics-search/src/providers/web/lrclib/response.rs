use serde::Deserialize;

/// LRCLIB 歌词条目。
///
/// 搜索结果与精确获取结果的结构完全相同，只保留一份定义。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LyricItem {
    /// LRCLIB 内的歌词 ID
    pub id: i32,
    /// 曲名
    pub track_name: String,
    /// 艺术家名
    pub artist_name: String,
    /// 专辑名
    pub album_name: String,
    /// 时长（秒）
    pub duration: f64,
    /// 纯文本歌词
    pub plain_lyrics: Option<String>,
    /// 同步歌词（LRC）
    pub synced_lyrics: Option<String>,
}

/// LRCLIB 搜索结果条目。
pub type SearchResultItem = LyricItem;

/// LRCLIB 歌词获取结果。
pub type GetLyricResult = LyricItem;
