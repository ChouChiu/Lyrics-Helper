# Lyrics Helper

Rust 歌词工具库，支持解析、生成、解密、搜索多种歌词格式。从 [WXRIW/Lyricify-Lyrics-Helper](https://github.com/WXRIW/Lyricify-Lyrics-Helper)（C#）重写而来。

## 快速开始

将 `lyrics-helper` 添加到你的项目：

```toml
[dependencies]
lyrics-helper = "0.1"
```

解析一段歌词（自动检测格式）：

```rust
use lyrics_helper::parse_auto;

fn main() {
    let content = "[00:12.00]Hello World\n[00:15.50]Second line";
    let data = parse_auto(content).unwrap();

    let lines = data.lines.unwrap();
    println!("共 {} 行歌词", lines.len());
    println!("第一行: {:?}", lines[0].text_from_any());
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

运行完整示例：

```bash
cd lyrics-helper
cargo run --example demo -- parsers-demo
cargo run --example demo -- parse tests/test_data/LrcDemo.txt lrc
cargo run --example demo -- generate tests/test_data/LrcDemo.txt lrc qrc
```

## 支持的格式

| 功能 | 格式 |
|------|------|
| **解析** | Lyricify Syllable, Lyricify Lines, LRC, QRC, KRC, YRC, TTML, Spotify JSON, Musixmatch JSON, Apple Music |
| **生成** | Lyricify Syllable, Lyricify Lines, LRC, QRC, KRC, YRC |
| **解密** | QRC, KRC |
| **搜索** | QQ 音乐, 网易云音乐, 酷狗音乐, 汽水音乐, Apple Music, Musixmatch, LRCLIB |

搜索功能需要启用 `search` feature（默认启用），依赖 `reqwest` 和 `tokio`。如需纯离线解析库，禁用默认 features：

```toml
lyrics-helper = { version = "0.1", default-features = false }
```

## 项目架构

所有源码位于 `lyrics-helper/src/`，目录结构如下：

```text
lyrics-helper/
├── src/
│   ├── lib.rs                  # 入口：re-export models, parse, generate_string
│   ├── models/                 # 歌词数据模型
│   │   ├── lyrics_data.rs      # LyricsData 核心结构
│   │   ├── line_info.rs        # LineInfo（行信息）
│   │   ├── syllable_info.rs    # SyllableInfo（音节信息）
│   │   ├── track_metadata.rs   # TrackMetadata（曲目元数据）
│   │   └── lyrics_types.rs     # 枚举：LyricsRawTypes, LyricsTypes, SyncTypes
│   ├── parsers/                # 每种格式一个解析器
│   ├── generators/             # 每种格式一个生成器
│   ├── decrypter/              # QRC 和 KRC 解密（AES/DES）
│   ├── helpers/                # 工具函数
│   │   ├── chinese_helper.rs   # 简繁转换
│   │   ├── string_helper.rs    # 字符串处理
│   │   ├── math_helper.rs      # 数学工具
│   │   ├── offset_helper.rs    # 偏移计算
│   │   ├── type_helper.rs      # 格式自动检测
│   │   └── optimization/       # 歌词优化（explicit, YRC, Musixmatch 等）
│   ├── searchers/              # 各平台歌曲搜索（search feature）
│   └── providers/web/          # 各平台 API 客户端（search feature）
├── tests/
│   ├── parser_tests.rs         # 集成测试
│   └── test_data/              # 各格式示例歌词文件
└── examples/
    ├── demo.rs                 # 解析/生成/解密演示
    └── search_test.rs          # 搜索 API 演示
```

## 开发

```bash
cd lyrics-helper

# 构建
cargo build

# 运行全部测试
cargo test

# 运行单个测试
cargo test test_parse_lrc

# Lint
cargo clippy

# 格式化
cargo fmt
```

项目使用 Rust 2024 edition（需要 Rust 1.85+），dev-dependency 包含 `pretty_assertions` 用于测试输出对比。

## 致谢

基于 [WXRIW/Lyricify-Lyrics-Helper](https://github.com/WXRIW/Lyricify-Lyrics-Helper)（C#）重写为 Rust 版本。
