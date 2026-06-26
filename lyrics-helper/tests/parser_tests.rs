use lyrics_helper::*;
use std::fs;

#[test]
fn test_parse_lrc() {
    let content = fs::read_to_string("tests/test_data/LrcDemo.txt").expect("Failed to read LRC file");
    let result = parse(&content, LyricsRawTypes::Lrc);
    assert!(result.is_some(), "LRC parsing should succeed");

    let data = result.unwrap();
    assert!(data.lines.is_some(), "LRC should have lines");
    let lines = data.lines.unwrap();
    assert!(!lines.is_empty(), "LRC should have at least one line");

    // Check first line has a timestamp
    let first = &lines[0];
    assert!(first.start_time().is_some(), "First LRC line should have start time");

    // Check metadata
    if let Some(ref meta) = data.track_metadata {
        println!("Title: {:?}", meta.title);
        println!("Artist: {:?}", meta.artist);
    }
}

#[test]
fn test_parse_qrc() {
    let content = fs::read_to_string("tests/test_data/QrcDemo.txt").expect("Failed to read QRC file");
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
    let content = fs::read_to_string("tests/test_data/YrcDemo.txt").expect("Failed to read YRC file");
    let result = parse(&content, LyricsRawTypes::Yrc);
    assert!(result.is_some(), "YRC parsing should succeed");

    let data = result.unwrap();
    assert!(data.lines.is_some(), "YRC should have lines");
    let lines = data.lines.unwrap();
    assert!(!lines.is_empty(), "YRC should have at least one line");
}

#[test]
fn test_parse_krc() {
    let content = fs::read_to_string("tests/test_data/KrcDemo.txt").expect("Failed to read KRC file");
    let result = parse(&content, LyricsRawTypes::Krc);
    assert!(result.is_some(), "KRC parsing should succeed");

    let data = result.unwrap();
    assert!(data.lines.is_some(), "KRC should have lines");
    let lines = data.lines.unwrap();
    assert!(!lines.is_empty(), "KRC should have at least one line");
}

#[test]
fn test_parse_spotify() {
    let content = fs::read_to_string("tests/test_data/SpotifyDemo.txt").expect("Failed to read Spotify file");
    let result = parse(&content, LyricsRawTypes::Spotify);
    assert!(result.is_some(), "Spotify parsing should succeed");

    let data = result.unwrap();
    assert!(data.lines.is_some(), "Spotify should have lines");
    let lines = data.lines.unwrap();
    assert!(!lines.is_empty(), "Spotify should have at least one line");
}

#[test]
fn test_parse_spotify_syllable() {
    let content = fs::read_to_string("tests/test_data/SpotifySyllableDemo.txt").expect("Failed to read Spotify syllable file");
    let result = parse(&content, LyricsRawTypes::Spotify);
    assert!(result.is_some(), "Spotify syllable parsing should succeed");

    let data = result.unwrap();
    assert!(data.lines.is_some(), "Spotify syllable should have lines");
}

#[test]
fn test_parse_spotify_unsynced() {
    let content = fs::read_to_string("tests/test_data/SpotifyUnsyncedDemo.txt").expect("Failed to read Spotify unsynced file");
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
    let content = fs::read_to_string("tests/test_data/MusixmatchDemo.txt").expect("Failed to read Musixmatch file");
    let result = parse(&content, LyricsRawTypes::Musixmatch);
    assert!(result.is_some(), "Musixmatch parsing should succeed");

    let data = result.unwrap();
    assert!(data.lines.is_some(), "Musixmatch should have lines");
}

#[test]
fn test_parse_lyricify_syllable() {
    let content = fs::read_to_string("tests/test_data/LyricifySyllableDemo.txt").expect("Failed to read Lyricify Syllable file");
    let result = parse(&content, LyricsRawTypes::LyricifySyllable);
    assert!(result.is_some(), "Lyricify Syllable parsing should succeed");

    let data = result.unwrap();
    assert!(data.lines.is_some(), "Lyricify Syllable should have lines");
    let lines = data.lines.unwrap();
    assert!(!lines.is_empty(), "Lyricify Syllable should have at least one line");
}

#[test]
fn test_parse_lyricify_lines() {
    let content = fs::read_to_string("tests/test_data/LyricifyLinesDemo.txt").expect("Failed to read Lyricify Lines file");
    let result = parse(&content, LyricsRawTypes::LyricifyLines);
    assert!(result.is_some(), "Lyricify Lines parsing should succeed");

    let data = result.unwrap();
    assert!(data.lines.is_some(), "Lyricify Lines should have lines");
    let lines = data.lines.unwrap();
    assert!(!lines.is_empty(), "Lyricify Lines should have at least one line");

    if let Some(ref file) = data.file {
        assert_eq!(file.sync_types, SyncTypes::LineSynced);
    }
}

#[test]
fn test_parse_auto_detect_lrc() {
    let content = fs::read_to_string("tests/test_data/LrcDemo.txt").expect("Failed to read LRC file");
    let result = parse_auto(&content);
    assert!(result.is_some(), "Auto-detect LRC should succeed");
}

#[test]
fn test_generate_lrc() {
    let content = fs::read_to_string("tests/test_data/LrcDemo.txt").expect("Failed to read LRC file");
    let data = parse(&content, LyricsRawTypes::Lrc).expect("Failed to parse LRC");

    let output = generate_string(&data, LyricsTypes::Lrc);
    assert!(output.is_some(), "LRC generation should succeed");

    let output = output.unwrap();
    assert!(!output.is_empty(), "LRC output should not be empty");
    assert!(output.contains("["), "LRC output should contain timestamps");
}

#[test]
fn test_generate_qrc() {
    let content = fs::read_to_string("tests/test_data/QrcDemo.txt").expect("Failed to read QRC file");
    let data = parse(&content, LyricsRawTypes::Qrc).expect("Failed to parse QRC");

    let output = generate_string(&data, LyricsTypes::Qrc);
    assert!(output.is_some(), "QRC generation should succeed");

    let output = output.unwrap();
    assert!(!output.is_empty(), "QRC output should not be empty");
}

#[test]
fn test_generate_yrc() {
    let content = fs::read_to_string("tests/test_data/YrcDemo.txt").expect("Failed to read YRC file");
    let data = parse(&content, LyricsRawTypes::Yrc).expect("Failed to parse YRC");

    let output = generate_string(&data, LyricsTypes::Yrc);
    assert!(output.is_some(), "YRC generation should succeed");

    let output = output.unwrap();
    assert!(!output.is_empty(), "YRC output should not be empty");
}

#[test]
fn test_roundtrip_lrc() {
    let content = fs::read_to_string("tests/test_data/LrcDemo.txt").expect("Failed to read LRC file");
    let data = parse(&content, LyricsRawTypes::Lrc).expect("Failed to parse LRC");

    let generated = generate_string(&data, LyricsTypes::Lrc).expect("Failed to generate LRC");
    let reparsed = parse(&generated, LyricsRawTypes::Lrc).expect("Failed to re-parse LRC");

    let original_lines = data.lines.as_ref().unwrap();
    let reparsed_lines = reparsed.lines.as_ref().unwrap();

    assert_eq!(original_lines.len(), reparsed_lines.len(), "Line count should match after round-trip");

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
    assert_eq!(
        helpers::chinese_helper::to_traditional("学习"),
        "學習"
    );
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
    assert_eq!(
        helpers::chinese_helper::to_simplified("了"),
        "了"
    );
    assert_eq!(
        helpers::chinese_helper::to_simplified("學習"),
        "学习"
    );
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
    let line = LineInfo::new_syllable(syllables);
    assert!(line.is_syllable());
    assert_eq!(line.start_time(), Some(0));
    assert_eq!(line.end_time(), Some(1000));
    assert_eq!(line.text_from_any(), "Hello");
}

#[test]
fn test_type_helper_detect() {
    let lrc = "[00:00.000]Hello World";
    assert_eq!(helpers::type_helper::get_lyrics_types(lrc), LyricsRawTypes::Lrc);
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
        assert_eq!(syllables[0].text.trim(), "Lately");
        assert_eq!(syllables[0].start_time, 358);
        assert_eq!(syllables[0].end_time, 1694);
        assert_eq!(syllables[1].text.trim(), "I've");
        assert_eq!(syllables[2].text.trim(), "been,");
    } else {
        panic!("Expected Syllable variant");
    }

    assert_eq!(data.file.as_ref().unwrap().sync_types, SyncTypes::SyllableSynced);
    assert_eq!(data.writers.as_ref().unwrap()[0], "Ryan Tedder");
}

#[test]
fn test_parse_ttml_duet_alignment() {
    let ttml = r#"<?xml version="1.0" encoding="UTF-8"?>
<tt xmlns="http://www.w3.org/ns/ttml" xmlns:ttm="http://www.w3.org/ns/ttml#metadata">
  <head>
    <metadata>
      <ttm:agent type="person" xml:id="v1"/>
      <ttm:agent type="group" xml:id="v2"/>
    </metadata>
  </head>
  <body>
    <div>
      <p begin="0" end="1000" ttm:agent="v1"><span begin="0" end="1000">Person line</span></p>
      <p begin="1000" end="2000" ttm:agent="v2"><span begin="1000" end="2000">Group line</span></p>
    </div>
  </body>
</tt>"#;

    let data = parsers::parsers::ttml_parser::parse(ttml);
    let lines = data.lines.as_ref().unwrap();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0].alignment(), LyricsAlignment::Left);
    assert_eq!(lines[1].alignment(), LyricsAlignment::Right);
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
    assert_eq!(data.file.as_ref().unwrap().sync_types, SyncTypes::LineSynced);
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
