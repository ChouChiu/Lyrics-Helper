# lyrics-parsers

歌词解析器库，支持将多种歌词格式解析为统一的 `LyricsData` 模型。

## 支持的格式

- LRC
- QRC
- KRC
- YRC
- TTML
- Spotify JSON
- Musixmatch JSON
- Apple Music
- Lyricify Syllable
- Lyricify Lines

## 依赖

```toml
[dependencies]
lyrics-parsers = "0.1"
```

## 使用

通常不需要直接依赖此 crate，建议使用门面库 `lyrics-helper`：

```rust
use lyrics_helper::parse_auto;

let data = parse_auto("[00:12.00]Hello World").unwrap();
```

## 许可证

Apache-2.0
