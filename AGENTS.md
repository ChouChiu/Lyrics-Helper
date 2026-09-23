# AGENTS.md

## Project overview

Cargo workspace for parsing, generating, decrypting, and searching song lyrics. Rust rewrite of [WXRIW/Lyricify-Lyrics-Helper](https://github.com/WXRIW/Lyricify-Lyrics-Helper) (C#). Chinese-language project by ChouChiu. README is in Chinese.

## Workspace structure

```
Cargo.toml              # workspace root — 6 member crates
lyrics-core/            # models (LyricsData, LineInfo, SyllableItem/FullSyllableInfo, TrackMetadata, enums),
                        # traits (LyricsParser, LyricsGenerator, LyricsDecrypter),
                        # helpers (chinese, string, math, offset, type detection, optimization, word timing)
lyrics-parsers/         # one parser per format: LRC, QRC, KRC, YRC, TTML, Spotify, Musixmatch, Lyricify
                        # + `xml_utils.rs` (QRC XML envelope sanitising/lookup, moved out of lyrics-crypto in 0.5.0)
lyrics-generators/      # one generator per format: LRC, QRC, KRC, YRC, Lyricify Syllable/Lines
lyrics-crypto/          # QRC and KRC decryption (Triple DES / XOR + zlib) behind `LyricsDecrypter`
                        # (`QrcDecrypter`, `KrcDecrypter`) + Netease eapi `params` encryption
                        # (MD5 + AES-128-ECB/PKCS7), depends on flate2+base64+aes+md-5 only
lyrics-search/          # song search by platform (QQ, Netease, Kugou, Soda, Apple, Musixmatch, LRCLIB, Spotify)
                        # `error.rs` = crate-level `SearchError`; `providers/` = per-platform HTTP clients
                        # (base_api: send/send_json/send_form + json/text); `searchers/` = Searcher trait + impls
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
- **Crate dependency graph**: `lyrics-core` ← `lyrics-parsers`, `lyrics-generators`, `lyrics-crypto` ← `lyrics-search` ← `lyrics-helper`. `lyrics-search` depends on `lyrics-core`, `lyrics-crypto` and `lyrics-parsers` (for `xml_utils`), all optional behind its `search` feature
- **Traits** in `lyrics-core/src/traits/`: `LyricsParser`, `LyricsGenerator`, `LyricsDecrypter`.
- **Dispatch**: every format has a zero-sized type implementing the core trait (`LrcParser`… in `lyrics_parsers::parsers`, `LrcGenerator`… in `lyrics_generators`), looked up by `parsers::parser_for(raw_type)` / `generator_for(type)`; `parse_lyrics`, `parse_lyrics_auto` and `generate_string` are thin wrappers over those tables. Adding a format = add its module, one line in the `define_*!` macro and one match arm
- **Line-format parser pipeline** (`lyrics-parsers/src/parsers/mod.rs`): `lyrics_data(type, sync, info)` builds the `LyricsData` shell, `parse_with_attributes(data, lines, parse_lines)` strips leading `[key:value]` lines and hands the `offset` to the format's `parse_lyrics`, which applies it via `apply_offset`. QRC, KRC, Lyricify Syllable/Lines share it
- **Offsets**: all time-shift logic lives in `lyrics-core/src/helpers/offset_helper.rs` (`add_offset`, `add_offset_to_syllable_items`, `add_offset_to_syllables`); `add_offset` shifts stored line times, syllable times and sub-lines together
- **Type detection** (`lyrics-core/src/helpers/type_helper.rs`, port of upstream `LyricsTypeDetector`): `get_lyrics_types(input) -> LyricsRawTypes` recognises LRC/QRC/KRC/YRC/Lyricify Syllable/Lyricify Lines by line regex, plus the structured envelopes `AppleJson`, `Spotify`, `Musixmatch`, `YrcFull` (JSON) and `QrcFull` (QRC XML). Per-format predicates (`is_lrc`, `is_qrc_full`, …), `LyricsRawTypes::lyrics_type()/display_name()`, `try_parse_raw_type`, `get_raw_type_display_name`, `is_lyrics_type(_any)` complete the API.
- **Raw envelopes are not parsed**: upstream-faithful `parse_lyrics` returns `None` for `QrcFull`/`YrcFull`/`AppleJson`; callers must unwrap them first (the QQ provider extracts `Lyric_1@LyricContent` via `lyrics_parsers::xml_utils`)
- **Syllable model**: `LineInfo::Syllable`/`FullSyllable` hold `Vec<SyllableItem>`, mirroring C# `ISyllableInfo`: `SyllableItem::Syllable(SyllableInfo)` or `SyllableItem::Full(FullSyllableInfo)` for merged words. Use `parts()`/`flatten_syllable_items()` in generators. `FullSyllableInfo` derives its text and start/end times from `sub_items` on every call — no cache, no `refresh_properties()`, and the whole model is `Sync`.
- **Optimizers** (`lyrics-core/src/helpers/optimization/`): full ports of `apple_music`, `info_lines`, `musixmatch`, `syllable_word_merger`, `sync_downgrade`, `yrc`, `explicit`.
- **Cross-document word timing** (`lyrics-core/src/helpers/word_timing.rs`): `apply_word_timings(&mut [LineInfo], &[LineInfo])` 把另一份逐词计时文档（如 YRC）的时间读到已解析的行级转录的文字上。行按开始时间顺序配对（容差 1000 毫秒，没有词或没有开始时间的逐词行不参与，每个逐词行只配一次）；每个词吃掉转录里「到它最后一个词字符为止」的字符，分隔符归前一个词的末尾，因此该行音节拼起来精确等于原文；对不齐就整行放弃（保留原行时间、不带词时间），绝不部分贴合。配上词的行变为 `Syllable`/`FullSyllable`（行时间、对齐、子行、翻译/拼音保留）；合并音节 `SyllableItem::Full` 按一个词处理，子音节按同一规则分字符、各自时间保留。
- **Top-level re-exports** in `lyrics-helper/src/lib.rs`: `parse`, `parse_auto`, `parser_for`, `generate_string`, `generator_for`, `decrypt_qrc`, `decrypt_krc`, `QrcDecrypter`, `KrcDecrypter`, `SearchError`（仅 `search` feature）
- **Decrypt errors**: `DecryptError` (`Copy + Eq + Display + Error`) distinguishes `InvalidInput` (bad hex/Base64, truncated), `DecompressionFailed` and `InvalidEncoding`; the free `decrypt_lyrics` functions are `.ok()` wrappers over the trait impls
- **Searchers layout**: `searchers/mod.rs` only holds the `Searchers` enum and re-exports; scoring (`compare_track`, `rank_by_match`, weights) lives in `compare_helper.rs`, query building and the refinement search in `refinement.rs`
- **Search error contract** (0.3.0): `lyrics-search/src/error.rs` 的 `SearchError`（`Http`/`Json`/`Status`/`Api`/`Captcha`/`Payload`/`InvalidConfig`）是搜索层唯一的失败通道；`Searcher` 与各 provider 的可失败入口都返回 `Result<_, SearchError>`。「没有数据」不是错误：`Ok(None)` = 该曲目没有这种歌词，空 `Vec` = 请求成功但没有结果。`base_api` 只有 `send`/`send_json`/`send_form` + `json`/`text` 五个函数，非 2xx 由 `json`/`text` 统一返回 `Status`（要把 404 当「没有这首歌」的调用方须先判 `response.status()`）。
- **`SyllableItem` 没有 `PartialEq`**（0.3.0 删除了只比时间、忽略文本的实现；上游 C# `ISyllableInfo` 也没有相等语义）。需要按时间比较时显式写 `start_time()`/`end_time()`；`LineInfo` 的 `PartialEq`/`Ord` 只比开始时间，是排序语义（对应上游 `IComparable`），保留不动。
- **Search feature gating**: `lyrics-search` deps (`reqwest`, `tokio`, `async-trait`, `rand`, `urlencoding`, `base64`) are all `optional = true` behind the `search` feature, and both of its modules (`searchers`, `providers`, `error`) are `#[cfg(feature = "search")]` — without the feature the crate compiles empty. `lyrics-helper` gates the entire `lyrics-search` crate behind its own `search` feature.
- **Musixmatch provider**: `providers::web::musixmatch::api_options::ApiOptions` selects the Android (default) or desktop API; `api::set_options(...) -> Result<(), SearchError>` overrides it globally（非法配置返回 `SearchError::InvalidConfig`，不 panic）。

## Important details

- **quick-xml 0.42 (names are `&str`)**: text events never contain entities — `&amp;`/`&#169;` arrive as separate `Event::GeneralRef`, so text must be accumulated across `Text` + `GeneralRef` events (`xml_utils::reference_text`, TTML `build_tree`). Never read attributes with `normalized_value`/`unescape_value`: both apply XML attribute normalisation and turn `\n` into spaces, which collapses the multi-line QRC `LyricContent`; use `xml_utils::attribute_value` (entity unescape only)
- **reqwest 0.13**: features are `json`, `form` (needed by `base_api::send_form`) and `rustls` (aws-lc-rs provider, needs a C toolchain + cmake to build)
- **Versions**: always full `x.y.z` — workspace `version` and every inter-crate `version = "…"` requirement (e.g. `"0.5.0"`), also in READMEs and prose
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
