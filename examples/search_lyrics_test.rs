use lyrics_helper::models::{LyricsRawTypes, TrackMetadata};
use lyrics_helper::search::providers::web::{kugou, lrclib, musixmatch, netease, qq_music};
use lyrics_helper::searchers::apple_music::AppleMusicSearcher;
use lyrics_helper::searchers::kugou::KugouSearcher;
use lyrics_helper::searchers::lrclib::LRCLIBSearcher;
use lyrics_helper::searchers::musixmatch::MusixmatchSearcher;
use lyrics_helper::searchers::netease::NeteaseSearcher;
use lyrics_helper::searchers::qq_music::QQMusicSearcher;
use lyrics_helper::searchers::search_for_best_result;
use lyrics_helper::searchers::soda_music::SodaMusicSearcher;
use lyrics_helper::searchers::spotify::SpotifySearcher;
use lyrics_helper::searchers::Searchers;

#[tokio::main]
async fn main() {
    let mut track = TrackMetadata::new();
    track.title = Some("Bang Bang".to_string());
    track.artist = Some("Jessie J, Ariana Grande, Nicki Minaj".to_string());
    track.album = Some("Sweet Talker".to_string());
    track.duration_ms = Some(199000);
    track.ensure_artists();

    println!("=== 全平台搜索并获取歌词: Bang Bang ===\n");

    // Phase 1: Search all platforms
    let spotify = SpotifySearcher::new(String::new());
    let apple_music = AppleMusicSearcher::new(String::new());

    let platforms: Vec<(&str, &dyn lyrics_helper::searchers::searcher::Searcher)> = vec![
        ("网易云", &NeteaseSearcher),
        ("QQ音乐", &QQMusicSearcher),
        ("酷狗", &KugouSearcher),
        ("Musixmatch", &MusixmatchSearcher),
        ("汽水音乐", &SodaMusicSearcher),
        ("Spotify", &spotify),
        ("Apple Music", &apple_music),
        ("LRCLIB", &LRCLIBSearcher),
    ];

    struct SearchResultWithPlatform {
        platform: String,
        searcher_type: Searchers,
        title: String,
        artists: String,
        album: String,
        match_type: String,
        id: String,
        numeric_id: Option<i64>,
        duration_ms: Option<i32>,
    }

    let mut all_results: Vec<SearchResultWithPlatform> = Vec::new();

    for (name, searcher) in &platforms {
        print!("  搜索 {} ... ", name);
        match search_for_best_result(*searcher, &track).await {
            Some(r) => {
                let artists = r.artist();
                println!("✅ {} - {} (匹配: {:?})", r.title, artists, r.match_type);
                all_results.push(SearchResultWithPlatform {
                    platform: name.to_string(),
                    searcher_type: r.searcher_type,
                    title: r.title,
                    artists,
                    album: r.album,
                    match_type: format!("{:?}", r.match_type),
                    id: r.id,
                    numeric_id: r.numeric_id,
                    duration_ms: r.duration_ms,
                });
            }
            None => println!("❌ 未找到"),
        }
    }

    println!("\n=== 搜索结果汇总: {} 个平台返回结果 ===\n", all_results.len());

    // Phase 2: Fetch lyrics from platforms that support it
    for r in &all_results {
        println!("--- {} ({}) ---", r.platform, r.match_type);
        println!("  {} - {} [ID: {}]", r.title, r.artists, r.id);

        let lyrics_text: Option<String> = match r.searcher_type {
            Searchers::Netease => {
                let song_id: i64 = r.id.parse().unwrap_or(0);
                match netease::api::get_lyrics(song_id).await {
                    Some((lyric, _trans)) => lyric,
                    None => {
                        println!("  ❌ 获取歌词失败");
                        println!();
                        continue;
                    }
                }
            }
            Searchers::QQMusic => {
                match qq_music::api::get_lyrics(
                    &r.id,
                    r.numeric_id,
                    &r.title,
                    &r.artists,
                    &r.album,
                    r.duration_ms,
                ).await {
                    Some((lyric, _trans)) => lyric,
                    None => {
                        println!("  ❌ 获取歌词失败");
                        println!();
                        continue;
                    }
                }
            }
            Searchers::Kugou => {
                let keyword = format!("{} {}", r.title, r.artists);
                let dur = r.duration_ms.unwrap_or(0);
                match kugou::api::get_lyrics(&keyword, &r.id, dur).await {
                    Some(l) => Some(l),
                    None => {
                        println!("  ❌ 获取歌词失败");
                        println!();
                        continue;
                    }
                }
            }
            Searchers::LRCLIB => {
                let id: i32 = r.id.parse().unwrap_or(0);
                match lrclib::api::get_by_id(id).await {
                    Some(lr) => lr.synced_lyrics.or(lr.plain_lyrics),
                    None => {
                        println!("  ❌ 获取歌词失败");
                        println!();
                        continue;
                    }
                }
            }
            Searchers::Musixmatch => {
                let track_id: i64 = r.id.parse().unwrap_or(0);
                match musixmatch::api::get_token().await {
                    Some(token) => {
                        musixmatch::api::get_synced_lyrics(track_id, &token)
                            .await
                            .or_else(|| {
                                // Block on the async plain lyrics fetch
                                // Since we're already in async context, use futures::executor
                                None // fallback: no synced, skip plain for simplicity
                            })
                    }
                    None => None,
                }
            }
            _ => {
                println!("  ⚠️ 该平台暂不支持获取歌词");
                println!();
                continue;
            }
        };

        if let Some(ref text) = lyrics_text {
            let line_count = text.lines().count();
            let preview: Vec<&str> = text.lines().take(5).collect();
            for line in &preview {
                println!("  {}", line);
            }
            if line_count > 5 {
                println!("  ... (共 {} 行)", line_count);
            }

            // Try to parse
            let raw_type = lyrics_helper::helpers::type_helper::get_lyrics_types(text);
            if raw_type != LyricsRawTypes::Unknown {
                if let Some(parsed) = lyrics_helper::parsers::parsers::parse_lyrics(text, raw_type) {
                    if let Some(ref lines) = parsed.lines {
                        println!("  ✅ 解析成功: {} 行, 格式: {:?}", lines.len(), raw_type);
                    }
                }
            } else {
                println!("  ⚠️ 无法自动检测格式");
            }
        } else {
            println!("  ❌ 未获取到歌词内容");
        }
        println!();
    }

    println!("=== 完成 ===");
}
