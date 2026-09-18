use std::collections::HashMap;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use serde::{Deserialize, Serialize};
use tokio::sync::OnceCell;

use super::response::{LyricsResponse, MusicuResponse};
use crate::error::SearchError;
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

/// 初始化 QQ 音乐匿名 session。
///
/// `Ok(None)` 表示平台在 2xx 里没有下发 session（缺 `request` / `data` / `session`），
/// 属于「没有数据」而非失败；请求失败或平台返回非 0 业务码时返回 [`SearchError`]。
/// 两种情况都由 [`get_session`] 降级成「没有 session」，不会打断搜索与取词。
async fn init_session() -> Result<Option<(String, String, String)>, SearchError> {
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
    let response = base_api::send_json(url, &body, QQ_HEADERS).await?;
    let session_resp: SessionResponse = base_api::json(response).await?;

    // 缺 `code` 字段按上游 C# 的语义视作成功（C# 的 `int` 默认值即 0），只有非 0 才是业务失败。
    match session_resp.code {
        None | Some(0) => {}
        Some(code) => {
            return Err(SearchError::Api(format!(
                "QQ 音乐 session 初始化失败：code {code}"
            )));
        }
    }

    let Some(session) = session_resp
        .request
        .and_then(|request| request.data)
        .and_then(|data| data.session)
    else {
        return Ok(None);
    };

    Ok(Some((
        session.uid.unwrap_or_else(|| "0".to_string()),
        session.sid.unwrap_or_default(),
        session.userip.unwrap_or_default(),
    )))
}

/// 取缓存的匿名 session。
///
/// 拿不到 session 时返回 `None`：[`init_session`] 的任何失败在这里都被有意降级为
/// 「没有 session」，调用方 [`get_comm`] 随即退化成与既有行为完全相同的匿名兜底身份
/// （`uid = "0"`，无 `sid` / `userip`），因此不影响搜索与取词。
async fn get_session() -> &'static Option<(String, String, String)> {
    SESSION
        .get_or_init(|| async { init_session().await.ok().flatten() })
        .await
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
    let millis = base_api::unix_millis() as u64;
    let part1 = (rand::random::<u64>() % 20 + 1) * 18014398509481984;
    let part2 = (rand::random::<u64>() % 4194305) * 4294967296;
    let part3 = millis % 86400000;
    (part1 + part2 + part3).to_string()
}

pub(crate) async fn search(keyword: &str) -> Result<MusicuResponse, SearchError> {
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

    let response = base_api::send_json(url, &body, QQ_HEADERS).await?;
    base_api::json(response).await
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
///
/// 请求成功但平台没给歌词（缺 `request` / `data`）或解密后没有正文时，对应元素为 `Ok(None)`；
/// 网络、状态码或响应格式失败返回 [`SearchError`]。
pub async fn get_lyrics(
    song_mid: &str,
    song_id: Option<i64>,
    title: &str,
    artist: &str,
    album: &str,
    duration_ms: Option<i32>,
) -> Result<(Option<String>, Option<String>), SearchError> {
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

    let response = base_api::send_json(url, &body, QQ_HEADERS).await?;
    let result: LyricsResponse = base_api::json(response).await?;

    // 平台没返回歌词数据属于「这首没有歌词」，不是错误。
    let Some(data) = result.request.and_then(|request| request.data) else {
        return Ok((None, None));
    };

    let lyric = decrypt_qrc_lyric(
        &data.lyric,
        data.qrc_t.unwrap_or(0),
        data.lrc_t.unwrap_or(0),
    )
    .and_then(|text| extract_lyric_content(&text));
    let trans = decrypt_qrc_lyric(&data.trans, data.trans_t.unwrap_or(0), 0)
        .and_then(|text| extract_lyric_content(&text));

    Ok((lyric, trans))
}

/// QRC XML 节点与结果名的映射，对应上游 `VerbatimXmlMappingDict`。
const QQ_XML_MAPPING: [(&str, &str); 4] = [
    ("content", "orig"),
    ("contentts", "ts"),
    ("contentroma", "roma"),
    ("Lyric_1", "lyric"),
];

/// 从 QRC XML 封装中取出歌词正文，非 XML 内容原样返回。
///
/// 腾讯音乐返回的正文是 `<QrcInfos><LyricInfo><Lyric_1 LyricContent="[ti:…]…"/>` 形式的封装，
/// 需要按上游 `QQMusic.Api.GetLyricsAsync` 的方式取出 `LyricContent` 属性；
/// 翻译等内容本身是纯文本，则直接返回。
fn extract_lyric_content(raw: &str) -> Option<String> {
    // 翻译等纯文本内容占多数，先判再拷贝，避免整段歌词无谓复制一次。
    if !raw.contains("<?xml") {
        return Some(raw.to_string());
    }

    let mut text = raw.to_string();

    // 封装可能嵌套多层，逐层解包（上游同样会在解密后再次解析 `<?xml`）。
    for _ in 0..2 {
        if !text.contains("<?xml") {
            return Some(text);
        }

        let document = lyrics_crypto::decrypter::qrc::xml_utils::create(&text)?;
        let mut found = HashMap::new();
        lyrics_crypto::decrypter::qrc::xml_utils::recursion_find_element(
            &document,
            &QQ_XML_MAPPING,
            &mut found,
        );

        text = found.get("lyric")?.attribute("LyricContent")?.to_string();
    }

    Some(text)
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
