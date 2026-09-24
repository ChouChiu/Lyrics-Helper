//! QRC XML 封装处理工具，对应上游 C# 的 `Decrypter/Qrc/XmlUtils.cs`。
//!
//! 上游把它放在解密器命名空间下，但它处理的是解密之后的 XML 信封，与密码学无关，
//! 因此这里归到解析器 crate：解密 crate 不再依赖 XML 与正则，离线解析 `QrcFull` 也不必
//! 启用网络搜索。
//!
//! 腾讯音乐返回的 QRC 歌词往往包裹在非标准 XML 中（属性值内含未转义的引号、`<`、`&`），
//! 本模块负责把这类内容修复为可解析的 XML，并提供按节点名递归查找的能力。

use std::collections::HashMap;
use std::sync::LazyLock;

use quick_xml::Reader;
use quick_xml::escape::resolve_predefined_entity;
use quick_xml::events::attributes::Attribute;
use quick_xml::events::{BytesRef, BytesStart, Event};
use regex::Regex;

/// 属性起始片段 `\s+[\w:.-]+\s*=\s*"`（对应上游正则的第 1 组，无环视）。
static ATTR_PREFIX_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"\s+[\w:.-]+\s*=\s*""#).unwrap());

/// 属性值的收尾判定 `\s+[\w:.-]+\s*="|\s*(?:/?|\?)>`（对应上游正则中第 5 组的环视条件）。
static VALUE_END_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"^(?:\s+[\w:.-]+\s*="|\s*(?:/?|\?)>)"#).unwrap());

/// 简单的 XML 节点，对应 C# `XmlNode`。
#[derive(Debug, Clone, Default)]
pub struct XmlNode {
    /// 节点名（含命名空间前缀）
    pub name: String,
    /// 属性列表
    pub attributes: Vec<(String, String)>,
    /// 子节点
    pub children: Vec<XmlNode>,
    /// 节点直接包含的文本
    pub text: String,
}

impl XmlNode {
    /// 返回节点属性值。
    pub fn attribute(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }

    /// 返回节点及其所有子节点的文本拼接，对应 C# `XmlNode.InnerText`。
    pub fn inner_text(&self) -> String {
        let mut text = self.text.clone();
        for child in &self.children {
            text.push_str(&child.inner_text());
        }
        text
    }
}

/// 创建可解析的 XML 文档，对应 C# `XmlUtils.Create`。
///
/// 先清理非法内容并转义 `&`、属性值中的引号后解析；解析失败时回退为
/// 仅清理非法内容与转义 `&` 的版本。
pub fn create(content: &str) -> Option<XmlNode> {
    let content = remove_illegal_content(content);
    let content = replace_amp(&content);
    let quoted = replace_quot(&content);

    parse_document(&quoted).or_else(|| parse_document(&content))
}

/// 递归查找节点，对应 C# `XmlUtils.RecursionFindElement`。
///
/// `mapping` 为「节点名 → 结果名」的映射，命中的节点写入 `res`。
pub fn recursion_find_element<'a>(
    node: &'a XmlNode,
    mapping: &[(&str, &str)],
    res: &mut HashMap<String, &'a XmlNode>,
) {
    if let Some((_, value)) = mapping.iter().find(|(key, _)| *key == node.name) {
        res.insert((*value).to_string(), node);
    }

    for child in &node.children {
        recursion_find_element(child, mapping, res);
    }
}

/// 转义未转义的 `&` 符号，对应上游 `AmpRegex = &(?![a-zA-Z]{2,6};|#[0-9]{2,4};)`。
///
/// Rust `regex` 不支持环视，因此按字符扫描实现等价逻辑。
pub fn replace_amp(content: &str) -> String {
    let mut result = String::with_capacity(content.len());

    for (index, character) in content.char_indices() {
        if character == '&' && !is_entity_start(&content[index + 1..]) {
            result.push_str("&amp;");
        } else {
            result.push(character);
        }
    }

    result
}

/// 判断 `rest` 是否以实体引用开头（`[a-zA-Z]{2,6};` 或 `#[0-9]{2,4};`）。
fn is_entity_start(rest: &str) -> bool {
    if let Some(digits) = rest.strip_prefix('#') {
        let count = digits.chars().take_while(char::is_ascii_digit).count();
        return (2..=4).contains(&count) && digits[count..].starts_with(';');
    }

    let letters = rest.chars().take_while(char::is_ascii_alphabetic).count();
    (2..=6).contains(&letters) && rest[letters..].starts_with(';')
}

/// 转义属性值中嵌套的引号与 `<`。
///
/// 对应上游 `QuotRegex`：属性值可以包含未转义的引号，值真正的结束位置是
/// 后面紧跟另一个属性或标签结尾（`>`、`/>`、`?>`）的那个引号。
/// Rust `regex` 不支持环视，因此拆成「属性前缀匹配」+「收尾扫描」两步。
pub fn replace_quot(content: &str) -> String {
    let mut sb = String::with_capacity(content.len());
    let mut current_pos = 0;

    while let Some(prefix) = ATTR_PREFIX_REGEX.find_at(content, current_pos) {
        let value_start = prefix.end();
        let Some(value_end) = find_value_end(content, value_start) else {
            break;
        };

        sb.push_str(&content[current_pos..value_start]);
        sb.push_str(
            &content[value_start..value_end]
                .replace('"', "&quot;")
                .replace('<', "&lt;"),
        );
        sb.push('"');

        current_pos = value_end + 1;
    }

    sb.push_str(&content[current_pos..]);
    sb
}

/// 查找属性值的结束引号位置。
fn find_value_end(content: &str, from: usize) -> Option<usize> {
    let bytes = content.as_bytes();
    let mut index = from;

    while index < bytes.len() {
        if bytes[index] == b'"' && VALUE_END_REGEX.is_match(&content[index + 1..]) {
            return Some(index);
        }
        index += 1;
    }

    None
}

/// 移除 XML 内容中无效的部分，对应 C# `XmlUtils.RemoveIllegalContent`。
pub fn remove_illegal_content(content: &str) -> String {
    let mut chars: Vec<char> = content.chars().collect();
    let mut left = 0;
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '<' {
            left = i;
        }

        // 闭区间
        if i > 0 && chars[i] == '>' && chars[i - 1] == '/' {
            // 下标与 `chars` 一致按字符计，避免非 ASCII 属性名下切片越界
            let part = &chars[left..=i];
            let first_eq = part.iter().position(|&c| c == '=');
            let last_eq = part.iter().rposition(|&c| c == '=');

            // 存在有且只有一个等号
            if let (Some(first), Some(last)) = (first_eq, last_eq)
                && first == last
            {
                // 等号和左括号之间没有空格 <a="b" />
                let prefix: String = chars[left..left + first].iter().collect();
                if !prefix.trim().contains(' ') {
                    chars.drain(left..=i);
                    i = 0;
                    continue;
                }
            }
        }

        i += 1;
    }

    chars.iter().collect::<String>().trim().to_string()
}

/// 解析 XML 文本为节点树，解析失败返回 `None`。
fn parse_document(content: &str) -> Option<XmlNode> {
    let mut reader = Reader::from_str(content);
    reader.config_mut().trim_text(false);
    let mut stack: Vec<XmlNode> = Vec::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                stack.push(element_node(&e));
            }
            Ok(Event::Empty(e)) => {
                let node = element_node(&e);
                match stack.last_mut() {
                    Some(parent) => parent.children.push(node),
                    None => return Some(node),
                }
            }
            Ok(Event::End(_)) => {
                let node = stack.pop()?;
                match stack.last_mut() {
                    Some(parent) => parent.children.push(node),
                    None => {
                        return Some(node);
                    }
                }
            }
            Ok(Event::Text(e)) => {
                if let Some(parent) = stack.last_mut() {
                    parent.text.push_str(&e.xml10_content());
                }
            }
            Ok(Event::GeneralRef(e)) => {
                if let Some(parent) = stack.last_mut() {
                    parent.text.push_str(&reference_text(&e));
                }
            }
            Ok(Event::CData(e)) => {
                if let Some(parent) = stack.last_mut() {
                    parent.text.push_str(&e.xml10_content());
                }
            }
            Ok(Event::Eof) => return None,
            Err(_) => return None,
            _ => {}
        }
    }
}

/// 由开始标签事件构造节点。
fn element_node(event: &BytesStart<'_>) -> XmlNode {
    XmlNode {
        name: event.name().as_ref().to_string(),
        attributes: event
            .attributes()
            .flatten()
            .map(|attribute| {
                (
                    attribute.key.as_ref().to_string(),
                    attribute_value(&attribute),
                )
            })
            .collect(),
        children: Vec::new(),
        text: String::new(),
    }
}

/// 还原属性值里的实体引用，无法识别的实体原样保留。
///
/// 只反转义、不做 XML 属性值规范化：quick-xml 的 `normalized_value` 会把换行换成空格，
/// 而 QQ 音乐把整段多行 QRC 歌词放在 `LyricContent` 属性里。
pub(crate) fn attribute_value(attribute: &Attribute<'_>) -> String {
    // quick-xml 的 `unescape` 遇到一个不认识的实体就整体报错，这里逐个还原，
    // 一个 `&nbsp;` 不会连累同一属性里的 `&amp;`。
    let mut value = String::with_capacity(attribute.value.len());
    let mut rest: &str = &attribute.value;
    while let Some(start) = rest.find('&') {
        value.push_str(&rest[..start]);
        rest = &rest[start..];
        match rest[1..].find(';') {
            Some(end) => {
                value.push_str(&reference_text(&BytesRef::new(&rest[1..=end])));
                rest = &rest[end + 2..];
            }
            // 没有结尾分号的 `&` 不是实体引用，原样保留。
            None => break,
        }
    }
    value.push_str(rest);
    value
}

/// 还原文本中的实体引用（`&amp;`、`&#x4E00;` 等），无法识别的原样保留。
///
/// quick-xml 0.38 起文本事件不再包含实体，实体引用单独作为 [`Event::GeneralRef`] 给出，
/// 调用方需要把它与前后的文本事件拼接起来。
pub(crate) fn reference_text(reference: &BytesRef<'_>) -> String {
    if let Ok(Some(character)) = reference.resolve_char_ref() {
        return character.to_string();
    }
    match resolve_predefined_entity(reference) {
        Some(text) => text.to_string(),
        None => format!("&{};", &**reference),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MAPPING: [(&str, &str); 1] = [("Lyric_1", "lyric")];

    #[test]
    fn create_parses_qrc_envelope_with_unescaped_ampersand() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<QrcInfos>
<QrcHeadInfo SaveTime="269" Version="100"/>
<LyricInfo LyricCount="1">
<Lyric_1 LyricType="1" LyricContent="[0,1000]A &amp; B(0,1000)"/>
</LyricInfo>
</QrcInfos>"#;

        let doc = create(xml).expect("should parse");
        let mut res = HashMap::new();
        recursion_find_element(&doc, &MAPPING, &mut res);

        assert_eq!(
            res["lyric"].attribute("LyricContent"),
            Some("[0,1000]A & B(0,1000)")
        );
    }

    /// 属性值只反转义、不做 XML 规范化：QRC 正文是多行，换行不能被换成空格。
    #[test]
    fn multi_line_lyric_content_keeps_its_line_breaks() {
        let xml = "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<QrcInfos><LyricInfo>\
<Lyric_1 LyricContent=\"[ti:A &amp; B]\n[0,1000]A(0,1000)\r\n[1000,1000]B(1000,1000)\"/>\
</LyricInfo></QrcInfos>";

        let doc = create(xml).expect("should parse");
        let mut res = HashMap::new();
        recursion_find_element(&doc, &MAPPING, &mut res);

        assert_eq!(
            res["lyric"].attribute("LyricContent"),
            Some("[ti:A & B]\n[0,1000]A(0,1000)\r\n[1000,1000]B(1000,1000)")
        );
    }

    #[test]
    fn text_keeps_resolved_and_unknown_entities() {
        let doc = create("<a>x &lt; y &#169; &nbsp;</a>").expect("should parse");
        assert_eq!(doc.inner_text(), "x < y \u{A9} &nbsp;");
    }

    /// 一个不认识的实体只保留它自己，同一属性里其他实体照样还原。
    #[test]
    fn attribute_keeps_only_the_unknown_entity() {
        let doc = create(r#"<a b="A &amp; B&nbsp;&#169;"/>"#).expect("should parse");
        assert_eq!(doc.attribute("b"), Some("A & B&nbsp;\u{A9}"));
    }

    #[test]
    fn replace_quot_escapes_nested_quotes_and_lt() {
        let content = r#"<a b="x" y="1 < 2" />"#;
        let replaced = replace_quot(content);
        assert!(replaced.contains("1 &lt; 2"));
        assert!(create(content).is_some());
    }

    #[test]
    fn remove_illegal_content_drops_unnamed_empty_attribute() {
        assert_eq!(
            remove_illegal_content(r#"<a ="b" /><c>text</c>"#),
            "<c>text</c>"
        );
    }

    #[test]
    fn replace_amp_keeps_existing_entities() {
        assert_eq!(
            replace_amp("A &amp; B & C &#39;"),
            "A &amp; B &amp; C &#39;"
        );
    }
}
