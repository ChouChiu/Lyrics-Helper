use crate::parsers::{apply_offset, lyrics_data, parse_with_attributes};
use lyrics_core::models::*;

/// 解析 QRC 格式歌词，返回包含属性信息的逐音节同步 [`LyricsData`]。
pub fn parse(input: &str) -> LyricsData {
    parse_with_attributes(
        lyrics_data(
            LyricsTypes::Qrc,
            SyncTypes::SyllableSynced,
            Some(AdditionalFileInfo::new_general()),
        ),
        input.trim().lines().map(str::to_string).collect(),
        parse_lyrics,
    )
}

/// 解析 QRC 歌词行列表，可选地应用时间偏移，返回歌词行列表。
pub fn parse_lyrics(lines: &[String], offset: Option<i32>) -> Vec<LineInfo> {
    let mut list: Vec<LineInfo> = lines.iter().map(|line| parse_lyrics_line(line)).collect();
    apply_offset(&mut list, offset);
    list
}

/// 解析单行 QRC 歌词，提取行头时间与音节信息，返回单个 [`LineInfo`]。
///
/// 行头 `[开始时间,时长]` 写的整行时长会一并写进 [`LineInfo`]：行时间与音节时间相互
/// 独立，末个音节唱完不等于这行该消失。
pub fn parse_lyrics_line(line: &str) -> LineInfo {
    let (line_time, body) = split_line_header(line);
    let (start_time, end_time) = match line_time {
        Some((start, duration)) => (Some(start), Some(start + duration)),
        None => (None, None),
    };

    LineInfo::new_syllable_with_time(
        to_syllable_items(parse_syllables(body)),
        start_time,
        end_time,
    )
}

/// 拆出行头 `[开始时间,时长]` 与行正文；行头不是两个整数时只返回正文。
fn split_line_header(line: &str) -> (Option<(i32, i32)>, &str) {
    let Some(bracket_end) = line.find(']') else {
        return (None, line);
    };

    let header = line[..bracket_end]
        .strip_prefix('[')
        .and_then(parse_time_pair);

    (header, &line[bracket_end + 1..])
}

/// 解析 `开始时间,时长` 形式的标签内容，两个字段都是整数才算有效。
fn parse_time_pair(tag: &str) -> Option<(i32, i32)> {
    let (start, duration) = tag.split_once(',')?;
    Some((start.trim().parse().ok()?, duration.trim().parse().ok()?))
}

/// 扫描行正文，把每个 `(开始时间,时长)` 时间戳之前的文本归给它所计时的音节。
///
/// QQ 音乐用括号包住重复乐句，例如 `((115,7)Remix(122,36))(158,7)`：只有内容是两个
/// 整数的时间戳才是词时间戳，其他括号都是正文。没有时间戳跟随的文本（行尾、未闭合
/// 括号之后）并入末个音节，否则这段歌词会整段丢失。
fn parse_syllables(body: &str) -> Vec<SyllableInfo> {
    let mut syllables: Vec<SyllableInfo> = Vec::new();
    let mut pending = String::new();
    let mut rest = body;

    while let Some(open) = rest.find('(') {
        let after_open = &rest[open + 1..];
        let Some(close) = after_open.find(')') else {
            break;
        };

        let Some((start_time, duration)) = parse_time_pair(&after_open[..close]) else {
            // 不是时间戳的括号属于正文，跟着后面的文本一起归给下一个被计时的词。
            pending.push_str(&rest[..=open]);
            rest = after_open;
            continue;
        };

        pending.push_str(&rest[..open]);
        syllables.push(SyllableInfo::new(
            std::mem::take(&mut pending),
            start_time,
            start_time + duration,
        ));
        rest = &after_open[close + 1..];
    }

    pending.push_str(rest);
    if let Some(last) = syllables.last_mut() {
        last.text.push_str(&pending);
    }

    syllables
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 渲染器画的是音节文本，不是行文本，所以音节拼起来必须等于整行。
    fn spelled(line: &LineInfo) -> String {
        LineInfo::text_from_syllables(line.syllables().expect("应为音节行"))
    }

    #[test]
    fn keeps_the_line_duration_written_in_the_header() {
        // 真实载荷（QQ 音乐）：末个音节 176463 唱完，但行头说这行到 194662 才结束。
        let line = parse_lyrics_line(
            "[170720,23942]Oh (170720,1578)you (172298,680)don't (172978,685)need(173663,2800)",
        );

        assert_eq!(line.start_time(), Some(170_720));
        assert_eq!(line.end_time(), Some(194_662));
        assert_eq!(line.duration(), Some(23_942));
        assert_eq!(
            line.syllables().unwrap().last().unwrap().end_time(),
            176_463,
            "音节时间不应被行时间改写"
        );
    }

    #[test]
    fn keeps_a_repeated_phrase_parenthesis_as_the_word_it_precedes() {
        // 真实载荷（QQ 音乐）：重复乐句 `(Come on, just…)` 的每个字符各配一个时间戳，
        // `(` 由紧跟它的 (109473,249) 计时（QRC 的标签写在其所计文本之后），
        // 内层时间戳不能被当成正文而丢掉。
        let line = parse_lyrics_line(
            "[109473,7768]((109473,249)Come (109722,192)on, (109914,209)just…)(110123,7118)",
        );

        let syllables = line.syllables().unwrap();
        assert_eq!(
            syllables
                .iter()
                .map(|s| (s.text(), s.start_time(), s.end_time()))
                .collect::<Vec<_>>(),
            vec![
                ("(".to_string(), 109_473, 109_722),
                ("Come ".to_string(), 109_722, 109_914),
                ("on, ".to_string(), 109_914, 110_123),
                ("just…)".to_string(), 110_123, 117_241),
            ]
        );
        assert_eq!(spelled(&line), "(Come on, just…)");
    }

    #[test]
    fn keeps_text_that_no_timestamp_follows() {
        // 末个时间戳之后的文本没有自己的时间，并入末个音节；丢掉它就等于丢掉这段歌词。
        let line = parse_lyrics_line("[17456,1000]Hello(17456,500) world");

        assert_eq!(spelled(&line), "Hello world");
    }

    #[test]
    fn falls_back_to_syllables_when_the_header_is_missing() {
        // 行头缺失或不是两个整数时不产生行时间，取首尾音节即可。
        let line = parse_lyrics_line("Hello(0,500) world(500,500)");

        assert!(line.syllables().is_some());
        assert_eq!(line.start_time(), Some(0));
        assert_eq!(line.end_time(), Some(1_000));
    }
}
