# lyrics-search

歌词搜索库，提供多平台歌曲搜索、匹配和歌词获取功能。

## 支持的平台

- QQ 音乐
- 网易云音乐
- 酷狗音乐
- 汽水音乐
- Apple Music
- Musixmatch
- LRCLIB
- Spotify

## 依赖

```toml
[dependencies]
lyrics-search = "0.1"
```

需要启用 `search` feature（默认启用），依赖 `reqwest` 和 `tokio`。

## 使用

通常不需要直接依赖此 crate，建议使用门面库 `lyrics-helper`：

```rust
use lyrics_helper::search::search_song;
```

## 许可证

Apache-2.0
