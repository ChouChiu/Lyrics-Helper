use async_trait::async_trait;

use super::search_result::SearchResult;
use super::Searchers;
use lyrics_core::models::TrackMetadata;

/// 歌词搜索器 trait，各平台搜索实现需实现此接口。
#[async_trait]
pub trait Searcher: Sync {
    /// 返回搜索器的内部标识名称。
    fn name(&self) -> &str;
    /// 返回搜索器的显示名称。
    fn display_name(&self) -> &str;
    /// 返回搜索器所属的平台类型。
    fn searcher_type(&self) -> Searchers;
    /// 使用搜索字符串执行搜索，返回匹配结果列表。
    async fn search_for_results_str(&self, search_string: &str) -> Option<Vec<SearchResult>>;

    /// 使用曲目元数据执行搜索，自动生成搜索字符串。
    async fn search_for_results(&self, track: &TrackMetadata) -> Option<Vec<SearchResult>> {
        let title = track.title.as_deref().unwrap_or("");
        let artist = track.artist.as_deref().unwrap_or("").replace(", ", " ");
        let album = track.album.as_deref().unwrap_or("");
        let search_string = format!("{} {} {}", title, artist, album)
            .replace(" - ", " ")
            .trim()
            .to_string();
        self.search_for_results_str(&search_string).await
    }
}
