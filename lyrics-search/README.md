# lyrics-search

歌词搜索库，提供多平台歌曲搜索、匹配和歌词获取功能。

## 支持的平台

- QQ 音乐（歌词及翻译）
- 网易云音乐（逐行 LRC 与逐字 YRC 歌词）
- 酷狗音乐（KRC 逐字歌词）
- 汽水音乐（曲目详情歌词）
- Apple Music
- Musixmatch
- LRCLIB
- Spotify

## 依赖

```toml
[dependencies]
lyrics-search = "0.3"
lyrics-core = "0.3"
```

需要启用 `search` feature（默认启用），依赖 `reqwest` 和 `tokio`。

## 使用

通常不需要直接依赖此 crate，建议使用门面库 `lyrics-helper`。

直接依赖 `lyrics-search` 时，搜索接口使用的 `TrackMetadata` 来自 `lyrics-core`，
因此 `lyrics-core` 需要一并引入（上面的 `[dependencies]` 已列出）。

各平台搜索结果统一由 `Searcher` trait 给出，失败原因通过类型化错误暴露：

```rust
use lyrics_search::searchers::netease::NeteaseSearcher;
use lyrics_search::searchers::search_for_best_result;
use lyrics_search::error::SearchError;
use lyrics_core::models::TrackMetadata;

let mut track = TrackMetadata::new();
track.title = Some("晴天".to_string());
track.artist = Some("周杰伦".to_string());
track.ensure_artists();

match search_for_best_result(&NeteaseSearcher, &track).await {
    Ok(Some(best)) => println!("{}", best.title),
    Ok(None) => println!("搜索成功但没有结果"),
    Err(SearchError::Captcha) => println!("命中验证码"),
    Err(error) => println!("搜索失败: {error}"),
}
```

`Ok(vec![])` 与 `Ok(None)` 表示「没有数据」，`Err` 才表示请求失败；具体变体见
`lyrics_search::error::SearchError`。

## 许可证

Apache-2.0
