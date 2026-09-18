//! 歌词搜索库，提供多平台歌曲搜索、匹配和歌词获取功能。
//!
//! 全部功能依赖网络请求，仅在启用 `search` feature（默认启用）时编译。

#[cfg(feature = "search")]
pub mod providers;
#[cfg(feature = "search")]
pub mod searchers;
