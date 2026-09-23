use std::collections::HashSet;
use std::sync::{Arc, LazyLock, PoisonError, RwLock};
use std::time::{Duration, Instant};

use reqwest::{Method, StatusCode};
use tokio::sync::Mutex;

use super::response::{LyricsEntry, RawIndexLine};
use crate::error::SearchError;
use crate::providers::web::base_api;

/// GitHub 原始文件地址（默认）。
pub const GITHUB_BASE_URL: &str =
    "https://raw.githubusercontent.com/amll-dev/amll-ttml-db/refs/heads/main";
/// 社区镜像 amlldb.bikonoo.com，目录结构与仓库一致。
pub const BIKONOO_BASE_URL: &str = "https://amlldb.bikonoo.com";
/// 社区镜像 GDBA，目录结构与仓库一致。
pub const GBCLSTUDIO_BASE_URL: &str = "https://amll-ttml-db.gbclstudio.cn";

/// 索引缓存的有效期：歌词库每天都有新提交，但没必要每次搜索都下载 1.6 MB 的索引。
const INDEX_TTL: Duration = Duration::from_secs(60 * 60);

/// 单次搜索最多返回的条目数，避免很短的关键词把整个库都返回回去。
const MAX_SEARCH_RESULTS: usize = 50;

static BASE_URL: LazyLock<RwLock<String>> =
    LazyLock::new(|| RwLock::new(GITHUB_BASE_URL.to_string()));

/// 串行化索引下载，并发的搜索只会触发一次下载。
static INDEX: LazyLock<Mutex<Option<CachedIndex>>> = LazyLock::new(|| Mutex::new(None));

struct CachedIndex {
    base_url: String,
    fetched_at: Instant,
    entries: Arc<[LyricsEntry]>,
}

/// 歌词文件所在的平台目录。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    /// 网易云音乐（`ncm-lyrics/`），ID 为歌曲 ID。
    Netease,
    /// QQ 音乐（`qq-lyrics/`），ID 为歌曲 mid。
    QQMusic,
    /// Apple Music（`am-lyrics/`），ID 为歌曲 ID。
    AppleMusic,
    /// Spotify（`spotify-lyrics/`），ID 为歌曲 ID。
    Spotify,
}

impl Platform {
    fn folder(self) -> &'static str {
        match self {
            Self::Netease => "ncm-lyrics",
            Self::QQMusic => "qq-lyrics",
            Self::AppleMusic => "am-lyrics",
            Self::Spotify => "spotify-lyrics",
        }
    }
}

/// 平台目录下提供的歌词格式（由仓库从 TTML 自动生成）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// 原始 TTML。
    Ttml,
    /// LRC。
    Lrc,
    /// 网易云 YRC。
    Yrc,
    /// QQ 音乐 QRC（明文）。
    Qrc,
    /// Lyricify Syllable。
    Lys,
    /// ESLyRiC。
    Eslrc,
}

impl Format {
    fn extension(self) -> &'static str {
        match self {
            Self::Ttml => "ttml",
            Self::Lrc => "lrc",
            Self::Yrc => "yrc",
            Self::Qrc => "qrc",
            Self::Lys => "lys",
            Self::Eslrc => "eslrc",
        }
    }
}

/// 当前使用的仓库地址。
pub fn base_url() -> String {
    BASE_URL
        .read()
        .unwrap_or_else(PoisonError::into_inner)
        .clone()
}

/// 切换仓库地址（如国内访问 GitHub 不畅时改用 [`BIKONOO_BASE_URL`]）。
///
/// 地址须是目录结构与仓库一致的 `http(s)` 镜像，否则返回 [`SearchError::InvalidConfig`]；
/// 切换后下一次搜索会从新地址重新下载索引。
pub fn set_base_url(url: &str) -> Result<(), SearchError> {
    let url = url.trim().trim_end_matches('/');
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return Err(SearchError::InvalidConfig(format!(
            "AMLL TTML DB 地址须以 http(s):// 开头：{url}"
        )));
    }
    *BASE_URL.write().unwrap_or_else(PoisonError::into_inner) = url.to_string();
    Ok(())
}

/// 按平台歌曲 ID 获取歌词文件。
///
/// 返回 `Ok(None)` 表示歌词库里没有这首歌（HTTP 404）。
pub async fn get_lyrics(
    platform: Platform,
    id: &str,
    format: Format,
) -> Result<Option<String>, SearchError> {
    let path = format!(
        "{}/{}.{}",
        platform.folder(),
        urlencoding::encode(id),
        format.extension()
    );
    get_text(&path).await
}

/// 按搜索结果里的文件名（[`LyricsEntry::raw_lyric_file`]）获取 TTML 歌词。
///
/// 返回 `Ok(None)` 表示文件不存在（HTTP 404）。
pub async fn get_raw_lyrics(raw_lyric_file: &str) -> Result<Option<String>, SearchError> {
    get_text(&format!(
        "raw-lyrics/{}",
        urlencoding::encode(raw_lyric_file)
    ))
    .await
}

/// 在歌词库索引中搜索，关键词按空白拆开后须全部出现在曲名、艺术家或专辑里（不区分大小写）。
///
/// 结果按提交时间从新到旧排列，最多 50 条；没有可用关键词时返回空列表。
pub async fn search(keyword: &str) -> Result<Vec<LyricsEntry>, SearchError> {
    let terms = search_terms(keyword);
    if terms.is_empty() {
        return Ok(Vec::new());
    }

    let entries = index().await?;
    Ok(entries
        .iter()
        .filter(|entry| matches_terms(entry, &terms))
        .take(MAX_SEARCH_RESULTS)
        .cloned()
        .collect())
}

/// 返回歌词库索引（每首歌只保留最新版本，从新到旧），一小时内复用已下载的索引。
pub async fn index() -> Result<Arc<[LyricsEntry]>, SearchError> {
    let base_url = base_url();
    let mut cache = INDEX.lock().await;
    if let Some(cached) = cache.as_ref()
        && cached.base_url == base_url
        && cached.fetched_at.elapsed() < INDEX_TTL
    {
        return Ok(cached.entries.clone());
    }

    let url = format!("{base_url}/metadata/raw-lyrics-index.jsonl");
    let response = base_api::send(Method::GET, &url, &[]).await?;
    let entries: Arc<[LyricsEntry]> = parse_index(&base_api::text(response).await?).into();

    *cache = Some(CachedIndex {
        base_url,
        fetched_at: Instant::now(),
        entries: entries.clone(),
    });
    Ok(entries)
}

async fn get_text(path: &str) -> Result<Option<String>, SearchError> {
    let url = format!("{}/{path}", base_url());
    let response = base_api::send(Method::GET, &url, &[]).await?;
    if response.status() == StatusCode::NOT_FOUND {
        return Ok(None);
    }
    Ok(Some(base_api::text(response).await?))
}

/// 解析索引并去掉旧版本。
///
/// 索引按提交时间从旧到新排列，同一首歌的每次修订各占一行。倒序遍历时，
/// 与更新条目共用任一平台 ID 的条目就是旧版本；没有任何平台 ID 的条目按曲名与艺术家判断。
/// 单行损坏只跳过该行，不影响整个索引。
fn parse_index(text: &str) -> Vec<LyricsEntry> {
    let mut seen = HashSet::new();
    let mut entries = Vec::new();

    for line in text.lines().rev() {
        let Ok(raw) = serde_json::from_str::<RawIndexLine>(line) else {
            continue;
        };
        let entry = LyricsEntry::from(raw);
        let keys = identity_keys(&entry);
        if keys.iter().any(|key| seen.contains(key)) {
            continue;
        }
        seen.extend(keys);
        entries.push(entry);
    }

    entries
}

fn identity_keys(entry: &LyricsEntry) -> Vec<String> {
    let platforms = [
        ("ncm", &entry.ncm_music_ids),
        ("qq", &entry.qq_music_ids),
        ("am", &entry.apple_music_ids),
        ("spotify", &entry.spotify_ids),
    ];
    let keys: Vec<String> = platforms
        .iter()
        .flat_map(|(platform, ids)| ids.iter().map(move |id| format!("{platform}:{id}")))
        .collect();
    if !keys.is_empty() {
        return keys;
    }
    vec![format!(
        "meta:{}|{}",
        entry.title().to_lowercase(),
        entry.artists.join(",").to_lowercase()
    )]
}

/// 拆出搜索词，只由标点构成的片段（如 `-`）不参与匹配。
fn search_terms(keyword: &str) -> Vec<String> {
    keyword
        .split_whitespace()
        .filter(|term| term.chars().any(char::is_alphanumeric))
        .map(str::to_lowercase)
        .collect()
}

fn matches_terms(entry: &LyricsEntry, terms: &[String]) -> bool {
    let haystack = entry
        .music_names
        .iter()
        .chain(&entry.artists)
        .chain(&entry.albums)
        .map(|field| field.to_lowercase())
        .collect::<Vec<_>>()
        .join("\n");
    terms.iter().all(|term| haystack.contains(term.as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const INDEX: &str = r#"{"metadata":[["album",["Idol"]],["artists",["YOASOBI"]],["musicName",["Idol"]],["ncmMusicId",["2048982668"]],["ttmlAuthorGithubLogin",["Steve-xmh"]]],"rawLyricFile":"1-old.ttml"}
not json
{"metadata":[["album",["ウェザーステーション"]],["artists",["稲葉曇","歌愛ユキ"]],["musicName",["ラグトレイン"]]],"rawLyricFile":"2-lagtrain.ttml"}
{"metadata":[["album",["Idol"]],["artists",["YOASOBI"]],["musicName",["Idol"]],["ncmMusicId",["2048982668"]],["qqMusicId",["003tS1Ss1q2Y9A"]],["Composer",["Ayase"]]],"rawLyricFile":"3-new.ttml"}"#;

    #[test]
    fn keeps_only_the_newest_revision_of_each_song() {
        let entries = parse_index(INDEX);

        assert_eq!(
            entries
                .iter()
                .map(|entry| entry.raw_lyric_file.as_str())
                .collect::<Vec<_>>(),
            vec!["3-new.ttml", "2-lagtrain.ttml"]
        );
        let idol = &entries[0];
        assert_eq!(idol.title(), "Idol");
        assert_eq!(idol.qq_music_ids, vec!["003tS1Ss1q2Y9A".to_string()]);
        assert_eq!(entries[1].artists, vec!["稲葉曇", "歌愛ユキ"]);
    }

    #[test]
    fn every_search_term_must_match_some_field() {
        let entries = parse_index(INDEX);
        let find = |keyword: &str| {
            let terms = search_terms(keyword);
            entries
                .iter()
                .filter(|entry| matches_terms(entry, &terms))
                .map(|entry| entry.raw_lyric_file.as_str())
                .collect::<Vec<_>>()
        };

        assert_eq!(find("idol yoasobi"), vec!["3-new.ttml"]);
        assert_eq!(find("ラグトレイン - 歌愛ユキ"), vec!["2-lagtrain.ttml"]);
        assert!(find("idol 稲葉曇").is_empty());
        assert!(search_terms(" - ").is_empty());
    }

    #[test]
    fn rejects_a_base_url_without_scheme() {
        assert!(matches!(
            set_base_url("amlldb.bikonoo.com"),
            Err(SearchError::InvalidConfig(_))
        ));
    }
}
