use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SearchResponse {
    pub(crate) data: Option<SearchData>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SearchData {
    pub(crate) info: Option<Vec<SongInfo>>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SongInfo {
    pub(crate) hash: String,
    pub(crate) songname: String,
    pub(crate) singername: String,
    pub(crate) album_name: Option<String>,
    pub(crate) duration: Option<i32>,
}

/// 酷狗歌词搜索响应。
#[derive(Debug, Clone, Deserialize)]
pub struct LyricSearchResponse {
    /// 候选歌词列表
    pub candidates: Option<Vec<LyricCandidate>>,
}

/// 酷狗歌词候选条目。
#[derive(Debug, Clone, Deserialize)]
pub struct LyricCandidate {
    /// 歌词 ID
    pub id: String,
    /// 访问密钥
    pub accesskey: String,
    /// 歌曲名称
    pub song: Option<String>,
    /// 艺术家名称
    pub singer: Option<String>,
    /// 歌曲时长（毫秒）
    pub duration: Option<i32>,
}

/// 酷狗歌词下载响应。
#[derive(Debug, Clone, Deserialize)]
pub struct LyricDownloadResponse {
    /// Base64 编码的歌词内容
    pub content: Option<String>,
}
