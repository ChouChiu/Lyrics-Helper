//! AMLL TTML DB（<https://github.com/amll-dev/amll-ttml-db>）：社区维护的逐词 TTML 歌词库。
//!
//! 仓库只有静态文件，没有搜索接口：按平台 ID 直接取歌词文件，按名称搜索则基于仓库的
//! 元数据索引 `metadata/raw-lyrics-index.jsonl` 在本地匹配。

pub mod api;
pub mod response;
