use async_trait::async_trait;

use super::Searchers;
use super::search_result::SearchResult;
use crate::error::SearchError;
use lyrics_core::models::TrackMetadata;

/// 歌词搜索器 trait，各平台搜索实现需实现此接口。
///
/// 错误契约：`Ok(vec![])` 表示搜索请求本身成功、但没有匹配结果；
/// `Err(_)` 表示请求失败（网络不通、被限流、命中验证码、平台返回业务错误码等）。
/// 「查无此歌」永远不是错误，调用方不必再从 `None` 里猜失败原因。
#[async_trait]
pub trait Searcher: Sync {
    /// 返回搜索器的内部标识名称。
    fn name(&self) -> &str;
    /// 返回搜索器的显示名称。
    fn display_name(&self) -> &str;
    /// 返回搜索器所属的平台类型。
    fn searcher_type(&self) -> Searchers;
    /// 使用搜索字符串执行搜索，返回匹配结果列表。
    ///
    /// `Ok(vec![])` 表示搜索成功但没有结果；`Err` 表示请求失败。
    async fn search_for_results_str(
        &self,
        search_string: &str,
    ) -> Result<Vec<SearchResult>, SearchError>;

    /// 使用曲目元数据执行搜索，自动生成搜索字符串。
    ///
    /// `Ok(vec![])` 表示搜索成功但没有结果；`Err` 表示请求失败。
    async fn search_for_results(
        &self,
        track: &TrackMetadata,
    ) -> Result<Vec<SearchResult>, SearchError> {
        self.search_for_results_str(&super::build_search_string(track))
            .await
    }
}
