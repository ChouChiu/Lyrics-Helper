use lyrics_core::models::*;

/// 把一条 `[key:value]` 属性写入 [`LyricsData`]。
///
/// `ti`/`ar`/`al`/`length` 写入曲目元数据，`offset` 通过 `offset` 参数带出，
/// `hash` 写入 KRC 的附加信息，其余属性追加到附加信息的属性列表。
fn apply_attribute(data: &mut LyricsData, key: String, value: String, offset: &mut Option<i32>) {
    let meta = data.track_metadata.get_or_insert_with(TrackMetadata::new);
    match key.as_str() {
        "ar" => meta.artist = Some(value.clone()),
        "al" => meta.album = Some(value.clone()),
        "ti" => meta.title = Some(value.clone()),
        "length" => meta.duration_ms = value.parse().ok(),
        "offset" => *offset = value.parse().ok(),
        _ => {}
    }

    let Some(file) = data.file.as_mut() else {
        return;
    };

    // KRC 的 `hash` 有专用字段；其他格式没有，按普通属性记录。
    if key == "hash"
        && let Some(AdditionalFileInfo::Krc { hash, .. }) = &mut file.additional_info
    {
        *hash = Some(value);
        return;
    }

    if let Some(attributes) = file
        .additional_info
        .as_mut()
        .and_then(AdditionalFileInfo::attributes_mut)
    {
        attributes.push((key, value));
    }
}

/// 从歌词行列表开头提取通用属性（如 `[ar:xxx]`），填充到 [`LyricsData`] 中，并从行列表中移除属性行。
///
/// 返回解析到的时间偏移量（offset）。
pub fn parse_general_attributes_to_lyrics_data_from_lines(
    data: &mut LyricsData,
    lines: &mut Vec<String>,
) -> Option<i32> {
    let mut offset = None;
    data.track_metadata.get_or_insert_with(TrackMetadata::new);

    let header_len = lines
        .iter()
        .position(|line| !is_attribute_line(line))
        .unwrap_or(lines.len());
    for line in lines.drain(..header_len) {
        let (key, value) = get_attribute(&line);
        apply_attribute(data, key, value, &mut offset);
    }

    offset
}

/// 从原始歌词字符串开头提取通用属性，填充到 [`LyricsData`] 中。
///
/// 返回时间偏移量和属性部分结束的字符索引。
pub fn parse_general_attributes_to_lyrics_data(
    data: &mut LyricsData,
    input: &str,
) -> (Option<i32>, usize) {
    let mut offset = None;
    let mut index = 0;

    data.track_metadata.get_or_insert_with(TrackMetadata::new);

    let chars: Vec<char> = input.chars().collect();
    while chars.get(index) == Some(&'[') {
        let end_index = chars[index..]
            .iter()
            .position(|&c| c == '\n')
            .map_or(chars.len(), |i| i + index);
        let info_line: String = chars[index..end_index].iter().collect();

        if !is_attribute_line(&info_line) {
            break;
        }

        let (key, value) = get_attribute(&info_line);
        apply_attribute(data, key, value, &mut offset);
        index = end_index;
    }

    (offset, index)
}

/// 判断一行文本是否为属性行（格式为 `[key:value]`）。
pub fn is_attribute_line(line: &str) -> bool {
    let line = line.trim();
    line.starts_with('[') && line.ends_with(']') && line.contains(':')
}

/// 从属性行中提取键值对，返回 `(key, value)` 元组。
///
/// 不是 `[key:value]` 形式时返回两个空串。
pub fn get_attribute(line: &str) -> (String, String) {
    let inner = line
        .trim()
        .strip_prefix('[')
        .and_then(|line| line.strip_suffix(']'))
        .unwrap_or_default();

    match inner.split_once(':') {
        Some((key, value)) => (key.to_string(), value.to_string()),
        None => (String::new(), String::new()),
    }
}
