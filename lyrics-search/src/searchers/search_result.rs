use super::Searchers;
use super::compare_helper::MatchType;

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
    /// 构造一条搜索结果，只填各平台都有的字段。
    ///
    /// `album_artists` / `numeric_id` 多数平台不提供，`match_type` 由
    /// [`compare_track_result`](super::compare_track_result) 在排序时写入，
    /// 需要时用 `..` 语法覆盖。
    pub fn new(
        searcher_type: Searchers,
        title: String,
        artists: Vec<String>,
        album: String,
        duration_ms: Option<i32>,
        id: String,
    ) -> Self {
        Self {
            searcher_type,
            title,
            artists,
            album,
            album_artists: None,
            duration_ms,
            match_type: None,
            id,
            numeric_id: None,
        }
    }

    /// 将艺术家列表以逗号拼接为单个字符串。
    pub fn artist(&self) -> String {
        self.artists.join(", ")
    }

    /// 将专辑艺术家列表以逗号拼接为单个字符串，无数据时返回 `None`。
    pub fn album_artist(&self) -> Option<String> {
        self.album_artists.as_ref().map(|a| a.join(", "))
    }
}

/// 艺术家串的拆分分隔符：既覆盖中文平台的顿号/斜杠，也覆盖英文平台的 `feat.` / `&`。
const ARTIST_SEPARATORS: [&str; 6] = [",", "、", "/", " & ", " feat. ", " ft. "];

/// 把平台返回的艺术家串拆成列表。
///
/// 各平台用的分隔符不同（酷狗用 `、`，LRCLIB 与 Musixmatch 用 `feat.` / `&`），
/// 这里统一按全部分隔符拆分，避免每个 searcher 各写一份。
pub fn split_artists(artists: &str) -> Vec<String> {
    let mut parts = vec![artists];

    for separator in ARTIST_SEPARATORS {
        parts = parts
            .into_iter()
            .flat_map(|part| part.split(separator))
            .collect();
    }

    parts
        .into_iter()
        .map(|part| part.trim().to_string())
        .filter(|part| !part.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::split_artists;

    #[test]
    fn splits_on_every_supported_separator() {
        assert_eq!(split_artists("周杰伦"), vec!["周杰伦".to_string()]);
        assert_eq!(
            split_artists("周杰伦、费玉清/方文山"),
            vec![
                "周杰伦".to_string(),
                "费玉清".to_string(),
                "方文山".to_string()
            ]
        );
        assert_eq!(
            split_artists("Taylor Swift feat. Ed Sheeran & Future"),
            vec![
                "Taylor Swift".to_string(),
                "Ed Sheeran".to_string(),
                "Future".to_string()
            ]
        );
        assert!(split_artists("  ,  ").is_empty());
    }
}
