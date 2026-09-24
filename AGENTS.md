# AGENTS.md

Cargo workspace for parsing, generating, decrypting and searching song lyrics; a Rust rewrite of [WXRIW/Lyricify-Lyrics-Helper](https://github.com/WXRIW/Lyricify-Lyrics-Helper) (C#) by ChouChiu; READMEs, doc comments and code comments are in Chinese

## Working rules

1. **Never guess an interface** Read the source, the rustdoc or the upstream C# before calling it; for a platform API read its `providers/web/<platform>/api.rs` and `response.rs`, not what the endpoint probably returns
2. **Clarify an ambiguous requirement first** Ask, get the answer, then write code
3. **Never invent the workflow** Behaviour of a format or optimizer is checked against upstream Lyricify-Lyrics-Helper; where upstream is silent or the behaviour is new, confirm with the maintainer instead of reconstructing it
4. **Reuse before you add** Look in `crates/lyrics-core/src/helpers/` (string, math, offset, chinese, type detection, optimization, word timing, conventions), `crates/lyrics-parsers/src/xml_utils.rs` and `providers/web/base_api.rs`, then in the crate you are changing
5. **Never break the architecture for convenience** The layering is enforced by the Cargo manifests: a crate only sees the crates it depends on; do not add a dependency edge to reach a helper, move the helper down into `lyrics-core` instead
6. **Say what you do not know** Name the gap (unknown upstream behaviour, untested platform response) instead of writing over it
7. **Map the blast radius before changing logic** Grep callers across all six crates; everything re-exported by `lyrics-helper` is public API; run `cargo test` for the whole workspace and build with `--no-default-features` when touching feature gates
8. **Test by risk, not by volume** Cover format edge cases, offsets, round-trips and error paths; skip trivial getters
9. **Verify before you fix** Reproduce with a fixture or a failing test first, do not hypothesise the bug
10. **No speculative defensive code** Handle the boundaries that actually occur in real lyrics files and API responses
11. **Comments are short and explain why** Write them in Chinese to match the codebase
12. **Do not reinvent the wheel** Prefer a maintained crate or what is already in the tree; new dependencies go through `[workspace.dependencies]`
13. **Write plainly** No padding; bold labels a term, it does not make a sentence truer
14. **Log deliberately** The library crates do not log and never print: failure goes through `Result` (`SearchError`, `DecryptError`), absence through `Option` or an empty `Vec`; console output belongs in examples only
15. **Never touch git history on your own** Commit, push, tag and branch wait for an instruction; when asked, messages are English `<type>: <summary>` (`feat` / `fix` / `refactor` / `style` / `docs` / `chore`) and commits stay GPG-signed, never `--no-gpg-sign`
16. **Reply in the user's language** Answer in whatever language the user writes in; this covers conversation only, code comments and commit messages keep the conventions above

## Workspace

```
Cargo.toml              workspace root, version 0.5.0, edition 2024, rust-version 1.85
crates/
  lyrics-core/          models (LyricsData, LineInfo, SyllableItem / FullSyllableInfo, TrackMetadata, enums)
                        traits (LyricsParser, LyricsGenerator, LyricsDecrypter)
                        helpers (chinese, string, math, offset, type detection, optimization, word timing, conventions)
  lyrics-parsers/       one parser per format: LRC, QRC, KRC, YRC, TTML, Spotify, Musixmatch, Lyricify Syllable / Lines
                        + xml_utils.rs (QRC XML envelope sanitising and lookup)
  lyrics-generators/    one generator per format: LRC, QRC, KRC, YRC, Lyricify Syllable / Lines
  lyrics-crypto/        QRC and KRC decryption (Triple DES / XOR + zlib) + Netease eapi `params` encryption (MD5 + AES-128-ECB)
  lyrics-search/        search by platform: QQ, Netease, Kugou, Soda, Apple, Musixmatch, LRCLIB, Spotify, AMLL TTML DB
                        error.rs = SearchError; providers/ = per-platform HTTP clients; searchers/ = Searcher trait + impls
  lyrics-helper/        facade: re-exports everything, holds integration tests and examples
    tests/parser_tests.rs   integration tests for parsers, generators and decrypters
    tests/test_data/        12 fixtures (LRC, QRC, KRC, YRC, Spotify, Musixmatch, Lyricify, Apple)
    examples/               demo.rs, search_test.rs, search_lyrics_test.rs (the last two need network)
```

Dependency graph: `lyrics-core` ← `lyrics-parsers`, `lyrics-generators`, `lyrics-crypto` ← `lyrics-search` ← `lyrics-helper`; `lyrics-search` depends on core, crypto and parsers (for `xml_utils`)

## Commands

Run from the workspace root

```bash
cargo build
cargo test                          # whole workspace, unit tests live in src/ as well
cargo test -p lyrics-helper         # integration tests only
cargo test test_parse_lrc           # single test
cargo build --no-default-features   # offline build without lyrics-search
cargo clippy
cargo fmt
cargo doc-local                     # cargo doc --no-deps --open, alias in .cargo/config.toml
cargo run --example demo -- parsers-demo
cargo run --example demo -- parse crates/lyrics-helper/tests/test_data/LrcDemo.txt lrc
cargo run --example demo -- generate crates/lyrics-helper/tests/test_data/QrcDemo.txt qrc lrc
```

No CI, no `rustfmt.toml`, no `clippy.toml`, no `rust-toolchain.toml`; standard defaults apply

## Architecture

- **Facade**: `crates/lyrics-helper/src/lib.rs` re-exports every crate; top level has `parse`, `parse_auto`, `parser_for`, `generate_string`, `generator_for`, `decrypt_qrc`, `decrypt_krc`, `QrcDecrypter`, `KrcDecrypter` and `SearchError` (only with `search`)
- **Dispatch**: each format is a zero-sized type implementing the core trait (`LrcParser` in `lyrics_parsers::parsers`, `LrcGenerator` in `lyrics_generators`), looked up by `parser_for(raw_type)` / `generator_for(type)`; `parse_lyrics`, `parse_lyrics_auto` and `generate_string` are thin wrappers; a new format = its module, one line in the `define_*!` macro and one match arm
- **Line-format parser pipeline** (`crates/lyrics-parsers/src/parsers/mod.rs`): `lyrics_data(type, sync, info)` builds the `LyricsData` shell, `parse_with_attributes(data, lines, parse_lines)` strips leading `[key:value]` lines and passes `offset` to the format's `parse_lyrics`, which applies it with `apply_offset`; QRC, KRC and Lyricify Syllable / Lines share it
- **Offsets**: all time shifting lives in `crates/lyrics-core/src/helpers/offset_helper.rs`; `add_offset` moves line times, syllable times and sub-lines together
- **Type detection** (`type_helper.rs`, port of upstream `LyricsTypeDetector`): `get_lyrics_types` recognises LRC / QRC / KRC / YRC / Lyricify Syllable / Lyricify Lines by line regex, plus the envelopes `AppleJson`, `Spotify`, `Musixmatch`, `YrcFull` (JSON) and `QrcFull` (QRC XML)
- **Raw envelopes are not parsed**: like upstream, `parse_lyrics` returns `None` for `QrcFull` / `YrcFull` / `AppleJson`; callers unwrap them first (the QQ provider extracts `Lyric_1@LyricContent` through `xml_utils`)
- **Syllable model**: `LineInfo::Syllable` / `FullSyllable` hold `Vec<SyllableItem>` (C# `ISyllableInfo`): `SyllableItem::Syllable` or `SyllableItem::Full` for merged words; generators use `parts()` / `flatten_syllable_items()`; `FullSyllableInfo` derives text and times from `sub_items` on every call, no cache, the model is `Sync`
- **Equality**: `SyllableItem` has no `PartialEq` on purpose, compare `start_time()` / `end_time()` explicitly; `LineInfo`'s `PartialEq` / `Ord` compare start time only, which is sort semantics (upstream `IComparable`)
- **Optimizers** (`helpers/optimization/`): full ports of `apple_music`, `info_lines`, `musixmatch`, `syllable_word_merger`, `sync_downgrade`, `yrc`, `explicit`; `utf16.rs` reproduces C# UTF-16 char semantics
- **Writing conventions** (`helpers/conventions.rs`): reads duet sides, background vocals and continued lines from the transcript text (`apply_speaker_labels`, `split_background_vocals`, `unwrap_brackets`, `fold_bracketed_echoes`, `merge_continued_lines`); never called by a parser, the caller runs them in the order given in the module docs
- **Cross-document word timing** (`helpers/word_timing.rs`): `apply_word_timings(&mut [LineInfo], &[LineInfo])` puts the word timings of another document (such as YRC) onto the text of a line-level transcript; lines pair by start time within 1000 ms (the nearest word line wins), each syllable line pairs once, lines without words or start time are skipped; each word takes the transcript characters up to its last word character, separators go to the previous word, so the syllables concatenate to exactly the original text; a line that does not align is left untouched, never partially fitted; merged `SyllableItem::Full` counts as one word, its sub-syllables split by the same rule and keep their times
- **Decrypt errors**: `DecryptError` (`Copy + Eq + Display + Error`) distinguishes `InvalidInput`, `DecompressionFailed` and `InvalidEncoding`; the free `decrypt_lyrics` functions are `.ok()` wrappers over the trait impls
- **Search errors**: `SearchError` (`Http` / `Json` / `Status` / `Api` / `Captcha` / `Payload` / `InvalidConfig`) is the only failure channel of the search layer; missing data is not an error, `Ok(None)` = the track has no such lyrics, an empty `Vec` = the request succeeded with no results; `base_api` has exactly `send` / `send_json` / `send_form` + `json` / `text`, and `json` / `text` turn non-2xx into `Status`, so a caller treating 404 as "no such song" checks `response.status()` first
- **Searchers**: `searchers/mod.rs` holds only the `Searchers` enum and re-exports; scoring (`compare_track`, `rank_by_match`, weights) is in `compare_helper.rs`, query building and refinement search in `refinement.rs`
- **Search feature gating**: every dependency of `lyrics-search` is optional behind its `search` feature and all three modules (`error`, `providers`, `searchers`) are `#[cfg(feature = "search")]`, so without it the crate compiles empty; `lyrics-helper` gates the whole crate behind its own `search` feature, on by default
- **AMLL TTML DB provider** (`providers::web::amll_ttml_db`): static files, no search API; `api::search` downloads `metadata/raw-lyrics-index.jsonl` (cached 1 h per base URL, deduplicated newest first by platform ID) and matches whitespace-separated terms against title, artists and album; result `id` is the `raw-lyrics/` file name for `get_raw_lyrics`; `get_lyrics(Platform, id, Format)` fetches per-platform files (404 → `Ok(None)`); `set_base_url` switches to the mirrors `BIKONOO_BASE_URL` and `GBCLSTUDIO_BASE_URL`
- **TTML inline annotations**: AMLL writes translations and romanisation as `<span ttm:role="x-translation" xml:lang>` / `x-roman` inside `<p>` (and inside `x-bg`); the parser keeps them out of syllables and line text and stores them in `translations` / `pronunciation` of the line or its background sub-line; `<head>` translations (Apple style) win
- **Musixmatch provider**: `api_options::ApiOptions` selects the Android (default) or desktop API; `api::set_options` returns `SearchError::InvalidConfig` for bad input instead of panicking

## Pitfalls

- **quick-xml 0.42**: names are `&str`; text events never contain entities, `&amp;` / `&#169;` arrive as separate `Event::GeneralRef`, so accumulate text across `Text` + `GeneralRef` (`xml_utils::reference_text`, TTML `build_tree`); never read attributes with `normalized_value` / `unescape_value`, they turn `\n` into spaces and collapse the multi-line QRC `LyricContent`, use `xml_utils::attribute_value`
- **reqwest 0.13**: features `json`, `form` (for `send_form`) and `rustls` (aws-lc-rs, needs a C toolchain and cmake)
- **Versions**: always full `x.y.z`, in the workspace `version`, every inter-crate requirement (`version = "0.5.0"`), READMEs and prose
- **Test data paths**: tests use `tests/test_data/*.txt`, relative to `crates/lyrics-helper/` because Cargo runs tests from the crate root
- **Naming**: packages use hyphens (`lyrics-helper`), crate paths use underscores (`lyrics_helper`)
- **Examples**: declared in `crates/lyrics-helper/Cargo.toml` with `path = "examples/..."`
- **`docs/compose/`**: gitignored planning notes, not a source of truth
- **Dev dependencies**: `lyrics-helper` has `tokio` for async tests; `pretty_assertions` sits in `[workspace.dependencies]` but no crate uses it
