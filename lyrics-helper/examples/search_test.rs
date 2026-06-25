use lyrics_helper::searchers::netease::NeteaseSearcher;
use lyrics_helper::searchers::qq_music::QQMusicSearcher;
use lyrics_helper::searchers::kugou::KugouSearcher;
use lyrics_helper::searchers::musixmatch::MusixmatchSearcher;
use lyrics_helper::searchers::soda_music::SodaMusicSearcher;
use lyrics_helper::searchers::spotify::SpotifySearcher;
use lyrics_helper::searchers::lrclib::LRCLIBSearcher;
use lyrics_helper::searchers::searcher::Searcher;
use lyrics_helper::searchers::search_for_best_result;
use lyrics_helper::models::TrackMetadata;

#[tokio::main]
async fn main() {
    let mut track = TrackMetadata::new();
    track.title = Some("Bang Bang".to_string());
    track.artist = Some("Jessie J, Ariana Grande, Nicki Minaj".to_string());
    track.album = Some("Sweet Talker".to_string());
    track.duration_ms = Some(199000);

    let spotify = SpotifySearcher::new(String::new());
    let platforms: Vec<(&str, &dyn Searcher)> = vec![
        ("网易云", &NeteaseSearcher),
        ("QQ音乐", &QQMusicSearcher),
        ("酷狗", &KugouSearcher),
        ("Musixmatch", &MusixmatchSearcher),
        ("汽水音乐", &SodaMusicSearcher),
        ("Spotify", &spotify),
        ("LRCLIB", &LRCLIBSearcher),
    ];

    println!("=== 搜索 'Bang Bang - Jessie J' ===\n");

    for (name, searcher) in &platforms {
        println!("--- {} ---", name);
        match search_for_best_result(*searcher, &track).await {
            Some(best) => {
                println!("  标题: {}", best.title);
                println!("  艺术家: {}", best.artist());
                println!("  专辑: {}", best.album);
                println!("  时长: {:?}ms", best.duration_ms);
                println!("  匹配: {:?}", best.match_type);
                println!("  ID: {}", best.id);
            }
            None => println!("  未找到匹配"),
        }
        println!();
    }
}
