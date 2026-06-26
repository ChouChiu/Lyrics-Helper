# lyrics-generators

歌词生成器库，支持将 `LyricsData` 模型导出为多种格式字符串。

## 支持的格式

- LRC
- QRC
- KRC
- YRC
- Lyricify Syllable
- Lyricify Lines

## 依赖

```toml
[dependencies]
lyrics-generators = "0.1"
```

## 使用

通常不需要直接依赖此 crate，建议使用门面库 `lyrics-helper`：

```rust
use lyrics_helper::{parse, generate_string, LyricsRawTypes, LyricsTypes};

let data = parse("[00:12.00]Hello World", LyricsRawTypes::Lrc).unwrap();
let qrc = generate_string(&data, LyricsTypes::Qrc).unwrap();
```

## 许可证

Apache-2.0
