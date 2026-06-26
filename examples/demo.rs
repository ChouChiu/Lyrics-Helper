use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        return;
    }

    match args[1].as_str() {
        "parse" => {
            if args.len() < 4 {
                eprintln!("Usage: demo parse <file> <format>");
                return;
            }
            let file_path = &args[2];
            let format = &args[3];
            cmd_parse(file_path, format);
        }
        "generate" => {
            if args.len() < 5 {
                eprintln!("Usage: demo generate <file> <from_format> <to_format>");
                return;
            }
            let file_path = &args[2];
            let from_format = &args[3];
            let to_format = &args[4];
            cmd_generate(file_path, from_format, to_format);
        }
        "detect" => {
            if args.len() < 3 {
                eprintln!("Usage: demo detect <file>");
                return;
            }
            let file_path = &args[2];
            cmd_detect(file_path);
        }
        "decrypt-qrc" => {
            if args.len() < 3 {
                eprintln!("Usage: demo decrypt-qrc <file>");
                return;
            }
            let file_path = &args[2];
            cmd_decrypt_qrc(file_path);
        }
        "decrypt-krc" => {
            if args.len() < 3 {
                eprintln!("Usage: demo decrypt-krc <file>");
                return;
            }
            let file_path = &args[2];
            cmd_decrypt_krc(file_path);
        }
        "parsers-demo" => {
            cmd_parsers_demo();
        }
        "generators-demo" => {
            cmd_generators_demo();
        }
        _ => {
            print_usage();
        }
    }
}

fn print_usage() {
    println!("Lyrics Helper - Demo");
    println!();
    println!("Usage: demo <subcommand>");
    println!();
    println!("Subcommands:");
    println!("  parse <file> <format>         Parse a lyrics file");
    println!("  generate <file> <from> <to>   Convert lyrics format");
    println!("  detect <file>                 Auto-detect lyrics format");
    println!("  decrypt-qrc <file>            Decrypt a QRC file");
    println!("  decrypt-krc <file>            Decrypt a KRC file");
    println!("  parsers-demo                  Run all parser demos");
    println!("  generators-demo               Run generator demos");
    println!();
    println!("Formats: lrc, qrc, yrc, krc, ttml, spotify, musixmatch, lyricify-syllable, lyricify-lines");
}

fn parse_format(format: &str) -> lyrics_helper::LyricsRawTypes {
    match format.to_lowercase().as_str() {
        "lrc" => lyrics_helper::LyricsRawTypes::Lrc,
        "qrc" => lyrics_helper::LyricsRawTypes::Qrc,
        "yrc" => lyrics_helper::LyricsRawTypes::Yrc,
        "krc" => lyrics_helper::LyricsRawTypes::Krc,
        "ttml" => lyrics_helper::LyricsRawTypes::Ttml,
        "spotify" => lyrics_helper::LyricsRawTypes::Spotify,
        "musixmatch" => lyrics_helper::LyricsRawTypes::Musixmatch,
        "lyricify-syllable" => lyrics_helper::LyricsRawTypes::LyricifySyllable,
        "lyricify-lines" => lyrics_helper::LyricsRawTypes::LyricifyLines,
        _ => lyrics_helper::LyricsRawTypes::Unknown,
    }
}

fn parse_gen_format(format: &str) -> lyrics_helper::LyricsTypes {
    match format.to_lowercase().as_str() {
        "lrc" => lyrics_helper::LyricsTypes::Lrc,
        "qrc" => lyrics_helper::LyricsTypes::Qrc,
        "yrc" => lyrics_helper::LyricsTypes::Yrc,
        "krc" => lyrics_helper::LyricsTypes::Krc,
        "lyricify-syllable" => lyrics_helper::LyricsTypes::LyricifySyllable,
        "lyricify-lines" => lyrics_helper::LyricsTypes::LyricifyLines,
        _ => lyrics_helper::LyricsTypes::Unknown,
    }
}

fn cmd_parse(file_path: &str, format: &str) {
    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            return;
        }
    };

    let raw_type = parse_format(format);
    if raw_type == lyrics_helper::LyricsRawTypes::Unknown {
        eprintln!("Unknown format: {}", format);
        return;
    }

    match lyrics_helper::parse(&content, raw_type) {
        Some(data) => {
            println!("=== Parse Result ===");
            if let Some(ref file) = data.file {
                println!("Type: {:?}", file.lyrics_type);
                println!("Sync: {:?}", file.sync_types);
            }
            if let Some(ref lines) = data.lines {
                println!("Lines: {}", lines.len());
                if let Some(first) = lines.first() {
                    println!("First line time: {:?}", first.start_time());
                    println!("First line text: {}", first.text_from_any());
                }
                if let Some(last) = lines.last() {
                    println!("Last line time: {:?}", last.start_time());
                    println!("Last line text: {}", last.text_from_any());
                }
            }
            if let Some(ref meta) = data.track_metadata {
                println!("Title: {:?}", meta.title);
                println!("Artist: {:?}", meta.artist);
                println!("Album: {:?}", meta.album);
            }
            if let Some(ref writers) = data.writers {
                println!("Writers: {:?}", writers);
            }
        }
        None => {
            eprintln!("Failed to parse lyrics");
        }
    }
}

fn cmd_generate(file_path: &str, from_format: &str, to_format: &str) {
    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            return;
        }
    };

    let raw_type = parse_format(from_format);
    let gen_type = parse_gen_format(to_format);

    if raw_type == lyrics_helper::LyricsRawTypes::Unknown {
        eprintln!("Unknown input format: {}", from_format);
        return;
    }
    if gen_type == lyrics_helper::LyricsTypes::Unknown {
        eprintln!("Unknown output format: {}", to_format);
        return;
    }

    match lyrics_helper::parse(&content, raw_type) {
        Some(data) => {
            match lyrics_helper::generate_string(&data, gen_type) {
                Some(output) => println!("{}", output),
                None => eprintln!("Failed to generate output"),
            }
        }
        None => {
            eprintln!("Failed to parse input lyrics");
        }
    }
}

fn cmd_detect(file_path: &str) {
    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            return;
        }
    };

    let detected = lyrics_helper::helpers::type_helper::get_lyrics_types(&content);
    println!("Detected format: {:?}", detected);
}

fn cmd_decrypt_qrc(file_path: &str) {
    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            return;
        }
    };

    match lyrics_helper::decrypter::decrypter::qrc::decrypter::decrypt_lyrics(&content) {
        Some(decrypted) => println!("{}", decrypted),
        None => eprintln!("Failed to decrypt QRC"),
    }
}

fn cmd_decrypt_krc(file_path: &str) {
    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            return;
        }
    };

    match lyrics_helper::decrypter::decrypter::krc::decrypter::decrypt_lyrics(&content) {
        Some(decrypted) => println!("{}", decrypted),
        None => eprintln!("Failed to decrypt KRC"),
    }
}

fn cmd_parsers_demo() {
    let test_data_dir = "lyrics-helper/tests/test_data";

    let demos = vec![
        ("LrcDemo.txt", "lrc"),
        ("QrcDemo.txt", "qrc"),
        ("YrcDemo.txt", "yrc"),
        ("KrcDemo.txt", "krc"),
        ("LyricifySyllableDemo.txt", "lyricify-syllable"),
        ("LyricifyLinesDemo.txt", "lyricify-lines"),
        ("SpotifyDemo.txt", "spotify"),
        ("SpotifySyllableDemo.txt", "spotify"),
        ("SpotifyUnsyncedDemo.txt", "spotify"),
        ("MusixmatchDemo.txt", "musixmatch"),
    ];

    for (file, format) in demos {
        let path = format!("{}/{}", test_data_dir, file);
        if !std::path::Path::new(&path).exists() {
            println!("Skipping {} (file not found)", file);
            continue;
        }

        println!("=== Parsing {} ({}) ===", file, format);
        cmd_parse(&path, format);
        println!();
    }
}

fn cmd_generators_demo() {
    let path = "lyrics-helper/tests/test_data/LyricifySyllableDemo.txt";
    if !std::path::Path::new(path).exists() {
        eprintln!("Demo file not found: {}", path);
        return;
    }

    let content = fs::read_to_string(path).unwrap();
    let data = lyrics_helper::parse(&content, lyrics_helper::LyricsRawTypes::LyricifySyllable);

    if let Some(data) = data {
        let formats = vec![
            ("LyricifySyllable", lyrics_helper::LyricsTypes::LyricifySyllable),
            ("LyricifyLines", lyrics_helper::LyricsTypes::LyricifyLines),
            ("LRC", lyrics_helper::LyricsTypes::Lrc),
            ("QRC", lyrics_helper::LyricsTypes::Qrc),
            ("KRC", lyrics_helper::LyricsTypes::Krc),
            ("YRC", lyrics_helper::LyricsTypes::Yrc),
        ];

        for (name, format) in formats {
            println!("=== Generate {} ===", name);
            match lyrics_helper::generate_string(&data, format) {
                Some(output) => println!("{}", output),
                None => println!("(no output)"),
            }
            println!();
        }
    } else {
        eprintln!("Failed to parse demo file");
    }
}
