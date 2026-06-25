# AGENTS.md

## Project overview

Rust library crate (`lyrics-helper`) for parsing, generating, decrypting, and searching song lyrics. Rewritten from C# ([WXRIW/Lyricify-Lyrics-Helper](https://github.com/WXRIW/Lyricify-Lyrics-Helper)). Chinese-language project by ChouChiu.

## Repo structure

```
lyrics-helper/          # The sole Rust crate (library, not binary)
  src/
    lib.rs              # Re-exports: models::*, parsers::parse, generators::generate_string
    models/             # LyricsData, LineInfo, SyllableInfo, TrackMetadata, enums
    parsers/            # One parser per format: LRC, QRC, KRC, YRC, TTML, Spotify, Musixmatch, Lyricify Syllable/Lines
    generators/         # Generate lyrics strings from parsed models
    decrypter/          # QRC and KRC decryption (AES/DES/ECB/CBC)
    helpers/            # Chinese (繁簡), string, math, offset, type helpers; optimization sub-modules
    searchers/          # Song search by platform (gated behind `search` feature)
    providers/web/      # HTTP client wrappers for lyrics APIs (gated behind `search` feature)
  tests/
    parser_tests.rs     # Integration tests for all parsers
    test_data/          # Sample lyrics files (LRC, QRC, KRC, YRC, TTML, Spotify, Musixmatch, Lyricify formats)
  examples/
    demo.rs             # Basic usage demo
    search_test.rs      # Search API demo
```

## Key commands

All commands run from the `lyrics-helper/` directory:

```bash
# Build
cargo build

# Run all tests
cargo test

# Run a single test by name
cargo test test_parse_lrc

# Run tests for a specific module
cargo test parsers::

# Check without building
cargo check

# Lint
cargo clippy

# Format
cargo fmt

# Run examples
cargo run --example demo
cargo run --example search_test    # requires network
```

## Important details

- **Rust edition 2024** — requires Rust 1.85+. No `rust-toolchain.toml` checked in.
- **Feature flags**: `search` (on by default) enables `reqwest` + `tokio` for network-dependent code (searchers, providers). Disable with `--no-default-features` to build a pure offline parsing library.
- **Test data**: `tests/test_data/` contains sample lyrics files. Tests read them with relative paths (`tests/test_data/*.txt`), so `cargo test` must run from `lyrics-helper/`.
- **No CI, no rustfmt.toml, no clippy.toml** — use standard Rust defaults.
- **Dev dependency**: `pretty_assertions` for readable test diffs.
- **Crate name vs package name**: package is `lyrics-helper`, the Rust crate name is `lyrics_helper` (hyphens → underscores). Use `use lyrics_helper::*` in code.
- **Key entrypoints**: `parsers::parse_lyrics(content, type)` and `parsers::parse_lyrics_auto(content)` for auto-detection. `generators::generate_string(...)` for output.
