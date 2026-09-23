use serde::Deserialize;

/// 索引文件 `raw-lyrics-index.jsonl` 的一行。
///
/// `metadata` 是 TTML 里 `amll:meta` 的原样转储：`[键, [值…]]` 的列表，
/// 同一个键可以有多个值（多位艺术家、多个平台 ID）。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawIndexLine {
    /// 元数据键值列表。
    pub metadata: Vec<(String, Vec<String>)>,
    /// `raw-lyrics/` 下的歌词文件名。
    pub raw_lyric_file: String,
}

/// 歌词库中的一份歌词（同一首歌只保留最新提交的版本）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LyricsEntry {
    /// `raw-lyrics/` 下的歌词文件名，可交给 [`get_raw_lyrics`](super::api::get_raw_lyrics)。
    pub raw_lyric_file: String,
    /// 曲名（可能有多个译名）。
    pub music_names: Vec<String>,
    /// 艺术家列表。
    pub artists: Vec<String>,
    /// 专辑名（可能有多个译名）。
    pub albums: Vec<String>,
    /// 网易云音乐歌曲 ID。
    pub ncm_music_ids: Vec<String>,
    /// QQ 音乐歌曲 mid。
    pub qq_music_ids: Vec<String>,
    /// Apple Music 歌曲 ID。
    pub apple_music_ids: Vec<String>,
    /// Spotify 歌曲 ID。
    pub spotify_ids: Vec<String>,
    /// ISRC。
    pub isrcs: Vec<String>,
    /// 歌词作者的 GitHub 用户名。
    pub ttml_authors: Vec<String>,
}

impl LyricsEntry {
    /// 首个曲名。
    pub fn title(&self) -> &str {
        self.music_names.first().map_or("", String::as_str)
    }

    /// 首个专辑名。
    pub fn album(&self) -> &str {
        self.albums.first().map_or("", String::as_str)
    }
}

impl From<RawIndexLine> for LyricsEntry {
    fn from(line: RawIndexLine) -> Self {
        let mut entry = Self {
            raw_lyric_file: line.raw_lyric_file,
            ..Self::default()
        };
        for (key, values) in line.metadata {
            let field = match key.as_str() {
                "musicName" => &mut entry.music_names,
                "artists" => &mut entry.artists,
                "album" => &mut entry.albums,
                "ncmMusicId" => &mut entry.ncm_music_ids,
                "qqMusicId" => &mut entry.qq_music_ids,
                "appleMusicId" => &mut entry.apple_music_ids,
                "spotifyId" => &mut entry.spotify_ids,
                "isrc" => &mut entry.isrcs,
                "ttmlAuthorGithubLogin" => &mut entry.ttml_authors,
                _ => continue,
            };
            field.extend(
                values
                    .into_iter()
                    .map(|value| value.trim().to_string())
                    .filter(|value| !value.is_empty()),
            );
        }
        entry
    }
}
