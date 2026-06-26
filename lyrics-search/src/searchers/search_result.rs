use super::compare_helper::MatchType;
use super::Searchers;

/// 歌曲搜索结果。
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// 来源搜索平台。
    pub searcher_type: Searchers,
    /// 歌曲标题。
    pub title: String,
    /// 艺术家列表。
    pub artists: Vec<String>,
    /// 专辑名称。
    pub album: String,
    /// 专辑艺术家列表。
    pub album_artists: Option<Vec<String>>,
    /// 歌曲时长（毫秒）。
    pub duration_ms: Option<i32>,
    /// 与目标曲目的匹配等级。
    pub match_type: Option<MatchType>,
    /// 平台内的歌曲标识符。
    pub id: String,
    /// 平台内的数字 ID（可选）。
    pub numeric_id: Option<i64>,
}

impl SearchResult {
    /// 将艺术家列表以逗号拼接为单个字符串。
    pub fn artist(&self) -> String {
        self.artists.join(", ")
    }

    /// 将专辑艺术家列表以逗号拼接为单个字符串，无数据时返回 `None`。
    pub fn album_artist(&self) -> Option<String> {
        self.album_artists.as_ref().map(|a| a.join(", "))
    }
}
