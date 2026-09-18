use lyrics_helper::*;
use std::fs;

#[test]
fn test_parse_lrc() {
    let content =
        fs::read_to_string("tests/test_data/LrcDemo.txt").expect("Failed to read LRC file");
    let result = parse(&content, LyricsRawTypes::Lrc);
    assert!(result.is_some(), "LRC parsing should succeed");

    let data = result.unwrap();
    assert!(data.lines.is_some(), "LRC should have lines");
    let lines = data.lines.unwrap();
    assert!(!lines.is_empty(), "LRC should have at least one line");

    // Check first line has a timestamp
    let first = &lines[0];
    assert!(
        first.start_time().is_some(),
        "First LRC line should have start time"
    );

    // Check metadata
    if let Some(ref meta) = data.track_metadata {
        println!("Title: {:?}", meta.title);
        println!("Artist: {:?}", meta.artist);
    }
}

#[test]
fn test_parse_qrc() {
    let content =
        fs::read_to_string("tests/test_data/QrcDemo.txt").expect("Failed to read QRC file");
    let result = parse(&content, LyricsRawTypes::Qrc);
    assert!(result.is_some(), "QRC parsing should succeed");

    let data = result.unwrap();
    assert!(data.lines.is_some(), "QRC should have lines");
    let lines = data.lines.unwrap();
    assert!(!lines.is_empty(), "QRC should have at least one line");

    // QRC should be syllable-synced
    if let Some(ref file) = data.file {
        assert_eq!(file.sync_types, SyncTypes::SyllableSynced);
    }
}

#[test]
fn test_parse_yrc() {
    let content =
        fs::read_to_string("tests/test_data/YrcDemo.txt").expect("Failed to read YRC file");
    let result = parse(&content, LyricsRawTypes::Yrc);
    assert!(result.is_some(), "YRC parsing should succeed");

    let data = result.unwrap();
    assert!(data.lines.is_some(), "YRC should have lines");
    let lines = data.lines.unwrap();
    assert!(!lines.is_empty(), "YRC should have at least one line");
}

#[test]
fn test_parse_yrc_credits_with_multibyte_text() {
    // 信息行含多字节字符时，行尾必须按字符索引定位（对应 C# `string.IndexOf`）；
    // 若按字节索引定位，扫描下标会与字符下标错位而无法前进。
    let content = "{\"t\":0,\"c\":[{\"tx\":\"Björk\"}]}\n\
                   {\"t\":1000,\"c\":[{\"tx\":\"作词: \"},{\"tx\":\"甲\"},{\"tx\":\"/\"},{\"tx\":\"乙\"}]}\n\
                   [0,1000](0,500,0)歌词(500,500,0)测试\n";
    let data = parse(content, LyricsRawTypes::Yrc).expect("YRC parsing should succeed");
    let lines = data.lines.as_ref().expect("YRC should have lines");

    assert_eq!(lines.len(), 3, "两条信息行 + 一条歌词行");
    assert_eq!(lines[0].text(), "Björk");
    assert_eq!(lines[0].start_time(), Some(0));
    assert_eq!(lines[1].text(), "作词: 甲/乙");
    assert_eq!(lines[1].start_time(), Some(1000));
    assert_eq!(
        data.writers.as_deref(),
        Some(["甲".to_string(), "乙".to_string()].as_slice()),
        "作词信息行应解析出作者"
    );
    assert_eq!(
        data.file.as_ref().map(|file| file.sync_types),
        Some(SyncTypes::MixedSynced),
        "含信息行时应标记为混合同步"
    );

    let syllables = lines[2].syllables().expect("第三条应为逐字歌词行");
    assert_eq!(syllables.len(), 2);
    assert_eq!(syllables[0].text(), "歌词");
    assert_eq!(
        (syllables[0].start_time(), syllables[0].end_time()),
        (0, 500)
    );
    assert_eq!(syllables[1].text(), "测试");
}

#[test]
fn test_parse_krc() {
    let content =
        fs::read_to_string("tests/test_data/KrcDemo.txt").expect("Failed to read KRC file");
    let result = parse(&content, LyricsRawTypes::Krc);
    assert!(result.is_some(), "KRC parsing should succeed");

    let data = result.unwrap();
    assert!(data.lines.is_some(), "KRC should have lines");
    let lines = data.lines.unwrap();
    assert!(!lines.is_empty(), "KRC should have at least one line");
}

#[test]
fn test_parse_spotify() {
    let content =
        fs::read_to_string("tests/test_data/SpotifyDemo.txt").expect("Failed to read Spotify file");
    let result = parse(&content, LyricsRawTypes::Spotify);
    assert!(result.is_some(), "Spotify parsing should succeed");

    let data = result.unwrap();
    assert!(data.lines.is_some(), "Spotify should have lines");
    let lines = data.lines.unwrap();
    assert!(!lines.is_empty(), "Spotify should have at least one line");
}

#[test]
fn test_parse_spotify_syllable() {
    let content = fs::read_to_string("tests/test_data/SpotifySyllableDemo.txt")
        .expect("Failed to read Spotify syllable file");
    let result = parse(&content, LyricsRawTypes::Spotify);
    assert!(result.is_some(), "Spotify syllable parsing should succeed");

    let data = result.unwrap();
    assert!(data.lines.is_some(), "Spotify syllable should have lines");
}

#[test]
fn test_parse_spotify_unsynced() {
    let content = fs::read_to_string("tests/test_data/SpotifyUnsyncedDemo.txt")
        .expect("Failed to read Spotify unsynced file");
    let result = parse(&content, LyricsRawTypes::Spotify);
    assert!(result.is_some(), "Spotify unsynced parsing should succeed");

    let data = result.unwrap();
    assert!(data.lines.is_some(), "Spotify unsynced should have lines");

    if let Some(ref file) = data.file {
        assert_eq!(file.sync_types, SyncTypes::Unsynced);
    }
}

#[test]
fn test_parse_musixmatch() {
    let content = fs::read_to_string("tests/test_data/MusixmatchDemo.txt")
        .expect("Failed to read Musixmatch file");
    let result = parse(&content, LyricsRawTypes::Musixmatch);
    assert!(result.is_some(), "Musixmatch parsing should succeed");

    let data = result.unwrap();
    assert!(data.lines.is_some(), "Musixmatch should have lines");
}

#[test]
fn test_parse_lyricify_syllable() {
    let content = fs::read_to_string("tests/test_data/LyricifySyllableDemo.txt")
        .expect("Failed to read Lyricify Syllable file");
    let result = parse(&content, LyricsRawTypes::LyricifySyllable);
    assert!(result.is_some(), "Lyricify Syllable parsing should succeed");

    let data = result.unwrap();
    assert!(data.lines.is_some(), "Lyricify Syllable should have lines");
    let lines = data.lines.unwrap();
    assert!(
        !lines.is_empty(),
        "Lyricify Syllable should have at least one line"
    );
}

#[test]
fn test_parse_lyricify_lines() {
    let content = fs::read_to_string("tests/test_data/LyricifyLinesDemo.txt")
        .expect("Failed to read Lyricify Lines file");
    let result = parse(&content, LyricsRawTypes::LyricifyLines);
    assert!(result.is_some(), "Lyricify Lines parsing should succeed");

    let data = result.unwrap();
    assert!(data.lines.is_some(), "Lyricify Lines should have lines");
    let lines = data.lines.unwrap();
    assert!(
        !lines.is_empty(),
        "Lyricify Lines should have at least one line"
    );

    if let Some(ref file) = data.file {
        assert_eq!(file.sync_types, SyncTypes::LineSynced);
    }
}

#[test]
fn test_parse_auto_detect_lrc() {
    let content =
        fs::read_to_string("tests/test_data/LrcDemo.txt").expect("Failed to read LRC file");
    let result = parse_auto(&content);
    assert!(result.is_some(), "Auto-detect LRC should succeed");
}

#[test]
fn test_generate_lrc() {
    let content =
        fs::read_to_string("tests/test_data/LrcDemo.txt").expect("Failed to read LRC file");
    let data = parse(&content, LyricsRawTypes::Lrc).expect("Failed to parse LRC");

    let output = generate_string(&data, LyricsTypes::Lrc);
    assert!(output.is_some(), "LRC generation should succeed");

    let output = output.unwrap();
    assert!(!output.is_empty(), "LRC output should not be empty");
    assert!(output.contains("["), "LRC output should contain timestamps");
}

#[test]
fn test_generate_qrc() {
    let content =
        fs::read_to_string("tests/test_data/QrcDemo.txt").expect("Failed to read QRC file");
    let data = parse(&content, LyricsRawTypes::Qrc).expect("Failed to parse QRC");

    let output = generate_string(&data, LyricsTypes::Qrc);
    assert!(output.is_some(), "QRC generation should succeed");

    let output = output.unwrap();
    assert!(!output.is_empty(), "QRC output should not be empty");
}

#[test]
fn test_generate_yrc() {
    let content =
        fs::read_to_string("tests/test_data/YrcDemo.txt").expect("Failed to read YRC file");
    let data = parse(&content, LyricsRawTypes::Yrc).expect("Failed to parse YRC");

    let output = generate_string(&data, LyricsTypes::Yrc);
    assert!(output.is_some(), "YRC generation should succeed");

    let output = output.unwrap();
    assert!(!output.is_empty(), "YRC output should not be empty");
}

#[test]
fn test_roundtrip_lrc() {
    let content =
        fs::read_to_string("tests/test_data/LrcDemo.txt").expect("Failed to read LRC file");
    let data = parse(&content, LyricsRawTypes::Lrc).expect("Failed to parse LRC");

    let generated = generate_string(&data, LyricsTypes::Lrc).expect("Failed to generate LRC");
    let reparsed = parse(&generated, LyricsRawTypes::Lrc).expect("Failed to re-parse LRC");

    let original_lines = data.lines.as_ref().unwrap();
    let reparsed_lines = reparsed.lines.as_ref().unwrap();

    assert_eq!(
        original_lines.len(),
        reparsed_lines.len(),
        "Line count should match after round-trip"
    );

    // Check that timestamps are preserved
    for i in 0..original_lines.len().min(reparsed_lines.len()) {
        let orig_time = original_lines[i].start_time();
        let reparsed_time = reparsed_lines[i].start_time();
        assert_eq!(orig_time, reparsed_time, "Timestamp mismatch at line {}", i);
    }
}

#[test]
fn test_string_helper_format_time() {
    let ts = helpers::string_helper::format_time_ms_to_timestamp_string(125500.0);
    assert_eq!(ts, "02:05.500");
}

#[test]
fn test_math_helper_min_max() {
    assert_eq!(helpers::math_helper::min_opt(Some(5), Some(3)), Some(3));
    assert_eq!(helpers::math_helper::min_opt(Some(5), None), Some(5));
    assert_eq!(helpers::math_helper::min_opt(None, None), None);

    assert_eq!(helpers::math_helper::max_opt(Some(5), Some(3)), Some(5));
    assert_eq!(helpers::math_helper::max_opt(Some(5), None), Some(5));
    assert_eq!(helpers::math_helper::max_opt(None, None), None);
}

#[test]
fn test_chinese_helper_to_traditional() {
    assert_eq!(
        helpers::chinese_helper::to_traditional("简体中文"),
        "簡體中文"
    );
    assert_eq!(
        helpers::chinese_helper::to_traditional("开放中文转换"),
        "開放中文轉換"
    );
    assert_eq!(
        helpers::chinese_helper::to_traditional("了"),
        "了" // 了 is identical in both scripts
    );
    assert_eq!(helpers::chinese_helper::to_traditional("学习"), "學習");
    assert_eq!(helpers::chinese_helper::to_traditional(""), "");
    assert_eq!(helpers::chinese_helper::to_traditional("abc123"), "abc123");
}

#[test]
fn test_chinese_helper_to_simplified() {
    assert_eq!(
        helpers::chinese_helper::to_simplified("繁體中文"),
        "繁体中文"
    );
    assert_eq!(
        helpers::chinese_helper::to_simplified("開放中文轉換"),
        "开放中文转换"
    );
    assert_eq!(helpers::chinese_helper::to_simplified("了"), "了");
    assert_eq!(helpers::chinese_helper::to_simplified("學習"), "学习");
    assert_eq!(helpers::chinese_helper::to_simplified(""), "");
    assert_eq!(helpers::chinese_helper::to_simplified("abc123"), "abc123");
}

#[test]
fn test_chinese_helper_roundtrip() {
    let original = "简体中文转换";
    let traditional = helpers::chinese_helper::to_traditional(original);
    let back = helpers::chinese_helper::to_simplified(&traditional);
    // s2t → t2s roundtrip should be largely idempotent
    assert!(!traditional.is_empty());
    assert_eq!(back, original);
}

#[test]
fn test_line_info_properties() {
    let line = LineInfo::new_line_with_time("Hello".to_string(), 1000);
    assert_eq!(line.start_time(), Some(1000));
    assert_eq!(line.text_from_any(), "Hello");
    assert!(!line.is_syllable());
    assert!(!line.is_full());
}

#[test]
fn test_line_info_syllable() {
    let syllables = vec![
        SyllableInfo::new("Hel".to_string(), 0, 500),
        SyllableInfo::new("lo".to_string(), 500, 1000),
    ];
    let line = LineInfo::new_syllable(to_syllable_items(syllables));
    assert!(line.is_syllable());
    assert_eq!(line.start_time(), Some(0));
    assert_eq!(line.end_time(), Some(1000));
    assert_eq!(line.text_from_any(), "Hello");
}

#[test]
fn test_type_helper_detect() {
    let lrc = "[00:00.000]Hello World";
    assert_eq!(
        helpers::type_helper::get_lyrics_types(lrc),
        LyricsRawTypes::Lrc
    );
}

#[test]
fn test_parse_ttml_syllable() {
    let ttml = r#"<?xml version="1.0" encoding="UTF-8"?>
<tt xmlns="http://www.w3.org/ns/ttml" xmlns:itunes="http://music.apple.com/lyric-ttml-internal" xmlns:ttm="http://www.w3.org/ns/ttml#metadata" itunes:timing="Word" xml:lang="en">
  <head>
    <metadata>
      <ttm:agent type="person" xml:id="v1"/>
      <iTunesMetadata leadingSilence="0.300">
        <songwriters><songwriter>Ryan Tedder</songwriter></songwriters>
      </iTunesMetadata>
    </metadata>
  </head>
  <body>
    <div>
      <p begin="0.358" end="4.933" itunes:key="L1" ttm:agent="v1">
        <span begin="0.358" end="1.694">Lately</span>
        <span begin="1.694" end="2.181">I've</span>
        <span begin="2.181" end="2.854">been,</span>
      </p>
    </div>
  </body>
</tt>"#;

    let data = parsers::parsers::ttml_parser::parse(ttml);
    let lines = data.lines.as_ref().unwrap();
    assert_eq!(lines.len(), 1);

    let line = &lines[0];
    assert!(line.is_syllable());
    assert_eq!(line.alignment(), LyricsAlignment::Left);

    if let LineInfo::Syllable { syllables, .. } = line {
        assert_eq!(syllables.len(), 3);
        assert_eq!(syllables[0].text().trim(), "Lately");
        assert_eq!(syllables[0].start_time(), 358);
        assert_eq!(syllables[0].end_time(), 1694);
        assert_eq!(syllables[1].text().trim(), "I've");
        assert_eq!(syllables[2].text().trim(), "been,");
    } else {
        panic!("Expected Syllable variant");
    }

    assert_eq!(
        data.file.as_ref().unwrap().sync_types,
        SyncTypes::SyllableSynced
    );
    assert_eq!(data.writers.as_ref().unwrap()[0], "Ryan Tedder");
}

#[test]
fn test_parse_ttml_duet_alignment() {
    let ttml = r#"<?xml version="1.0" encoding="UTF-8"?>
<tt xmlns="http://www.w3.org/ns/ttml" xmlns:ttm="http://www.w3.org/ns/ttml#metadata">
  <head>
    <metadata>
      <ttm:agent type="person" xml:id="v1"/>
      <ttm:agent type="person" xml:id="v2"/>
      <ttm:agent type="group" xml:id="v3"/>
      <ttm:agent type="other" xml:id="v4"/>
    </metadata>
  </head>
  <body>
    <div>
      <p begin="0" end="1000" ttm:agent="v1"><span begin="0" end="1000">First person</span></p>
      <p begin="1000" end="2000" ttm:agent="v2"><span begin="1000" end="2000">Second person</span></p>
      <p begin="2000" end="3000" ttm:agent="v3"><span begin="2000" end="3000">Group line</span></p>
      <p begin="3000" end="4000" ttm:agent="v4"><span begin="3000" end="4000">Other line</span></p>
      <p begin="4000" end="5000"><span begin="4000" end="5000">No agent line</span></p>
      <p begin="5000" end="6000" ttm:agent="v2"><span begin="5000" end="6000">Second person again</span></p>
    </div>
  </body>
</tt>"#;

    // Apple 语义：person agent 交替左右侧（首个演唱者取左侧），group 固定左侧，
    // other 固定右侧，缺失 agent 的行继承当前演唱者一侧。
    let data = parsers::parsers::ttml_parser::parse(ttml);
    let lines = data.lines.as_ref().unwrap();
    assert_eq!(lines.len(), 6);
    assert_eq!(lines[0].alignment(), LyricsAlignment::Left);
    assert_eq!(lines[1].alignment(), LyricsAlignment::Right);
    assert_eq!(lines[2].alignment(), LyricsAlignment::Left);
    assert_eq!(lines[3].alignment(), LyricsAlignment::Right);
    assert_eq!(lines[4].alignment(), LyricsAlignment::Right);
    assert_eq!(lines[5].alignment(), LyricsAlignment::Right);

    // agent 状态按 begin 时间顺序推进，而不是文档顺序；
    // 行开始时间缺失时取子 span 的最早 begin。
    let out_of_order = r#"<?xml version="1.0" encoding="UTF-8"?>
<tt xmlns="http://www.w3.org/ns/ttml" xmlns:ttm="http://www.w3.org/ns/ttml#metadata">
  <head>
    <metadata>
      <ttm:agent type="person" xml:id="v1"/>
      <ttm:agent type="person" xml:id="v2"/>
    </metadata>
  </head>
  <body>
    <div>
      <p ttm:agent="v2"><span begin="2000" end="3000">Second</span></p>
      <p ttm:agent="v1"><span begin="0" end="1000">First</span></p>
    </div>
  </body>
</tt>"#;

    let data = parsers::parsers::ttml_parser::parse(out_of_order);
    let lines = data.lines.as_ref().unwrap();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0].alignment(), LyricsAlignment::Right);
    assert_eq!(lines[1].alignment(), LyricsAlignment::Left);
}

#[test]
fn test_parse_ttml_line_synced() {
    let ttml = r#"<?xml version="1.0" encoding="UTF-8"?>
<tt xmlns="http://www.w3.org/ns/ttml">
  <body>
    <div>
      <p begin="0.5" end="3.0">Hello World</p>
      <p begin="3.5" end="6.0">Second line</p>
    </div>
  </body>
</tt>"#;

    let data = parsers::parsers::ttml_parser::parse(ttml);
    let lines = data.lines.as_ref().unwrap();
    assert_eq!(lines.len(), 2);
    assert_eq!(
        data.file.as_ref().unwrap().sync_types,
        SyncTypes::LineSynced
    );
    assert_eq!(lines[0].start_time(), Some(500));
    assert_eq!(lines[0].end_time(), Some(3000));
}

#[test]
fn test_parse_ttml_background_vocals() {
    let ttml = r#"<?xml version="1.0" encoding="UTF-8"?>
<tt xmlns="http://www.w3.org/ns/ttml" xmlns:ttm="http://www.w3.org/ns/ttml#metadata">
  <body>
    <div>
      <p begin="0" end="5000">
        <span begin="0" end="2000">Main</span>
        <span begin="2000" end="5000" ttm:role="x-bg">(<span begin="2000" end="3500">bg</span> <span begin="3500" end="5000">vocals</span>)</span>
      </p>
    </div>
  </body>
</tt>"#;

    let data = parsers::parsers::ttml_parser::parse(ttml);
    let lines = data.lines.as_ref().unwrap();
    assert_eq!(lines.len(), 1);

    let line = &lines[0];
    assert!(line.is_syllable());
    let sub = line.sub_line();
    assert!(sub.is_some(), "should have background sub_line");
    let sub = sub.unwrap();
    assert!(sub.is_syllable());
}

#[test]
fn test_parse_ttml_empty() {
    let data = parsers::parsers::ttml_parser::parse("");
    assert!(data.lines.as_ref().unwrap().is_empty());
    let data = parsers::parsers::ttml_parser::parse("   ");
    assert!(data.lines.as_ref().unwrap().is_empty());
}

#[test]
fn test_detect_fixture_raw_types() {
    // 上游 LyricsTypeDetector 的识别结果（逐条对应 fixtures）
    let cases = [
        ("LrcDemo.txt", LyricsRawTypes::Lrc),
        ("QrcDemo.txt", LyricsRawTypes::Qrc),
        ("KrcDemo.txt", LyricsRawTypes::Krc),
        ("YrcDemo.txt", LyricsRawTypes::Yrc),
        ("LyricifySyllableDemo.txt", LyricsRawTypes::LyricifySyllable),
        ("LyricifyLinesDemo.txt", LyricsRawTypes::LyricifyLines),
        ("SpotifyDemo.txt", LyricsRawTypes::Spotify),
        ("SpotifySyllableDemo.txt", LyricsRawTypes::Spotify),
        ("SpotifyUnsyncedDemo.txt", LyricsRawTypes::Spotify),
        ("MusixmatchDemo.txt", LyricsRawTypes::Musixmatch),
        ("AppleSyllableDemo.txt", LyricsRawTypes::AppleJson),
    ];

    for (file, expected) in cases {
        let content = fs::read_to_string(format!("tests/test_data/{file}"))
            .unwrap_or_else(|e| panic!("读取 {file} 失败: {e}"));
        assert_eq!(
            helpers::type_helper::get_lyrics_types(&content),
            expected,
            "fixture {file} 的识别结果不符合预期"
        );
    }

    // 逐字格式不能被误判为 Lyricify Lines（上游用 AnySyllableTiming 做了负向保护）
    let qrc = fs::read_to_string("tests/test_data/QrcDemo.txt").unwrap();
    assert!(!helpers::type_helper::is_lyricify_lines(&qrc));
}

#[test]
fn test_apple_json_requires_unwrapping() {
    // 上游对 Apple Music API JSON、QRC XML 与网易云完整 YRC JSON 只做识别，
    // ParseHelper 不解析这类外层封装，需要调用方先取出内嵌歌词。
    let content = fs::read_to_string("tests/test_data/AppleSyllableDemo.txt").unwrap();
    assert_eq!(
        helpers::type_helper::get_lyrics_types(&content),
        LyricsRawTypes::AppleJson
    );
    assert!(
        parse_auto(&content).is_none(),
        "Apple JSON 封装不应被直接解析"
    );
}

#[test]
fn test_musixmatch_standardize_keeps_text_and_drops_whitespace_fragments() {
    let content = fs::read_to_string("tests/test_data/MusixmatchDemo.txt").unwrap();
    let mut data = parse(&content, LyricsRawTypes::Musixmatch).unwrap();
    let before: Vec<String> = data
        .lines
        .as_ref()
        .unwrap()
        .iter()
        .map(|line| line.text_from_any())
        .collect();

    let lines = data.lines.as_mut().unwrap();
    helpers::optimization::musixmatch::standardize_musixmatch_lyrics(lines);

    let after: Vec<String> = lines.iter().map(|line| line.text_from_any()).collect();
    assert_eq!(before, after, "标准化不应改变歌词文本");

    let syllable_lines: Vec<&LineInfo> = lines.iter().filter(|l| l.is_syllable()).collect();
    assert!(!syllable_lines.is_empty(), "fixture 应包含逐字歌词行");

    for line in syllable_lines {
        let syllables = line.syllables().unwrap();
        assert!(
            syllables
                .iter()
                .skip(1)
                .all(|s| !s.text().chars().all(char::is_whitespace)),
            "下标 ≥1 的纯空白片段应被附加到前一个音节: {:?}",
            syllables.iter().map(|s| s.text()).collect::<Vec<_>>()
        );
    }
}

#[test]
fn test_merged_syllables_keep_sub_timings_in_generation_and_offset() {
    // 合并后的单词（FullSyllableInfo）在生成时必须展开为子音节时间，
    // 且时间偏移要作用到子音节上（缓存需随之刷新）。
    let merged = SyllableItem::Full(FullSyllableInfo::new(vec![
        SyllableInfo::new("Hel".to_string(), 0, 500),
        SyllableInfo::new("lo".to_string(), 500, 1000),
    ]));
    let data = LyricsData {
        file: Some(FileInfo {
            lyrics_type: LyricsTypes::LyricifySyllable,
            sync_types: SyncTypes::SyllableSynced,
            additional_info: None,
        }),
        lines: Some(vec![LineInfo::new_syllable(vec![merged])]),
        writers: None,
        track_metadata: Some(TrackMetadata::new()),
    };

    let qrc = generate_string(&data, LyricsTypes::Qrc).unwrap();
    assert!(qrc.contains("Hel(0,500)"), "生成结果缺少首个子音节: {qrc}");
    assert!(
        qrc.contains("lo(500,500)"),
        "生成结果缺少第二个子音节: {qrc}"
    );

    let mut lines = data.lines.clone().unwrap();
    helpers::offset_helper::add_offset(&mut lines, 200);
    let full = lines[0].syllables().unwrap()[0].as_full().unwrap();
    assert_eq!(full.start_time(), -200, "偏移后聚合开始时间应重新计算");
    assert_eq!(full.end_time(), 800, "偏移后聚合结束时间应重新计算");
}

#[test]
fn test_ttml_chinese_conversion_preference() {
    // 86119d8：默认优先使用内嵌的简体中文替换文本；
    // 关闭该选项且根语言为 zh-Hant 时，跳过 zh-Hans 替换并记录原始语言。
    let ttml = r#"<?xml version="1.0" encoding="UTF-8"?>
<tt xmlns="http://www.w3.org/ns/ttml" xmlns:itunes="http://music.apple.com/lyric-ttml-internal" xml:lang="zh-Hant">
  <body>
    <div>
      <p begin="0.0" end="2.0" itunes:key="L1">
        <span begin="0.0" end="2.0">原來如此</span>
      </p>
    </div>
  </body>
  <itunes:translations>
    <itunes:translation type="replacement" xml:lang="zh-Hans">
      <itunes:text for="L1">原来如此</itunes:text>
    </itunes:translation>
  </itunes:translations>
</tt>"#;

    let embedded = parsers::parsers::ttml_parser::parse_with_options(ttml, true);
    assert_eq!(
        embedded.lines.as_ref().unwrap()[0].text_from_any(),
        "原来如此"
    );
    assert_eq!(
        embedded
            .track_metadata
            .as_ref()
            .unwrap()
            .language
            .as_deref(),
        Some(&["zh-Hans".to_string()][..])
    );

    let native = parsers::parsers::ttml_parser::parse_with_options(ttml, false);
    assert_eq!(
        native.lines.as_ref().unwrap()[0].text_from_any().trim(),
        "原來如此"
    );
    assert_eq!(
        native.track_metadata.as_ref().unwrap().language.as_deref(),
        Some(&["zh-Hant".to_string()][..])
    );
}

#[test]
fn test_info_lines_heading_detection() {
    let content = fs::read_to_string("tests/test_data/LrcDemo.txt").unwrap();
    let data = parse(&content, LyricsRawTypes::Lrc).unwrap();
    let lines = data.lines.as_ref().unwrap();
    let metadata = data.track_metadata.as_ref();

    let flags = helpers::optimization::info_lines::check_info_lines(lines, metadata);
    assert_eq!(flags.len(), lines.len());

    // fixture 开头是「作词 / 作曲」等制作人员信息行
    assert!(flags[0], "首行制作人员信息应被识别为信息行");
    assert!(
        helpers::optimization::info_lines::heading_info_lines_count(lines, metadata) >= 2,
        "应至少识别出 2 行开头信息行"
    );
    assert!(!flags.last().unwrap(), "末尾歌词行不应被识别为信息行");
    assert!(helpers::optimization::info_lines::is_info_line(
        "作词 : Ryan Tedder",
        metadata
    ));
    assert!(!helpers::optimization::info_lines::is_info_line(
        "I've been reading books",
        metadata
    ));
}

#[test]
fn test_generate_krc_matches_upstream_format() {
    // KRC 的逐音节格式为 `[行开始,行时长]<相对开始,时长,0>文本`（上游 KrcGenerator），
    // 音节之间不应出现多余的 `,0>` 分隔符。
    let content = fs::read_to_string("tests/test_data/KrcDemo.txt").unwrap();
    let data = parse(&content, LyricsRawTypes::Krc).unwrap();
    let output = generate_string(&data, LyricsTypes::Krc).unwrap();

    assert!(
        !output.contains(",0><"),
        "生成结果不应包含多余的 ,0> 分隔符"
    );
    let line = output
        .lines()
        .find(|l| l.starts_with("[790,3661]"))
        .expect("应生成 Lately 行");
    assert_eq!(
        line,
        "[790,3661]<0,1072,0>Lately <1072,533,0>I've <1605,471,0>been <2076,343,0>I've <2419,327,0>been <2746,348,0>losing <3094,567,0>sleep"
    );

    let reparsed = parse(&output, LyricsRawTypes::Krc).unwrap();
    let before: Vec<String> = data
        .lines
        .as_ref()
        .unwrap()
        .iter()
        .filter(|l| l.is_syllable())
        .map(|l| l.text_from_any())
        .collect();
    let after: Vec<String> = reparsed
        .lines
        .as_ref()
        .unwrap()
        .iter()
        .filter(|l| l.is_syllable())
        .map(|l| l.text_from_any())
        .collect();
    assert_eq!(before, after, "KRC 往返后文本应保持一致");
}

#[test]
fn test_lrc_last_line_keeps_its_trailing_characters() {
    // 回归：结尾处的截断曾经按字节进行，会丢掉最后一行最后一个字符，
    // 遇到多字节字符还会 panic。
    for input in ["[00:00.00]Hello", "[00:00.00]Hello\n"] {
        let data = parse(input, LyricsRawTypes::Lrc).unwrap();
        let lines = data.lines.unwrap();
        assert_eq!(lines[0].text_from_any(), "Hello", "输入 {input:?}");
    }

    let data = parse("[00:00.00]你好\n", LyricsRawTypes::Lrc).unwrap();
    assert_eq!(data.lines.unwrap()[0].text_from_any(), "你好");

    let data = parse("[00:00.00]Hello\n[00:02.00]World\n", LyricsRawTypes::Lrc).unwrap();
    let texts: Vec<String> = data
        .lines
        .unwrap()
        .iter()
        .map(|line| line.text_from_any())
        .collect();
    assert_eq!(texts, vec!["Hello".to_string(), "World".to_string()]);
}
