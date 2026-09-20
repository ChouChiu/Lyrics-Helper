# Lyrics Helper

[![crates.io](https://img.shields.io/crates/v/lyrics-helper.svg)](https://crates.io/crates/lyrics-helper)
[![docs.rs](https://img.shields.io/docsrs/lyrics-helper)](https://docs.rs/lyrics-helper)
[![license](https://img.shields.io/crates/l/lyrics-helper.svg)](LICENSE)

Rust 歌词工具库：解析、生成、解密、搜索多种歌词格式。从 [WXRIW/Lyricify-Lyrics-Helper](https://github.com/WXRIW/Lyricify-Lyrics-Helper)（C#）重写而来。

## 安装

```bash
cargo add lyrics-helper
```

## 快速开始

```rust
use lyrics_helper::{generate_string, parse_auto, LyricsTypes};

fn main() {
    // 自动识别格式（这里是带音节时间的 QRC）
    let qrc = "[0,1500]Hello(0,500) (500,500)World(1000,500)";
    let data = parse_auto(qrc).unwrap();

    // 逐字歌词降级为逐行 LRC
    let lrc = generate_string(&data, LyricsTypes::Lrc).unwrap();
    println!("{lrc}");
}
```

更多用法见 Wiki 的[快速开始](https://github.com/ChouChiu/Lyrics-Helper/wiki/Getting-Started)。

## 支持的格式

| 功能 | 格式 |
|------|------|
| **解析** | Lyricify Syllable、Lyricify Lines、LRC、QRC、KRC、YRC、TTML、Spotify JSON、Musixmatch JSON、Apple Music JSON |
| **生成** | Lyricify Syllable、Lyricify Lines、LRC、QRC、KRC、YRC |
| **解密** | QRC、KRC |
| **搜索** | QQ 音乐、网易云音乐、酷狗音乐、汽水音乐、Apple Music、Musixmatch、LRCLIB、Spotify |

格式转换有方向性：逐字歌词可以降级为逐行，反过来不行。完整的转换矩阵见
[支持格式](https://github.com/ChouChiu/Lyrics-Helper/wiki/Supported-Formats)。

## 离线使用

搜索功能由 `search` feature 控制（默认开启），会引入 `reqwest` 与 `tokio`。
只需要解析/生成/解密时可以关掉：

```bash
cargo add lyrics-helper --no-default-features
```

## 文档

完整文档在 [Wiki](https://github.com/ChouChiu/Lyrics-Helper/wiki)，API 详情见 [docs.rs](https://docs.rs/lyrics-helper)。

| | |
|---|---|
| [快速开始](https://github.com/ChouChiu/Lyrics-Helper/wiki/Getting-Started) | 安装与基本用法 |
| [支持格式](https://github.com/ChouChiu/Lyrics-Helper/wiki/Supported-Formats) | 各格式详解与转换矩阵 |
| [API 概览](https://github.com/ChouChiu/Lyrics-Helper/wiki/API-Reference) | 顶层函数、数据结构、枚举 |
| [搜索功能](https://github.com/ChouChiu/Lyrics-Helper/wiki/Search) | 多平台搜索与歌词获取 |
| [歌词解密](https://github.com/ChouChiu/Lyrics-Helper/wiki/Decryption) | QRC / KRC 解密 |
| [辅助工具](https://github.com/ChouChiu/Lyrics-Helper/wiki/Helpers) | 类型检测、时间偏移、歌词优化 |
| [项目架构](https://github.com/ChouChiu/Lyrics-Helper/wiki/Architecture) | 6 个 crate 的划分与依赖 |
| [开发与构建](https://github.com/ChouChiu/Lyrics-Helper/wiki/Development) | 构建、测试、lint |

### 版本升级

| | |
|---|---|
| [0.3 → 0.4](https://github.com/ChouChiu/Lyrics-Helper/wiki/Migration-0.4) | 移除聚合缓存、LRCLIB 结构合并、KRC 解密修复 |
| [0.2 → 0.3](https://github.com/ChouChiu/Lyrics-Helper/wiki/Migration-0.3) | 搜索层改为类型化错误、`SyllableItem` 相等语义移除 |
| [0.1 → 0.2](https://github.com/ChouChiu/Lyrics-Helper/wiki/Migration-0.2) | 音节模型改为 `SyllableItem` |

## 致谢

基于 [WXRIW/Lyricify-Lyrics-Helper](https://github.com/WXRIW/Lyricify-Lyrics-Helper)（C#）重写为 Rust 版本。

## License

[Apache-2.0](LICENSE)
