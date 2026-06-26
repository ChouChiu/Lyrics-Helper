use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use serde::{Deserialize, Serialize};
use tokio::sync::OnceCell;

use super::response::{LyricsResponse, MusicuResponse};
use crate::providers::web::base_api;

const QQ_HEADERS: &[(&str, &str)] = &[
    ("User-Agent", "okhttp/3.14.9"),
    ("Cookie", "tmeLoginType=-1;"),
    ("Content-Type", "application/json"),
];

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Comm {
    ct: i32,
    cv: String,
    v: String,
    os_ver: String,
    phonetype: String,
    rom: String,
    #[serde(rename = "tmeAppID")]
    tme_app_id: String,
    nettype: String,
    udid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    uid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    userip: Option<String>,
}

#[derive(Serialize)]
struct SearchParam {
    search_id: String,
    remoteplace: String,
    query: String,
    search_type: i32,
    num_per_page: i32,
    page_num: i32,
    highlight: i32,
    nqc_flag: i32,
    page_id: i32,
    grp: i32,
}

#[derive(Serialize)]
struct RequestBody {
    method: String,
    module: String,
    param: SearchParam,
}

#[derive(Serialize)]
struct MusicuBody {
    comm: Comm,
    request: RequestBody,
}

#[derive(Serialize)]
struct SessionParam {
    caller: i32,
    uid: String,
    vkey: i32,
}

#[derive(Serialize)]
struct SessionBody {
    comm: Comm,
    request: SessionRequestBody,
}

#[derive(Serialize)]
struct SessionRequestBody {
    method: String,
    module: String,
    param: SessionParam,
}

#[derive(Debug, Clone, Deserialize)]
struct SessionResponse {
    code: Option<i32>,
    request: Option<SessionReqData>,
}

#[derive(Debug, Clone, Deserialize)]
struct SessionReqData {
    #[serde(rename = "code")]
    _code: Option<i32>,
    data: Option<SessionData>,
}

#[derive(Debug, Clone, Deserialize)]
struct SessionData {
    session: Option<SessionInfo>,
}

#[derive(Debug, Clone, Deserialize)]
struct SessionInfo {
    uid: Option<String>,
    sid: Option<String>,
    userip: Option<String>,
}

static SESSION: OnceCell<Option<(String, String, String)>> = OnceCell::const_new();

async fn init_session() -> Option<(String, String, String)> {
    let comm = Comm {
        ct: 11,
        cv: "1003006".to_string(),
        v: "1003006".to_string(),
        os_ver: "15".to_string(),
        phonetype: "24122RKC7C".to_string(),
        rom: "Redmi/miro/miro:15/AE3A.240806.005/OS2.0.105.0.VOMCNXM:user/release-keys".to_string(),
        tme_app_id: "qqmusiclight".to_string(),
        nettype: "NETWORK_WIFI".to_string(),
        udid: "0".to_string(),
        uid: None,
        sid: None,
        userip: None,
    };

    let body = SessionBody {
        comm,
        request: SessionRequestBody {
            method: "GetSession".to_string(),
            module: "music.getSession.session".to_string(),
            param: SessionParam {
                caller: 0,
                uid: "0".to_string(),
                vkey: 0,
            },
        },
    };

    let url = "https://u.y.qq.com/cgi-bin/musicu.fcg";
    let resp = base_api::post_json_raw_with_headers(url, &body, QQ_HEADERS).await?;
    let session_resp: SessionResponse = serde_json::from_str(&resp).ok()?;

    if session_resp.code? != 0 {
        eprintln!("  [QQMusic] Session init failed: code {}", session_resp.code.unwrap_or(-1));
        return None;
    }

    let data = session_resp.request?.data?;
    let session = data.session?;
    Some((
        session.uid.unwrap_or_else(|| "0".to_string()),
        session.sid.unwrap_or_default(),
        session.userip.unwrap_or_default(),
    ))
}

async fn get_session() -> &'static Option<(String, String, String)> {
    SESSION.get_or_init(init_session).await
}

async fn get_comm() -> Comm {
    let (uid, sid, userip) = get_session()
        .await
        .as_ref()
        .map(|(u, s, ip)| (Some(u.clone()), Some(s.clone()), Some(ip.clone())))
        .unwrap_or((Some("0".to_string()), None, None));

    Comm {
        ct: 11,
        cv: "1003006".to_string(),
        v: "1003006".to_string(),
        os_ver: "15".to_string(),
        phonetype: "24122RKC7C".to_string(),
        rom: "Redmi/miro/miro:15/AE3A.240806.005/OS2.0.105.0.VOMCNXM:user/release-keys".to_string(),
        tme_app_id: "qqmusiclight".to_string(),
        nettype: "NETWORK_WIFI".to_string(),
        udid: "0".to_string(),
        uid,
        sid,
        userip,
    }
}

fn generate_search_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    let part1 = (rand::random::<u64>() % 20 + 1) * 18014398509481984;
    let part2 = (rand::random::<u64>() % 4194305) * 4294967296;
    let part3 = millis % 86400000;
    (part1 + part2 + part3).to_string()
}

pub(crate) async fn search(keyword: &str) -> Option<MusicuResponse> {
    let url = "https://u.y.qq.com/cgi-bin/musicu.fcg";
    let body = MusicuBody {
        comm: get_comm().await,
        request: RequestBody {
            method: "DoSearchForQQMusicLite".to_string(),
            module: "music.search.SearchCgiService".to_string(),
            param: SearchParam {
                search_id: generate_search_id(),
                remoteplace: "search.android.keyboard".to_string(),
                query: keyword.to_string(),
                search_type: 0,
                num_per_page: 20,
                page_num: 1,
                highlight: 0,
                nqc_flag: 0,
                page_id: 1,
                grp: 1,
            },
        },
    };

    let resp = base_api::post_json_raw_with_headers(url, &body, QQ_HEADERS).await;
    match resp {
        Some(text) => {
            let result: Option<MusicuResponse> = serde_json::from_str(&text).ok();
            if result.is_none() {
                eprintln!("  [QQMusic] Failed to parse response: {}", &text[..text.len().min(500)]);
            }
            result
        }
        None => {
            eprintln!("  [QQMusic] HTTP request failed");
            None
        }
    }
}

#[derive(Serialize)]
struct LyricsParam {
    #[serde(rename = "songMID")]
    song_mid: String,
    #[serde(rename = "songID")]
    song_id: i64,
    #[serde(rename = "songName")]
    song_name: String,
    #[serde(rename = "singerName")]
    singer_name: String,
    #[serde(rename = "albumName")]
    album_name: String,
    interval: i32,
    #[serde(rename = "lrc_t")]
    lrc_t: i32,
    #[serde(rename = "qrc_t")]
    qrc_t: i32,
    #[serde(rename = "trans_t")]
    trans_t: i32,
    #[serde(rename = "roma_t")]
    roma_t: i32,
    crypt: i32,
    ct: i32,
    cv: i32,
    qrc: i32,
    roma: i32,
    trans: i32,
    #[serde(rename = "type")]
    lyric_type: i32,
}

#[derive(Serialize)]
struct LyricsRequestBody {
    method: String,
    module: String,
    param: LyricsParam,
}

#[derive(Serialize)]
struct LyricsBody {
    comm: Comm,
    request: LyricsRequestBody,
}

/// 获取 QQ 音乐歌词，返回 `(原文歌词, 翻译歌词)` 元组。
pub async fn get_lyrics(
    song_mid: &str,
    song_id: Option<i64>,
    title: &str,
    artist: &str,
    album: &str,
    duration_ms: Option<i32>,
) -> Option<(Option<String>, Option<String>)> {
    let url = "https://u.y.qq.com/cgi-bin/musicu.fcg";
    let interval = duration_ms.unwrap_or(0) / 1000;

    let body = LyricsBody {
        comm: get_comm().await,
        request: LyricsRequestBody {
            method: "GetPlayLyricInfo".to_string(),
            module: "music.musichallSong.PlayLyricInfo".to_string(),
            param: LyricsParam {
                song_mid: song_mid.to_string(),
                song_id: song_id.unwrap_or(0),
                song_name: BASE64.encode(title.as_bytes()),
                singer_name: BASE64.encode(artist.as_bytes()),
                album_name: BASE64.encode(album.as_bytes()),
                interval,
                lrc_t: 0,
                qrc_t: 0,
                trans_t: 0,
                roma_t: 0,
                crypt: 1,
                ct: 19,
                cv: 2111,
                qrc: 1,
                roma: 1,
                trans: 1,
                lyric_type: 0,
            },
        },
    };

    let resp = base_api::post_json_raw_with_headers(url, &body, QQ_HEADERS).await?;
    let result: Option<LyricsResponse> = serde_json::from_str(&resp).ok();
    if result.is_none() {
        eprintln!("  [QQMusic] Failed to parse lyrics response: {}", &resp[..resp.len().min(500)]);
    }

    let data = result?.request?.data?;

    let lyric = decrypt_qrc_lyric(&data.lyric, data.qrc_t.unwrap_or(0), data.lrc_t.unwrap_or(0));
    let trans = decrypt_qrc_lyric(&data.trans, data.trans_t.unwrap_or(0), 0);

    Some((lyric, trans))
}

fn decrypt_qrc_lyric(encrypted: &Option<String>, qrc_t: i32, lrc_t: i32) -> Option<String> {
    let text = encrypted.as_ref()?;
    if text.is_empty() {
        return None;
    }
    let t = if qrc_t != 0 { qrc_t } else { lrc_t };
    if t == 0 {
        return None;
    }
    lyrics_crypto::decrypter::qrc::decrypter::decrypt_lyrics(text)
}
