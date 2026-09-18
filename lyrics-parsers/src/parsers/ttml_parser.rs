use lyrics_core::models::*;
use quick_xml::Reader;
use quick_xml::events::Event;
use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

static SPACES_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s+").unwrap());
static OPEN_BRACKET_SPACE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\s+(\(|（)").unwrap());
static CLOSE_BRACKET_SPACE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(\)|）)\s+").unwrap());
static BRACKET_CONTENT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\(([^)]*)\)|（([^）]*)）").unwrap());

struct Agent {
    id: String,
    agent_type: String,
}

/// 模拟 Apple Music 的有状态 agent 对齐。person 类型 agent 在演唱者切换时交替左右侧，
/// group 与 other 类型 agent 则分别固定为正常/翻转对齐。
struct AgentAlignmentState {
    agents: HashMap<String, Agent>,
    current_person_id: Option<String>,
    is_flipped: bool,
}

impl AgentAlignmentState {
    fn new(agents: Vec<Agent>) -> Self {
        let mut map: HashMap<String, Agent> = HashMap::new();
        for agent in agents {
            map.entry(agent.id.clone()).or_insert(agent);
        }
        Self {
            agents: map,
            current_person_id: None,
            is_flipped: false,
        }
    }

    /// 返回该行 agent 对应的对齐方式，并在过程中更新内部的演唱者状态。
    fn get_alignment(&mut self, agent_id: Option<&str>) -> LyricsAlignment {
        if self.agents.is_empty() {
            return LyricsAlignment::Unspecified;
        }

        // Apple 将单演唱者模式与每行的 normal/flipped 值分开暴露，
        // 而本模型只暴露 alignment，因此单演唱者保持正常（左侧）对齐。
        if self.agents.len() == 1 {
            return LyricsAlignment::Left;
        }

        if let Some(id) = agent_id.map(str::trim).filter(|id| !id.is_empty()) {
            if let Some(agent) = self.agents.get(id) {
                if agent.agent_type.eq_ignore_ascii_case("person") {
                    match &self.current_person_id {
                        None => self.current_person_id = Some(agent.id.clone()),
                        Some(current) if current != &agent.id => {
                            self.is_flipped = !self.is_flipped;
                            self.current_person_id = Some(agent.id.clone());
                        }
                        _ => {}
                    }
                } else if agent.agent_type.eq_ignore_ascii_case("group") {
                    return LyricsAlignment::Left;
                } else if agent.agent_type.eq_ignore_ascii_case("other") {
                    return LyricsAlignment::Right;
                }
            }
        }

        // 缺失的 agent 和未知类型在 Apple Music 中继承当前演唱者一侧，
        // 而不是丢失对唱对齐。
        if self.is_flipped {
            LyricsAlignment::Right
        } else {
            LyricsAlignment::Left
        }
    }
}

#[derive(Clone)]
struct TranslationValue {
    text: String,
    span_texts: Vec<String>,
}

enum XmlNode {
    Element {
        name: String,
        attributes: Vec<(String, String)>,
        children: Vec<XmlNode>,
    },
    Text(String),
}

fn local_name(qname: &str) -> &str {
    qname.rsplit(':').next().unwrap_or(qname)
}

fn attr_value(attrs: &[(String, String)], name: &str) -> Option<String> {
    for (k, v) in attrs {
        if k == name || local_name(k) == name {
            return Some(v.clone());
        }
    }
    None
}

fn build_tree(reader: &mut Reader<&[u8]>, end_tag: &[u8]) -> Vec<XmlNode> {
    let mut nodes = Vec::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let attributes: Vec<(String, String)> = e
                    .attributes()
                    .flatten()
                    .map(|a| {
                        (
                            String::from_utf8_lossy(a.key.as_ref()).to_string(),
                            String::from_utf8_lossy(&a.value).to_string(),
                        )
                    })
                    .collect();
                let children = build_tree(reader, e.name().as_ref());
                nodes.push(XmlNode::Element {
                    name,
                    attributes,
                    children,
                });
            }
            Ok(Event::Empty(ref e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let attributes: Vec<(String, String)> = e
                    .attributes()
                    .flatten()
                    .map(|a| {
                        (
                            String::from_utf8_lossy(a.key.as_ref()).to_string(),
                            String::from_utf8_lossy(&a.value).to_string(),
                        )
                    })
                    .collect();
                nodes.push(XmlNode::Element {
                    name,
                    attributes,
                    children: Vec::new(),
                });
            }
            Ok(Event::Text(ref e)) => {
                let text = e.unescape().unwrap_or_default().to_string();
                if !text.is_empty() {
                    nodes.push(XmlNode::Text(text));
                }
            }
            Ok(Event::End(ref e)) => {
                if e.name().as_ref() == end_tag {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    nodes
}

/// 解析 TTML（Timed Text Markup Language）格式歌词，支持逐音节同步、翻译和背景人声，返回 [`LyricsData`]。
pub fn parse(ttml: &str) -> LyricsData {
    parse_with_options(ttml, true)
}

/// 解析 TTML 歌词，`use_embedded_simplified_chinese_lyrics` 控制是否优先采用内嵌的简体中文转换：
/// 为 `false` 且原文为繁体中文（zh-Hant）时，保留原文语言并忽略内嵌的简体中文 replacement 翻译。
pub fn parse_with_options(ttml: &str, use_embedded_simplified_chinese_lyrics: bool) -> LyricsData {
    let mut data = LyricsData {
        track_metadata: Some(TrackMetadata::new()),
        file: Some(FileInfo {
            lyrics_type: LyricsTypes::Ttml,
            sync_types: SyncTypes::SyllableSynced,
            additional_info: Some(AdditionalFileInfo::new_general()),
        }),
        lines: Some(Vec::new()),
        writers: None,
    };

    if ttml.trim().is_empty() {
        return data;
    }

    let mut reader = Reader::from_str(ttml);
    let doc = build_tree(&mut reader, b"");

    parse_itunes_metadata(&doc, &mut data);
    let translations = parse_translations(&doc);
    let agents = parse_agents(&doc);
    let mut agent_alignment = AgentAlignmentState::new(agents);
    let root_language = root_element(&doc).and_then(|node| element_attr(node, "lang"));
    let root_is_zh_hant = is_language(root_language.as_deref(), "zh-Hant");
    // 整篇固定：繁体正文且未启用内嵌简体时，才需要过滤掉 zh-Hans 的 replacement。
    let keep_all_replacement_langs = use_embedded_simplified_chinese_lyrics || !root_is_zh_hant;
    if !use_embedded_simplified_chinese_lyrics && root_is_zh_hant {
        if let Some(lang) = &root_language {
            data.track_metadata
                .get_or_insert_with(TrackMetadata::new)
                .language = Some(vec![lang.trim().to_string()]);
        }
    }

    let p_nodes = find_elements(&doc, "p");

    // 按开始时间（缺失则视为最后）预计算每行对齐方式，agent 状态随之顺序推进。
    let mut ordered_indices: Vec<usize> = (0..p_nodes.len()).collect();
    ordered_indices
        .sort_by_cached_key(|&index| get_line_start_time(p_nodes[index]).unwrap_or(i32::MAX));
    let mut alignments = vec![LyricsAlignment::Unspecified; p_nodes.len()];
    for index in ordered_indices {
        let agent_id = element_attr(p_nodes[index], "agent");
        alignments[index] = agent_alignment.get_alignment(agent_id.as_deref());
    }

    let mut any_line_synced = false;
    let mut any_syllable_synced = false;

    for (index, p) in p_nodes.iter().enumerate() {
        let (p_attrs, p_children) = match p {
            XmlNode::Element {
                attributes,
                children,
                ..
            } => (attributes, children),
            _ => continue,
        };

        let key = attr_value(p_attrs, "key");

        let mut main_syllables = Vec::new();
        let mut bg_syllables = Vec::new();
        collect_syllables_from_nodes(p_children, &mut main_syllables, &mut bg_syllables, false);

        let mut line: Option<LineInfo> = None;

        if !main_syllables.is_empty() {
            any_syllable_synced = true;
            line = Some(LineInfo::new_syllable(to_syllable_items(main_syllables)));
        } else {
            let begin = attr_value(p_attrs, "begin").and_then(|v| parse_time_ms(&v));
            let end = attr_value(p_attrs, "end").and_then(|v| parse_time_ms(&v));
            let text = normalize_text(&element_text_value(p_children))
                .trim()
                .to_string();

            if !text.is_empty() {
                if let Some(begin_ms) = begin {
                    any_line_synced = true;
                    line = Some(LineInfo::new_line(text, Some(begin_ms), end));
                }
            }
        }

        let mut line = match line {
            Some(l) => l,
            None => continue,
        };

        let align = alignments[index];
        line.set_alignment(align);

        if !bg_syllables.is_empty() {
            normalize_bracket_inner_spacing_for_bg(&mut bg_syllables);
            let mut sub = LineInfo::new_syllable(to_syllable_items(bg_syllables));
            sub.set_alignment(align);
            line.set_sub_line(Some(Box::new(sub)));
        }

        if let Some(ref key_val) = key {
            if let Some(tmap) = translations.get(key_val) {
                let replacement = tmap
                    .iter()
                    .filter(|((t, _), _)| t.eq_ignore_ascii_case("replacement"))
                    .filter(|((_, lang), _)| {
                        keep_all_replacement_langs || !is_language(Some(lang.as_str()), "zh-Hans")
                    })
                    .map(|(_, v)| v)
                    .find(|v| !v.text.trim().is_empty());

                if let Some(rep) = replacement {
                    line = apply_replacement(line, rep);
                    line.set_alignment(align);
                }

                let subtitles: Vec<_> = tmap
                    .iter()
                    .filter(|((t, _), _)| t.eq_ignore_ascii_case("subtitle"))
                    .map(|((_, lang), val)| (lang.clone(), val.text.clone()))
                    .collect();

                if !subtitles.is_empty() {
                    line = apply_subtitle_translations(line, &subtitles);
                    line.set_alignment(align);
                }
            }
        }

        data.lines.as_mut().unwrap().push(line);
    }

    if any_line_synced && any_syllable_synced {
        data.file.as_mut().unwrap().sync_types = SyncTypes::MixedSynced;
    } else if any_line_synced {
        data.file.as_mut().unwrap().sync_types = SyncTypes::LineSynced;
    } else if any_syllable_synced {
        data.file.as_mut().unwrap().sync_types = SyncTypes::SyllableSynced;
    } else {
        data.file.as_mut().unwrap().sync_types = SyncTypes::Unknown;
    }

    data
}

fn find_elements<'a>(nodes: &'a [XmlNode], local: &str) -> Vec<&'a XmlNode> {
    let mut result = Vec::new();
    for node in nodes {
        match node {
            XmlNode::Element { name, children, .. } => {
                if local_name(name) == local {
                    result.push(node);
                }
                result.extend(find_elements(children, local));
            }
            XmlNode::Text(_) => {}
        }
    }
    result
}

fn find_first_element<'a>(nodes: &'a [XmlNode], local: &str) -> Option<&'a XmlNode> {
    for node in nodes {
        match node {
            XmlNode::Element { name, children, .. } => {
                if local_name(name) == local {
                    return Some(node);
                }
                if let Some(found) = find_first_element(children, local) {
                    return Some(found);
                }
            }
            XmlNode::Text(_) => {}
        }
    }
    None
}

fn element_text_value(nodes: &[XmlNode]) -> String {
    let mut s = String::new();
    for node in nodes {
        match node {
            XmlNode::Text(t) => s.push_str(t),
            XmlNode::Element { children, .. } => s.push_str(&element_text_value(children)),
        }
    }
    s
}

fn collect_syllables_from_nodes(
    nodes: &[XmlNode],
    main: &mut Vec<SyllableInfo>,
    bg: &mut Vec<SyllableInfo>,
    is_bg_context: bool,
) {
    for node in nodes {
        match node {
            XmlNode::Text(text) => {
                if is_bg_context {
                    append_to_previous(text, bg);
                } else {
                    append_to_previous(text, main);
                }
            }
            XmlNode::Element {
                name,
                attributes,
                children,
            } => {
                if local_name(name) == "span" {
                    let role = attr_value(attributes, "role");
                    let is_bg = is_bg_context
                        || role
                            .as_deref()
                            .is_some_and(|r| r.eq_ignore_ascii_case("x-bg"));

                    let begin_attr = attr_value(attributes, "begin");
                    if let Some(begin_str) = begin_attr {
                        let begin_ms = parse_time_ms(&begin_str);
                        let end_ms = attr_value(attributes, "end")
                            .and_then(|v| parse_time_ms(&v))
                            .or(begin_ms);

                        if let Some(b) = begin_ms {
                            let mut raw = normalize_text(&element_text_value(children));
                            if is_bg {
                                raw = move_leading_spaces_to_previous(&raw, bg);
                                if !raw.is_empty() {
                                    bg.push(SyllableInfo::new(raw, b, end_ms.unwrap_or(b)));
                                }
                            } else {
                                raw = move_leading_spaces_to_previous(&raw, main);
                                if !raw.is_empty() {
                                    main.push(SyllableInfo::new(raw, b, end_ms.unwrap_or(b)));
                                }
                            }
                        }
                    } else {
                        collect_syllables_from_nodes(children, main, bg, is_bg);
                    }
                } else {
                    collect_syllables_from_nodes(children, main, bg, is_bg_context);
                }
            }
        }
    }
}

fn append_to_previous(text: &str, list: &mut [SyllableInfo]) {
    if text.is_empty() || list.is_empty() {
        return;
    }
    let normalized = normalize_text(text);
    if normalized.is_empty() {
        return;
    }
    list.last_mut().unwrap().text.push_str(&normalized);
}

fn move_leading_spaces_to_previous(text: &str, list: &mut [SyllableInfo]) -> String {
    if text.is_empty() || list.is_empty() {
        return text.to_string();
    }
    let trimmed = text.trim_start();
    let leading_len = text.len() - trimmed.len();
    if leading_len == 0 {
        return text.to_string();
    }
    list.last_mut().unwrap().text.push_str(&text[..leading_len]);
    trimmed.to_string()
}

fn parse_agents(nodes: &[XmlNode]) -> Vec<Agent> {
    let mut agents = Vec::new();
    for node in nodes {
        match node {
            XmlNode::Element {
                name, attributes, ..
            } if local_name(name) == "agent" => {
                let id = attr_value(attributes, "id").unwrap_or_default();
                let id = id.trim().to_string();
                if id.is_empty() {
                    continue;
                }
                let agent_type = attr_value(attributes, "type")
                    .unwrap_or_default()
                    .trim()
                    .to_string();
                agents.push(Agent { id, agent_type });
            }
            XmlNode::Element { children, .. } => {
                agents.extend(parse_agents(children));
            }
            _ => {}
        }
    }
    agents
}

fn element_attr(node: &XmlNode, name: &str) -> Option<String> {
    match node {
        XmlNode::Element { attributes, .. } => attr_value(attributes, name),
        XmlNode::Text(_) => None,
    }
}

/// 返回文档的根元素，等价于 C# 的 `XDocument.Root`（顶层文本节点不计入）。
fn root_element(nodes: &[XmlNode]) -> Option<&XmlNode> {
    nodes
        .iter()
        .find(|node| matches!(node, XmlNode::Element { .. }))
}

fn get_line_start_time(node: &XmlNode) -> Option<i32> {
    let (attributes, children) = match node {
        XmlNode::Element {
            attributes,
            children,
            ..
        } => (attributes, children),
        XmlNode::Text(_) => return None,
    };

    if let Some(start) = attr_value(attributes, "begin").and_then(|value| parse_time_ms(&value)) {
        return Some(start);
    }

    find_elements(children, "span")
        .iter()
        .filter_map(|span| element_attr(span, "begin").and_then(|value| parse_time_ms(&value)))
        .min()
}

fn parse_itunes_metadata(doc: &[XmlNode], data: &mut LyricsData) {
    let metadata = find_first_element(doc, "metadata");
    let meta = find_first_element(doc, "iTunesMetadata");

    parse_track_metadata(doc, metadata, meta, data);

    let meta = match meta {
        Some(m) => m,
        None => return,
    };

    let (meta_attrs, meta_children) = match meta {
        XmlNode::Element {
            attributes,
            children,
            ..
        } => (attributes, children),
        _ => return,
    };

    if let Some(leading) = attr_value(meta_attrs, "leadingSilence") {
        if !leading.trim().is_empty() {
            if let Some(AdditionalFileInfo::General { attributes }) =
                data.file.as_mut().unwrap().additional_info.as_mut()
            {
                attributes.push(("leadingSilence".to_string(), leading));
            }
        }
    }

    let writers: Vec<String> = find_elements(meta_children, "songwriter")
        .iter()
        .map(|e| {
            if let XmlNode::Element { children, .. } = e {
                element_text_value(children).trim().to_string()
            } else {
                String::new()
            }
        })
        .filter(|s| !s.is_empty())
        .collect();

    if !writers.is_empty() {
        data.writers = Some(writers);
    }
}

static METADATA_KEY_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[^A-Za-z0-9]").unwrap());

fn parse_track_metadata(
    doc: &[XmlNode],
    metadata: Option<&XmlNode>,
    itunes_metadata: Option<&XmlNode>,
    data: &mut LyricsData,
) {
    let track = data.track_metadata.get_or_insert_with(TrackMetadata::new);

    let mut roots: Vec<&XmlNode> = Vec::new();
    if let Some(m) = itunes_metadata {
        roots.push(m);
    }
    if let Some(m) = metadata {
        roots.push(m);
    }
    for node in doc {
        roots.push(node);
    }

    set_if_empty(
        &mut track.title,
        find_metadata_value(
            &roots,
            &["title", "trackTitle", "songTitle", "songName", "musicName"],
        ),
    );
    set_if_empty(
        &mut track.artist,
        find_metadata_value(
            &roots,
            &[
                "artist",
                "artists",
                "artistName",
                "songArtist",
                "singer",
                "performer",
                "performers",
            ],
        ),
    );
    set_if_empty(
        &mut track.album,
        find_metadata_value(&roots, &["album", "albumName"]),
    );
    set_if_empty(
        &mut track.album_artist,
        find_metadata_value(&roots, &["albumArtist", "albumArtistName"]),
    );
    set_if_empty(&mut track.isrc, find_metadata_value(&roots, &["isrc"]));

    if track.duration_ms.is_none() {
        let body_dur = find_elements(doc, "body").iter().find_map(|b| {
            if let XmlNode::Element { attributes, .. } = b {
                attr_value(attributes, "dur")
            } else {
                None
            }
        });

        let duration = body_dur
            .as_deref()
            .and_then(parse_time_ms)
            .or_else(|| parse_metadata_duration(&roots));
        if let Some(d) = duration {
            track.duration_ms = Some(d);
        }
    }

    let root_lang = root_element(doc).and_then(|node| element_attr(node, "lang"));

    let simplified_replacement_lang = find_elements(doc, "translation")
        .iter()
        .find(|node| {
            if let XmlNode::Element { attributes, .. } = node {
                let typ = attr_value(attributes, "type").unwrap_or_default();
                typ.trim().eq_ignore_ascii_case("replacement")
            } else {
                false
            }
        })
        .and_then(|node| {
            if let XmlNode::Element { attributes, .. } = node {
                attr_value(attributes, "lang")
            } else {
                None
            }
        })
        .filter(|x| {
            is_language(root_lang.as_deref(), "zh-Hant") && is_language(Some(x), "zh-Hans")
        });

    if simplified_replacement_lang.is_some() {
        track.language = Some(vec!["zh-Hans".to_string()]);
    } else if let Some(ref rl) = root_lang {
        let trimmed = rl.trim().to_string();
        if !trimmed.is_empty() {
            track.language = Some(vec![trimmed]);
        }
    }
}

fn set_if_empty(target: &mut Option<String>, new_value: Option<String>) {
    if target.as_ref().is_none_or(|v| v.trim().is_empty()) {
        if let Some(v) = new_value {
            let trimmed = v.trim().to_string();
            if !trimmed.is_empty() {
                *target = Some(trimmed);
            }
        }
    }
}

fn find_metadata_value(roots: &[&XmlNode], keys: &[&str]) -> Option<String> {
    let normalized_keys: std::collections::HashSet<String> =
        keys.iter().map(|k| normalize_metadata_key(k)).collect();

    for root in roots {
        for element in descendants_and_self(root) {
            if let XmlNode::Element {
                name,
                attributes,
                children,
            } = element
            {
                for (attr_name, attr_val) in attributes.iter() {
                    if !attr_val.trim().is_empty()
                        && normalized_keys.contains(&normalize_metadata_key(attr_name))
                    {
                        return Some(attr_val.trim().to_string());
                    }
                }

                let key = get_metadata_key(attributes);
                if let Some(ref k) = key {
                    if !k.trim().is_empty() && normalized_keys.contains(&normalize_metadata_key(k))
                    {
                        let value = get_metadata_value(attributes, children);
                        if let Some(ref v) = value {
                            if !v.trim().is_empty() {
                                return Some(v.trim().to_string());
                            }
                        }
                    }
                }

                if normalized_keys.contains(&normalize_metadata_key(name)) {
                    let value = get_element_own_text(children);
                    if !value.trim().is_empty() {
                        return Some(value.trim().to_string());
                    }
                }
            }
        }
    }

    None
}

fn descendants_and_self(node: &XmlNode) -> Vec<&XmlNode> {
    let mut result = vec![node];
    if let XmlNode::Element { children, .. } = node {
        for child in children {
            result.extend(descendants_and_self(child));
        }
    }
    result
}

fn parse_metadata_duration(roots: &[&XmlNode]) -> Option<i32> {
    for key in &["durationMs", "durationInMillis", "duration", "length"] {
        let value = find_metadata_value(roots, &[key])?;
        let trimmed = value.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Ok(int_val) = trimmed.parse::<i32>() {
            if key.contains("Ms") || key.contains("Millis") || int_val > 10000 {
                return Some(int_val);
            }
        }

        if let Some(ms) = parse_time_ms(trimmed) {
            return Some(ms);
        }
    }
    None
}

fn get_metadata_key(attrs: &[(String, String)]) -> Option<String> {
    attr_value(attrs, "key")
        .or_else(|| attr_value(attrs, "name"))
        .or_else(|| attr_value(attrs, "property"))
}

fn get_metadata_value(attrs: &[(String, String)], children: &[XmlNode]) -> Option<String> {
    attr_value(attrs, "value")
        .or_else(|| attr_value(attrs, "content"))
        .or_else(|| {
            let text = get_element_own_text(children);
            if text.is_empty() { None } else { Some(text) }
        })
}

fn get_element_own_text(nodes: &[XmlNode]) -> String {
    nodes
        .iter()
        .filter_map(|n| {
            if let XmlNode::Text(t) = n {
                Some(t.as_str())
            } else {
                None
            }
        })
        .collect::<Vec<&str>>()
        .concat()
}

fn normalize_metadata_key(key: &str) -> String {
    METADATA_KEY_RE.replace_all(key, "").to_string()
}

fn is_language(lang: Option<&str>, language_prefix: &str) -> bool {
    let lang = match lang {
        Some(l) if !l.trim().is_empty() => l.trim(),
        _ => return false,
    };
    lang.eq_ignore_ascii_case(language_prefix)
        || lang
            .to_lowercase()
            .starts_with(&format!("{}-", language_prefix).to_lowercase())
}

fn parse_translations(
    doc: &[XmlNode],
) -> HashMap<String, HashMap<(String, String), TranslationValue>> {
    let mut result: HashMap<String, HashMap<(String, String), TranslationValue>> = HashMap::new();

    let translation_nodes = find_elements(doc, "translation");
    for node in translation_nodes {
        let (node_attrs, node_children) = match node {
            XmlNode::Element {
                attributes,
                children,
                ..
            } => (attributes, children),
            _ => continue,
        };

        let typ = attr_value(node_attrs, "type")
            .unwrap_or_default()
            .trim()
            .to_string();
        let lang = attr_value(node_attrs, "lang")
            .unwrap_or_default()
            .trim()
            .to_string();

        for child in node_children {
            if let XmlNode::Element {
                name,
                attributes,
                children,
            } = child
            {
                if local_name(name) != "text" {
                    continue;
                }
                let key = match attr_value(attributes, "for") {
                    Some(k) if !k.trim().is_empty() => k.trim().to_string(),
                    _ => continue,
                };
                let value = normalize_text(&element_text_value(children))
                    .trim()
                    .to_string();
                if value.is_empty() {
                    continue;
                }

                result.entry(key).or_default().insert(
                    (typ.clone(), lang.clone()),
                    TranslationValue {
                        text: value,
                        span_texts: extract_timed_span_texts(children),
                    },
                );
            }
        }
    }

    result
}

fn normalize_text(text: &str) -> String {
    text.replace(['\r', '\n'], "")
}

fn normalize_spaces(s: &str) -> String {
    SPACES_RE.replace_all(s.trim(), " ").to_string()
}

fn normalize_lang_key(lang: &str) -> String {
    let lang = lang.trim();
    if lang.is_empty() {
        return "und".to_string();
    }
    if lang.to_lowercase().starts_with("zh") {
        return "zh".to_string();
    }
    lang.to_string()
}

fn normalize_bracket_inner_spacing_for_bg(syllables: &mut [SyllableInfo]) {
    if syllables.is_empty() {
        return;
    }
    let first = &mut syllables[0].text;
    *first = OPEN_BRACKET_SPACE_RE.replace_all(first, "$1").to_string();

    let last = syllables.last_mut().unwrap();
    last.text = CLOSE_BRACKET_SPACE_RE
        .replace_all(&last.text, "$1")
        .to_string();
}

fn extract_timed_span_texts(nodes: &[XmlNode]) -> Vec<String> {
    nodes
        .iter()
        .filter_map(|node| {
            if let XmlNode::Element {
                name,
                attributes,
                children,
            } = node
            {
                if local_name(name) == "span"
                    && attr_value(attributes, "begin").is_some_and(|b| !b.trim().is_empty())
                {
                    let text = normalize_text(&element_text_value(children));
                    if !text.is_empty() {
                        return Some(text);
                    }
                }
            }
            None
        })
        .collect()
}

fn replace_syllable_line_parts(
    syllables: &[SyllableInfo],
    new_parts: &[String],
) -> Option<Vec<SyllableInfo>> {
    if new_parts.is_empty() || new_parts.len() != syllables.len() {
        return None;
    }

    let new_syllables: Vec<SyllableInfo> = syllables
        .iter()
        .zip(new_parts.iter())
        .map(|(syllable, new_part)| {
            let leading: String = syllable
                .text
                .chars()
                .take_while(|c| c.is_whitespace())
                .collect();
            let trailing: String = syllable
                .text
                .chars()
                .rev()
                .take_while(|c| c.is_whitespace())
                .collect::<String>()
                .chars()
                .rev()
                .collect();
            let replacement = normalize_text(new_part);
            SyllableInfo::new(
                format!("{}{}{}", leading, replacement, trailing),
                syllable.start_time,
                syllable.end_time,
            )
        })
        .collect();

    Some(new_syllables)
}

fn apply_replacement(mut line: LineInfo, replacement: &TranslationValue) -> LineInfo {
    let replacement_text = normalize_text(&replacement.text);
    let existing_sub = line.sub_line().cloned();

    let (main_replacement, bg_replacement) = if existing_sub.is_some() {
        let (main_text, bracket) = split_first_bracket_segment(&replacement_text);
        let bg = bracket.map(|b| normalize_bracket_outer_spaces(&b));
        (main_text, bg)
    } else {
        (normalize_spaces(&replacement_text), None)
    };

    match &mut line {
        LineInfo::Syllable { syllables, .. } | LineInfo::FullSyllable { syllables, .. } => {
            let mut flat = flatten_syllable_items(syllables);
            let new_sylls = if existing_sub.is_none() {
                replace_syllable_line_parts(&flat, &replacement.span_texts)
            } else {
                None
            };
            if let Some(new_sylls) = new_sylls {
                flat = new_sylls;
            } else {
                replace_syllable_line_text(&mut flat, &main_replacement);
            }
            *syllables = to_syllable_items(flat);
        }
        LineInfo::Line { text, .. } | LineInfo::FullLine { text, .. } => {
            *text = normalize_text(&main_replacement);
        }
    }

    if let Some(mut sub) = existing_sub {
        if let Some(bg_rep) = &bg_replacement {
            match &mut sub {
                LineInfo::Syllable { syllables, .. } | LineInfo::FullSyllable { syllables, .. } => {
                    let mut flat = flatten_syllable_items(syllables);
                    replace_syllable_line_text(&mut flat, bg_rep);
                    *syllables = to_syllable_items(flat);
                }
                LineInfo::Line { text, .. } | LineInfo::FullLine { text, .. } => {
                    *text = normalize_text(bg_rep);
                }
            }
        }
        line.set_sub_line(Some(Box::new(sub)));
    }

    line
}

fn apply_subtitle_translations(mut line: LineInfo, subtitles: &[(String, String)]) -> LineInfo {
    let align = line.alignment();
    let mut existing_sub = line.sub_line().cloned();

    let mut dict: HashMap<String, String> = HashMap::new();
    for (lang, value) in subtitles {
        let lang_key = normalize_lang_key(lang);
        dict.entry(lang_key).or_insert_with(|| value.clone());
    }

    let mut bg_dict: Option<HashMap<String, String>> = None;
    if existing_sub.is_some() {
        let mut bg = HashMap::new();
        for key in dict.keys().cloned().collect::<Vec<_>>() {
            let val = dict[&key].clone();
            let (main_text, bg_text) = split_subtitle_by_parentheses(&val);
            dict.insert(key.clone(), main_text);
            if let Some(bg_val) = bg_text {
                bg.insert(key, bg_val);
            }
        }
        if !bg.is_empty() {
            bg_dict = Some(bg);
        }
    }

    match &mut line {
        LineInfo::Syllable {
            syllables,
            alignment,
            sub_line,
            ..
        } => {
            let translations = dict;
            let mut full = LineInfo::FullSyllable {
                syllables: std::mem::take(syllables),
                alignment: *alignment,
                sub_line: sub_line.take(),
                translations,
                pronunciation: None,
            };

            if let Some(ref mut sub) = existing_sub {
                if let Some(ref bg) = bg_dict {
                    apply_bg_translations_to_subline(sub, bg);
                }
            }
            if let Some(sub) = existing_sub {
                full.set_sub_line(Some(Box::new(sub)));
            }
            full.set_alignment(align);
            full
        }
        LineInfo::Line {
            text,
            start_time,
            end_time,
            alignment,
            sub_line,
            ..
        } => {
            let translations = dict;
            let mut full = LineInfo::FullLine {
                text: std::mem::take(text),
                start_time: *start_time,
                end_time: *end_time,
                alignment: *alignment,
                sub_line: sub_line.take(),
                translations,
                pronunciation: None,
            };

            if let Some(ref mut sub) = existing_sub {
                if let Some(ref bg) = bg_dict {
                    apply_bg_translations_to_subline(sub, bg);
                }
            }
            if let Some(sub) = existing_sub {
                full.set_sub_line(Some(Box::new(sub)));
            }
            full.set_alignment(align);
            full
        }
        other => {
            other.set_alignment(align);
            if let Some(sub) = existing_sub {
                other.set_sub_line(Some(Box::new(sub)));
            }
            other.clone()
        }
    }
}

fn apply_bg_translations_to_subline(sub: &mut LineInfo, bg_dict: &HashMap<String, String>) {
    match sub {
        LineInfo::Syllable {
            syllables,
            alignment,
            sub_line,
            ..
        } => {
            let existing = sub_line.take();
            let full = LineInfo::FullSyllable {
                syllables: std::mem::take(syllables),
                alignment: *alignment,
                sub_line: existing,
                translations: bg_dict.clone(),
                pronunciation: None,
            };
            *sub = full;
        }
        LineInfo::Line {
            text,
            start_time,
            end_time,
            alignment,
            sub_line,
            ..
        } => {
            let existing = sub_line.take();
            let full = LineInfo::FullLine {
                text: std::mem::take(text),
                start_time: *start_time,
                end_time: *end_time,
                alignment: *alignment,
                sub_line: existing,
                translations: bg_dict.clone(),
                pronunciation: None,
            };
            *sub = full;
        }
        LineInfo::FullLine { translations, .. } | LineInfo::FullSyllable { translations, .. } => {
            for (k, v) in bg_dict {
                translations.entry(k.clone()).or_insert_with(|| v.clone());
            }
        }
    }
}

fn split_first_bracket_segment(text: &str) -> (String, Option<String>) {
    let text = normalize_text(text);
    match BRACKET_CONTENT_RE.find(&text) {
        Some(m) => {
            let bracket_seg = m.as_str().to_string();
            let mut main = text.clone();
            main.replace_range(m.start()..m.end(), "");
            (normalize_spaces(&main), Some(bracket_seg))
        }
        None => (normalize_spaces(&text), None),
    }
}

fn normalize_bracket_outer_spaces(s: &str) -> String {
    let s = normalize_text(s);
    let s = OPEN_BRACKET_SPACE_RE.replace_all(&s, "$1").to_string();
    CLOSE_BRACKET_SPACE_RE.replace_all(&s, "$1").to_string()
}

fn replace_syllable_line_text(syllables: &mut Vec<SyllableInfo>, new_text: &str) {
    let new_text = normalize_text(new_text);
    let lens: Vec<usize> = syllables.iter().map(|s| s.text.len()).collect();
    let total_len: usize = lens.iter().sum();

    if total_len == new_text.len() && total_len > 0 {
        let mut idx = 0;
        for (i, syll) in syllables.iter_mut().enumerate() {
            let len = lens[i];
            syll.text = new_text[idx..idx + len].to_string();
            idx += len;
        }
    } else if !syllables.is_empty() {
        let start = syllables[0].start_time;
        let end = syllables.last().unwrap().end_time;
        syllables.clear();
        syllables.push(SyllableInfo::new(new_text, start, end));
    }
}

fn split_subtitle_by_parentheses(value: &str) -> (String, Option<String>) {
    let value = normalize_text(value);
    match BRACKET_CONTENT_RE.captures(&value) {
        Some(caps) => {
            let inner = if let Some(m) = caps.get(1) {
                m.as_str()
            } else if let Some(m) = caps.get(2) {
                m.as_str()
            } else {
                ""
            };
            let inner = normalize_spaces(inner);
            let m = caps.get(0).unwrap();
            let mut main = value.clone();
            main.replace_range(m.start()..m.end(), "");
            let main = normalize_spaces(&main);
            if inner.trim().is_empty() {
                (main, None)
            } else {
                (main, Some(inner))
            }
        }
        None => (normalize_spaces(&value), None),
    }
}

fn parse_time_ms(value: &str) -> Option<i32> {
    let value = value.trim();
    let value = if value.ends_with('s') || value.ends_with('S') {
        &value[..value.len() - 1]
    } else {
        value
    };

    if value.contains(':') {
        let parts: Vec<&str> = value.split(':').collect();
        let seconds: f64 = match parts.len() {
            2 => {
                let minutes: f64 = parts[0].parse().ok()?;
                let sec: f64 = parts[1].parse().ok()?;
                minutes * 60.0 + sec
            }
            3 => {
                let hours: f64 = parts[0].parse().ok()?;
                let minutes: f64 = parts[1].parse().ok()?;
                let sec: f64 = parts[2].parse().ok()?;
                hours * 3600.0 + minutes * 60.0 + sec
            }
            _ => {
                let replaced = value.replace(':', ".");
                let seconds: f64 = replaced.parse().ok()?;
                return Some((seconds * 1000.0).round() as i32);
            }
        };
        Some((seconds * 1000.0).round() as i32)
    } else {
        let seconds: f64 = value.parse().ok()?;
        Some((seconds * 1000.0).round() as i32)
    }
}
