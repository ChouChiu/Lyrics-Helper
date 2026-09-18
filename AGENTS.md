# AGENTS.md

## Project overview

Cargo workspace for parsing, generating, decrypting, and searching song lyrics. Rust rewrite of [WXRIW/Lyricify-Lyrics-Helper](https://github.com/WXRIW/Lyricify-Lyrics-Helper) (C#). Chinese-language project by ChouChiu. README is in Chinese.

## Workspace structure

```
Cargo.toml              # workspace root — 6 member crates
lyrics-core/            # models (LyricsData, LineInfo, SyllableItem/FullSyllableInfo, TrackMetadata, enums),
                        # traits (LyricsParser, LyricsGenerator, LyricsDecrypter),
                        # helpers (chinese, string, math, offset, type detection, optimization)
lyrics-parsers/         # one parser per format: LRC, QRC, KRC, YRC, TTML, Spotify, Musixmatch, Lyricify
lyrics-generators/      # one generator per format: LRC, QRC, KRC, YRC, Lyricify Syllable/Lines
lyrics-crypto/          # QRC and KRC decryption (AES/DES/ECB/CBC) + QRC XML sanitising
                        # (`decrypter/qrc/xml_utils.rs`) + Netease eapi `params` encryption
                        # (MD5 + AES-128-ECB/PKCS7), depends on flate2+base64+regex+quick-xml+aes+md-5
lyrics-search/          # song search by platform (QQ, Netease, Kugou, Soda, Apple, Musixmatch, LRCLIB, Spotify)
                        # gated behind `search` feature; depends on reqwest+tokio+lyrics-crypto
lyrics-helper/          # facade crate — re-exports all above; holds tests and examples
  tests/
    parser_tests.rs     # integration tests for all parsers + generators + decrypters
    test_data/          # 12 fixture files (LRC, QRC, KRC, YRC, TTML, Spotify, Musixmatch, Lyricify, Apple)
  examples/
    demo.rs               # CLI demo: parse, generate, detect, decrypt, parsers-demo, generators-demo
    search_test.rs        # search API demo (requires network)
    search_lyrics_test.rs # search+fetch lyrics demo (requires network)
```

## Key commands

All commands run from the **workspace root** (`/path/to/Lyricify-Lyrics-Helper`), not from a sub-crate:

```bash
cargo build                      # build all crates
cargo test                       # run all tests (workspace-wide)
cargo test -p lyrics-helper      # run tests in facade crate only
cargo test test_parse_lrc        # run a single test by name
cargo clippy                     # lint all crates
cargo fmt                        # format all crates
cargo doc --no-deps              # generate docs (alias: cargo doc-local)
```

Run examples from workspace root:
```bash
cargo run --example demo -- parsers-demo
cargo run --example demo -- parse lyrics-helper/tests/test_data/LrcDemo.txt lrc
cargo run --example demo -- generate lyrics-helper/tests/test_data/QrcDemo.txt qrc lrc
```

## Architecture

- **Facade pattern**: `lyrics-helper` re-exports everything. Users only need `use lyrics_helper::*`.
- **Crate dependency graph**: `lyrics-core` ← `lyrics-parsers`, `lyrics-generators`, `lyrics-crypto` ← `lyrics-search` ← `lyrics-helper`. Note: `lyrics-search` depends on both `lyrics-core` and `lyrics-crypto`.
- **Traits** in `lyrics-core/src/traits/`: `LyricsParser`, `LyricsGenerator`, `LyricsDecrypter`.
- **Dispatch**: `parsers::parse_lyrics(input, raw_type)` and `parsers::parse_lyrics_auto(input)` for auto-detection. `generators::generate_string(lyrics_data, type)` for output.
- **Type detection** (`lyrics-core/src/helpers/type_helper.rs`, port of upstream `LyricsTypeDetector`): `get_lyrics_types(input) -> LyricsRawTypes` recognises LRC/QRC/KRC/YRC/Lyricify Syllable/Lyricify Lines by line regex, plus the structured envelopes `AppleJson`, `Spotify`, `Musixmatch`, `YrcFull` (JSON) and `QrcFull` (QRC XML). Per-format predicates (`is_lrc`, `is_qrc_full`, …), `LyricsRawTypes::lyrics_type()/display_name()`, `try_parse_raw_type`, `get_raw_type_display_name`, `is_lyrics_type(_any)` complete the API.
- **Raw envelopes are not parsed**: upstream-faithful `parse_lyrics` returns `None` for `QrcFull`/`YrcFull`/`AppleJson`; callers must unwrap them first (the QQ provider extracts `Lyric_1@LyricContent` via `lyrics-crypto::decrypter::qrc::xml_utils`).
- **Syllable model**: `LineInfo::Syllable`/`FullSyllable` hold `Vec<SyllableItem>`, mirroring C# `ISyllableInfo`: `SyllableItem::Syllable(SyllableInfo)` or `SyllableItem::Full(FullSyllableInfo)` for merged words. Use `parts()`/`flatten_syllable_items()` in generators; `sub_items_mut()` requires a following `refresh_properties()`.
- **Optimizers** (`lyrics-core/src/helpers/optimization/`): full ports of `apple_music`, `info_lines`, `musixmatch`, `syllable_word_merger`, `sync_downgrade`, `yrc`, `explicit`.
- **Top-level re-exports** in `lyrics-helper/src/lib.rs`: `parse`, `parse_auto`, `generate_string`, `decrypt_qrc`, `decrypt_krc`.
- **Search feature gating**: `lyrics-search` deps (`reqwest`, `tokio`, `async-trait`, `rand`, `urlencoding`) are all `optional = true` behind the `search` feature, and both of its modules (`searchers`, `providers`) are `#[cfg(feature = "search")]` — without the feature the crate compiles empty. `lyrics-helper` gates the entire `lyrics-search` crate behind its own `search` feature.
- **Musixmatch provider**: `providers::web::musixmatch::api_options::ApiOptions` selects the Android (default) or desktop API; `api::set_options(...)` overrides it globally.

## Important details

- **Rust edition 2024** — requires Rust 1.85+. No `rust-toolchain.toml`.
- **Feature flags**: `search` (default on) in `lyrics-search` and `lyrics-helper` enables `reqwest` + `tokio`. Disable with `--no-default-features` for offline-only builds.
- **Test data paths**: tests use relative paths like `tests/test_data/*.txt`, which resolve from the `lyrics-helper/` crate directory (Cargo sets CWD to the crate root when running tests). `cargo test -p lyrics-helper` works correctly from the workspace root.
- **No CI, no rustfmt.toml, no clippy.toml** — use standard Rust defaults.
- **Dev dependency**: `pretty_assertions` for readable test diffs.
- **Crate names**: package names use hyphens (`lyrics-helper`), Rust crate names use underscores (`lyrics_helper`).
- **Doc alias**: `cargo doc-local` = `cargo doc --no-deps --open` (defined in `.cargo/config.toml`).
- **Examples path**: examples live at `lyrics-helper/examples/`, declared in `lyrics-helper/Cargo.toml` with `path = "examples/..."`.
- **Docs directory**: `docs/compose/` is gitignored — contains planning/design docs, not source of truth.
- **Key entrypoints**: `lyrics-helper/src/lib.rs` is the facade; `lyrics-parsers/src/parsers/mod.rs` has `parse_lyrics`/`parse_lyrics_auto`; `lyrics-generators/src/lib.rs` has `generate_string`; `lyrics-core/src/helpers/type_helper.rs` has `get_lyrics_types`; `lyrics-parsers/src/parsers/ttml_parser.rs` has `parse`/`parse_with_options` (the latter takes `use_embedded_simplified_chinese_lyrics`).
