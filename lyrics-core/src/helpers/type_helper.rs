use crate::models::LyricsRawTypes;

pub fn get_lyrics_types(input: &str) -> LyricsRawTypes {
    let trimmed = input.trim();

    if trimmed.starts_with('{') && trimmed.contains('"') {
        // Could be Spotify or Musixmatch JSON
        if trimmed.contains("colorLyrics") || trimmed.contains("lyrics") && trimmed.contains("syncType") {
            return LyricsRawTypes::Spotify;
        }
        if trimmed.contains("message") && trimmed.contains("macro_calls") {
            return LyricsRawTypes::Musixmatch;
        }
    }

    if trimmed.contains("<tt ") && trimmed.contains("xmlns") {
        return LyricsRawTypes::Ttml;
    }

    // Check for Lyricify formats
    if trimmed.contains("[type:LyricifySyllable]") || has_lyricify_syllable_pattern(trimmed) {
        return LyricsRawTypes::LyricifySyllable;
    }
    if trimmed.contains("[type:LyricifyLines]") || is_lyricify_lines_pattern(trimmed) {
        return LyricsRawTypes::LyricifyLines;
    }

    // Check for YRC format: [start,duration](wordStart,duration,0)text
    if trimmed.contains("],(") && trimmed.contains(",0)") {
        return LyricsRawTypes::Yrc;
    }

    // Check for KRC format: [start,duration]<offset,duration,0>text
    if trimmed.contains(",0>") {
        return LyricsRawTypes::Krc;
    }

    // Check for QRC format: text(start,duration)
    if trimmed.contains('(') && trimmed.contains(',') && trimmed.contains(')') {
        let has_syllable_pattern = regex::Regex::new(r"\(\d+,\d+\)")
            .map(|re| re.is_match(trimmed))
            .unwrap_or(false);
        if has_syllable_pattern {
            return LyricsRawTypes::Qrc;
        }
    }

    // Check for LRC format: [mm:ss.xx]
    if trimmed.contains("[") && trimmed.contains("]") && trimmed.contains(":") {
        return LyricsRawTypes::Lrc;
    }

    LyricsRawTypes::Unknown
}

pub fn has_lyricify_syllable_pattern(input: &str) -> bool {
    // Lyricify Syllable: lines with syllable patterns like text(start,duration)
    // and optional alignment prefix like [0], [1], etc.
    let lines: Vec<&str> = input.lines().take(5).collect();
    for line in lines {
        if line.starts_with('[') && line.contains("](") && line.contains(",") {
            return true;
        }
    }
    false
}

pub fn is_lyricify_lines_pattern(input: &str) -> bool {
    use once_cell::sync::Lazy;
    use regex::Regex;

    static LINE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\[\d+,\d+\].*").unwrap());
    static SYLLABLE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\w+\(\d+,\d+\)").unwrap());

    let line_matches = LINE_RE.find_iter(input).count();
    if line_matches == 0 {
        return false;
    }
    let syllable_matches = SYLLABLE_RE.find_iter(input).count();
    syllable_matches <= line_matches
}

pub fn is_lrc(input: &str) -> bool {
    let trimmed = input.trim();
    // Check if it looks like LRC format
    trimmed.contains("[") && trimmed.contains("]") && trimmed.contains(":")
        && !trimmed.contains(",0>")
        && !trimmed.contains("],(")
}
