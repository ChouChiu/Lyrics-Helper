//! 信息行（标题行）相关判断及处理，对应 C# `Helpers/Optimization/InfoLines.cs`。
//!
//! 信息行指作词/作曲等署名行、演唱者标签以外的版权声明行；[`check_info_lines`] 返回与
//! 歌词行等长的标记列表，[`heading_info_lines_count`] / [`ending_info_lines_count`] 只统计
//! 开头/结尾的连续信息行。

use std::borrow::Cow;

use std::sync::LazyLock;

use regex::{Regex, RegexBuilder};

use crate::helpers::chinese_helper::{to_simplified, to_simplified_forced};
use crate::helpers::string_helper::has_chinese;
use crate::models::{LineInfo, LyricsData, TrackMetadata};

/// 标题行（信息行）关键词字典，对应 C# `InfoLines.TitleLineInfoDict`，内容与顺序保持一致。
pub static TITLE_LINE_INFO_DICT: &[&str] = &[
    "音",
    "声",
    "词",
    "曲",
    "鼓",
    "笛",
    "编曲",
    "吉他",
    "吉它",
    "贝斯",
    "缩混",
    "胡琴",
    "扬琴",
    "提琴",
    "录音",
    "弦乐",
    "键盘",
    "钢琴",
    "童声",
    "助理",
    "古筝",
    "琵琶",
    "笛子",
    "二胡",
    "唢呐",
    "出品",
    "和声",
    "和音",
    "编写",
    "混音",
    "封面",
    "营销",
    "发行",
    "制作",
    "监制",
    "策划",
    "企划",
    "推广",
    "母带",
    "编写",
    "文案",
    "编辑",
    "统筹",
    "总监",
    "鸣谢",
    "感谢",
    "设计",
    "视觉",
    "小号",
    "长号",
    "管乐",
    "管弦",
    "工程",
    "簧管",
    "巴松",
    "原唱",
    "配唱",
    "伴唱",
    "轨道",
    "演唱",
    "尺八",
    "乐队",
    "调校",
    "伴奏",
    "主唱",
    "曲绘",
    "呼麦",
    "校对",
    "设计",
    "商务",
    "合作",
    "合唱",
    "指挥",
    "经纪",
    "戏腔",
    "团队",
    "协作",
    "顾问",
    "翻译",
    "摄影",
    "协力",
    "艺人",
    "行销",
    "媒介",
    "运营",
    "宣传",
    "管理",
    "单位",
    "支持",
    "平台",
    "录混",
    "宣发",
    "念白",
    "厂牌",
    "分轨",
    "贴混",
    "古琴",
    "调教",
    "题字",
    "海报",
    "图绘",
    "影像",
    "导演",
    "造型",
    "妆发",
    "单簧管",
    "萨克斯",
    "打击乐",
    "合成器",
    "马头琴",
    "热瓦普",
    "葫芦丝",
    "冬不拉",
    "工作室",
    "科布兹",
    "专辑介绍",
    "巴拉莱卡",
    "OA",
    "OP",
    "OT",
    "PV",
    "SP",
    "A&R",
    "PGM",
    "ISRC",
    "Bass",
    "Drum",
    "Pads",
    "Brass",
    "Cello",
    "Choir",
    "Horns",
    "Mixed",
    "Mixer",
    "Piano",
    "Synth",
    "Viola",
    "Vocal",
    "Winds",
    "Kobza",
    "Assist",
    "Violin",
    "Mixing",
    "String",
    "Guitar",
    "Master",
    "Chorus",
    "Record",
    "violins",
    "Arrange",
    "Conduct",
    "Editing",
    "Hormony",
    "Ocarina",
    "Produce",
    "Strings",
    "Whistle",
    "Stylist",
    "Engineer",
    "Keyboard",
    "Director",
    "Harmonica",
    "Mastering",
    "Recording",
    "Balalaika",
    "Percussion",
    "Programing",
    "Production",
    "Programming",
    "Additional programming",
    "Irish whistle",
];

/// [`TITLE_LINE_INFO_DICT`] 的小写形式，用于等价于 C# `StringComparison.OrdinalIgnoreCase` 的包含匹配。
///
/// 构建时跳过空/空白条目，对应 C# `ContainsAnyKeyword` 中的 `IsNullOrWhiteSpace` 判断。
static TITLE_LINE_INFO_DICT_LOWER: LazyLock<Vec<String>> = LazyLock::new(|| {
    TITLE_LINE_INFO_DICT
        .iter()
        .filter(|keyword| !keyword.trim().is_empty())
        .map(|keyword| keyword.to_lowercase())
        .collect()
});

/// 拆分「标题 - 歌手」式标题的分隔正则，对应 C# `Regex.Split(..., " - ", RegexOptions.IgnoreCase)`。
/// 署名行标记，对应 C# `InfoLines.IsStringCreditBy` 中的列表（预先小写化）。
static CREDIT_BY_MARKERS_LOWER: LazyLock<Vec<String>> = LazyLock::new(|| {
    ["st:", "or:", "Lyrics:", " by:", " By:"]
        .iter()
        .filter(|marker| !marker.is_empty())
        .map(|marker| marker.to_lowercase())
        .collect()
});

static TITLE_DASH_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    RegexBuilder::new(" - ")
        .case_insensitive(true)
        .build()
        .expect("invalid regex pattern")
});

/// 统计歌词中的信息行，返回与 `lines` 等长的列表：`true` => 信息/署名/版权声明行。
///
/// 对应 C# `InfoLines.CheckInfoLines(List<ILineInfo>, ITrackMetadata?)`。
/// C# 在 `lines` 为 `null` 时返回 `null`，Rust 无对应表示，此处对空列表返回空列表。
pub fn check_info_lines(lines: &[LineInfo], track_info: Option<&TrackMetadata>) -> Vec<bool> {
    let n = lines.len();
    let mut flags = vec![false; n];
    if n == 0 {
        return flags;
    }

    let start_count = heading_info_lines_count(lines, track_info);
    let end_count = ending_info_lines_count(lines, track_info);

    for flag in flags.iter_mut().take(start_count.min(n)) {
        *flag = true;
    }

    for flag in flags.iter_mut().skip(n.saturating_sub(end_count)) {
        *flag = true;
    }

    // 中间部分
    let mid_start = start_count.min(n);
    let mid_end_exclusive = n.saturating_sub(end_count).min(n);

    for i in mid_start..mid_end_exclusive {
        if is_info_line(&get_line_text(&lines[i]), track_info) {
            flags[i] = true;
        }
    }

    flags
}

/// 统计 `lyrics` 中的信息行，对应 C# `InfoLines.CheckInfoLines(LyricsData)`。
pub fn check_info_lines_for_lyrics(lyrics: &LyricsData) -> Vec<bool> {
    check_info_lines(
        lyrics.lines.as_deref().unwrap_or(&[]),
        lyrics.track_metadata.as_ref(),
    )
}

/// 统计开头连续的信息行数量。
///
/// 对应 C# `InfoLines.GetHeadingInfoLinesCount(List<ILineInfo>, ITrackMetadata?)`。
pub fn heading_info_lines_count(lines: &[LineInfo], track_info: Option<&TrackMetadata>) -> usize {
    if lines.is_empty() {
        return 0;
    }

    // 特例：「标题 + 歌手」行
    let mut i = 0usize;
    if let Some(track) = track_info {
        let first = get_line_text(&lines[0]);

        if looks_like_title_and_artist_line(&first, track) {
            i += 1;
        } else if lines.len() >= 3 && is_info_line(&get_line_text(&lines[1]), track_info) {
            i += 2;
        }
    }

    while i < lines.len() {
        let text = get_line_text(&lines[i]);

        if text.trim().is_empty() {
            if has_upcoming_info(lines, i, track_info) {
                i += 1;
                continue;
            }

            break;
        }

        if is_info_line(&text, track_info) {
            i += 1;
            continue;
        }

        if i + 1 < lines.len() && is_info_line(&get_line_text(&lines[i + 1]), track_info) {
            i += 2;
            continue;
        }

        break;
    }

    i
}

/// 统计 `lyrics` 开头连续的信息行数量，对应 C# `InfoLines.GetHeadingInfoLinesCount(LyricsData)`。
pub fn heading_info_lines_count_for_lyrics(lyrics: &LyricsData) -> usize {
    heading_info_lines_count(
        lyrics.lines.as_deref().unwrap_or(&[]),
        lyrics.track_metadata.as_ref(),
    )
}

/// 统计结尾连续的信息行数量。
///
/// 对应 C# `InfoLines.GetEndingInfoLinesCount(List<ILineInfo>, ITrackMetadata?)`。
pub fn ending_info_lines_count(lines: &[LineInfo], track_info: Option<&TrackMetadata>) -> usize {
    if lines.is_empty() {
        return 0;
    }

    let mut count = 0usize;

    for i in (0..lines.len()).rev() {
        let text = get_line_text(&lines[i]);

        if text.trim().is_empty() {
            if has_previous_info(lines, i, track_info) {
                count += 1;
                continue;
            }

            break;
        }

        if is_info_line(&text, track_info) {
            count += 1;
            continue;
        }

        break;
    }

    count
}

/// 统计 `lyrics` 结尾连续的信息行数量，对应 C# `InfoLines.GetEndingInfoLinesCount(LyricsData)`。
pub fn ending_info_lines_count_for_lyrics(lyrics: &LyricsData) -> usize {
    ending_info_lines_count(
        lyrics.lines.as_deref().unwrap_or(&[]),
        lyrics.track_metadata.as_ref(),
    )
}

/// 判断单行文本是否为信息行（署名行/版权声明行）。
///
/// 对应 C# `InfoLines.IsInfoLine(string?, ITrackMetadata?)`。
/// 注：C# 中 `text.Replace("：", ": ")` 后已不存在全角冒号，`str.Contains('：')` 恒为 false，
/// 此处保留该判断以与上游一致。
pub fn is_info_line(text: &str, track_info: Option<&TrackMetadata>) -> bool {
    if text.trim().is_empty() {
        return false;
    }

    let str_sc = to_simplified(&text.replace('：', ": "));
    let s = str_sc.trim();

    if looks_like_artist_speaker_label(text, track_info) {
        return false;
    }

    if is_string_tencent_claiming(s) {
        return true;
    }

    if is_string_credit_by(s) {
        return true;
    }

    // 先判冒号：没有冒号时关键词命中与否都不影响结果，可跳过整部字典的扫描。
    let has_colon = s.contains(':') || s.contains('：');
    if !has_colon || !contains_any_keyword(s, &TITLE_LINE_INFO_DICT_LOWER) {
        return is_string_copyright_claiming(s);
    }

    true
}

/// 取行文本，对应 C# `InfoLines.GetLineText(ILineInfo?)`（音节行取音节拼接文本）。
fn get_line_text(line: &LineInfo) -> Cow<'_, str> {
    match line {
        LineInfo::Line { text, .. } | LineInfo::FullLine { text, .. } => Cow::Borrowed(text),
        _ => Cow::Owned(line.text_from_any()),
    }
}

/// 文本中是否包含字典中任一关键词（忽略大小写），对应 C# `InfoLines.ContainsAnyKeyword`。
///
/// `dict_lower` 为预先小写化的 [`TITLE_LINE_INFO_DICT_LOWER`]。
fn contains_any_keyword(s: &str, dict_lower: &[String]) -> bool {
    let s_lower = s.to_lowercase();
    dict_lower
        .iter()
        .any(|keyword| s_lower.contains(keyword.as_str()))
}

/// 从 `index` 起向后最多查看 2 个非空行，判断其中是否存在信息行。
///
/// 对应 C# `InfoLines.HasUpcomingInfo`。
fn has_upcoming_info(lines: &[LineInfo], index: usize, track_info: Option<&TrackMetadata>) -> bool {
    let mut seen = 0;
    let mut i = index + 1;
    while i < lines.len() && seen < 2 {
        let text = get_line_text(&lines[i]);
        i += 1;

        if text.trim().is_empty() {
            continue;
        }

        seen += 1;

        if is_info_line(&text, track_info) {
            return true;
        }
    }

    false
}

/// 从 `index` 起向前最多查看 2 个非空行，判断其中是否存在信息行。
///
/// 对应 C# `InfoLines.HasPreviousInfo`。
fn has_previous_info(lines: &[LineInfo], index: usize, track_info: Option<&TrackMetadata>) -> bool {
    let mut seen = 0;
    let mut i = index;
    while i > 0 && seen < 2 {
        i -= 1;
        let text = get_line_text(&lines[i]);

        if text.trim().is_empty() {
            continue;
        }

        seen += 1;

        if is_info_line(&text, track_info) {
            return true;
        }
    }

    false
}

/// 判断 `raw` 是否为「歌手名:」形式的演唱者标签行。
///
/// 对应 C# `InfoLines.LooksLikeArtistSpeakerLabel`（任意歌手名加冒号即视为标签行）。
fn looks_like_artist_speaker_label(raw: &str, track_info: Option<&TrackMetadata>) -> bool {
    let trimmed = raw.trim();
    if !trimmed.ends_with(':') {
        return false;
    }

    let Some(track) = track_info else {
        return false;
    };

    let trimmed_lower = trimmed.to_lowercase();

    // 多歌手
    if let Some(artists) = &track.artists {
        for artist in artists {
            if artist.trim().is_empty() {
                continue;
            }

            if format!("{}:", artist.trim()).to_lowercase() == trimmed_lower {
                return true;
            }
        }
    }

    // 单歌手
    let single = artist_text(track);
    if !single.trim().is_empty() && format!("{}:", single.trim()).to_lowercase() == trimmed_lower {
        return true;
    }

    false
}

/// 判断整行是否形如「标题 + 歌手」，对应 C# `InfoLines.LooksLikeTitleAndArtistLine`。
fn looks_like_title_and_artist_line(line: &str, track_info: &TrackMetadata) -> bool {
    if line.trim().is_empty() {
        return false;
    }

    let title = track_info.title.clone().unwrap_or_default();
    let artist = artist_text(track_info);

    let line_norm = line.to_lowercase().replace('’', "'");
    let title_norm = to_simplified_forced(&title.to_lowercase()).replace('’', "'");
    let artist_norm = to_simplified_forced(&artist.to_lowercase())
        .replace('’', "'")
        .replace(", ", "/");

    let title_hit = contains_title(&line_norm, &title_norm);
    let artist_hit = contains_artists(&line_norm, track_info, &artist_norm);

    if title_hit && artist_hit {
        return true;
    }

    if title.contains(" - ") {
        let lowered = title.to_lowercase().replace('’', "'");
        if let Some(first) = TITLE_DASH_PATTERN.split(&lowered).next() {
            if !first.trim().is_empty()
                && line_norm.contains(first)
                && line_norm.contains(&artist_norm)
            {
                return true;
            }
        }
    }

    if has_chinese(&title)
        && line.contains(to_simplified(&title).as_str())
        && line.contains(to_simplified(&artist).as_str())
    {
        return true;
    }

    false
}

/// 判断行是否包含标题，对应 C# `InfoLines.ContainsTitle`。
fn contains_title(line_lower: &str, title_lower: &str) -> bool {
    if title_lower.trim().is_empty() {
        return false;
    }

    if line_lower.contains(title_lower) {
        return true;
    }

    // 行内含 "(...)" 时比较 "(" 之前的部分
    if let Some(idx) = line_lower.find('(') {
        if idx > 0 && line_lower[..idx].contains(title_lower) {
            return true;
        }
    }

    // 标题内含 "(...)" 时比较 "(" 之前的部分
    if let Some(title_idx) = title_lower.find('(') {
        if title_idx > 0 {
            let before = title_lower[..title_idx].trim();
            if !before.is_empty() && line_lower.contains(before) {
                return true;
            }
        }
    }

    // 标题内含 " - " 时比较 " - " 之前的部分
    if let Some(dash) = title_lower.find(" - ") {
        if dash > 0 {
            let before = title_lower[..dash].trim();
            if !before.is_empty() && line_lower.contains(before) {
                return true;
            }
        }
    }

    false
}

/// 判断行是否包含歌手名，对应 C# `InfoLines.ContainsArtists`。
fn contains_artists(
    line_lower: &str,
    track_info: &TrackMetadata,
    single_artist_norm: &str,
) -> bool {
    if !single_artist_norm.trim().is_empty() && line_lower.contains(single_artist_norm) {
        return true;
    }

    if let Some(artists) = &track_info.artists {
        let hit = artists
            .iter()
            .filter(|artist| !artist.trim().is_empty())
            .filter(|artist| {
                let norm = to_simplified_forced(&artist.to_lowercase())
                    .replace('’', "'")
                    .replace(", ", "/");
                !norm.is_empty() && line_lower.contains(&norm)
            })
            .count();

        if hit > 1 || (hit == 1 && artists.len() == 1) {
            return true;
        }
    }

    false
}

/// 判断文本是否为腾讯音乐的翻译权声明，对应 C# `InfoLines.IsStringTencentClaiming`。
///
/// `s` 须为已简体化的文本（[`is_info_line`] 在调用前已转换）。
fn is_string_tencent_claiming(s: &str) -> bool {
    (s.contains("腾讯") || s.contains("TME"))
        && s.contains("享有")
        && s.contains("翻译")
        && s.contains('权')
}

/// 判断文本是否为署名行（`st:`/`or:`/`er:` 等标记），对应 C# `InfoLines.IsStringCreditBy`。
fn is_string_credit_by(s: &str) -> bool {
    let s_lower = s.to_lowercase();

    if CREDIT_BY_MARKERS_LOWER
        .iter()
        .any(|marker| s_lower.contains(marker.as_str()))
    {
        return true;
    }

    s_lower.contains("er:") && !s_lower.contains("tedder:") && !s_lower.contains("bieber:")
}

/// 判断文本是否为版权声明，对应 C# `InfoLines.IsStringCopyrightClaiming`（7 项关键词命中 4 项及以上）。
fn is_string_copyright_claiming(s: &str) -> bool {
    const KEYWORDS: [&str; 7] = ["未经", "许可", "授权", "不得", "请勿", "使用", "版权"];

    KEYWORDS
        .iter()
        .filter(|keyword| s.contains(**keyword))
        .count()
        >= 4
}

/// 返回元数据中的歌手字符串。
///
/// 对应 C# `ITrackMetadata.Artist`：单歌手元数据取 `Artist`，多歌手元数据取 `Artists` 的 `", "` 连接。
fn artist_text(track_info: &TrackMetadata) -> String {
    match (&track_info.artist, &track_info.artists) {
        (Some(artist), _) => artist.clone(),
        (None, Some(artists)) => artists.join(", "),
        (None, None) => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(text: &str) -> LineInfo {
        LineInfo::new_line_simple(text.to_string())
    }

    fn metadata(title: &str, artist: &str) -> TrackMetadata {
        TrackMetadata {
            title: Some(title.to_string()),
            artist: Some(artist.to_string()),
            ..Default::default()
        }
    }

    /// `作词 : X` 这类署名行是信息行；普通歌词行（即使含字典关键词）不是。
    #[test]
    fn info_line_detection() {
        assert!(is_info_line("作词 : X", None));
        assert!(is_info_line("作词：X", None));
        assert!(!is_info_line("第一句歌词", None));
        assert!(!is_info_line("", None));
        assert!(!is_info_line("   ", None));
        // ℗/© 行同样需要「字典关键词 + 冒号」才判定为信息行
        assert!(is_info_line("℗ : Example Records", None));
        assert!(!is_info_line("℗ 2024 Example Records", None));
    }

    /// 开头/结尾连续信息行的统计。
    #[test]
    fn heading_and_ending_counts() {
        let heading = vec![
            line("作词 : A"),
            line("作曲 : B"),
            line("编曲 : C"),
            line("第一句歌词"),
            line("第二句歌词"),
        ];
        assert_eq!(heading_info_lines_count(&heading, None), 3);
        assert_eq!(ending_info_lines_count(&heading, None), 0);

        let ending = vec![
            line("第一句歌词"),
            line("第二句歌词"),
            line("混音 : D"),
            line("℗ : Example Records"),
        ];
        assert_eq!(heading_info_lines_count(&ending, None), 0);
        assert_eq!(ending_info_lines_count(&ending, None), 2);
    }

    /// 首尾的空白行在有相邻信息行时一并计入信息行。
    #[test]
    fn blank_lines_around_info_lines_are_counted() {
        let heading = vec![
            line(""),
            line("   "),
            line("作词 : A"),
            line("第一句歌词"),
            line("第二句歌词"),
        ];
        assert_eq!(heading_info_lines_count(&heading, None), 3);

        let ending = vec![
            line("第一句歌词"),
            line("第二句歌词"),
            line("混音 : D"),
            line(""),
            line("  "),
        ];
        assert_eq!(ending_info_lines_count(&ending, None), 3);
    }

    /// 中间的信息行由逐行判定标记；`LyricsData` 包装函数结果一致。
    #[test]
    fn check_info_lines_flags_middle_info_line() {
        let lines = vec![
            line("第一句歌词"),
            line("第二句歌词"),
            line("混音 : D"),
            line("第三句歌词"),
        ];
        assert_eq!(
            check_info_lines(&lines, None),
            vec![false, false, true, false]
        );

        let data = LyricsData {
            lines: Some(lines),
            ..Default::default()
        };
        assert_eq!(
            check_info_lines_for_lyrics(&data),
            vec![false, false, true, false]
        );
        assert_eq!(
            check_info_lines_for_lyrics(&LyricsData::default()),
            Vec::<bool>::new()
        );
    }

    /// 署名/版权声明判定，包含 4 项关键词的阈值。
    #[test]
    fn credit_and_copyright_claiming() {
        assert!(is_info_line("Lyrics: someone", None));
        assert!(is_info_line("腾讯享有翻译权", None));
        assert!(is_info_line("未经许可不得使用本作品", None));
        assert!(!is_info_line("未经许可使用本作品", None));
    }

    /// 歌手名标签行不算信息行（同一文本无元数据时算）。
    #[test]
    fn artist_speaker_label_is_not_info_line() {
        let track = metadata("Song", "Mixer");
        assert!(!is_info_line("Mixer:", Some(&track)));
        assert!(is_info_line("Mixer:", None));
        assert!(is_info_line("混音 : X", Some(&track)));
    }

    /// 多歌手元数据的歌手列表同样参与标签行匹配。
    #[test]
    fn multi_artist_metadata_matches_speaker_label() {
        let track = TrackMetadata {
            title: Some("Song".to_string()),
            artists: Some(vec!["Mixer".to_string(), "Vocal".to_string()]),
            ..Default::default()
        };
        assert!(!is_info_line("Vocal:", Some(&track)));
        assert!(is_info_line("Vocal:", None));
    }

    /// 「标题 - 歌手」行计入开头信息行。
    #[test]
    fn title_and_artist_line_counts_as_heading() {
        let track = metadata("Song Title", "Artist Name");
        let lines = vec![
            line("Song Title - Artist Name"),
            line("第一句歌词"),
            line("第二句歌词"),
        ];
        assert_eq!(heading_info_lines_count(&lines, Some(&track)), 1);
        assert_eq!(
            check_info_lines(&lines, Some(&track)),
            vec![true, false, false]
        );
    }
}
