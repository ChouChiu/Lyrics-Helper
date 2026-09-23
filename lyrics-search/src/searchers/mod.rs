pub mod apple_music;
pub mod compare_helper;
pub mod kugou;
pub mod lrclib;
pub mod musixmatch;
pub mod netease;
pub mod qq_music;
pub mod refinement;
pub mod search_result;
pub mod searcher;
pub mod soda_music;
pub mod spotify;

pub use compare_helper::{MatchType, compare_track, compare_track_result, rank_by_match};
pub use refinement::{
    build_refinement_queries, build_search_string, search_for_best_result,
    search_for_best_result_with_match, search_with_refinement, strip_feat,
};

/// 支持的歌词搜索平台枚举。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Searchers {
    /// QQ 音乐
    QQMusic,
    /// 网易云音乐
    Netease,
    /// 酷狗音乐
    Kugou,
    /// Musixmatch
    Musixmatch,
    /// 汽水音乐
    SodaMusic,
    /// Apple Music
    AppleMusic,
    /// Spotify
    Spotify,
    /// LRCLIB
    LRCLIB,
}
