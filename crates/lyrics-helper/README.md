# Lyrics Helper

[![crates.io](https://img.shields.io/crates/v/lyrics-helper.svg)](https://crates.io/crates/lyrics-helper)
[![docs.rs](https://img.shields.io/docsrs/lyrics-helper)](https://docs.rs/lyrics-helper)
[![license](https://img.shields.io/crates/l/lyrics-helper.svg)](LICENSE)

Rust 歌词处理工具库，提供常见歌词格式的解析、生成、解密以及跨平台在线搜索。移植自 C# 项目 [WXRIW/Lyricify-Lyrics-Helper](https://github.com/WXRIW/Lyricify-Lyrics-Helper)。

## 功能概览

- **格式解析与生成**：支持 LRC、QRC、KRC、YRC 以及 Lyricify Syllable / Lines 的解析与生成；支持 TTML（含 Apple Music / AMLL 扩展）、Spotify JSON、Musixmatch JSON 的解析。逐字歌词可降级导出为逐行 LRC。
- **歌词解密**：支持 QQ 音乐 QRC（Triple DES）、酷狗音乐 KRC（XOR）解密，以及网易云音乐 eapi 请求参数加密。
- **多平台搜索**：支持在网易云音乐、QQ 音乐、酷狗音乐、汽水音乐、Musixmatch、LRCLIB、AMLL TTML DB 搜索歌曲并获取歌词；支持 Spotify 与 Apple Music 的曲目搜索（需启用 `search` feature）。

更多技术细节、外层封装展开与格式转换矩阵见 [项目 Wiki](https://github.com/ChouChiu/Lyrics-Helper/wiki)。

## 安装与配置

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
lyrics-helper = "0.5.0"
```

搜索模块依赖 `reqwest` 和 `tokio`，由 `search` feature 控制（默认开启）。若仅需本地歌词解析、生成或解密，可关闭默认特性以保持零网络依赖：

```toml
[dependencies]
lyrics-helper = { version = "0.5.0", default-features = false }
```

## 快速上手

### 1. 格式解析与转换

```rust
use lyrics_helper::{generate_string, parse_auto, LyricsTypes};

fn main() {
    // 自动识别歌词格式（以带音节时间的 QRC 为例）
    let qrc = "[0,1500]Hello(0,500) (500,500)World(1000,500)";
    let data = parse_auto(qrc).expect("解析失败");

    // 逐字歌词降级生成逐行 LRC
    let lrc = generate_string(&data, LyricsTypes::Lrc).expect("生成失败");
    println!("{lrc}");
}
```

### 2. 歌词解密

```rust
use lyrics_helper::decrypt_qrc;

fn main() {
    // 一段带属性头的 QRC 加密密文（Triple DES + zlib）
    let encrypted = "61EA2D770702AE2B2B52DA9EDDEC07BB35F01431C529E8AE46B70CD635C127867\
                     E1AB832ABFF18CF7AABF1313EF7EF537021F03A5E957206";

    if let Some(decrypted) = decrypt_qrc(encrypted) {
        println!("{decrypted}");
    }
}
```

### 3. 在线搜索（需要 `search` feature）

```rust
use lyrics_helper::models::TrackMetadata;
use lyrics_helper::searchers::netease::NeteaseSearcher;
use lyrics_helper::searchers::search_for_best_result;

#[tokio::main]
async fn main() {
    let mut track = TrackMetadata::new();
    track.title = Some("晴天".to_string());
    track.artist = Some("周杰伦".to_string());
    track.ensure_artists();

    match search_for_best_result(&NeteaseSearcher, &track).await {
        Ok(Some(best)) => println!("匹配到: {} - {}", best.title, best.artist()),
        Ok(None) => println!("未找到匹配歌曲"),
        Err(err) => eprintln!("搜索出错: {err}"),
    }
}
```

## 文档

技术规范与 API 详情请参阅：

- [API 文档 (docs.rs)](https://docs.rs/lyrics-helper)
- [项目 Wiki](https://github.com/ChouChiu/Lyrics-Helper/wiki)
  - [支持格式与转换矩阵](https://github.com/ChouChiu/Lyrics-Helper/wiki/Supported-Formats)
  - [搜索平台与接口说明](https://github.com/ChouChiu/Lyrics-Helper/wiki/Search)
  - [歌词解密](https://github.com/ChouChiu/Lyrics-Helper/wiki/Decryption)
  - [辅助工具（时间偏移、类型检测、歌词优化）](https://github.com/ChouChiu/Lyrics-Helper/wiki/Helpers)
  - [项目架构](https://github.com/ChouChiu/Lyrics-Helper/wiki/Architecture)
  - [版本升级指南](https://github.com/ChouChiu/Lyrics-Helper/wiki/Migration-0.5)

## 致谢

- 基于 [WXRIW/Lyricify-Lyrics-Helper](https://github.com/WXRIW/Lyricify-Lyrics-Helper)（C#）重写为 Rust 版本。
- 逐词 TTML 歌词源来自社区维护的 [AMLL TTML DB](https://github.com/amll-dev/amll-ttml-db)。

## License

[Apache-2.0](LICENSE)
