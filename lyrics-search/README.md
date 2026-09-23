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
- AMLL TTML DB（社区逐词 TTML 歌词库）

## 依赖

```toml
[dependencies]
lyrics-search = "0.5.0"
lyrics-core = "0.5.0"
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

## 平台说明

网易云音乐的搜索依次尝试两个接口：web 接口（`api/search/get/web`）与 eapi 接口
（`cloudsearch/pc`）。前者在海外 IP 下返回加密结果、在风控网络环境下返回 `-460`，两种情况
都会自动改用 eapi 接口，并记住该选择供后续请求使用（因此首个请求可能比后续请求慢）。
两个接口都失败时返回第一次尝试的错误。

AMLL TTML DB 只有静态文件：`amll_ttml_db::api::search` 下载仓库的元数据索引（约 1.6 MB，
缓存一小时，同一首歌只保留最新版本）后在本地按关键词匹配，结果的 `id` 是 `raw-lyrics/` 下的文件名，
交给 `get_raw_lyrics` 取 TTML；已知平台 ID 时可用 `get_lyrics(Platform, id, Format)` 直接取
TTML、LRC、YRC、QRC 等格式，没有这首歌时返回 `Ok(None)`。国内访问 GitHub 不畅时可用
`set_base_url(BIKONOO_BASE_URL)` 等切换到目录结构一致的镜像

搜索结果与运行环境有关：曲目受地区版权限制，海外 IP 通常只能搜到翻唱等条目，
所以不要对搜索结果做断言。

## 许可证

Apache-2.0
