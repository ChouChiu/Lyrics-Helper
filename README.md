# Lyrics Helper

Rust 歌词工具库，支持解析、生成、解密、搜索多种歌词格式。从 [WXRIW/Lyricify-Lyrics-Helper](https://github.com/WXRIW/Lyricify-Lyrics-Helper)（C#）重写而来。

## 快速开始

将 `lyrics-helper` 添加到你的项目：

```toml
[dependencies]
lyrics-helper = "0.2.0"
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
cargo run --example demo -- generate lyrics-helper/tests/test_data/QrcDemo.txt qrc lrc
```

## 从 0.1 升级到 0.2

0.2.0 改变了逐字歌词的音节模型：`LineInfo::Syllable` 与 `LineInfo::FullSyllable`
的 `syllables` 字段由 `Vec<SyllableInfo>` 变为 `Vec<SyllableItem>`，对应上游 C# 的
`ISyllableInfo`。同一单词内被合并的音节表示为 `SyllableItem::Full(FullSyllableInfo)`，
它保留各子音节各自的时间信息，聚合文本与首尾时间由子项推导。

只读取文本或时间的代码改动很小，把字段访问换成同名方法即可：

```rust
// 0.1
let text = &syllables[0].text;
let start = syllables[0].start_time;

// 0.2
let text = syllables[0].text();
let start = syllables[0].start_time();
```

需要拿回扁平的 `SyllableInfo` 序列时（例如自己生成逐字格式）：

```rust
use lyrics_helper::{flatten_syllable_items, to_syllable_items};

let flat = flatten_syllable_items(&syllables);   // Vec<SyllableItem> -> Vec<SyllableInfo>
let items = to_syllable_items(flat);             // 反向包装
```

单个音节项也可以用 `parts()` 取到不分配的 `&[SyllableInfo]`。就地修改
`FullSyllableInfo` 的子音节后，必须调用 `refresh_properties()` 让缓存失效。

## 支持的格式

| 功能 | 格式 |
|------|------|
| **解析** | Lyricify Syllable, Lyricify Lines, LRC, QRC, KRC, YRC, TTML, Spotify JSON, Musixmatch JSON, Apple Music |
| **生成** | Lyricify Syllable, Lyricify Lines, LRC, QRC, KRC, YRC |
| **解密** | QRC, KRC |
| **搜索** | QQ 音乐, 网易云音乐, 酷狗音乐, 汽水音乐, Apple Music, Musixmatch, LRCLIB, Spotify |

除上述歌词格式外，`helpers::type_helper` 还能识别 QRC XML、网易云完整 YRC JSON 与 Apple Music API JSON 等原始封装类型：

```rust
use lyrics_helper::helpers::type_helper::{get_lyrics_types, is_qrc_full, try_parse_raw_type};
use lyrics_helper::LyricsRawTypes;

let raw = /* 平台原始响应 */ "";
let raw_type = get_lyrics_types(raw);      // -> LyricsRawTypes（未知格式返回 Unknown）
let parsed = try_parse_raw_type("qrc (xml)"); // -> Some(LyricsRawTypes::QrcFull)
```

注意：与上游一致，`parse_lyrics` 不解析这些外层封装（返回 `None`），需要调用方先取出内嵌歌词；
仓库内的各平台 Provider 已经完成这一步（如 QQ 音乐会自动展开 QRC XML）。

各平台的在线歌词接口位于 `search::providers::web`，如网易云音乐同时提供逐行与逐字歌词：

```rust
use lyrics_helper::LyricsRawTypes;
use lyrics_helper::search::providers::web::netease;

// 逐字歌词：eapi 接口返回的 YRC 文本（含信息行），可直接按 LyricsRawTypes::Yrc 解析
if let Some(lyrics) = netease::api::get_syllable_lyrics(423997333).await
    && let Some(yrc) = lyrics.yrc
{
    let parsed = lyrics_helper::parse(&yrc, LyricsRawTypes::Yrc);
}

// 逐行歌词：旧接口返回的 LRC 文本与翻译；曲目没有逐字歌词时可用它回退
let (lrc, translation) = netease::api::get_lyrics(423997333).await.unwrap();
```

## 歌词处理优化

`helpers::optimization` 提供与上游一一对应的优化函数：

| 模块 | 作用 |
|------|------|
| `syllable_word_merger` | 把同一单词内的连续音节合并为合并音节（保留各子音节时间） |
| `apple_music` | Apple Music TTML 歌词预处理与大小写规范化 |
| `info_lines` | 制作人员等「信息行」识别与首尾计数 |
| `musixmatch` | Musixmatch richsync 歌词标准化 |
| `sync_downgrade` | 逐字歌词降级为逐行 |
| `yrc` | YRC 歌词标准化 |
| `explicit` | explicit 标记处理与修复 |

搜索功能需要启用 `search` feature（默认启用），依赖 `reqwest` 和 `tokio`。如需纯离线解析库，禁用默认 features：

```toml
lyrics-helper = { version = "0.2.0", default-features = false }
```

## 项目架构

项目采用 Cargo workspace，由 6 个 crate 组成：

```text
Lyricify-Lyrics-Helper/          # workspace 根目录
├── Cargo.toml                   # workspace 定义
├── lyrics-core/                 # 核心模型与 traits
│   └── src/
│       ├── models/              # LyricsData, LineInfo, SyllableItem, TrackMetadata, 枚举
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
│   ├── examples/
│   │   ├── demo.rs              # 解析/生成/解密演示
│   │   ├── search_test.rs       # 搜索 API 演示
│   │   └── search_lyrics_test.rs # 搜索+获取歌词演示
│   └── src/lib.rs               # 顶层 API：parse, parse_auto, generate_string
└── AGENTS.md
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
