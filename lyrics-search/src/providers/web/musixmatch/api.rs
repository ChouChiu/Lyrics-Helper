use super::api_options::ApiOptions;
use super::response::{Track, TrackResponse};
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use std::sync::LazyLock;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};

/// 请求重试次数（对应 C# `RequestRetryCount`）。
const REQUEST_RETRY_COUNT: usize = 5;

/// 结果重试次数（对应 C# `ResultRetryCount`）。
const RESULT_RETRY_COUNT: usize = 5;

/// 两次请求之间的最小间隔（对应 C# `GetResponseAsync` 中的 250ms）。
const MIN_REQUEST_INTERVAL: Duration = Duration::from_millis(250);

static OPTIONS: LazyLock<RwLock<ApiOptions>> = LazyLock::new(|| RwLock::new(ApiOptions::new()));
static CLIENT: LazyLock<RwLock<Client>> =
    LazyLock::new(|| RwLock::new(create_client(&ApiOptions::new())));
static USER_TOKEN: LazyLock<RwLock<Option<String>>> = LazyLock::new(|| RwLock::new(None));
static TOKEN_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));
/// 串行化所有请求，并记录上一次请求的完成时间用于最小间隔控制。
static REQUEST_LOCK: LazyLock<Mutex<Option<Instant>>> = LazyLock::new(|| Mutex::new(None));

/// Musixmatch 请求错误，对应 C# 的 `RequestCaptchaException` 与 `HttpRequestException`。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MusixmatchError {
    /// 命中 401 且 hint 为 captcha（对应 C# `RequestCaptchaException`）。
    Captcha,
    /// 其他请求错误（对应 C# `HttpRequestException`）。
    Request(String),
}

/// 返回当前配置的副本。
pub async fn options() -> ApiOptions {
    OPTIONS.read().await.clone()
}

/// 覆盖全局配置（对应 C# `new Api(options => ...)` 中的配置动作）。
///
/// 配置会先经过校验（对应 C# `ValidateOptions`），非法配置会 panic。
pub async fn set_options(mut options: ApiOptions) {
    validate_options(&mut options);
    let client = create_client(&options);
    *OPTIONS.write().await = options;
    *CLIENT.write().await = client;
}

/// 设置 UserToken（例如之前缓存的 Token），不可用的 Token 会被忽略。
pub async fn set_user_token(token: &str) {
    *USER_TOKEN.write().await = if is_usable_token(token) {
        Some(token.to_string())
    } else {
        None
    };
}

/// 获取当前的 UserToken（例如用于缓存）。
pub async fn get_user_token() -> Option<String> {
    USER_TOKEN.read().await.clone()
}

/// 获取 Musixmatch 用户令牌（用于后续 API 调用）。
pub async fn get_token() -> Option<String> {
    let response = request_token().await.ok().flatten()?;
    response["message"]["body"]["user_token"]
        .as_str()
        .map(str::to_string)
}

/// 搜索曲目列表（对应 C# `SearchTracksAsync`）。
///
/// 请求失败时返回 `None`；请求成功但没有通过相关性校验时返回空列表。
pub async fn search_tracks(
    keyword: Option<&str>,
    track: Option<&str>,
    artist: Option<&str>,
    duration_secs: Option<i32>,
) -> Option<Vec<Track>> {
    let mut parameters = vec![
        "page_size=10".to_string(),
        "page=1".to_string(),
        "s_track_rating=desc".to_string(),
    ];
    add_parameter(&mut parameters, "q", keyword);
    add_parameter(&mut parameters, "q_track", track);
    add_parameter(&mut parameters, "q_artist", artist);
    if let Some(duration) = duration_secs.filter(|duration| *duration > 0) {
        parameters.push(format!("q_duration={duration}"));
    }

    let request = format!("track.search?{}", parameters.join("&"));
    for attempt in 0..RESULT_RETRY_COUNT {
        let response = send_api_request(&request).await.ok()?;
        if let Some(list) = body_of(&response)["track_list"].as_array() {
            let results: Vec<Track> = list
                .iter()
                .filter_map(|item| Track::deserialize(&item["track"]).ok())
                .collect();
            if !results.is_empty() && has_related_result(&results, keyword, track, artist) {
                return Some(results);
            }
        }

        if attempt + 1 < RESULT_RETRY_COUNT {
            delay_before_result_retry(attempt).await;
        }
    }

    Some(Vec::new())
}

/// 获取曲目（对应 C# `GetTrack`），返回搜索结果的第一条。
pub async fn search_track(
    q_track: &str,
    q_artist: &str,
    user_token: &str,
) -> Option<TrackResponse> {
    set_user_token(user_token).await;
    let track = search_tracks(None, Some(q_track), Some(q_artist), None)
        .await?
        .into_iter()
        .next()?;
    Some(TrackResponse::from_track(track))
}

/// 获取完整歌词的原始响应（对应 C# `GetFullLyricsRaw(string trackId)`）。
///
/// `expected_vanity_id` 非空时会额外校验曲目的 vanity ID。
pub async fn get_full_lyrics_raw(
    track_id: &str,
    expected_vanity_id: Option<&str>,
) -> Option<String> {
    let response = get_full_lyrics_value(track_id, expected_vanity_id).await?;
    serde_json::to_string(&response).ok()
}

/// 获取完整歌词的宏调用响应，供内部提取字段使用（避免序列化后再解析一次）。
async fn get_full_lyrics_value(track_id: &str, expected_vanity_id: Option<&str>) -> Option<Value> {
    let id: i64 = track_id.parse().ok()?;

    for attempt in 0..RESULT_RETRY_COUNT {
        let response = get_lyrics_response(id).await.ok()?;
        if let Some(track) = get_matched_track(&response) {
            if track.track_id == id
                && vanity_matches(expected_vanity_id, track.commontrack_vanity_id.as_deref())
            {
                return Some(response);
            }
        }

        if attempt + 1 < RESULT_RETRY_COUNT {
            delay_before_result_retry(attempt).await;
        }
    }

    None
}

/// 获取 Musixmatch 非同步歌词文本。
pub async fn get_lyrics(track_id: i64, user_token: &str) -> Option<String> {
    set_user_token(user_token).await;
    let response = get_full_lyrics_value(&track_id.to_string(), None).await?;
    unsynced_lyrics_from(&response)
}

/// 获取 Musixmatch 同步歌词（LRC 格式）。
pub async fn get_synced_lyrics(track_id: i64, user_token: &str) -> Option<String> {
    set_user_token(user_token).await;
    let response = get_full_lyrics_value(&track_id.to_string(), None).await?;
    synced_lyrics_from(&response)
}

/// 请求 UserToken（对应 C# `RequestTokenAsync`）；响应体为空时返回 `None`。
async fn request_token() -> Result<Option<Value>, MusixmatchError> {
    let options = OPTIONS.read().await.clone();
    let url = format!(
        "{}token.get?user_language=en&app_id={}&t={}",
        options.api_base_url,
        urlencoding::encode(&options.app_id),
        urlencoding::encode(&options.request_id_factory.create()),
    );

    let response = get_response(&url).await?;
    if !response.is_success() {
        return Err(http_error(response.status));
    }
    if response.content.trim().is_empty() {
        return Ok(None);
    }

    let json = parse_json(&response.content)?;
    let header = &json["message"]["header"];
    let status_code = header["status_code"].as_i64();
    let hint = header["hint"].as_str();
    if status_code == Some(401) && is_hint(hint, "captcha") {
        return Err(MusixmatchError::Captcha);
    }
    if status_code != Some(200) {
        return Err(MusixmatchError::Request(api_status_message(
            status_code,
            hint,
        )));
    }

    Ok(Some(json))
}

/// 发送 API 请求并解析 JSON（对应 C# `SendApiRequestAsync`）。
async fn send_api_request(request: &str) -> Result<Value, MusixmatchError> {
    let mut last_error = None;

    for attempt in 0..REQUEST_RETRY_COUNT {
        match send_api_request_once(request).await {
            Ok(json) => return Ok(json),
            Err(error @ MusixmatchError::Captcha) => return Err(error),
            Err(error) => last_error = Some(error),
        }

        if attempt + 1 < REQUEST_RETRY_COUNT {
            delay_before_request_retry(attempt).await;
        }
    }

    Err(MusixmatchError::Request(format!(
        "Musixmatch request failed after all retries. ({})",
        last_error
            .map(|error| error_message(&error).to_string())
            .unwrap_or_else(|| "unknown error".to_string())
    )))
}

/// 发送单次 API 请求（对应 C# `SendApiRequestAsync` 循环体）。
async fn send_api_request_once(request: &str) -> Result<Value, MusixmatchError> {
    let token = ensure_user_token().await?;
    let options = OPTIONS.read().await.clone();
    let separator = if request.contains('?') { '&' } else { '?' };
    let url = format!(
        "{}{}{}usertoken={}&format=json&app_id={}&t={}",
        options.api_base_url,
        request,
        separator,
        urlencoding::encode(&token),
        urlencoding::encode(&options.app_id),
        urlencoding::encode(&options.request_id_factory.create()),
    );

    let response = get_response(&url).await?;
    if response.content.trim().is_empty() {
        return Err(http_error(response.status));
    }

    let json = parse_json(&response.content)?;
    let header = &json["message"]["header"];
    let status_code = header["status_code"].as_i64();
    let hint = header["hint"].as_str();

    if status_code == Some(404) {
        return Ok(json);
    }
    if !response.is_success() {
        return Err(http_error(response.status));
    }
    if status_code == Some(200) {
        return Ok(json);
    }

    if status_code == Some(401) && is_hint(hint, "renew") {
        invalidate_token().await;
    } else if status_code == Some(401) && is_hint(hint, "captcha") {
        return Err(MusixmatchError::Captcha);
    }

    Err(MusixmatchError::Request(api_status_message(
        status_code,
        hint,
    )))
}

/// 确保存在可用的 UserToken（对应 C# `EnsureUserTokenAsync`）。
async fn ensure_user_token() -> Result<String, MusixmatchError> {
    if let Some(token) = usable_token().await {
        return Ok(token);
    }

    let _guard = TOKEN_LOCK.lock().await;
    if let Some(token) = usable_token().await {
        return Ok(token);
    }

    let response = request_token().await?;
    let token = response
        .as_ref()
        .and_then(|json| json["message"]["body"]["user_token"].as_str())
        .map(str::to_string)
        .filter(|token| is_usable_token(token));

    match token {
        Some(token) => {
            *USER_TOKEN.write().await = Some(token.clone());
            Ok(token)
        }
        None => Err(MusixmatchError::Request(
            "Musixmatch token request failed.".to_string(),
        )),
    }
}

/// 读取当前可用的 UserToken。
async fn usable_token() -> Option<String> {
    let token = USER_TOKEN.read().await.clone();
    token.filter(|token| is_usable_token(token))
}

/// 清除当前 UserToken（对应 C# `InvalidateToken`）。
async fn invalidate_token() {
    *USER_TOKEN.write().await = None;
}

/// 发送请求并读取响应内容（对应 C# `GetResponseAsync`），同一时刻只允许一个请求。
async fn get_response(url: &str) -> Result<RawResponse, MusixmatchError> {
    let mut last_request = REQUEST_LOCK.lock().await;

    if let Some(previous) = *last_request {
        let elapsed = previous.elapsed();
        if elapsed < MIN_REQUEST_INTERVAL {
            tokio::time::sleep(MIN_REQUEST_INTERVAL - elapsed).await;
        }
    }

    let options = OPTIONS.read().await.clone();
    let client = CLIENT.read().await.clone();
    let mut request = client.get(url);
    if let Some(user_agent) = options
        .user_agent
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        request = request.header("User-Agent", user_agent);
    }
    if let Some(cookie) = options
        .cookie
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        request = request.header("Cookie", cookie);
    }

    let response = request
        .send()
        .await
        .map_err(|error| MusixmatchError::Request(error.to_string()))?;
    let status = response.status();
    let content = response
        .text()
        .await
        .map_err(|error| MusixmatchError::Request(error.to_string()))?;
    *last_request = Some(Instant::now());

    Ok(RawResponse {
        status: status.as_u16(),
        content,
    })
}

/// 请求歌词宏调用响应（对应 C# `GetLyricsResponseAsync`）。
async fn get_lyrics_response(track_id: i64) -> Result<Value, MusixmatchError> {
    send_api_request(&format!(
        "macro.subtitles.get?namespace=lyrics_richsynched\
         &optional_calls=track.richsync\
         &subtitle_format=lrc\
         &track_id={track_id}\
         &f_subtitle_length_max_deviation=40"
    ))
    .await
}

/// 从宏调用响应中取出匹配到的曲目（对应 C# `GetMatchedTrack`）。
fn get_matched_track(response: &Value) -> Option<Track> {
    let track = &response["message"]["body"]["macro_calls"]["matcher.track.get"]["message"]["body"]
        ["track"];
    Track::deserialize(track).ok()
}

/// 从宏调用响应中提取非同步歌词文本（`track.lyrics.get`）。
fn unsynced_lyrics_from(response: &Value) -> Option<String> {
    response["message"]["body"]["macro_calls"]["track.lyrics.get"]["message"]["body"]["lyrics"]
        ["lyrics_body"]
        .as_str()
        .map(str::to_string)
}

/// 从宏调用响应中提取同步歌词文本（`track.subtitles.get`，LRC 格式）。
fn synced_lyrics_from(response: &Value) -> Option<String> {
    response["message"]["body"]["macro_calls"]["track.subtitles.get"]["message"]["body"]
        ["subtitle_list"][0]["subtitle"]["subtitle_body"]
        .as_str()
        .map(str::to_string)
}

/// 对应 C# `GetBody`：`response.message.body`。
fn body_of(response: &Value) -> &Value {
    &response["message"]["body"]
}

fn parse_json(content: &str) -> Result<Value, MusixmatchError> {
    serde_json::from_str(content).map_err(|error| MusixmatchError::Request(error.to_string()))
}

/// 对应 C# `AddParameter`：非空白值才会加入查询参数。
fn add_parameter(parameters: &mut Vec<String>, name: &str, value: Option<&str>) {
    if let Some(value) = value.filter(|value| !value.trim().is_empty()) {
        parameters.push(format!("{name}={}", urlencoding::encode(value)));
    }
}

/// 对应 C# `HasRelatedResult`。
fn has_related_result(
    results: &[Track],
    keyword: Option<&str>,
    title: Option<&str>,
    artist: Option<&str>,
) -> bool {
    let keyword_tokens = tokenize(keyword);
    let title_tokens = tokenize(title);
    let artist_tokens = tokenize(artist);
    if keyword_tokens.is_empty() && title_tokens.is_empty() && artist_tokens.is_empty() {
        return true;
    }

    results.iter().any(|result| {
        let actual_title = result.track_name.to_lowercase();
        let actual_artists = result.artist_name.to_lowercase();
        if !title_tokens.is_empty()
            && !title_tokens
                .iter()
                .all(|token| actual_title.contains(token))
        {
            return false;
        }
        if !artist_tokens.is_empty()
            && !artist_tokens
                .iter()
                .any(|token| actual_artists.contains(token))
        {
            return false;
        }

        if keyword_tokens.is_empty() {
            return true;
        }

        let actual = format!("{actual_title} {actual_artists}");
        let required_matches =
            std::cmp::max(1, (keyword_tokens.len() as f64 * 0.6).ceil() as usize);
        keyword_tokens
            .iter()
            .filter(|token| actual.contains(*token))
            .count()
            >= required_matches
    })
}

/// 对应 C# `Tokenize`。
fn tokenize(value: Option<&str>) -> Vec<String> {
    value
        .unwrap_or_default()
        .to_lowercase()
        .split([' ', '-', '_', '/', ',', '.', '(', ')', '[', ']', '&'])
        .filter(|token| token.chars().count() > 1)
        .map(str::to_string)
        .collect()
}

/// 对应 C# `IsUsableToken`。
fn is_usable_token(token: &str) -> bool {
    !token.trim().is_empty() && token != "null" && token.chars().any(|c| c != '0')
}

/// 对应 C# `NormalizeVanity`。
fn normalize_vanity(value: Option<&str>) -> String {
    let value = value.unwrap_or_default();
    let decoded = urlencoding::decode(value)
        .map(|decoded| decoded.into_owned())
        .unwrap_or_else(|_| value.to_string());
    decoded
        .trim()
        .trim_start_matches('/')
        .trim_end_matches('/')
        .to_string()
}

/// 校验曲目 vanity ID 是否与预期一致，预期为空时视为匹配。
fn vanity_matches(expected: Option<&str>, actual: Option<&str>) -> bool {
    match expected.filter(|expected| !expected.trim().is_empty()) {
        Some(expected) => {
            normalize_vanity(Some(expected)).eq_ignore_ascii_case(&normalize_vanity(actual))
        }
        None => true,
    }
}

fn is_hint(hint: Option<&str>, expected: &str) -> bool {
    hint.is_some_and(|hint| hint.eq_ignore_ascii_case(expected))
}

fn api_status_message(status_code: Option<i64>, hint: Option<&str>) -> String {
    let status = status_code
        .map(|status| status.to_string())
        .unwrap_or_else(|| "unknown".to_string());
    match hint.filter(|hint| !hint.trim().is_empty()) {
        Some(hint) => format!("Musixmatch returned API status {status} ({hint})."),
        None => format!("Musixmatch returned API status {status}."),
    }
}

fn error_message(error: &MusixmatchError) -> &str {
    match error {
        MusixmatchError::Captcha => "Hit 401 error with Captcha hint.",
        MusixmatchError::Request(message) => message,
    }
}

/// 请求重试前的等待（对应 C# `DelayBeforeRequestRetryAsync`）。
async fn delay_before_request_retry(attempt: usize) {
    let delay = std::cmp::min(500 * (1 << attempt), 2000);
    tokio::time::sleep(Duration::from_millis(delay)).await;
}

/// 结果重试前的等待（对应 C# `DelayBeforeResultRetryAsync`）。
async fn delay_before_result_retry(attempt: usize) {
    let delay = std::cmp::min(200 * (attempt as u64 + 1), 800);
    tokio::time::sleep(Duration::from_millis(delay)).await;
}

/// 对应 C# `CreateClient`。
fn create_client(options: &ApiOptions) -> Client {
    Client::builder()
        .timeout(options.timeout)
        .build()
        .expect("Failed to create HTTP client")
}

/// 对应 C# `ValidateOptions`。
fn validate_options(options: &mut ApiOptions) {
    if options.api_base_url.trim().is_empty() {
        panic!("Musixmatch API base URL is required.");
    }
    if !options.api_base_url.ends_with('/') {
        options.api_base_url.push('/');
    }
    if options.app_id.trim().is_empty() {
        panic!("Musixmatch app ID is required.");
    }
    if options.timeout.is_zero() {
        panic!("Musixmatch request timeout must be greater than zero.");
    }
}

/// 请求结果（对应 C# `GetResponseAsync` 返回的元组）。
struct RawResponse {
    status: u16,
    content: String,
}

impl RawResponse {
    /// HTTP 状态码是否为 2xx。
    fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }
}

/// 构造「返回了非成功状态码」的错误。
fn http_error(status: u16) -> MusixmatchError {
    MusixmatchError::Request(format!("Musixmatch returned HTTP {}.", status))
}
