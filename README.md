# Lyrics Helper

Rust 歌词工具库，支持解析、生成、解密、搜索多种歌词格式。从 [WXRIW/Lyricify-Lyrics-Helper](https://github.com/WXRIW/Lyricify-Lyrics-Helper)（C#）重写而来。

## 快速开始

将 `lyrics-helper` 添加到你的项目：

```toml
[dependencies]
lyrics-helper = "0.1"
```

自动检测格式并解析歌词：

```rust
use lyrics_helper::parse_auto;

fn main() {
    let content = "[00:12.00]Hello World\n[00:15.50]Second line";
    let data = parse_auto(content).unwrap();

    if let Some(lines) = &data.lines {
        println!("共 {} 行歌词", lines.len());
        if let Some(first) = lines.first() {
            println!("第一行: {:?}", first.text_from_any());
        }
    }
}
```

指定格式解析并转换：

```rust
use lyrics_helper::{parse, generate_string, LyricsRawTypes, LyricsTypes};

fn main() {
    let lrc_content = "[00:12.00]Hello World\n[00:15.50]Second line";
    let data = parse(lrc_content, LyricsRawTypes::Lrc).unwrap();

    // LRC → QRC
    let qrc_output = generate_string(&data, LyricsTypes::Qrc).unwrap();
    println!("{}", qrc_output);
}
```

运行完整示例（从 workspace 根目录）：

```bash
cargo run --example demo -- parsers-demo
cargo run --example demo -- parse lyrics-helper/tests/test_data/LrcDemo.txt lrc
cargo run --example demo -- generate lyrics-helper/tests/test_data/LrcDemo.txt lrc qrc
```

## 支持的格式

| 功能 | 格式 |
|------|------|
| **解析** | Lyricify Syllable, Lyricify Lines, LRC, QRC, KRC, YRC, TTML, Spotify JSON, Musixmatch JSON, Apple Music |
| **生成** | Lyricify Syllable, Lyricify Lines, LRC, QRC, KRC, YRC |
| **解密** | QRC, KRC |
| **搜索** | QQ 音乐, 网易云音乐, 酷狗音乐, 汽水音乐, Apple Music, Musixmatch, LRCLIB, Spotify |

搜索功能需要启用 `search` feature（默认启用），依赖 `reqwest` 和 `tokio`。如需纯离线解析库，禁用默认 features：

```toml
lyrics-helper = { version = "0.1", default-features = false }
```

## 项目架构

项目采用 Cargo workspace，由 6 个 crate 组成：

```text
Lyricify-Lyrics-Helper/          # workspace 根目录
├── Cargo.toml                   # workspace 定义
├── lyrics-core/                 # 核心模型与 traits
│   └── src/
│       ├── models/              # LyricsData, LineInfo, SyllableInfo, TrackMetadata, 枚举
│       ├── traits/              # LyricsParser, LyricsGenerator, LyricsDecrypter
│       └── helpers/             # chinese, string, math, offset, type detection, optimization
├── lyrics-parsers/              # 每种格式一个解析器
├── lyrics-generators/           # 每种格式一个生成器
├── lyrics-crypto/               # QRC 和 KRC 解密（AES/DES/ECB/CBC）
├── lyrics-search/               # 各平台歌曲搜索（search feature）
├── lyrics-helper/               # 门面 crate，re-export 所有子 crate
│   ├── tests/
│   │   ├── parser_tests.rs      # 集成测试
│   │   └── test_data/           # 各格式示例歌词文件
│   └── src/lib.rs               # 顶层 API：parse, parse_auto, generate_string
└── examples/
    ├── demo.rs                  # 解析/生成/解密演示
    ├── search_test.rs           # 搜索 API 演示
    └── search_lyrics_test.rs    # 搜索+获取歌词演示
```

**依赖关系**：`lyrics-core` ← `lyrics-parsers` / `lyrics-generators` / `lyrics-crypto` ← `lyrics-search` ← `lyrics-helper`

用户只需依赖 `lyrics-helper`，通过 `use lyrics_helper::*` 即可访问全部功能。

## 开发

所有命令在 workspace 根目录执行：

```bash
cargo build                      # 构建全部 crate
cargo test                       # 运行全部测试
cargo test -p lyrics-helper      # 仅运行门面 crate 测试
cargo test test_parse_lrc        # 运行单个测试
cargo clippy                     # lint
cargo fmt                        # 格式化
```

项目使用 Rust 2024 edition（需要 Rust 1.85+），dev-dependency 包含 `pretty_assertions` 用于测试输出对比。

## 致谢

基于 [WXRIW/Lyricify-Lyrics-Helper](https://github.com/WXRIW/Lyricify-Lyrics-Helper)（C#）重写为 Rust 版本。
