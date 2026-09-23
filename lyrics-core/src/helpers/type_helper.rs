//! 歌词原始类型识别与类型名称辅助函数。
//!
//! 移植自上游 `Helpers/Types/LyricsTypeDetector.cs`、`Helpers/Types/LyricsTypes.cs`
//! 与 `Helpers/TypeHelper.cs`。

use crate::models::{LyricsRawTypes, LyricsTypes};
use quick_xml::NsReader;
use quick_xml::events::{BytesStart, Event};
use quick_xml::name::{Namespace, ResolveResult};
use regex::Regex;
use serde_json::Value;
use std::sync::LazyLock;

/// TTML 文档的命名空间。
const TTML_NAMESPACE: &str = "http://www.w3.org/ns/ttml";

// 格式：一个或多个 [分:秒.毫秒] 时间戳，毫秒部分可省略，也支持使用冒号分隔毫秒。
// 示例：[00:12.345]Hello world
static LRC_LINE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^[ \t]*(?:\[\d+:\d{1,2}(?:[.:]\d{1,3})?\])+[^\r\n]*").unwrap()
});

// 格式：[开始时间,结束时间]歌词文本；这是 Lyricify Lines 与逐字格式共有的行头。
// 示例：[12000,15000]Hello world
static BRACKETED_LINE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^[ \t]*\[\d+,\d+\][^\r\n]*").unwrap());

// 格式：[行开始时间,行时长]文本(字开始时间,字时长)。
// 示例：[12000,3000]Hel(12000,400)lo(12400,500)
static QRC_LINE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^[ \t]*\[\d+,\d+\][^\r\n]*\(-?\d+,\d+\)[^\r\n]*").unwrap());

// 格式：[行开始时间,行时长]<相对开始时间,字时长,保留值>文本。
// 示例：[12000,3000]<0,400,0>Hel<400,500,0>lo
static KRC_LINE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^[ \t]*\[\d+,\d+\]<-?\d+,\d+,\d+>[^\r\n]*").unwrap());

// 格式：[行开始时间,行时长](字开始时间,字时长,保留值)文本。
// 示例：[12000,3000](12000,400,0)Hel(12400,500,0)lo
static YRC_LINE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^[ \t]*\[\d+,\d+\]\(-?\d+,\d+,\d+\)[^\r\n]*").unwrap());

// 格式：[行属性]文本(字开始时间,字时长)，行属性表示主/背景人声及对齐方式。
// 示例：[4]Hel(12000,400)lo(12400,500)
static LYRICIFY_SYLLABLE_LINE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^[ \t]*\[\d+\][^\r\n]*\(-?\d+,\d+\)[^\r\n]*").unwrap());

// 匹配任意受支持的逐字时间片段，用于避免把逐字歌词误判为 Lyricify Lines。
// 示例：(12000,400)、(12000,400,0)、<0,400,0>
static ANY_SYLLABLE_TIMING: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:\(-?\d+,\d+(?:,\d+)?\)|<-?\d+,\d+,\d+>)").unwrap());

// 匹配无法作为标准 XML 解析时的 QRC Full 特征节点与歌词属性。
// 示例：<Lyric_1 LyricContent="[0,100]Hi(0,100)" />
static QRC_FULL_FALLBACK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)<Lyric_1\b[^>]*\bLyricContent\s*=").unwrap());

/// 识别歌词文本的原始格式类型，无法识别时返回 [`LyricsRawTypes::Unknown`]。
///
/// 判定顺序与上游 `LyricsTypeDetector.Detect` 完全一致：
/// 空白文本 → JSON 类型 → XML 类型 → `[type:LyricifyLines]` 标记 → KRC → YRC →
/// Lyricify Syllable → QRC → Lyricify Lines → LRC。
pub fn get_lyrics_types(input: &str) -> LyricsRawTypes {
    if input.trim().is_empty() {
        return LyricsRawTypes::Unknown;
    }

    let structured_type = get_json_type(input);
    if structured_type != LyricsRawTypes::Unknown {
        return structured_type;
    }

    let structured_type = get_xml_type(input);
    if structured_type != LyricsRawTypes::Unknown {
        return structured_type;
    }

    if has_lyricify_lines_type_marker(input) {
        return LyricsRawTypes::LyricifyLines;
    }
    if is_krc(input) {
        return LyricsRawTypes::Krc;
    }
    if is_yrc(input) {
        return LyricsRawTypes::Yrc;
    }
    if is_lyricify_syllable(input) {
        return LyricsRawTypes::LyricifySyllable;
    }
    if is_qrc(input) {
        return LyricsRawTypes::Qrc;
    }
    if is_lyricify_lines(input) {
        return LyricsRawTypes::LyricifyLines;
    }
    if is_lrc(input) {
        return LyricsRawTypes::Lrc;
    }

    LyricsRawTypes::Unknown
}

/// 判断文本是否为标准 LRC 格式。
pub fn is_lrc(input: &str) -> bool {
    LRC_LINE.is_match(input)
}

/// 判断文本是否为 Lyricify Lines 格式。
///
/// 带 `[type:LyricifyLines]` 标记时直接命中；否则要求存在 `[开始,结束]` 行头，
/// 且全文没有任何逐字时间片段（避免把逐字歌词误判为 Lyricify Lines）。
pub fn is_lyricify_lines(input: &str) -> bool {
    if has_lyricify_lines_type_marker(input) {
        return true;
    }

    BRACKETED_LINE.is_match(input) && !ANY_SYLLABLE_TIMING.is_match(input)
}

/// 判断文本是否为 Lyricify Syllable 格式。
pub fn is_lyricify_syllable(input: &str) -> bool {
    LYRICIFY_SYLLABLE_LINE.is_match(input)
}

/// 判断文本是否为 QRC 格式。
pub fn is_qrc(input: &str) -> bool {
    QRC_LINE.is_match(input)
}

/// 判断文本是否为 QRC Full（XML 信封）格式。
pub fn is_qrc_full(input: &str) -> bool {
    get_xml_type(input) == LyricsRawTypes::QrcFull
}

/// 判断文本是否为 KRC 格式。
pub fn is_krc(input: &str) -> bool {
    KRC_LINE.is_match(input)
}

/// 判断文本是否为 YRC 格式。
pub fn is_yrc(input: &str) -> bool {
    YRC_LINE.is_match(input)
}

/// 判断文本是否为 YRC Full（API 原始 JSON）格式。
pub fn is_yrc_full(input: &str) -> bool {
    get_json_type(input) == LyricsRawTypes::YrcFull
}

/// 判断文本是否为 TTML 格式。
pub fn is_ttml(input: &str) -> bool {
    get_xml_type(input) == LyricsRawTypes::Ttml
}

/// 判断文本是否为 Apple Music 原始 JSON 格式。
pub fn is_apple_json(input: &str) -> bool {
    get_json_type(input) == LyricsRawTypes::AppleJson
}

/// 判断文本是否为 Spotify 原始 JSON 格式。
pub fn is_spotify(input: &str) -> bool {
    get_json_type(input) == LyricsRawTypes::Spotify
}

/// 判断文本是否为 Musixmatch 原始 JSON 格式。
pub fn is_musixmatch(input: &str) -> bool {
    get_json_type(input) == LyricsRawTypes::Musixmatch
}

/// 按 ASCII 大小写不敏感的方式查找子串（对应上游 `IndexOf(OrdinalIgnoreCase)`）。
fn contains_ignore_case(input: &str, needle: &str) -> bool {
    let haystack = input.as_bytes();
    let needle = needle.as_bytes();
    haystack.len() >= needle.len()
        && haystack
            .windows(needle.len())
            .any(|w| w.eq_ignore_ascii_case(needle))
}

/// 文本是否带有 `[type:LyricifyLines]` 标记（大小写不敏感）。
fn has_lyricify_lines_type_marker(input: &str) -> bool {
    contains_ignore_case(input, "[type:LyricifyLines]")
}

/// 大小写不敏感地取对象属性（对应上游 `JObject.GetValue(name, OrdinalIgnoreCase)`）。
fn get<'a>(value: &'a Value, name: &str) -> Option<&'a Value> {
    value
        .as_object()?
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(name))
        .map(|(_, value)| value)
}

/// 属性是否为非空白字符串（对应上游 `HasNonEmptyString`）。
fn has_non_empty_string(value: &Value, name: &str) -> bool {
    get(value, name)
        .and_then(Value::as_str)
        .is_some_and(|text| !text.trim().is_empty())
}

/// 判断 JSON 根对象的歌词原始类型。
fn get_json_type(input: &str) -> LyricsRawTypes {
    if !input.trim_start().starts_with('{') {
        return LyricsRawTypes::Unknown;
    }

    // 上游使用 Newtonsoft `JObject.Parse`，对注释、尾随逗号等更宽容；
    // 这里使用严格 JSON 解析，只对合法 JSON 生效。
    let Ok(root) = serde_json::from_str::<Value>(input) else {
        return LyricsRawTypes::Unknown;
    };

    if is_apple_json_value(&root) {
        return LyricsRawTypes::AppleJson;
    }
    if is_spotify_value(&root) {
        return LyricsRawTypes::Spotify;
    }
    if is_musixmatch_value(&root) {
        return LyricsRawTypes::Musixmatch;
    }
    if is_yrc_full_value(&root) {
        return LyricsRawTypes::YrcFull;
    }

    LyricsRawTypes::Unknown
}

/// Apple Music 原始 JSON：`data[]` 或其 `relationships["syllable-lyrics"].data[]` 中存在音节歌词项。
fn is_apple_json_value(root: &Value) -> bool {
    let Some(data) = get(root, "data").and_then(Value::as_array) else {
        return false;
    };

    data.iter().any(|item| {
        is_syllable_lyrics_item(item)
            || get(item, "relationships")
                .and_then(|relationships| get(relationships, "syllable-lyrics"))
                .and_then(|syllable_lyrics| get(syllable_lyrics, "data"))
                .and_then(Value::as_array)
                .is_some_and(|data| data.iter().any(is_syllable_lyrics_item))
    })
}

/// 单个 Apple Music 音节歌词项：`type` 为 `syllable-lyrics`（或缺失/为空），且 `attributes` 带非空 `ttml`/`ttmlLocalizations`。
fn is_syllable_lyrics_item(item: &Value) -> bool {
    if let Some(item_type) = get(item, "type") {
        let matches = match item_type {
            Value::String(text) => text.is_empty() || text.eq_ignore_ascii_case("syllable-lyrics"),
            Value::Null => true,
            _ => false,
        };
        if !matches {
            return false;
        }
    }

    let Some(attributes) = get(item, "attributes") else {
        return false;
    };
    has_non_empty_string(attributes, "ttml")
        || has_non_empty_string(attributes, "ttmlLocalizations")
}

/// Spotify 原始 JSON：`lyrics.syncType` 为字符串且 `lyrics.lines` 为数组。
fn is_spotify_value(root: &Value) -> bool {
    let Some(lyrics) = get(root, "lyrics") else {
        return false;
    };

    get(lyrics, "syncType").is_some_and(Value::is_string)
        && get(lyrics, "lines").is_some_and(Value::is_array)
}

/// Musixmatch 原始 JSON：`message.body.macro_calls` 含任意一个歌词接口。
fn is_musixmatch_value(root: &Value) -> bool {
    let Some(calls) = get(root, "message")
        .and_then(|message| get(message, "body"))
        .and_then(|body| get(body, "macro_calls"))
    else {
        return false;
    };

    get(calls, "track.richsync.get").is_some()
        || get(calls, "track.subtitles.get").is_some()
        || get(calls, "track.lyrics.get").is_some()
}

/// YRC Full 原始 JSON：`yrc.lyric` 为字符串。
fn is_yrc_full_value(root: &Value) -> bool {
    get(root, "yrc").is_some_and(|yrc| get(yrc, "lyric").is_some_and(Value::is_string))
}

/// XML 扫描结果。
struct XmlScan {
    /// 是否存在带 `LyricContent` 属性的 `Lyric_1` 元素（含根元素自身）。
    has_lyric_content: bool,
    /// 根元素是否为 TTML 命名空间下的 `tt`。
    root_is_ttml: bool,
}

/// 判断 XML 文本的歌词原始类型。
///
/// 与上游一致：先看是否存在 `Lyric_1` + `LyricContent`（QRC Full），再看根元素是否为
/// TTML 命名空间下的 `tt`；只在解析失败时才回退到 [`QRC_FULL_FALLBACK`] 正则。
///
/// 与 `XDocument` 的宽松程度存在以下刻意差异（仅影响畸形文档）：
/// - 不要求文档只有一个根元素；`XDocument.Parse` 会因此报错并走正则回退；
/// - 未声明的前缀不会导致解析失败（只能使命名空间解析为 `Unknown`）；
/// - 不处理 DTD 实体展开。正常 QRC Full 与 TTML 文档行为与上游一致。
fn get_xml_type(input: &str) -> LyricsRawTypes {
    if !input.trim_start().starts_with('<') {
        return LyricsRawTypes::Unknown;
    }

    match scan_xml(input) {
        Ok(scan) => {
            if scan.has_lyric_content {
                LyricsRawTypes::QrcFull
            } else if scan.root_is_ttml {
                LyricsRawTypes::Ttml
            } else {
                LyricsRawTypes::Unknown
            }
        }
        Err(()) => {
            if QRC_FULL_FALLBACK.is_match(input) {
                LyricsRawTypes::QrcFull
            } else {
                LyricsRawTypes::Unknown
            }
        }
    }
}

/// 完整扫描 XML 文档，失败时返回 `Err(())`（对应上游的解析异常）。
fn scan_xml(input: &str) -> Result<XmlScan, ()> {
    let mut reader = NsReader::from_str(input);
    let mut scan = XmlScan {
        has_lyric_content: false,
        root_is_ttml: false,
    };
    let mut is_root = true;

    loop {
        let (namespace, event) = reader.read_resolved_event().map_err(|_| ())?;

        match event {
            Event::Start(start) | Event::Empty(start) => {
                let local_name = start.local_name();
                let local_name = local_name.as_ref();

                if local_name.eq_ignore_ascii_case("Lyric_1")
                    && start_has_attribute(&start, "LyricContent")?
                {
                    scan.has_lyric_content = true;
                }

                if is_root {
                    scan.root_is_ttml = local_name.eq_ignore_ascii_case("tt")
                        && matches!(namespace, ResolveResult::Bound(Namespace(ns)) if ns == TTML_NAMESPACE);
                    is_root = false;
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }

    Ok(scan)
}

/// 元素是否带指定局部名的属性（大小写不敏感，忽略命名空间前缀）。
fn start_has_attribute(start: &BytesStart<'_>, name: &str) -> Result<bool, ()> {
    for attribute in start.attributes() {
        let attribute = attribute.map_err(|_| ())?;
        if attribute
            .key
            .local_name()
            .as_ref()
            .eq_ignore_ascii_case(name)
        {
            return Ok(true);
        }
    }
    Ok(false)
}

impl LyricsRawTypes {
    /// 将原始类型映射到歌词类型（对应上游 `GetLyricsType`）。
    pub fn lyrics_type(&self) -> LyricsTypes {
        match self {
            LyricsRawTypes::Unknown => LyricsTypes::Unknown,
            LyricsRawTypes::LyricifySyllable => LyricsTypes::LyricifySyllable,
            LyricsRawTypes::LyricifyLines => LyricsTypes::LyricifyLines,
            LyricsRawTypes::Lrc => LyricsTypes::Lrc,
            LyricsRawTypes::Qrc | LyricsRawTypes::QrcFull => LyricsTypes::Qrc,
            LyricsRawTypes::Krc => LyricsTypes::Krc,
            LyricsRawTypes::Yrc | LyricsRawTypes::YrcFull => LyricsTypes::Yrc,
            LyricsRawTypes::Ttml | LyricsRawTypes::AppleJson => LyricsTypes::Ttml,
            LyricsRawTypes::Spotify => LyricsTypes::Spotify,
            LyricsRawTypes::Musixmatch => LyricsTypes::Musixmatch,
        }
    }

    /// 原始类型的显示名称（对应上游 `GetDisplayName`），未知类型返回空字符串。
    pub fn display_name(&self) -> &'static str {
        match self {
            LyricsRawTypes::Lrc => "LRC",
            LyricsRawTypes::Qrc => "QRC",
            LyricsRawTypes::QrcFull => "QRC (Full)",
            LyricsRawTypes::Krc => "KRC",
            LyricsRawTypes::Yrc => "YRC",
            LyricsRawTypes::YrcFull => "YRC (Full)",
            LyricsRawTypes::Ttml => "TTML",
            LyricsRawTypes::AppleJson => "Apple Music (JSON)",
            LyricsRawTypes::LyricifyLines => "Lyricify Lines",
            LyricsRawTypes::LyricifySyllable => "Lyricify Syllable",
            LyricsRawTypes::Musixmatch => "Musixmatch (JSON)",
            LyricsRawTypes::Spotify => "Spotify (JSON)",
            LyricsRawTypes::Unknown => "",
        }
    }
}

/// 解析原始类型名称（对应上游 `TryParseRawType`）。
///
/// 空白文本、数字开头的文本以及无法识别的名称返回 `None`；先按枚举变体名（大小写不敏感）
/// 解析，失败后再匹配别名表（`QRC (FULL)`/`QRC (XML)`、`YRC (FULL)`/`YRC (JSON)`、
/// `APPLE MUSIC (JSON)`/`APPLE MUSIC JSON`/`APPLE MUSIC`、`LYRICIFY LINE(S)`、
/// `LYRICIFY SYLLABLE(S)`、`MUSIXMATCH (JSON)`/`MUSIXMATCH JSON`/`MUSIXMATCHJSON`、
/// `SPOTIFY (JSON)`/`SPOTIFY JSON`/`SPOTIFYJSON`）。
pub fn try_parse_raw_type(name: &str) -> Option<LyricsRawTypes> {
    let value = name.trim();
    if value.is_empty() {
        return None;
    }

    let upper = value.to_uppercase();

    // 上游使用 `char.IsDigit`（Unicode Nd），此处以 `char::is_numeric` 近似。
    if !value.chars().next().is_some_and(char::is_numeric) {
        let variant = match upper.as_str() {
            "LYRICIFYSYLLABLE" => Some(LyricsRawTypes::LyricifySyllable),
            "LYRICIFYLINES" => Some(LyricsRawTypes::LyricifyLines),
            "LRC" => Some(LyricsRawTypes::Lrc),
            "QRC" => Some(LyricsRawTypes::Qrc),
            "QRCFULL" => Some(LyricsRawTypes::QrcFull),
            "KRC" => Some(LyricsRawTypes::Krc),
            "YRC" => Some(LyricsRawTypes::Yrc),
            "YRCFULL" => Some(LyricsRawTypes::YrcFull),
            "TTML" => Some(LyricsRawTypes::Ttml),
            "APPLEJSON" => Some(LyricsRawTypes::AppleJson),
            "SPOTIFY" => Some(LyricsRawTypes::Spotify),
            "MUSIXMATCH" => Some(LyricsRawTypes::Musixmatch),
            _ => None,
        };
        if variant.is_some() {
            return variant;
        }
    }

    match upper.as_str() {
        "QRC (FULL)" | "QRC (XML)" => Some(LyricsRawTypes::QrcFull),
        "YRC (FULL)" | "YRC (JSON)" => Some(LyricsRawTypes::YrcFull),
        "APPLE MUSIC (JSON)" | "APPLE MUSIC JSON" | "APPLE MUSIC" => {
            Some(LyricsRawTypes::AppleJson)
        }
        "LYRICIFY LINE" | "LYRICIFY LINES" => Some(LyricsRawTypes::LyricifyLines),
        "LYRICIFY SYLLABLE" | "LYRICIFY SYLLABLES" => Some(LyricsRawTypes::LyricifySyllable),
        "MUSIXMATCH (JSON)" | "MUSIXMATCH JSON" | "MUSIXMATCHJSON" => {
            Some(LyricsRawTypes::Musixmatch)
        }
        "SPOTIFY (JSON)" | "SPOTIFY JSON" | "SPOTIFYJSON" => Some(LyricsRawTypes::Spotify),
        _ => None,
    }
}

/// 取原始类型名称的显示名称（对应上游 `GetRawTypeDisplayName`）：
/// 解析成功时返回规范显示名，否则返回去除首尾空白的原名。
pub fn get_raw_type_display_name(name: &str) -> String {
    match try_parse_raw_type(name) {
        Some(raw_type) => raw_type.display_name().to_string(),
        None => name.trim().to_string(),
    }
}

/// 判断歌词文本的歌词类型是否为指定类型（对应上游 `IsLyricsType(string, LyricsTypes)`）。
pub fn is_lyrics_type(lyrics: &str, lyrics_type: LyricsTypes) -> bool {
    lyrics_type != LyricsTypes::Unknown && get_lyrics_types(lyrics).lyrics_type() == lyrics_type
}

/// 判断歌词文本的歌词类型是否在指定类型列表中（对应上游 `IsLyricsType(string, LyricsTypes[])`）。
pub fn is_lyrics_type_any(lyrics: &str, lyrics_types: &[LyricsTypes]) -> bool {
    let lyrics_type = get_lyrics_types(lyrics).lyrics_type();
    lyrics_type != LyricsTypes::Unknown && lyrics_types.contains(&lyrics_type)
}

#[cfg(test)]
mod tests {
    use super::*;

    const LRC: &str = "[00:00.000]Hello World\n[00:01.500]Second line\n[00:03.000]Third";
    const QRC: &str = "[12000,3000]Hel(12000,400)lo(12400,500)\n[15000,2000]World(15000,800)";
    const KRC: &str = "[12000,3000]<0,400,0>Hel<400,500,0>lo\n[15000,2000]<0,900,0>World";
    const YRC: &str = "[12000,3000](12000,400,0)Hel(12400,500,0)lo\n[15000,2000](15000,900,0)World";
    const LYRICIFY_SYLLABLE: &str = "[4]Hel(12000,400)lo(12400,500)\n[0]World(15000,900)";
    const LYRICIFY_LINES: &str = "[12000,3000]Hello world\n[15000,2000]Second line";
    const TTML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<tt xmlns="http://www.w3.org/ns/ttml" xml:lang="en">
  <body><div><p begin="0.358" end="4.933">Lately</p></div></body>
</tt>"#;
    const QRC_FULL_XML: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<QQLyric><QrcInfos><LyricInfo LyricCount="1"/></QrcInfos><Lyric_1 LyricType="1" LyricContent="[0,100]Hi(0,100)"/></QQLyric>"#;

    #[test]
    fn test_detect_line_formats() {
        assert_eq!(get_lyrics_types(LRC), LyricsRawTypes::Lrc);
        assert_eq!(get_lyrics_types(QRC), LyricsRawTypes::Qrc);
        assert_eq!(get_lyrics_types(KRC), LyricsRawTypes::Krc);
        assert_eq!(get_lyrics_types(YRC), LyricsRawTypes::Yrc);
        assert_eq!(
            get_lyrics_types(LYRICIFY_SYLLABLE),
            LyricsRawTypes::LyricifySyllable
        );
        assert_eq!(
            get_lyrics_types(LYRICIFY_LINES),
            LyricsRawTypes::LyricifyLines
        );
    }

    #[test]
    fn test_detect_unknown_and_marker() {
        assert_eq!(get_lyrics_types(""), LyricsRawTypes::Unknown);
        assert_eq!(get_lyrics_types("   \n\t  "), LyricsRawTypes::Unknown);
        assert_eq!(
            get_lyrics_types("just some plain text"),
            LyricsRawTypes::Unknown
        );
        assert_eq!(
            get_lyrics_types("[type:LyricifyLines]\nHello world"),
            LyricsRawTypes::LyricifyLines
        );
        assert_eq!(
            get_lyrics_types("[TYPE:lyricifylines]\nHello world"),
            LyricsRawTypes::LyricifyLines
        );
    }

    #[test]
    fn test_is_lyricify_lines_rejects_syllable_lyrics() {
        assert!(is_lyricify_lines(LYRICIFY_LINES));
        assert!(is_lyricify_lines("[12000,3000]Hello\n[type:LyricifyLines]"));
        // 逐字歌词带有 [开始,结束] 行头，但必须保持不变判为逐字格式。
        assert!(!is_lyricify_lines(QRC));
        assert_eq!(get_lyrics_types(QRC), LyricsRawTypes::Qrc);
        assert!(!is_lyricify_lines(YRC));
        assert!(!is_lyricify_lines(KRC));
        assert!(!is_lyricify_lines("plain text"));
    }

    #[test]
    fn test_detect_xml_formats() {
        assert_eq!(get_lyrics_types(TTML), LyricsRawTypes::Ttml);
        assert!(is_ttml(TTML));
        assert_eq!(get_lyrics_types(QRC_FULL_XML), LyricsRawTypes::QrcFull);
        assert!(is_qrc_full(QRC_FULL_XML));
        assert_eq!(
            get_lyrics_types("<foo><bar/></foo>"),
            LyricsRawTypes::Unknown
        );
        // 非 TTML 命名空间的 tt 根元素不算 TTML。
        assert_eq!(
            get_lyrics_types("<tt><body/></tt>"),
            LyricsRawTypes::Unknown
        );
        assert_eq!(
            get_lyrics_types("<tt xmlns=\"http://www.w3.org/ns/ttml#metadata\"><body/></tt>"),
            LyricsRawTypes::Unknown
        );
        // 解析失败时回退到 QRC Full 特征正则。
        assert_eq!(
            get_lyrics_types("<Lyric_1 LyricContent=\"[0,100]Hi(0,100)\"><unclosed>"),
            LyricsRawTypes::QrcFull
        );
        assert_eq!(
            get_lyrics_types("<broken><unclosed>"),
            LyricsRawTypes::Unknown
        );
    }

    #[test]
    fn test_detect_json_formats() {
        let apple = r#"{"data":[{"id":"1","type":"syllable-lyrics","attributes":{"ttml":"<tt/>","ttmlLocalizations":null}}]}"#;
        assert_eq!(get_lyrics_types(apple), LyricsRawTypes::AppleJson);
        assert!(is_apple_json(apple));

        let apple_relationship = r#"{"data":[{"id":"1","type":"songs","relationships":{"syllable-lyrics":{"data":[{"type":"syllable-lyrics","attributes":{"ttmlLocalizations":"{\"en\":\"<tt/>\"}"}}]}}}]}"#;
        assert_eq!(
            get_lyrics_types(apple_relationship),
            LyricsRawTypes::AppleJson
        );

        let spotify = r#"{"lyrics":{"syncType":"LINE_SYNCED","lines":[{"startTimeMs":"1000"}]}}"#;
        assert_eq!(get_lyrics_types(spotify), LyricsRawTypes::Spotify);
        assert!(is_spotify(spotify));

        let musixmatch = r#"{"message":{"header":{},"body":{"macro_calls":{"track.richsync.get":{"message":{}}}}}}"#;
        assert_eq!(get_lyrics_types(musixmatch), LyricsRawTypes::Musixmatch);
        assert!(is_musixmatch(musixmatch));

        let yrc_full =
            r#"{"code":200,"yrc":{"version":18,"lyric":"[12000,3000](12000,400,0)Hel"}}"#;
        assert_eq!(get_lyrics_types(yrc_full), LyricsRawTypes::YrcFull);
        assert!(is_yrc_full(yrc_full));

        // 键名大小写不敏感。
        assert_eq!(
            get_lyrics_types(r#"{"Lyrics":{"SyncType":"LINE_SYNCED","Lines":[]}}"#),
            LyricsRawTypes::Spotify
        );

        // 形状不符的 JSON 与非法 JSON 均为 Unknown。
        assert_eq!(get_lyrics_types(r#"{"data":[]}"#), LyricsRawTypes::Unknown);
        assert_eq!(
            get_lyrics_types(r#"{"lyrics":{"syncType":1,"lines":[]}}"#),
            LyricsRawTypes::Unknown
        );
        assert_eq!(
            get_lyrics_types(r#"{"yrc":{"lyric":123}}"#),
            LyricsRawTypes::Unknown
        );
        assert_eq!(get_lyrics_types("{not json"), LyricsRawTypes::Unknown);
    }

    #[test]
    fn test_lyrics_type_mapping() {
        assert_eq!(LyricsRawTypes::Unknown.lyrics_type(), LyricsTypes::Unknown);
        assert_eq!(LyricsRawTypes::QrcFull.lyrics_type(), LyricsTypes::Qrc);
        assert_eq!(LyricsRawTypes::Qrc.lyrics_type(), LyricsTypes::Qrc);
        assert_eq!(LyricsRawTypes::YrcFull.lyrics_type(), LyricsTypes::Yrc);
        assert_eq!(LyricsRawTypes::AppleJson.lyrics_type(), LyricsTypes::Ttml);
        assert_eq!(LyricsRawTypes::Ttml.lyrics_type(), LyricsTypes::Ttml);
        assert_eq!(
            LyricsRawTypes::Musixmatch.lyrics_type(),
            LyricsTypes::Musixmatch
        );
        assert_eq!(LyricsRawTypes::Spotify.lyrics_type(), LyricsTypes::Spotify);
        assert_eq!(
            LyricsRawTypes::LyricifySyllable.lyrics_type(),
            LyricsTypes::LyricifySyllable
        );
        assert_eq!(
            LyricsRawTypes::LyricifyLines.lyrics_type(),
            LyricsTypes::LyricifyLines
        );
    }

    #[test]
    fn test_try_parse_raw_type() {
        assert_eq!(try_parse_raw_type("lrc"), Some(LyricsRawTypes::Lrc));
        assert_eq!(
            try_parse_raw_type("  LyricifySyllable  "),
            Some(LyricsRawTypes::LyricifySyllable)
        );
        assert_eq!(try_parse_raw_type("qrcFull"), Some(LyricsRawTypes::QrcFull));
        assert_eq!(
            try_parse_raw_type("QRC (FULL)"),
            Some(LyricsRawTypes::QrcFull)
        );
        assert_eq!(
            try_parse_raw_type("qrc (xml)"),
            Some(LyricsRawTypes::QrcFull)
        );
        assert_eq!(
            try_parse_raw_type("YRC (JSON)"),
            Some(LyricsRawTypes::YrcFull)
        );
        assert_eq!(
            try_parse_raw_type("APPLE MUSIC"),
            Some(LyricsRawTypes::AppleJson)
        );
        assert_eq!(
            try_parse_raw_type("Apple Music JSON"),
            Some(LyricsRawTypes::AppleJson)
        );
        assert_eq!(
            try_parse_raw_type("lyricify line"),
            Some(LyricsRawTypes::LyricifyLines)
        );
        assert_eq!(
            try_parse_raw_type("LYRICIFY LINES"),
            Some(LyricsRawTypes::LyricifyLines)
        );
        assert_eq!(
            try_parse_raw_type("Lyricify Syllables"),
            Some(LyricsRawTypes::LyricifySyllable)
        );
        assert_eq!(
            try_parse_raw_type("MUSIXMATCHJSON"),
            Some(LyricsRawTypes::Musixmatch)
        );
        assert_eq!(
            try_parse_raw_type("spotify (json)"),
            Some(LyricsRawTypes::Spotify)
        );
        assert_eq!(
            try_parse_raw_type("SpotifyJSON"),
            Some(LyricsRawTypes::Spotify)
        );

        assert_eq!(try_parse_raw_type(""), None);
        assert_eq!(try_parse_raw_type("   "), None);
        assert_eq!(try_parse_raw_type("Unknown"), None);
        assert_eq!(try_parse_raw_type("12"), None);
        assert_eq!(try_parse_raw_type("nonsense"), None);
    }

    #[test]
    fn test_display_names() {
        assert_eq!(LyricsRawTypes::QrcFull.display_name(), "QRC (Full)");
        assert_eq!(LyricsRawTypes::YrcFull.display_name(), "YRC (Full)");
        assert_eq!(
            LyricsRawTypes::AppleJson.display_name(),
            "Apple Music (JSON)"
        );
        assert_eq!(LyricsRawTypes::Unknown.display_name(), "");

        assert_eq!(get_raw_type_display_name("  qrc (xml)  "), "QRC (Full)");
        assert_eq!(
            get_raw_type_display_name("Apple Music"),
            "Apple Music (JSON)"
        );
        assert_eq!(get_raw_type_display_name("  未知格式  "), "未知格式");
        assert_eq!(get_raw_type_display_name("   "), "");
    }

    #[test]
    fn test_is_lyrics_type() {
        assert!(is_lyrics_type(QRC, LyricsTypes::Qrc));
        assert!(is_lyrics_type(QRC_FULL_XML, LyricsTypes::Qrc));
        assert!(is_lyrics_type(TTML, LyricsTypes::Ttml));
        assert!(is_lyrics_type(LRC, LyricsTypes::Lrc));
        assert!(!is_lyrics_type(QRC, LyricsTypes::Ttml));
        assert!(!is_lyrics_type("plain text", LyricsTypes::Lrc));
        assert!(!is_lyrics_type(LRC, LyricsTypes::Unknown));

        assert!(is_lyrics_type_any(
            LRC,
            &[LyricsTypes::Ttml, LyricsTypes::Lrc]
        ));
        assert!(!is_lyrics_type_any(LRC, &[LyricsTypes::Ttml]));
        assert!(!is_lyrics_type_any(LRC, &[]));
        assert!(!is_lyrics_type_any("plain text", &[LyricsTypes::Lrc]));
    }
}
