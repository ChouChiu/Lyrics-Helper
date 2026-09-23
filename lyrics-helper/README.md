# Lyrics Helper

Rust 歌词工具库，支持解析、生成、解密、搜索多种歌词格式。从 [WXRIW/Lyricify-Lyrics-Helper](https://github.com/WXRIW/Lyricify-Lyrics-Helper)（C#）重写而来。

## 快速开始

将 `lyrics-helper` 添加到你的项目：

```toml
[dependencies]
lyrics-helper = "0.5.0"
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
    // QRC 是带音节时间的逐字格式
    let qrc_content = "[0,1500]Hello(0,500) (500,500)World(1000,500)";
    let data = parse(qrc_content, LyricsRawTypes::Qrc).unwrap();

    // QRC → LRC
    let lrc_output = generate_string(&data, LyricsTypes::Lrc).unwrap();
    println!("{}", lrc_output);
}
```

注意方向性：QRC / KRC / YRC 的生成器只输出带音节的行，因此逐字歌词可以降级为
逐行的 LRC，反过来用 LRC 生成 QRC 只会得到空字符串（`generate_string` 返回
`None`）。需要反向转换时，得先用 QRC / KRC / YRC / TTML 等含音节的输入解析。

运行完整示例（从 workspace 根目录）：

```bash
cargo run --example demo -- parsers-demo
cargo run --example demo -- parse lyrics-helper/tests/test_data/LrcDemo.txt lrc
cargo run --example demo -- generate lyrics-helper/tests/test_data/QrcDemo.txt qrc lrc
```

## 从 0.1.0 升级到 0.2.0

0.2.0 改变了逐字歌词的音节模型：`LineInfo::Syllable` 与 `LineInfo::FullSyllable`
的 `syllables` 字段由 `Vec<SyllableInfo>` 变为 `Vec<SyllableItem>`，对应上游 C# 的
`ISyllableInfo`。同一单词内被合并的音节表示为 `SyllableItem::Full(FullSyllableInfo)`，
它保留各子音节各自的时间信息，聚合文本与首尾时间由子项推导。

只读取文本或时间的代码改动很小，把字段访问换成同名方法即可：

```rust
// 0.1.0
let text = &syllables[0].text;
let start = syllables[0].start_time;

// 0.2.0
let text = syllables[0].text();
let start = syllables[0].start_time();
```

需要拿回扁平的 `SyllableInfo` 序列时（例如自己生成逐字格式）：

```rust
use lyrics_helper::{
    flatten_syllable_items, to_syllable_items, SyllableInfo, SyllableItem,
};

let syllables = vec![SyllableItem::from(SyllableInfo::new("Hello".to_string(), 0, 500))];

let flat = flatten_syllable_items(&syllables);   // Vec<SyllableItem> -> Vec<SyllableInfo>
let items = to_syllable_items(flat);             // 反向包装

assert_eq!(items.len(), 1);
```

单个音节项也可以用 `parts()` 取到不分配的 `&[SyllableInfo]`。就地修改
`FullSyllableInfo` 的子音节后，必须调用 `refresh_properties()` 让缓存失效。

## 从 0.2.0 升级到 0.3.0

0.3.0 是破坏性版本：搜索层改为返回类型化错误，`SyllableItem` 的相等语义被移除。

### 搜索层返回 `SearchError` 而不是 `None`

`Searcher::search_for_results*`、`search_with_refinement`、`search_for_best_result*`
以及各平台 Provider 的可失败入口，全部改为 `Result<_, SearchError>`：

```rust
use lyrics_helper::models::TrackMetadata;
use lyrics_helper::searchers::netease::NeteaseSearcher;
use lyrics_helper::searchers::search_for_best_result;
use lyrics_helper::SearchError;

let mut track = TrackMetadata::new();
track.title = Some("晴天".to_string());
track.artist = Some("周杰伦".to_string());
track.ensure_artists();

match search_for_best_result(&NeteaseSearcher, &track).await {
    Ok(Some(best)) => println!("{}", best.title),
    Ok(None) => println!("没有搜到"), // 搜索成功，但没有结果
    Err(SearchError::Status(429)) => println!("被限流"),
    Err(SearchError::Captcha) => println!("命中验证码"),
    Err(SearchError::Http(error)) if error.is_timeout() => println!("超时"),
    Err(error) => println!("搜索失败: {error}"),
}
```

「没有数据」不再是错误：`Ok(None)` 表示该曲目没有这种歌词（例如没有逐字歌词），
空 `Vec` 表示搜索成功但没有结果；只有真正的失败才返回 `Err`。

Provider 侧：`netease::api::get_lyrics` 等改返回
`Result<(Option<String>, Option<String>), SearchError>`；Musixmatch 的 `MusixmatchError`
被 `SearchError` 取代，`api::set_options` 不再 panic，非法配置返回
`Err(SearchError::InvalidConfig(_))`；`base_api` 的 7 个请求函数收敛为
`send` / `send_json` / `send_form` 加 `json` / `text` 两个终结方法，
非 2xx 状态码统一返回 `SearchError::Status`（签名里用到的 `Method`、`StatusCode`、
`Response` 已从 `base_api` 再导出）：

```rust
use lyrics_helper::providers::web::base_api;
use lyrics_helper::providers::web::base_api::{Method, StatusCode};
use lyrics_helper::providers::web::lrclib::response::SearchResultItem;

let response = base_api::send(Method::GET, "https://lrclib.net/api/search?track_name=Yesterday", &[])
    .await
    .expect("请求失败");
if response.status() == StatusCode::NOT_FOUND {
    // 404 这类「没有这首歌」由调用方自己判定
    return;
}
let items: Vec<SearchResultItem> = base_api::json(response).await.expect("解码失败");
```

### `SyllableItem` 不再实现 `PartialEq`

0.2.0 的 `SyllableItem` 只比较 `start_time`/`end_time`、完全忽略文本，
两个文本不同的音节只要时间相同就被判为相等，`contains` / `dedup` / `assert_eq!`
都会因此给出违反直觉的结果；上游 C# 的 `ISyllableInfo` 本来也没有任何相等语义，
0.3.0 直接删掉了这个实现。需要按时间比较时显式写：

```rust
use lyrics_helper::{SyllableInfo, SyllableItem};

let a = SyllableItem::from(SyllableInfo::new("晴".to_string(), 1000, 1500));
let b = SyllableItem::from(SyllableInfo::new("天".to_string(), 1000, 1500));

// 0.2.0 里 a == b 为 true（只比时间），0.3.0 起没有 PartialEq，必须显式比较
assert!(a.start_time() == b.start_time() && a.end_time() == b.end_time());
```

`LineInfo` 的 `PartialEq`/`Ord`（只比开始时间）保留不动——它是给排序用的，
对应上游 C# 的 `IComparable`。

## 支持的格式

| 功能 | 格式 |
|------|------|
| **解析** | Lyricify Syllable, Lyricify Lines, LRC, QRC, KRC, YRC, TTML, Spotify JSON, Musixmatch JSON, Apple Music |
| **生成** | Lyricify Syllable, Lyricify Lines, LRC, QRC, KRC, YRC |
| **解密** | QRC, KRC |
| **搜索** | QQ 音乐, 网易云音乐, 酷狗音乐, 汽水音乐, Apple Music, Musixmatch, LRCLIB, Spotify, AMLL TTML DB |

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
// Ok(None) 表示该曲目没有逐字歌词（不是错误），此时可回退到下面的逐行歌词
if let Ok(Some(lyrics)) = netease::api::get_syllable_lyrics(423997333).await
    && let Some(yrc) = lyrics.yrc
{
    let _parsed = lyrics_helper::parse(&yrc, LyricsRawTypes::Yrc);
}

// 逐行歌词：旧接口返回的 LRC 文本与翻译；曲目没有逐字歌词时可用它回退
if let Ok((lrc, translation)) = netease::api::get_lyrics(423997333).await {
    println!("{lrc:?} {translation:?}");
}
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
lyrics-helper = { version = "0.5.0", default-features = false }
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
