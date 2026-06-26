use lyrics_core::models::*;
use quick_xml::events::Event;
use quick_xml::Reader;
use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

static SPACES_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s+").unwrap());
static OPEN_BRACKET_SPACE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s+(\(|（)").unwrap());
static CLOSE_BRACKET_SPACE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(\)|）)\s+").unwrap());
static BRACKET_CONTENT_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\(([^)]*)\)|（([^）]*)）").unwrap());

struct Agent {
    id: String,
    agent_type: String,
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

pub fn parse(ttml: &str) -> LyricsData {
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

    let p_nodes = find_elements(&doc, "p");
    let mut any_line_synced = false;
    let mut any_syllable_synced = false;

    for p in &p_nodes {
        let (p_attrs, p_children) = match p {
            XmlNode::Element { attributes, children, .. } => (attributes, children),
            _ => continue,
        };

        let key = attr_value(p_attrs, "key");
        let agent_id = attr_value(p_attrs, "agent");

        let mut main_syllables = Vec::new();
        let mut bg_syllables = Vec::new();
        collect_syllables_from_nodes(p_children, &mut main_syllables, &mut bg_syllables, false);

        let mut line: Option<LineInfo> = None;

        if !main_syllables.is_empty() {
            any_syllable_synced = true;
            line = Some(LineInfo::new_syllable(main_syllables));
        } else {
            let begin = attr_value(p_attrs, "begin").and_then(|v| parse_time_ms(&v));
            let end = attr_value(p_attrs, "end").and_then(|v| parse_time_ms(&v));
            let text = normalize_text(&element_text_value(p_children)).trim().to_string();

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

        let align = get_alignment_from_agent(&agent_id, &agents);
        line.set_alignment(align);

        if !bg_syllables.is_empty() {
            normalize_bracket_inner_spacing_for_bg(&mut bg_syllables);
            let mut sub = LineInfo::new_syllable(bg_syllables);
            sub.set_alignment(align);
            line.set_sub_line(Some(Box::new(sub)));
        }

        if let Some(ref key_val) = key {
            if let Some(tmap) = translations.get(key_val) {
                let replacement = tmap
                    .iter()
                    .find(|((t, _), v)| t.eq_ignore_ascii_case("replacement") && !v.trim().is_empty())
                    .map(|(_, v)| v.clone());

                if let Some(rep) = replacement {
                    line = apply_replacement(line, &rep);
                    line.set_alignment(align);
                }

                let subtitles: Vec<_> = tmap
                    .iter()
                    .filter(|((t, _), _)| t.eq_ignore_ascii_case("subtitle"))
                    .map(|((_, lang), val)| (lang.clone(), val.clone()))
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
            XmlNode::Element {
                name, children, ..
            } => {
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
            XmlNode::Element {
                name, children, ..
            } => {
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

fn get_alignment_from_agent(agent_id: &Option<String>, agents: &[Agent]) -> LyricsAlignment {
    if agents.is_empty() {
        return LyricsAlignment::Unspecified;
    }
    if agents.len() == 1 {
        return LyricsAlignment::Left;
    }

    let agent_id = match agent_id {
        Some(id) if !id.trim().is_empty() => id.trim(),
        _ => return LyricsAlignment::Unspecified,
    };

    let hit = match agents.iter().find(|a| a.id == agent_id) {
        Some(a) => a,
        None => return LyricsAlignment::Unspecified,
    };

    let mut type_groups: Vec<(&str, Vec<&Agent>)> = Vec::new();
    for agent in agents {
        let t = agent.agent_type.as_str();
        if let Some(group) = type_groups.iter_mut().find(|(gt, _)| *gt == t) {
            group.1.push(agent);
        } else {
            type_groups.push((t, vec![agent]));
        }
    }

    if agents.len() == 2
        && type_groups.len() == 2
        && type_groups.iter().all(|(_, g)| g.len() == 1)
        && type_groups.iter().any(|(t, _)| t.eq_ignore_ascii_case("person"))
    {
        return if hit.agent_type.eq_ignore_ascii_case("person") {
            LyricsAlignment::Left
        } else {
            LyricsAlignment::Right
        };
    }

    let idx = type_groups
        .iter()
        .find(|(t, _)| *t == hit.agent_type.as_str())
        .map(|(_, group)| group.iter().position(|a| a.id == hit.id).unwrap_or(0))
        .unwrap_or(0);

    if idx % 2 == 0 {
        LyricsAlignment::Left
    } else {
        LyricsAlignment::Right
    }
}

fn parse_itunes_metadata(doc: &[XmlNode], data: &mut LyricsData) {
    let meta = match find_first_element(doc, "iTunesMetadata") {
        Some(m) => m,
        None => return,
    };

    let (meta_attrs, meta_children) = match meta {
        XmlNode::Element { attributes, children, .. } => (attributes, children),
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

fn parse_translations(doc: &[XmlNode]) -> HashMap<String, HashMap<(String, String), String>> {
    let mut result: HashMap<String, HashMap<(String, String), String>> = HashMap::new();

    let translation_nodes = find_elements(doc, "translation");
    for node in translation_nodes {
        let (node_attrs, node_children) = match node {
            XmlNode::Element { attributes, children, .. } => (attributes, children),
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

                result
                    .entry(key)
                    .or_default()
                    .insert((typ.clone(), lang.clone()), value);
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
    last.text = CLOSE_BRACKET_SPACE_RE.replace_all(&last.text, "$1").to_string();
}

fn apply_replacement(mut line: LineInfo, replacement: &str) -> LineInfo {
    let replacement = normalize_text(replacement);
    let existing_sub = line.sub_line().cloned();

    let (main_replacement, bg_replacement) = if existing_sub.is_some() {
        let (main_text, bracket) = split_first_bracket_segment(&replacement);
        let bg = bracket.map(|b| normalize_bracket_outer_spaces(&b));
        (main_text, bg)
    } else {
        (normalize_spaces(&replacement), None)
    };

    match &mut line {
        LineInfo::Syllable { syllables, .. } => {
            replace_syllable_line_text(syllables, &main_replacement);
        }
        LineInfo::Line { text, .. } => {
            *text = normalize_text(&main_replacement);
        }
        LineInfo::FullSyllable { syllables, .. } => {
            replace_syllable_line_text(syllables, &main_replacement);
        }
        LineInfo::FullLine { text, .. } => {
            *text = normalize_text(&main_replacement);
        }
    }

    if let Some(mut sub) = existing_sub {
        if let Some(ref bg_rep) = bg_replacement {
            match &mut sub {
                LineInfo::Syllable { syllables, .. } => {
                    replace_syllable_line_text(syllables, bg_rep);
                }
                LineInfo::Line { text, .. } => {
                    *text = normalize_text(bg_rep);
                }
                LineInfo::FullSyllable { syllables, .. } => {
                    replace_syllable_line_text(syllables, bg_rep);
                }
                LineInfo::FullLine { text, .. } => {
                    *text = normalize_text(bg_rep);
                }
            }
        }
        line.set_sub_line(Some(Box::new(sub)));
    }

    line
}

fn apply_subtitle_translations(
    mut line: LineInfo,
    subtitles: &[(String, String)],
) -> LineInfo {
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
        LineInfo::FullLine { translations, .. }
        | LineInfo::FullSyllable { translations, .. } => {
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
