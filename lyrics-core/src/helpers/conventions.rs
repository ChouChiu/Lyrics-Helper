//! 从歌词文本自身的书写约定里读出对唱分边、背景和声与跨行续句。
//!
//! QRC、LRC、KRC、YRC 都没有「这一段是第二个声部」「这一句是背景和声」这样的字段——转录
//! 者是把它们写进文本里的：行内括号短语是背景和声（网易云《Umbrella》开头
//! `Ahuh Ahuh （Yea Rihanna）`），`歌手名：` 开头的行或独占一行的标签是对唱分边，逗号
//! 结尾的行由下一行续上。这里把这三件事读出来，填进 [`LineInfo`] 本来就有的 `sub_line`
//! 与 `alignment`（对应下游的 `background` 与 `voice`：署名里第一位是左、其余是右）。
//!
//! 三条约定都只认转录明写的东西：对唱标签必须点到该曲目的歌手之一，匹配不上的标签与时间
//! 重叠都不构成第二个声部的证据；括号里要有字母或数字才是被唱出来的一句；行尾的标点与
//! 大小写说了句子是否继续。宁可漏判，也不瞎判。`pronunciation` 不属于这三件事——它是读音
//! 而不是书写约定，本模块不填它。
//!
//! # 由调用方显式调用
//!
//! 这些变换**不会**被任何解析器自动调用。它们改变既有格式的解析结果——文本里的括号会被
//! 搬进 `sub_line`、只有标签的行会消失、断句的行会并成一行——自动执行就是对所有格式的
//! 破坏性改动；它们还需要曲目的歌手列表，并且要跟其它后处理争顺序。与
//! `helpers::optimization` 里的那些变换一样，这里是调用方按需执行的一组函数。
//!
//! 建议的顺序：
//!
//! 1. [`apply_speaker_labels`]——只有标签的行要赶在被丢掉之前把分边读出来，而分边决定了
//!    后面「同一侧才合并」的判断；
//! 2. [`split_background_vocals`]——要在逐词时间被合并成单词之前切开括号短语；
//! 3. 配对翻译——这时每一行只带着自己那一行的翻译（[`unwrap_brackets`] 用来去掉短语译文
//!    外面的括号）；
//! 4. [`fold_bracketed_echoes`]——独占一行的括号短语先收下属于它的翻译，再并进它回答的那
//!    一行；
//! 5. [`merge_continued_lines`]——最后把断句的行并成一行，连各自的翻译一起并上。
//!
//! # 示例
//!
//! ```
//! use lyrics_core::helpers::conventions::{apply_speaker_labels, split_background_vocals};
//! use lyrics_core::{LineInfo, LyricsAlignment, SyllableInfo};
//!
//! // 转录写下来的样子：独占一行的标签，加上一行里带括号短语的歌词。
//! let mut lines = vec![
//!     LineInfo::new_line("某某：".to_string(), Some(0), None),
//!     LineInfo::new_syllable_with_time(
//!         vec![
//!             SyllableInfo::new("Hold ".to_string(), 1_000, 1_500).into(),
//!             SyllableInfo::new("on (ohyeah)".to_string(), 1_500, 2_500).into(),
//!         ],
//!         Some(1_000),
//!         None,
//!     ),
//! ];
//!
//! apply_speaker_labels(&mut lines, &["某某".to_string()]);
//! split_background_vocals(&mut lines);
//!
//! assert_eq!(lines.len(), 1, "只有标签的那一行被删掉了");
//! assert_eq!(lines[0].text_from_any(), "Hold on");
//! assert_eq!(lines[0].alignment(), LyricsAlignment::Left);
//! assert_eq!(
//!     lines[0].sub_line().map(LineInfo::text_from_any).as_deref(),
//!     Some("ohyeah")
//! );
//! ```

use std::collections::HashMap;

use crate::helpers::string_helper::collapse_whitespace;
use crate::models::{FullSyllableInfo, LineInfo, LyricsAlignment, SyllableInfo, SyllableItem};

/// 联合署名写下的一位表演者与另一位之间的分隔符。
const LABEL_SEPARATORS: [char; 5] = ['/', '&', ',', '，', '、'];

/// 转录写「两人一起唱」的几种叫法。
const JOINT_LABELS: [&str; 5] = ["both", "all", "合唱", "齐唱", "合"];

/// 转录把一句话留着没说完的几种标点。
const OPEN_MARKS: [char; 7] = [',', '，', '、', ';', '；', ':', '：'];

/// 续上的那一行最晚可以在它所续的那一行之后多久开始（毫秒）。
///
/// 这里所有材料里写成续句的行都在它所续的那一行的一百毫秒之内开始，而在一行下面重复的
/// 副歌是每半秒一行。
const CONTINUATION_GAP_MS: i32 = 250;

/// 说前一句话已经说完的标点。
const CLOSING_MARKS: [char; 13] = [
    '.', '。', '!', '！', '?', '？', '…', '"', '“', '”', ')', '）', '】',
];

/// 括号短语最晚可以在它回答的那一行之后多久开始（毫秒）。
///
/// QQ 音乐通常把一句回声定在它回答的那一行唱完的时刻，但它回答的是那句话而不是那一行：
/// 《Saddle Up》的伴唱在那一行留出的停顿之后才起，晚了一秒半。两秒足够吸收这种停顿，
/// 又不会够到后面另一段的括号句。
const ECHO_GAP_MS: i32 = 2_000;

/// 把 `lines` 里的说话人标签读成每一行的分边。
///
/// 标签得点到 `artists` 里的某一位——就是提供这批歌词的那一方署名的那几位，所以一句歌词
/// 里普通的冒号不会被当成标签。独占一行的标签给它后面的行定下分边，标签本身被删掉；写在行
/// 里的标签从显示文本与逐词时间里都去掉。
///
/// 分边按 [`LyricsAlignment`] 写：署名里排第一的那位是左，其余是右；合唱、合、Both、All
/// 这类标签把这一段交回第一个声部，与下游把两人一起唱的一段交回主声部的处理一致。任何一行
/// 都没有标签时，本函数不改动 `alignment` —— 转录什么都没说，就不替它说话。
pub fn apply_speaker_labels(lines: &mut Vec<LineInfo>, artists: &[String]) {
    let mut current_alignment = LyricsAlignment::Unspecified;

    lines.retain_mut(|line| {
        let text = line.text_from_any();
        if let Some((alignment, content_start)) = speaker_label(&text, artists) {
            current_alignment = alignment;
            strip_speaker_label(line, content_start, text[..content_start].chars().count());
        }
        // 一行都没有标签时不动它原来的分边（解析器可能已经写过）。
        if current_alignment != LyricsAlignment::Unspecified {
            set_alignment(line, current_alignment);
        }
        !is_blank(line)
    });
}

/// 读出一个说话人标签，它可能一次署名好几位表演者。
///
/// 联合署名的写法是主唱在前，所以标签点到的第一位也是提供方列出来的表演者决定分边：一起
/// 唱的一段归署名里靠前的那位。点到一位就够了，这样提供方没列进歌手表的那位客串可以跟着
/// 旁边的名字一起进来。一个名字都没点、只说两人一起唱的标签由 [`is_joint_label`] 交给主声部。
fn speaker_label(text: &str, artists: &[String]) -> Option<(LyricsAlignment, usize)> {
    let (colon_start, colon) = text
        .char_indices()
        .find(|(_, character)| matches!(character, ':' | '：'))?;
    let label = text[..colon_start].trim();
    if label.is_empty() {
        return None;
    }

    let after_colon = colon_start + colon.len_utf8();
    let content = text[after_colon..].trim_start();
    let content_start = text.len() - content.len();

    // 一个名字都没点到的标签说两人一起唱，它打开的那一行跟点了名字的那一行一样读。
    if is_joint_label(label) {
        return Some((LyricsAlignment::Left, content_start));
    }

    let leading = label.split(LABEL_SEPARATORS).find_map(|name| {
        let name = normalize_artist(name);
        artists
            .iter()
            .position(|artist| !name.is_empty() && name == normalize_artist(artist))
    })?;

    let alignment = if leading == 0 {
        LyricsAlignment::Left
    } else {
        LyricsAlignment::Right
    };
    Some((alignment, content_start))
}

/// 返回 `label` 是否说它打开的那些行由两人一起唱。
///
/// 把一首歌分给两位表演者的转录，把分边写成点到其中一位的标签，而两人一起唱的一段写成点到
/// 两位的标签；提供方对这种段落没有名字时写 `Both：`——《Save Your Tears (Remix)》的最后
/// 一段副歌就是这样交回两人手上的。这样的一段属于主声部，也就是 AMLL 歌词库给两人一起唱的
/// 副歌的那一边。
fn is_joint_label(label: &str) -> bool {
    JOINT_LABELS.contains(&normalize_artist(label).as_str())
}

/// 从一行里去掉说话人标签。
///
/// `content_start` 是标签之后那段内容在文本里的字节下标，`characters` 是它前面有几个字符：
/// 文本按字节切，词按字符去。
fn strip_speaker_label(line: &mut LineInfo, content_start: usize, characters: usize) {
    match line {
        LineInfo::Line { text, .. } | LineInfo::FullLine { text, .. } => {
            text.drain(..content_start);
        }
        LineInfo::Syllable { syllables, .. } | LineInfo::FullSyllable { syllables, .. } => {
            strip_leading_characters(syllables, characters);
        }
    }
}

/// 去掉音节列表开头的几个字符，连因此空掉的词一起。
fn strip_leading_characters(items: &mut Vec<SyllableItem>, characters: usize) {
    let mut remaining = characters;
    items.retain_mut(|item| {
        if remaining == 0 {
            return true;
        }
        let count = item_characters(item);
        if remaining >= count {
            remaining -= count;
            return false;
        }
        strip_item_leading(item, remaining);
        remaining = 0;
        true
    });
}

/// 从一个音节项开头去掉几个字符（合并音节从它的第一个子音节去掉）。
fn strip_item_leading(item: &mut SyllableItem, characters: usize) {
    let mut remaining = characters;
    for part in item.parts_mut() {
        if remaining == 0 {
            break;
        }
        let count = part.text.chars().count();
        if remaining >= count {
            part.text.clear();
            remaining -= count;
            continue;
        }
        let byte_offset = part
            .text
            .char_indices()
            .nth(remaining)
            .map_or(part.text.len(), |(index, _)| index);
        part.text.drain(..byte_offset);
        remaining = 0;
    }
}

/// 设置一行的分边，子行跟着它一起（子行与主行由同一位唱）。
fn set_alignment(line: &mut LineInfo, alignment: LyricsAlignment) {
    line.set_alignment(alignment);
    if let Some(sub_line) = line.sub_line_mut() {
        sub_line.set_alignment(alignment);
    }
}

/// 返回这一行是否一个字符都没写（音节行看它的词）。
fn is_blank(line: &LineInfo) -> bool {
    match line {
        LineInfo::Line { text, .. } | LineInfo::FullLine { text, .. } => text.trim().is_empty(),
        LineInfo::Syllable { .. } | LineInfo::FullSyllable { .. } => {
            !line.syllables().unwrap_or_default().iter().any(has_text)
        }
    }
}

/// 归一化歌手名或点到某个歌手的标签。
///
/// 提供方的署名与它写在歌词里的标签在大小写、空格和标点上都不一样，所以只比字母与数字。
fn normalize_artist(value: &str) -> String {
    let cleaned: String = value
        .to_lowercase()
        .chars()
        .map(|character| {
            if character.is_alphanumeric() {
                character
            } else {
                ' '
            }
        })
        .collect();
    collapse_whitespace(&cleaned)
}

/// 切开一个括号短语，把它变成它所在行的背景和声。
///
/// QRC 与 LRC 都没有背景和声字段，有的是转录写下来的约定：伴唱写在它所回答的那一行里，
/// 作为一段带自己时间的括号短语。网易云《Umbrella》开场是 `Ahuh Ahuh （Yea Rihanna）`，
/// 括号本身是两个零长度的音节；《I'm In Love With a Monster》把短语写在行中间——
/// `I'm in love (we're in love) with a monster`——两人合唱的词就站在把短语引出来的词与
/// 接下去的词之间。两种写法都留下这一行自己的词，在短语原来的位置上接起来。
///
/// 规则保持很窄，因为括号本身不构成证据：短语旁边得留着唱出来的文字，短语里得有字母或
/// 数字，而且这一行得有逐词时间。因此 `(instrumental)` 这样的旁白、整行只有一段括号的
/// 行、只有行级时间的来源，都保留括号、仍旧是一行普通歌词。
pub fn split_background_vocals(lines: &mut [LineInfo]) {
    for line in lines {
        split_background_vocal(line);
    }
}

fn split_background_vocal(line: &mut LineInfo) {
    if line.sub_line().is_some() || !is_word_timed(line) {
        return;
    }
    let text = line.text_from_any();
    let Some(phrase) = bracketed_phrase_in_line(&text) else {
        return;
    };

    let before = &text[..phrase.open];
    let after = &text[phrase.close..];
    // 短语不是这一行显示的内容，所以它前后的词按断句的两行那样接起来：中间留出视图画词
    // 流时的那一个空格。
    let head_text = join_around(before, after);
    if head_text.is_empty() {
        return;
    }

    // 短语里的括号不会空着（`bracketed_phrase_in_line` 只收有字母或数字的），所以这里
    // 只用管这一行还留没留下词。
    let mut head = take_syllables(line);
    let mut enclosed = split_syllables_at(&mut head, before.chars().count());
    let trailing = split_syllables_at(
        &mut enclosed,
        text[phrase.open..phrase.close].chars().count(),
    );
    let background = unwrap_syllable_brackets(enclosed);
    // 这一行的词就是短语被写在中间的那些词，短语在行尾时它后面没有词要接；这时短语前面
    // 的分隔符留在行尾就多余了——一行的词拼起来要正好等于这一行的文字。
    if trailing.is_empty() {
        trim_end_of_last_word(&mut head);
    } else {
        join_words(&mut head, trailing);
    }

    // 短语唱在这一行里面，所以它由这一行自己的翻译覆盖，不带自己的翻译。
    let mut sub_line = LineInfo::new_syllable(background);
    sub_line.set_alignment(line.alignment());
    line.set_sub_line(Some(Box::new(sub_line)));
    set_syllables(line, head);
}

/// 把括号短语被写在中间的那段文本接起来。
fn join_around(before: &str, after: &str) -> String {
    match (before.trim_end(), after.trim_start()) {
        ("", after) => after.to_string(),
        (before, "") => before.to_string(),
        (before, after) => format!("{before} {after}"),
    }
}

/// 把跟在括号短语后面的词接回它前面的词上。
///
/// 短语后面那个空格是转录写下它的地方的单位，短语与括号共用这个词时切开会把那个词分开：
/// `love) ` 变成 `love)` 与 ` `，那个空格两半都不属于。接回来时把它重新写上一次，因此
/// 只装着它的那半会被丢掉，接缝两头的词去掉接缝取代掉的那些空白。
fn join_words(head: &mut Vec<SyllableItem>, mut trailing: Vec<SyllableItem>) {
    let separators = trailing
        .iter()
        .take_while(|item| !item.parts().iter().any(|part| !part.text.trim().is_empty()))
        .count();
    trailing.drain(..separators);
    trim_start_of_first_word(&mut trailing);
    trim_end_of_last_word(head);
    continue_words(head, trailing);
}

/// 去掉包在背景和声外面的括号。
///
/// 括号属于这句短语，不属于唱它的那些词，因此只装着括号的词跟着括号一起消失。
fn unwrap_syllable_brackets(mut items: Vec<SyllableItem>) -> Vec<SyllableItem> {
    if let Some(part) = items.first_mut().and_then(first_part_mut) {
        let mut rest = part.text.trim_start();
        rest = rest.strip_prefix(['(', '（']).unwrap_or(rest);
        rest = rest.trim_start();
        let skip = part.text.len() - rest.len();
        part.text.drain(..skip);
    }
    if let Some(part) = items.last_mut().and_then(last_part_mut) {
        let mut rest = part.text.trim_end();
        rest = rest.strip_suffix([')', '）']).unwrap_or(rest);
        rest = rest.trim_end();
        part.text.truncate(rest.len());
    }
    items.retain(|item| item.parts().iter().any(|part| !part.text.is_empty()));
    items
}

/// 把独占一行的括号短语并进它所回答的那一行。
///
/// [`split_background_vocals`] 读的那一半约定：网易云把伴唱写成同一行里的尾巴，QQ 音乐则
/// 写成独占一行的行，用括号括起来、定在它回答的那一行的时间上——《hate that i made you
/// love me》里的 `(My way from you)` 跟在 `Know that I will find my way from you` 后面。
/// 比一行长的短语写成连续的几行，开括号在第一行、闭括号在最后一行，《Saddle Up》就是用
/// 这种写法把副歌重复在尾声下面；而它回答的是那句话留出的停顿之后，不总是紧接上一个词。
///
/// 渲染时画出来的歌词把伴唱挂在它回答的那一行上，所以这里就把两者并起来，而不是让它们
/// 各自滚过去。回声得从它并进去的那一行内部或紧接着之后开始：歌里别处单独站着的括号句
/// 不是随便哪一行的回答。
pub fn fold_bracketed_echoes(lines: &mut Vec<LineInfo>) {
    let mut index = 1;
    while index < lines.len() {
        let rows = if lines[index - 1].sub_line().is_none() {
            echo_rows(lines, index)
        } else {
            0
        };
        if rows == 0 {
            index += 1;
            continue;
        }
        // 回声唱在它自己的位置上，所以它自己的翻译属于它，而不是它回答的那一行。
        let echo: Vec<LineInfo> = lines.drain(index..index + rows).collect();
        let alignment = lines[index - 1].alignment();
        let mut background = echo_background(echo);
        background.set_alignment(alignment);
        lines[index - 1].set_sub_line(Some(Box::new(background)));
    }
}

/// 返回从 `lines[index]` 开始的回声写成了几行。
///
/// 站在那里的东西不回答它前面那一行时返回 0。
fn echo_rows(lines: &[LineInfo], index: usize) -> usize {
    let parent = &lines[index - 1];
    let Some(rows) = bracketed_phrase(lines, index) else {
        return 0;
    };
    let phrase = &lines[index..index + rows];
    // `(instrumental)` 这样的旁白什么都不回答。
    if !phrase_text(phrase).chars().any(char::is_alphanumeric) {
        return 0;
    }
    let answers = phrase[0].start_time().is_some_and(|start| {
        start >= parent.start_time().unwrap_or_default()
            && start <= latest_time_ms(parent).saturating_add(ECHO_GAP_MS)
    });
    // 比一行长的短语是连着唱下来的，所以歌里别处没配上的括号够不到它这里来收尾。
    let back_to_back = phrase.windows(2).all(|rows| {
        let (Some(start), Some(next)) = (rows[1].start_time(), rows[0].start_time()) else {
            return false;
        };
        start <= latest_time_ms(&rows[0]).saturating_add(ECHO_GAP_MS) && next <= start
    });
    if answers && back_to_back { rows } else { 0 }
}

/// 读从 `lines[index]` 开始的那段括号短语。
///
/// 短语在文本以括号开头的那一行打开，在把它结束的那一行合上——独占一行时整行都在括号里，
/// 更长时开括号在第一行、闭括号在最后一行。返回它占了几行；文本没有写出一段合上的括号时
/// 返回 `None`。
fn bracketed_phrase(lines: &[LineInfo], index: usize) -> Option<usize> {
    if !lines[index]
        .text_from_any()
        .trim_start()
        .starts_with(['(', '（'])
    {
        return None;
    }

    let mut depth = 0_usize;
    for (offset, line) in lines[index..].iter().enumerate() {
        for character in line.text_from_any().chars() {
            match character {
                '(' | '（' => depth += 1,
                ')' | '）' => depth = depth.saturating_sub(1),
                _ => {}
            }
        }
        if depth == 0 {
            // 合上短语的那一行以合上它的括号收尾；它后面还有文字，说明括号不是这句短语的。
            return line
                .text_from_any()
                .trim_end()
                .ends_with([')', '）'])
                .then_some(offset + 1);
        }
    }
    None
}

/// 一段括号短语的文本，去掉标记它的括号。
fn phrase_text(rows: &[LineInfo]) -> String {
    rows.iter()
        .enumerate()
        .map(|(offset, row)| {
            let text = row.text_from_any();
            let mut piece = text.trim();
            if offset == 0 {
                piece = piece.strip_prefix(['(', '（']).unwrap_or(piece);
            }
            if offset + 1 == rows.len() {
                piece = piece.strip_suffix([')', '）']).unwrap_or(piece);
            }
            // `text` 是本函数的临时值，所以每段都得留下自己的字符串。
            piece.trim().to_string()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// 把回声的几行并成它所回答的那一行的背景和声。
fn echo_background(rows: Vec<LineInfo>) -> LineInfo {
    let mut rows = rows;
    let mut syllables = Vec::new();
    for row in rows.iter_mut() {
        // 词保留回声唱出来的时间，去掉短语被写进去的那对括号。
        continue_words(
            &mut syllables,
            unwrap_syllable_brackets(take_syllables(row)),
        );
    }

    let text = phrase_text(&rows);
    let translations = echo_translations(&rows);
    if syllables.is_empty() {
        let start = rows.first().and_then(|row| row.start_time());
        let end = rows.last().and_then(|row| row.end_time());
        return line_with_text(text, start, end, translations);
    }
    line_with_syllables(syllables, translations)
}

/// 返回一段短语的几行被翻译成的那个译文。
fn echo_translations(rows: &[LineInfo]) -> HashMap<String, String> {
    let mut pieces: HashMap<String, Vec<String>> = HashMap::new();
    for row in rows {
        if let Some(translations) = row.translations() {
            for (language, text) in translations {
                pieces
                    .entry(language.clone())
                    .or_default()
                    .push(unwrap_brackets(text));
            }
        }
    }

    pieces
        .into_iter()
        .filter_map(|(language, pieces)| join_pieces(pieces).map(|joined| (language, joined)))
        .collect()
}

/// 去掉短语与它的翻译被写进去的那对括号。
///
/// 提供方把自己重复的那句短语括起来，翻译也跟着一起括——`(来吧 尽管…)` 翻译的正是
/// `(Come on, just…)`。括号是转录标记这句短语的方式，不是它说的话，所以背景和声已经单独
/// 画出来时就不再画一次。
pub fn unwrap_brackets(text: &str) -> String {
    let trimmed = text.trim();
    let Some(open) = trimmed.chars().next() else {
        return String::new();
    };
    let Some(close) = matching_bracket(open) else {
        return trimmed.to_string();
    };
    if !trimmed.ends_with(close) {
        return trimmed.to_string();
    }
    trimmed[open.len_utf8()..trimmed.len() - close.len_utf8()]
        .trim()
        .to_string()
}

fn matching_bracket(open: char) -> Option<char> {
    match open {
        '(' => Some(')'),
        '（' => Some('）'),
        '[' => Some(']'),
        '【' => Some('】'),
        _ => None,
    }
}

/// 括号短语写在一行里的哪个位置。
struct BracketedPhrase {
    /// 开括号在这一行文本里的字节下标。
    open: usize,
    /// 闭括号之后的字节下标。
    close: usize,
}

/// 读出一行里最后的那段括号短语，不管它写在行的什么位置。
///
/// 短语括在它所回答的那一行里，而一行回答的是它最后括起来的那句：网易云把短语写在行尾，
/// 而 `I'm in love (we're in love) with a monster` 的中间正是短语回答的位置。
/// `(instrumental)` 这样的旁白读不出东西，因为唱出来的短语是在说一件事：括号里有字母或
/// 数字。
fn bracketed_phrase_in_line(text: &str) -> Option<BracketedPhrase> {
    let mut phrase = None;
    let mut open = None;
    let mut inner = None;
    let mut depth = 0_usize;

    for (index, character) in text.char_indices() {
        match character {
            '(' | '（' => {
                if depth == 0 {
                    open = Some(index);
                    inner = Some(index + character.len_utf8());
                }
                depth += 1;
            }
            ')' | '）' => {
                if depth == 0 {
                    continue;
                }
                depth -= 1;
                if depth > 0 {
                    continue;
                }
                let (Some(bracket), Some(start)) = (open.take(), inner.take()) else {
                    continue;
                };
                if text[start..index].trim().chars().any(char::is_alphanumeric) {
                    phrase = Some(BracketedPhrase {
                        open: bracket,
                        close: index + character.len_utf8(),
                    });
                }
            }
            _ => {}
        }
    }

    phrase
}

/// 按字符下标切开音节列表，返回尾部。
///
/// 跨过边界的那个音节按字符数分它的时间，因此两半在时间上仍然接连；下标在末尾之后时返回
/// 空，调用方读作「不切」。
fn split_syllables_at(items: &mut Vec<SyllableItem>, character_index: usize) -> Vec<SyllableItem> {
    let mut seen = 0_usize;
    for index in 0..items.len() {
        if seen == character_index {
            return items.split_off(index);
        }
        let count = item_characters(&items[index]);
        if seen + count > character_index {
            let divided = divide_item(&mut items[index], character_index - seen);
            let mut tail = items.split_off(index + 1);
            tail.insert(0, divided);
            return tail;
        }
        seen += count;
    }
    Vec::new()
}

/// 把一个音节项按其中的字符下标切开，返回后半。
///
/// 合并音节在它的子音节上递归切开，两半都还是合并音节。
fn divide_item(item: &mut SyllableItem, character_offset: usize) -> SyllableItem {
    match item {
        SyllableItem::Syllable(syllable) => {
            SyllableItem::Syllable(divide_syllable(syllable, character_offset))
        }
        SyllableItem::Full(full) => SyllableItem::Full(FullSyllableInfo::new(split_sub_items(
            full,
            character_offset,
        ))),
    }
}

/// 把一个合并音节按其中的字符下标切开，返回后半的子音节。
fn split_sub_items(full: &mut FullSyllableInfo, character_offset: usize) -> Vec<SyllableInfo> {
    let mut seen = 0_usize;
    for index in 0..full.sub_items().len() {
        if seen == character_offset {
            return full.sub_items_mut().split_off(index);
        }
        let count = full.sub_items()[index].text.chars().count();
        if seen + count > character_offset {
            let divided =
                divide_syllable(&mut full.sub_items_mut()[index], character_offset - seen);
            let mut tail = full.sub_items_mut().split_off(index + 1);
            tail.insert(0, divided);
            return tail;
        }
        seen += count;
    }
    Vec::new()
}

/// 把一个普通音节按其中的字符下标切开，返回后半。
fn divide_syllable(syllable: &mut SyllableInfo, character_offset: usize) -> SyllableInfo {
    let characters = syllable.text.chars().count();
    let byte_offset = syllable
        .text
        .char_indices()
        .nth(character_offset)
        .map_or(syllable.text.len(), |(index, _)| index);
    let span = syllable.end_time.saturating_sub(syllable.start_time);
    // 只有文本跨过边界的音节才会被切开，所以它的字符数与其中的下标都至少是 1。
    let boundary = syllable
        .start_time
        .saturating_add(span.saturating_mul(character_offset as i32) / characters as i32);

    let text = syllable.text.split_off(byte_offset);
    let divided = SyllableInfo::new(text, boundary, syllable.end_time);
    syllable.end_time = boundary;
    divided
}

/// 把 `words` 接到先写下的那一段词后面。
///
/// 续上的那一行写作时中间是不带空格的，因为断句的地方就在那里，所以接缝前的最后一个词带着
/// 那个分隔符。视图画的是词流而不是文本，留在词流之外的空格会让两段词在屏幕上粘在一起。
fn continue_words(target: &mut Vec<SyllableItem>, mut words: Vec<SyllableItem>) {
    if let Some(part) = target.last_mut().and_then(last_part_mut) {
        if !part.text.ends_with(char::is_whitespace) {
            part.text.push(' ');
        }
    }
    target.append(&mut words);
}

/// 把分几行写下的文本按中间的空格接成一句。
///
/// 哪一段没有内容那里就什么都没说，只装着分隔符的一段也不是文本。
fn join_pieces(pieces: impl IntoIterator<Item = String>) -> Option<String> {
    let text = pieces
        .into_iter()
        .map(|piece| piece.trim().to_string())
        .filter(|piece| !piece.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    (!text.is_empty()).then_some(text)
}

/// 按 `translations` 是否为空构造一行文本行。
fn line_with_text(
    text: String,
    start_time: Option<i32>,
    end_time: Option<i32>,
    translations: HashMap<String, String>,
) -> LineInfo {
    if translations.is_empty() {
        LineInfo::new_line(text, start_time, end_time)
    } else {
        LineInfo::new_full_line(text, start_time, end_time, translations, None)
    }
}

/// 按 `translations` 是否为空构造一行音节行（时间由音节推导）。
fn line_with_syllables(
    syllables: Vec<SyllableItem>,
    translations: HashMap<String, String>,
) -> LineInfo {
    if translations.is_empty() {
        LineInfo::new_syllable(syllables)
    } else {
        LineInfo::new_full_syllable(syllables, translations, None)
    }
}

/// 取走一行的音节，行里留下空列表。
fn take_syllables(line: &mut LineInfo) -> Vec<SyllableItem> {
    line.syllables_mut().map(std::mem::take).unwrap_or_default()
}

/// 把音节放回一行。
fn set_syllables(line: &mut LineInfo, items: Vec<SyllableItem>) {
    if let Some(syllables) = line.syllables_mut() {
        *syllables = items;
    }
}

/// 返回这一行是否带逐词时间。
fn is_word_timed(line: &LineInfo) -> bool {
    line.syllables()
        .is_some_and(|items| items.iter().any(has_text))
}

/// 返回这一行提到的最晚时刻：行时间与音节时间的较晚者（不含子行）。
fn latest_time_ms(line: &LineInfo) -> i32 {
    let start = line.start_time().unwrap_or_default();
    let from_syllables = line
        .syllables()
        .unwrap_or_default()
        .iter()
        .map(SyllableItem::end_time)
        .max()
        .unwrap_or(start);
    line.end_time()
        .unwrap_or(start)
        .max(from_syllables)
        .max(start)
}

/// 返回一个音节项是否写了内容。
fn has_text(item: &SyllableItem) -> bool {
    item.parts().iter().any(|part| !part.text.trim().is_empty())
}

/// 返回一个音节项写下的字符数。
fn item_characters(item: &SyllableItem) -> usize {
    item.parts()
        .iter()
        .map(|part| part.text.chars().count())
        .sum()
}

/// 返回一个音节项第一个普通音节的可变引用（合并音节取它的第一个子音节）。
fn first_part_mut(item: &mut SyllableItem) -> Option<&mut SyllableInfo> {
    item.parts_mut().first_mut()
}

/// 返回一个音节项最后一个普通音节的可变引用（合并音节取它的最后一个子音节）。
fn last_part_mut(item: &mut SyllableItem) -> Option<&mut SyllableInfo> {
    item.parts_mut().last_mut()
}

/// 去掉音节列表开头那个词前面的空白。
fn trim_start_of_first_word(items: &mut [SyllableItem]) {
    if let Some(part) = items.first_mut().and_then(first_part_mut) {
        let blank = part.text.len() - part.text.trim_start().len();
        part.text.drain(..blank);
    }
}

/// 去掉音节列表末尾那个词后面的空白。
fn trim_end_of_last_word(items: &mut [SyllableItem]) {
    if let Some(part) = items.last_mut().and_then(last_part_mut) {
        let end = part.text.trim_end().len();
        part.text.truncate(end);
    }
}

/// 把一句话断开写的几行并进开始这句话的那一行。
///
/// 一行写不下的转录者把这句话剩下的部分写在它后面那一行，断在哪里由行的长度决定：
/// 《Saddle Up》先写 `But I had enough,` 再写 `so I move onto the next thing`，先写
/// `Put your money` 再写 `where your mouth is`，后一行都定在它所续的那一行唱完的时刻。
/// 几行是一句话，所以这里把它们并成渲染时画出来的那一行。
///
/// 后面那一行说的是什么，决定它属不属于前面那一行：以小写词开头的行把句子接下去，因为
/// 转录者开始新的一句时用大写。英文的 `I` 在它是这句话所依靠的那个词时大写，而它自己
/// 起一句的时候也一样多，所以它只在前面那一行把标点留着没合上时才接下去，就像
/// `A tragedy, Ms. RIP,` 那样。前面那一行得说得出这件事：被标点合上的行结束了它那一句，
/// 而结尾是没有大小写的文字的行什么都不说，因为那种语言的转录者断在行写不下的地方，
/// 而不是句子结束的地方。前面那一行只用最后一个字母的大小写说了话时，时间也得说同样的话：
/// 给每个词都计时的转录把续句写成从它所续的那一行的最后一个词开始，所以跟前面那一行时间
/// 分开的行是它自己的一行——《WDA (Whole Different Animal)》的副歌先写
/// `She a Whole Different Animal` 再写 `different animal`，晚了半秒、还带着自己的译文，
/// 两行都留着。
///
/// 这件事在译文配对之后做，因为在那之前一行带着的是它自己那一行的译文；也在括号回声被并走
/// 之后做，这样一行所回答的那句短语留在它原来写的地方。
pub fn merge_continued_lines(lines: &mut Vec<LineInfo>) {
    let mut index = 0;
    while index + 1 < lines.len() {
        if !continues(&lines[index], &lines[index + 1]) {
            index += 1;
            continue;
        }
        // 一句话可能断不止一次，所以接过一行的行要再看一次，而不是跨过去。
        let tail = lines.remove(index + 1);
        let head = std::mem::replace(&mut lines[index], LineInfo::new_line_simple(String::new()));
        lines[index] = join_continuation(head, tail);
    }
}

/// 返回 `tail` 是不是 `first` 开始的那句话剩下的部分。
fn continues(first: &LineInfo, tail: &LineInfo) -> bool {
    let text = first.text_from_any();
    let Some(last) = text.trim_end().chars().next_back() else {
        return false;
    };
    // 被标点合上的行说它那一句结束了，后面跟着什么都一样。
    if CLOSING_MARKS.contains(&last) {
        return false;
    }
    // 一行用留着的标点说这句话还要继续，或者用转录者选了大小的那个字母说；没有大小写的
    // 文字断在行写不下的地方，什么都没说。
    let left_open = OPEN_MARKS.contains(&last);
    let sentence_goes_on = left_open || has_case(last);
    first.alignment() == tail.alignment()
        // 一行只带一个背景和声，把两句并起来就会丢掉其中一个。
        && !(first.sub_line().is_some() && tail.sub_line().is_some())
        // 接起来的一行用的词是两行各自的词，所以只有两行写法相同（都逐词计时或都只有行级
        // 时间，文字才拼得一样）时才接。
        && is_word_timed(first) == is_word_timed(tail)
        // 留着没合上的标点是它自己的证据，不管后面那一行的时间；只用大小写说话的行，只有
        // 在紧接它唱完的地方开始的那一行才接得下去。
        && (left_open || follows_flush(first, tail))
        && opens_a_continuation(tail.text_from_any().trim_start(), sentence_goes_on, left_open)
}

/// 返回 `tail` 是否从它所续的那一行唱完的地方开始。
///
/// 给每个词都计时的转录把续句写成从它所续的那一行的最后一个词开始——《Saddle Up》的
/// `where your mouth is` 从 `Put your money` 结束的地方开始——所以跟前面那一行时间分开的
/// 行是它自己的一行：《WDA (Whole Different Animal)》的副歌先写
/// `She a Whole Different Animal` 再写 `different animal`，晚了半秒、还带着自己的译文，
/// 那是这句吟唱的另一行而不是它的续句。只有行级时间的那一行关于它写下的后一行什么都没说，
/// 那就只看大小写。
fn follows_flush(first: &LineInfo, tail: &LineInfo) -> bool {
    let ends_at = latest_time_ms(first);
    if ends_at == first.start_time().unwrap_or_default() {
        return true;
    }
    tail.start_time()
        .is_some_and(|start| start <= ends_at.saturating_add(CONTINUATION_GAP_MS))
}

/// 返回 `text` 是否以把句子接下去的词开头。
///
/// `sentence_goes_on` 是它前面那一行有没有说这句话还要继续，`left_open` 是它有没有用留着
/// 的标点这么说。
fn opens_a_continuation(text: &str, sentence_goes_on: bool, left_open: bool) -> bool {
    let mut characters = text.chars();
    match characters.next() {
        Some(first) if first.is_lowercase() => sentence_goes_on,
        // 英文的 `I` 自己起一句跟接一句一样多，所以只有说这句话还要继续的标点才算数。
        Some('I') => left_open && matches!(characters.next(), None | Some(' ' | '\'' | '’')),
        _ => false,
    }
}

/// 返回 `character` 是不是转录者选了大小的字母。
fn has_case(character: char) -> bool {
    character.is_uppercase() || character.is_lowercase()
}

/// 把一句话剩下的部分接到开始它的那一行上。
///
/// 一行的读法是在解析之后、按这里并完的样子生成的，所以只把转录说了的东西带过来。
fn join_continuation(mut first: LineInfo, mut tail: LineInfo) -> LineInfo {
    let end_time = Some(latest_time_ms(&first).max(latest_time_ms(&tail)));

    // 一句话的文字是两行各自的文字接起来的：视图画的是词流，所以接的是词。
    match &mut first {
        LineInfo::Line { text, .. } | LineInfo::FullLine { text, .. } => {
            *text = format!("{} {}", text.trim_end(), tail.text_from_any().trim_start());
        }
        LineInfo::Syllable { syllables, .. } | LineInfo::FullSyllable { syllables, .. } => {
            continue_words(syllables, take_syllables(&mut tail));
        }
    }

    // 一行带着自己那一行的译文，所以接起来的一句话带着两句的译文。拼音不像译文那样接起来
    // （下游也不接）：接起来的一行保留第一行原有的拼音。
    let tail_translations = take_translations(&mut tail);
    if !tail_translations.is_empty() {
        let merged = join_translations(take_translations(&mut first), tail_translations);
        set_translations(&mut first, merged);
    }

    // 回声是接着它回答的那句话写的，所以它跟着这句话一起并过去。
    if first.sub_line().is_none() {
        first.set_sub_line(tail.take_sub_line());
    }
    set_end_time(&mut first, end_time);
    first
}

/// 把译文写到行上（不是 Full 变体的行升级为 Full 变体，拼音保持不变）。
fn set_translations(line: &mut LineInfo, translations: HashMap<String, String>) {
    match line {
        LineInfo::FullLine {
            translations: existing,
            ..
        }
        | LineInfo::FullSyllable {
            translations: existing,
            ..
        } => *existing = translations,
        other => {
            let owned = std::mem::replace(other, LineInfo::new_line_simple(String::new()));
            *other = owned.to_full_line(translations, None);
        }
    }
}

/// 把两行的译文按语言接成一句（某一行的某一种语言空着时就不接）。
fn join_translations(
    mut first: HashMap<String, String>,
    tail: HashMap<String, String>,
) -> HashMap<String, String> {
    for (language, text) in tail {
        match first.get(&language) {
            Some(piece) => {
                let joined = join_pieces([piece.clone(), text]).unwrap_or_default();
                first.insert(language, joined);
            }
            None => {
                first.insert(language, text);
            }
        }
    }
    first
}

/// 取走一行的译文。
fn take_translations(line: &mut LineInfo) -> HashMap<String, String> {
    line.translations_mut()
        .map(std::mem::take)
        .unwrap_or_default()
}

/// 把一句话结束的时刻写到行上。
fn set_end_time(line: &mut LineInfo, end_time: Option<i32>) {
    match line {
        LineInfo::Line { end_time: end, .. }
        | LineInfo::Syllable { end_time: end, .. }
        | LineInfo::FullLine { end_time: end, .. }
        | LineInfo::FullSyllable { end_time: end, .. } => *end = end_time,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 一个音节。
    fn syllable(start_time: i32, end_time: i32, text: &str) -> SyllableItem {
        SyllableInfo::new(text.to_string(), start_time, end_time).into()
    }

    /// 一行歌词：没有音节时是文本行，有音节时是音节行（音节必须正好拼出 `text`）。
    fn line(
        start_time: i32,
        end_time: Option<i32>,
        text: &str,
        syllables: Vec<SyllableItem>,
    ) -> LineInfo {
        if syllables.is_empty() {
            return LineInfo::new_line(text.to_string(), Some(start_time), end_time);
        }
        let spelled = LineInfo::text_from_syllables(&syllables);
        assert_eq!(spelled, text, "测试写的音节要正好拼出这一行的文字");
        LineInfo::new_syllable_with_time(syllables, Some(start_time), end_time)
    }

    /// 一行的背景和声文本。
    fn background_text(line: &LineInfo) -> Option<String> {
        line.sub_line().map(LineInfo::text_from_any)
    }

    /// 一行的词，合并音节展开成子音节。
    fn texts(line: &LineInfo) -> Vec<String> {
        line.syllables()
            .unwrap_or_default()
            .iter()
            .map(SyllableItem::text)
            .collect()
    }

    #[test]
    fn a_bracketed_tail_becomes_a_background_vocal() {
        // 网易云写《Umbrella》伴唱的形状：写在它所回答的那一行里的括号尾巴，括号本身是
        // 两个零长度的音节。
        let mut lines = vec![line(
            0,
            Some(4_650),
            "Ahuh Ahuh （Yea Rihanna）",
            vec![
                syllable(0, 180, "Ahuh "),
                syllable(180, 270, "Ahuh "),
                syllable(270, 270, "（"),
                syllable(270, 3_990, "Yea "),
                syllable(3_990, 4_650, "Rihanna"),
                syllable(4_650, 4_650, "）"),
            ],
        )];

        split_background_vocals(&mut lines);

        assert_eq!(lines[0].text_from_any(), "Ahuh Ahuh");
        assert_eq!(background_text(&lines[0]).as_deref(), Some("Yea Rihanna"));
        assert_eq!(
            texts(&lines[0]),
            vec!["Ahuh ", "Ahuh"],
            "行尾不再留着短语前那个分隔符"
        );
        let background = lines[0].sub_line().expect("尾巴被切了出来");
        assert_eq!(
            (background.start_time(), background.end_time()),
            (Some(270), Some(4_650)),
            "尾巴唱在它自己的词所在的位置上"
        );
    }

    #[test]
    fn a_bracketed_tail_inside_a_syllable_is_divided_by_its_characters() {
        let mut lines = vec![line(
            1_000,
            Some(3_000),
            "Hold on (ohyeah)",
            vec![
                syllable(1_000, 1_500, "Hold "),
                syllable(1_500, 2_500, "on (ohyeah)"),
            ],
        )];

        split_background_vocals(&mut lines);

        assert_eq!(lines[0].text_from_any(), "Hold on");
        assert_eq!(background_text(&lines[0]).as_deref(), Some("ohyeah"));
        assert_eq!(texts(&lines[0]), vec!["Hold ", "on"]);
        assert_eq!(
            lines[0]
                .sub_line()
                .map(|background| (background.start_time(), background.end_time())),
            Some((Some(1_772), Some(2_500))),
            "尾巴留下的那一半从它留下的字符开始"
        );
        // 被切开的两半在时间上仍然接连。
        assert_eq!(
            lines[0].syllables().unwrap()[1].end_time(),
            1_500 + 1_000 * 3 / 11
        );
    }

    /// 给一行写上中文译文（没有 Full 变体时升级为 Full 变体）。
    fn with_chinese(line: LineInfo, text: &str) -> LineInfo {
        line.to_full_line(HashMap::from([("zh".to_string(), text.to_string())]), None)
    }

    #[test]
    fn a_bracketed_phrase_inside_the_line_becomes_a_background_vocal() {
        // 《I'm In Love With a Monster》把两人合唱的词写在它所回答的那一行中间，这一行
        // 在短语之后还接着唱下去。
        let mut lines = vec![line(
            1_849,
            Some(5_627),
            "I'm in love (we're in love) with a monster",
            vec![
                syllable(1_849, 2_029, "I'm "),
                syllable(2_029, 2_219, "in "),
                syllable(2_219, 2_928, "love "),
                syllable(2_928, 3_308, "("),
                syllable(3_308, 3_488, "we're "),
                syllable(3_488, 3_918, "in "),
                syllable(3_918, 4_437, "love) "),
                syllable(4_437, 4_607, "with "),
                syllable(4_607, 4_787, "a "),
                syllable(4_787, 5_627, "monster"),
            ],
        )];

        split_background_vocals(&mut lines);

        assert_eq!(lines[0].text_from_any(), "I'm in love with a monster");
        assert_eq!(
            texts(&lines[0]),
            vec!["I'm ", "in ", "love ", "with ", "a ", "monster"],
            "这一行画出来的词是短语两边的词"
        );
        let background = lines[0].sub_line().expect("短语被切了出来");
        assert_eq!(background.text_from_any(), "we're in love");
        assert_eq!(
            texts(background),
            vec!["we're ", "in ", "love"],
            "括号不是唱出来的，所以不是短语的词"
        );
        assert_eq!(
            (background.start_time(), background.end_time()),
            (Some(3_308), Some(4_350)),
            "短语唱在它自己的词所在的位置上，在这一行里面"
        );
    }

    #[test]
    fn a_bracketed_phrase_the_line_opens_with_becomes_a_background_vocal() {
        let mut lines = vec![line(
            0,
            Some(2_000),
            "(Oh) I love it",
            vec![
                syllable(0, 300, "(Oh) "),
                syllable(300, 700, "I "),
                syllable(700, 1_200, "love "),
                syllable(1_200, 2_000, "it"),
            ],
        )];

        split_background_vocals(&mut lines);

        assert_eq!(lines[0].text_from_any(), "I love it");
        assert_eq!(texts(&lines[0]), vec!["I ", "love ", "it"]);
        assert_eq!(background_text(&lines[0]).as_deref(), Some("Oh"));
    }

    #[test]
    fn a_bracketed_tail_without_words_is_left_alone() {
        let mut lines = vec![line(
            1_000,
            Some(3_000),
            "Wait (...)",
            vec![
                syllable(1_000, 2_000, "Wait "),
                syllable(2_000, 3_000, "(...)"),
            ],
        )];

        split_background_vocals(&mut lines);

        assert_eq!(lines[0].text_from_any(), "Wait (...)");
        assert!(lines[0].sub_line().is_none());
    }

    #[test]
    fn a_line_timed_source_keeps_its_brackets() {
        // 只有行级时间的 LRC 行没有可以用来切开尾巴的逐词时间。
        let mut lines = vec![line(0, None, "Know the way (My way)", Vec::new())];

        split_background_vocals(&mut lines);

        assert_eq!(lines[0].text_from_any(), "Know the way (My way)");
        assert!(lines[0].sub_line().is_none());
    }

    #[test]
    fn a_wholly_bracketed_line_joins_the_line_it_echoes() {
        // QQ 音乐写《hate that i made you love me》的形状：独占一行的括号句，定在它所回答
        // 的那一行唱完的时刻。
        let mut lines = vec![
            line(
                1_000,
                Some(3_000),
                "Know the way",
                vec![
                    syllable(1_000, 2_000, "Know "),
                    syllable(2_000, 3_000, "the way"),
                ],
            ),
            line(
                3_000,
                Some(4_000),
                "(My way)",
                vec![
                    syllable(3_000, 3_100, "("),
                    syllable(3_100, 4_000, "My way)"),
                ],
            ),
        ];

        fold_bracketed_echoes(&mut lines);

        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].text_from_any(), "Know the way");
        assert_eq!(background_text(&lines[0]).as_deref(), Some("My way"));
    }

    #[test]
    fn a_folded_echo_keeps_its_own_translation() {
        // 回声译在它被唱出来的地方，所以它的译文跟着它走，而不是跟着它不再是的那个行丢掉。
        let mut lines = vec![
            with_chinese(
                line(
                    1_000,
                    Some(3_000),
                    "Know the way",
                    vec![syllable(1_000, 3_000, "Know the way")],
                ),
                "要知道方法",
            ),
            with_chinese(
                line(
                    3_000,
                    Some(4_000),
                    "(My way)",
                    vec![syllable(3_000, 4_000, "(My way)")],
                ),
                "（从我身边离开的方法）",
            ),
        ];

        fold_bracketed_echoes(&mut lines);

        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].chinese_translation(), Some("要知道方法"));
        let background = lines[0].sub_line().expect("回声被并了进来");
        assert_eq!(background.text_from_any(), "My way");
        assert_eq!(
            (background.start_time(), background.end_time()),
            (Some(3_000), Some(4_000)),
            "回声唱在它自己的词所在的位置上，不是它前面那一行唱完的地方"
        );
        assert_eq!(
            texts(background),
            vec!["My way"],
            "括号不是唱出来的，所以不是回声的词"
        );
        assert_eq!(
            background.chinese_translation(),
            Some("从我身边离开的方法"),
            "短语被写进去的那对括号不在它的译文里重复"
        );
    }

    #[test]
    fn a_bracketed_echo_answers_the_rest_its_line_leaves() {
        // 《Saddle Up》的伴唱：`Are you man enough to hold it down` 唱完之后留出停顿，
        // 它的回声一秒半之后才起。
        let mut lines = vec![
            line(
                1_000,
                Some(3_172),
                "Are you man enough to hold it down",
                vec![syllable(1_000, 3_172, "Are you man enough to hold it down")],
            ),
            line(
                4_639,
                Some(6_751),
                "(I wanna see you hold it down for me)",
                vec![syllable(
                    4_639,
                    6_751,
                    "(I wanna see you hold it down for me)",
                )],
            ),
        ];

        fold_bracketed_echoes(&mut lines);

        assert_eq!(lines.len(), 1);
        assert_eq!(
            lines[0].text_from_any(),
            "Are you man enough to hold it down"
        );
        assert_eq!(
            background_text(&lines[0]).as_deref(),
            Some("I wanna see you hold it down for me")
        );
    }

    #[test]
    fn a_bracketed_phrase_written_across_rows_joins_the_line_it_answers() {
        // 《Saddle Up》把副歌重复在尾声下面的写法：短语在它开始的那一行打开，在中间几行
        // 接着写，在它结束的那一行合上，而它后面那一行又自成一行。
        let mut lines = vec![
            line(
                1_000,
                Some(2_586),
                "come and drive me crazy",
                vec![syllable(1_000, 2_586, "come and drive me crazy")],
            ),
            with_chinese(
                line(
                    2_586,
                    Some(3_000),
                    "(If you walk it",
                    vec![
                        syllable(2_586, 2_700, "(If "),
                        syllable(2_700, 2_800, "you "),
                        syllable(2_800, 2_900, "walk "),
                        syllable(2_900, 3_000, "it"),
                    ],
                ),
                "如果你言行一致",
            ),
            with_chinese(
                line(
                    3_000,
                    Some(3_300),
                    "Baby, stand up",
                    vec![
                        syllable(3_000, 3_100, "Baby, "),
                        syllable(3_100, 3_200, "stand "),
                        syllable(3_200, 3_300, "up"),
                    ],
                ),
                "宝贝 请挺身而出",
            ),
            line(
                3_300,
                Some(3_391),
                "Baby, we go up)",
                vec![
                    syllable(3_300, 3_330, "Baby, "),
                    syllable(3_330, 3_360, "we "),
                    syllable(3_360, 3_390, "go "),
                    syllable(3_390, 3_391, "up)"),
                ],
            ),
            line(
                4_000,
                Some(5_000),
                "Put your money",
                vec![syllable(4_000, 5_000, "Put your money")],
            ),
        ];

        fold_bracketed_echoes(&mut lines);

        assert_eq!(lines.len(), 2);
        assert_eq!(lines[1].text_from_any(), "Put your money");
        assert_eq!(lines[0].text_from_any(), "come and drive me crazy");
        let background = lines[0].sub_line().expect("这段短语被并了进来");
        assert_eq!(
            background.text_from_any(),
            "If you walk it Baby, stand up Baby, we go up"
        );
        assert_eq!(
            LineInfo::text_from_syllables(background.syllables().unwrap()),
            background.text_from_any(),
            "词拼起来正是这段短语写下的样子"
        );
        assert_eq!(
            (background.start_time(), background.end_time()),
            (Some(2_586), Some(3_391)),
            "短语从它第一行开始的地方唱到它最后一行结束的地方"
        );
        assert_eq!(
            background.chinese_translation(),
            Some("如果你言行一致 宝贝 请挺身而出"),
            "分几行写的短语每一行都有译文"
        );
    }

    #[test]
    fn an_unmatched_bracket_does_not_reach_across_the_song() {
        // 一行开着的括号跟着它后面的几行并不是它们所属的短语：它们离本该回答的那一行太远。
        let mut lines = vec![
            line(
                1_000,
                Some(2_000),
                "Know the way",
                vec![syllable(1_000, 2_000, "Know the way")],
            ),
            line(
                2_000,
                Some(3_000),
                "(hold on",
                vec![syllable(2_000, 3_000, "(hold on")],
            ),
            line(
                20_000,
                Some(21_000),
                "sing it",
                vec![syllable(20_000, 21_000, "sing it")],
            ),
            line(
                21_000,
                Some(22_000),
                "again)",
                vec![syllable(21_000, 22_000, "again)")],
            ),
        ];

        fold_bracketed_echoes(&mut lines);

        assert_eq!(lines.len(), 4);
        assert!(lines[0].sub_line().is_none());
    }

    #[test]
    fn an_unbracketed_translation_keeps_its_text() {
        assert_eq!(unwrap_brackets("来吧 尽管…"), "来吧 尽管…");
        assert_eq!(unwrap_brackets("  (来吧 尽管…)  "), "来吧 尽管…");
        assert_eq!(unwrap_brackets("【来吧 尽管…】"), "来吧 尽管…");
        assert_eq!(
            unwrap_brackets("(来吧"),
            "(来吧",
            "落单的括号是文本的一部分"
        );
        assert_eq!(unwrap_brackets("("), "(");
    }

    #[test]
    fn a_bracketed_line_far_from_the_previous_one_stays_its_own_line() {
        // 歌里别处的括号句什么都不回答；只有跟前面那一行时间对得上的行是它的回声。
        let mut lines = vec![
            line(
                1_000,
                Some(2_000),
                "Know the way",
                vec![syllable(1_000, 2_000, "Know the way")],
            ),
            line(
                9_000,
                Some(10_000),
                "(Instrumental)",
                vec![syllable(9_000, 10_000, "(Instrumental)")],
            ),
        ];

        fold_bracketed_echoes(&mut lines);

        assert_eq!(lines.len(), 2);
        assert_eq!(lines[1].text_from_any(), "(Instrumental)");
        assert!(lines[0].sub_line().is_none());
    }

    #[test]
    fn a_bracketed_opening_line_stays_one_ordinary_line() {
        // 它前面什么都没有，没有它可以归属的那一行。
        let mut lines = vec![line(
            0,
            Some(1_000),
            "（Ella ella）",
            vec![syllable(0, 1_000, "（Ella ella）")],
        )];

        fold_bracketed_echoes(&mut lines);

        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].text_from_any(), "（Ella ella）");
        assert!(lines[0].sub_line().is_none());
    }

    /// 一行逐词时间：把 `text` 的词从 `start_ms` 到 `end_ms` 均匀计时。
    ///
    /// 《Saddle Up》的转录给每一行的每个词都计了时，断句的行就是这么接起来的。
    fn word_timed_line(start_ms: i32, end_ms: i32, text: &str) -> LineInfo {
        let words = text.split(' ').collect::<Vec<_>>();
        let last = words.len() - 1;
        let step = (end_ms - start_ms) / words.len() as i32;
        let syllables = words
            .iter()
            .enumerate()
            .map(|(index, word)| {
                let word_start = start_ms + step * index as i32;
                let word_end = if index == last {
                    end_ms
                } else {
                    word_start + step
                };
                let text = if index == last {
                    (*word).to_string()
                } else {
                    format!("{word} ")
                };
                syllable(word_start, word_end, &text)
            })
            .collect();
        line(start_ms, Some(end_ms), text, syllables)
    }

    /// 一个背景和声（只看文本的用例用它占位）。
    fn echo(text: &str) -> LineInfo {
        LineInfo::new_line(text.to_string(), Some(0), None)
    }

    /// 提供方为这首歌署名的歌手。
    fn artists(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| (*name).to_string()).collect()
    }

    #[test]
    fn a_joint_label_takes_its_side_from_the_performer_it_names_first() {
        // QQ 音乐用联合署名的标签分开《Problem》，其中一次点了 Big Sean，而提供方并没有把他
        // 列成歌手。
        let mut lines = vec![
            line(0, Some(500), "Iggy Azalea/Ariana Grande：", Vec::new()),
            line(1_000, Some(2_000), "Uh-huh it's Iggy", Vec::new()),
            line(2_000, Some(2_500), "Big Sean/Ariana Grande：", Vec::new()),
            line(3_000, Some(4_000), "One less problem", Vec::new()),
        ];

        apply_speaker_labels(&mut lines, &artists(&["Ariana Grande", "Iggy Azalea"]));

        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].text_from_any(), "Uh-huh it's Iggy");
        assert_eq!(lines[0].alignment(), LyricsAlignment::Right);
        assert_eq!(lines[1].text_from_any(), "One less problem");
        assert_eq!(lines[1].alignment(), LyricsAlignment::Left);
    }

    #[test]
    fn a_label_naming_nobody_on_the_record_is_not_a_label() {
        let mut lines = vec![line(
            1_000,
            Some(2_000),
            "Big Sean/Some Guy: line",
            vec![
                syllable(1_000, 1_500, "Big Sean/Some Guy: "),
                syllable(1_500, 2_000, "line"),
            ],
        )];

        apply_speaker_labels(&mut lines, &artists(&["Ariana Grande", "Iggy Azalea"]));

        assert_eq!(lines[0].text_from_any(), "Big Sean/Some Guy: line");
        assert_eq!(
            lines[0].alignment(),
            LyricsAlignment::Unspecified,
            "一个歌手都没点到的标签不给这一行定分边"
        );
    }

    #[test]
    fn an_inline_label_leaves_the_word_timing_behind() {
        let mut lines = vec![line(
            1_000,
            Some(2_000),
            "Doja Cat: sing it",
            vec![
                syllable(1_000, 1_200, "Doja "),
                syllable(1_200, 1_500, "Cat: "),
                syllable(1_500, 2_000, "sing it"),
            ],
        )];

        apply_speaker_labels(&mut lines, &artists(&["Doja Cat", "SZA"]));

        assert_eq!(lines[0].text_from_any(), "sing it");
        assert_eq!(lines[0].alignment(), LyricsAlignment::Left);
        assert_eq!(texts(&lines[0]), vec!["sing it"]);
    }

    #[test]
    fn a_second_performer_switches_the_voice() {
        let mut lines = vec![
            line(0, Some(1_000), "The Weeknd：", Vec::new()),
            line(1_000, Some(2_000), "I can't feel my face", Vec::new()),
            line(2_000, Some(3_000), "Take my hand", Vec::new()),
        ];

        apply_speaker_labels(&mut lines, &artists(&["Ariana Grande", "The Weeknd"]));

        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].alignment(), LyricsAlignment::Right);
        assert_eq!(lines[1].alignment(), LyricsAlignment::Right);
    }

    #[test]
    fn a_sung_colon_without_a_label_keeps_the_line() {
        let mut lines = vec![line(0, Some(1_000), "Love: it hurts", Vec::new())];

        apply_speaker_labels(&mut lines, &artists(&["Ariana Grande"]));

        assert_eq!(lines[0].text_from_any(), "Love: it hurts");
        assert_eq!(lines[0].alignment(), LyricsAlignment::Unspecified);
    }

    #[test]
    fn a_joint_label_gives_its_part_back_to_the_main_voice() {
        // 《Save Your Tears (Remix)》把主歌分给两位表演者，把最后一段副歌交给两人，QQ 音乐
        // 给这一段写了它自己的标签。
        let mut lines = vec![
            line(0, Some(500), "The Weeknd：", Vec::new()),
            line(
                1_000,
                Some(2_000),
                "I saw you dancing in a crowded room",
                Vec::new(),
            ),
            line(2_000, Some(2_500), "Ariana Grande：", Vec::new()),
            line(
                3_000,
                Some(4_000),
                "Met you once under a Pisces moon",
                Vec::new(),
            ),
            line(4_000, Some(4_500), "Both：", Vec::new()),
            line(
                5_000,
                Some(6_000),
                "I don't know why I run away",
                Vec::new(),
            ),
        ];

        apply_speaker_labels(&mut lines, &artists(&["The Weeknd", "Ariana Grande"]));

        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].alignment(), LyricsAlignment::Left);
        assert_eq!(lines[1].alignment(), LyricsAlignment::Right);
        assert_eq!(
            (lines[2].text_from_any().as_str(), lines[2].alignment()),
            ("I don't know why I run away", LyricsAlignment::Left),
            "两人一起唱的一段交回第一个声部"
        );
    }

    #[test]
    fn a_joint_label_written_in_chinese_gives_its_part_back_to_the_main_voice() {
        // 中文转录把同一件事写成 `合：` 或 `合唱：`，独占一行或在两人唱的词前面。
        let mut lines = vec![
            line(0, Some(500), "合：", Vec::new()),
            line(1_000, Some(2_000), "合唱：我们一起走吧", Vec::new()),
        ];

        apply_speaker_labels(&mut lines, &artists(&["某人"]));

        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].text_from_any(), "我们一起走吧");
        assert_eq!(lines[0].alignment(), LyricsAlignment::Left);
    }

    #[test]
    fn a_sentence_broken_at_a_comma_is_joined_into_one_line() {
        // 《Saddle Up》把 `Don't be shy, come and drive me crazy` 写成两行，后一行定在
        // 前一行唱完的时刻。
        let mut lines = vec![
            word_timed_line(194_619, 195_413, "Don't be shy,"),
            word_timed_line(195_413, 196_999, "come and drive me crazy"),
        ];
        lines[0] = with_chinese(lines[0].clone(), "不要害羞");
        lines[1] = with_chinese(lines[1].clone(), "让我陷入疯狂");

        merge_continued_lines(&mut lines);

        assert_eq!(lines.len(), 1, "两行是一句话");
        assert_eq!(
            lines[0].text_from_any(),
            "Don't be shy, come and drive me crazy"
        );
        assert_eq!(
            lines[0].chinese_translation(),
            Some("不要害羞 让我陷入疯狂"),
            "每一行的译文跟着它自己那一行"
        );
        assert_eq!(
            texts(&lines[0]),
            vec![
                "Don't ", "be ", "shy, ", "come ", "and ", "drive ", "me ", "crazy"
            ],
            "这句话的词是两行各自画出来的词"
        );
        assert_eq!(lines[0].start_time(), Some(194_619));
        assert_eq!(lines[0].end_time(), Some(196_999));
    }

    #[test]
    fn the_english_pronoun_carries_the_rest_of_a_sentence() {
        let mut lines = vec![
            word_timed_line(108_000, 108_582, "A tragedy, Ms. RIP,"),
            word_timed_line(108_582, 109_800, "I came for a reason"),
        ];

        merge_continued_lines(&mut lines);

        assert_eq!(lines.len(), 1);
        assert_eq!(
            lines[0].text_from_any(),
            "A tragedy, Ms. RIP, I came for a reason"
        );
    }

    #[test]
    fn the_rest_of_a_sentence_is_joined_without_punctuation_at_the_break() {
        // 《Saddle Up》把 `Put your money` 与 `where your mouth is` 写成两行：断在行写不下的
        // 地方，而不是某个标点处。
        let mut lines = vec![
            word_timed_line(197_804, 198_300, "Put your money"),
            word_timed_line(198_300, 199_100, "where your mouth is"),
        ];

        merge_continued_lines(&mut lines);

        assert_eq!(lines.len(), 1);
        assert_eq!(
            lines[0].text_from_any(),
            "Put your money where your mouth is"
        );
        assert_eq!(lines[0].end_time(), Some(199_100));
    }

    #[test]
    fn a_hook_timed_apart_from_the_row_before_it_keeps_its_rows() {
        // 《WDA (Whole Different Animal)》把副歌写成每半秒一行、彼此隔着一个停顿，并且每一行
        // 都有自己的译文；只有一句话剩下的部分才从它所续的那一行唱完的地方开始。
        let mut lines = vec![
            word_timed_line(28_415, 29_417, "She a Whole Different Animal"),
            word_timed_line(29_953, 30_779, "different animal"),
        ];

        merge_continued_lines(&mut lines);

        assert_eq!(lines.len(), 2, "两行是副歌，不是一句话");
    }

    #[test]
    fn a_row_the_punctuation_left_open_is_joined_however_late_it_is_timed() {
        // 《LEMONADE》的行之间隔着一秒，而前一行结尾的逗号说的正是这句话还要继续，所以时间
        // 对它们没有话说。
        let mut lines = vec![
            word_timed_line(39_491, 39_991, "Like zip,"),
            word_timed_line(40_572, 40_984, "I don't care"),
        ];

        merge_continued_lines(&mut lines);

        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].text_from_any(), "Like zip, I don't care");
    }

    #[test]
    fn a_line_timed_payload_joins_its_rows_by_case_alone() {
        // 只有行级时间的一行没有结束时间，所以它关于后面那一行什么时候开始什么都没说，
        // 能读的只有字母的大小写。
        let mut lines = vec![
            line(1_000, None, "Put your money", Vec::new()),
            line(5_000, None, "where your mouth is", Vec::new()),
        ];

        merge_continued_lines(&mut lines);

        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn a_sentence_the_punctuation_closed_keeps_its_rows() {
        let mut lines = vec![
            word_timed_line(0, 1_000, "I'm not your enemy."),
            word_timed_line(1_000, 2_000, "i already know"),
        ];

        merge_continued_lines(&mut lines);

        assert_eq!(lines.len(), 2, "前一行说它那一句已经说完");
    }

    #[test]
    fn the_english_pronoun_begins_a_sentence_without_the_punctuation_to_carry_on() {
        // 什么都没说的一行把决定权交给后面那一行，而 `I` 跟任何大写的词一样什么都没说——
        // 16 Bit 把 `These days` 与 `I can't picture my face` 写成两行。
        let mut lines = vec![
            word_timed_line(0, 1_000, "These days"),
            word_timed_line(1_000, 2_000, "I can't picture my face"),
        ];

        merge_continued_lines(&mut lines);

        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn a_row_in_a_script_without_letter_case_says_nothing_about_the_sentence() {
        // 只有行级时间的一份歌词写 `[00:01.00]こんにちは世界` 再写 `[00:05.00]bye`：小写那一行
        // 前面的一行结尾是没有大小写的文字，所以它关于句子是否继续什么都没说，后面那一行自成
        // 一行。
        let mut lines = vec![
            line(1_000, Some(3_000), "こんにちは世界", Vec::new()),
            line(3_000, Some(5_000), "bye", Vec::new()),
        ];

        merge_continued_lines(&mut lines);

        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn a_row_that_begins_a_sentence_keeps_its_own_line() {
        let mut lines = vec![
            word_timed_line(158_851, 159_400, "Boy, Saddle Up,"),
            word_timed_line(159_400, 160_800, "Don't waste my time"),
        ];

        merge_continued_lines(&mut lines);

        assert_eq!(lines.len(), 2, "大写的词自己起一句");
    }

    #[test]
    fn a_script_without_letter_case_keeps_its_rows() {
        let mut lines = vec![
            word_timed_line(148_870, 149_500, "这还远远不够，"),
            word_timed_line(149_500, 150_600, "我直言不讳"),
        ];

        merge_continued_lines(&mut lines);

        assert_eq!(lines.len(), 2, "逗号在那里断开行，而不是把它留着");
    }

    #[test]
    fn a_duet_answer_keeps_its_own_row() {
        let mut lines = vec![
            word_timed_line(0, 1_000, "hold on,"),
            word_timed_line(1_000, 2_000, "i got you"),
        ];
        set_alignment(&mut lines[1], LyricsAlignment::Right);

        merge_continued_lines(&mut lines);

        assert_eq!(lines.len(), 2, "另一位唱的是他自己的一行");
    }

    #[test]
    fn a_sentence_broken_twice_is_joined_into_one_line() {
        let mut lines = vec![
            word_timed_line(0, 1_000, "Take the reins,"),
            word_timed_line(1_000, 2_000, "buckle up,"),
            word_timed_line(2_000, 3_000, "my baby"),
        ];

        merge_continued_lines(&mut lines);

        assert_eq!(lines.len(), 1);
        assert_eq!(
            lines[0].text_from_any(),
            "Take the reins, buckle up, my baby"
        );
        assert_eq!(lines[0].end_time(), Some(3_000));
    }

    #[test]
    fn the_rest_of_a_sentence_keeps_the_echo_it_answers_with() {
        let mut lines = vec![
            word_timed_line(194_619, 195_413, "Don't be shy,"),
            word_timed_line(195_413, 196_999, "come and drive me crazy"),
        ];
        lines[1].set_sub_line(Some(Box::new(echo("If you walk it like you talk it"))));

        merge_continued_lines(&mut lines);

        assert_eq!(lines.len(), 1);
        assert_eq!(
            background_text(&lines[0]).as_deref(),
            Some("If you walk it like you talk it"),
            "回声回答的是它写在下面的那句话"
        );
    }

    #[test]
    fn rows_timed_differently_keep_their_own_lines() {
        // 只有行级时间的一行没有自己的词，接起来的一行就没法用两行各自的词画出来。
        let mut lines = vec![
            word_timed_line(0, 1_000, "hold on,"),
            line(1_000, Some(2_000), "i got you", Vec::new()),
        ];

        merge_continued_lines(&mut lines);

        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn two_echoes_are_not_joined_into_one_line() {
        let mut lines = vec![
            word_timed_line(0, 1_000, "hold on,"),
            word_timed_line(1_000, 2_000, "i got you"),
        ];
        lines[0].set_sub_line(Some(Box::new(echo("yeah"))));
        lines[1].set_sub_line(Some(Box::new(echo("oh"))));

        merge_continued_lines(&mut lines);

        assert_eq!(lines.len(), 2, "一行只带一个背景和声");
    }
}
