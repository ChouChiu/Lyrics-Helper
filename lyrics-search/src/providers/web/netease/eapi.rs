//! 网易云音乐 eapi 接口客户端，对应 C# `EapiHelper`。
//!
//! `params` 的构造（MD5 摘要与 AES-128-ECB 加密）由
//! [`lyrics_crypto::decrypter::netease::eapi`] 提供，本模块负责组装请求地址、请求头与
//! 请求正文。

use std::sync::LazyLock;

use rand::Rng;
use regex::Regex;
use serde_json::{Value, json};

use crate::providers::web::base_api;

/// eapi 专用 User-Agent，对应 C# `EapiHelper.userAgent`。
const USER_AGENT: &str = "Mozilla/5.0 (Linux; Android 9; PCT-AL10) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/70.0.3538.64 HuaweiBrowser/10.0.3.311 Mobile Safari/537.36";

const REFERER: &str = "https://music.163.com/";

/// eapi 请求地址重写规则，对应 C# `Regex.Replace(url, @"\w*api", "eapi")`。
static EAPI_URL_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\w*api").unwrap());

/// 以 eapi 协议向 `url` 发送 POST 请求，返回响应文本。
///
/// `data` 必须为 JSON 对象，函数会按 eapi 要求追加 `header` 字段。
pub(super) async fn post(url: &str, data: Value) -> Option<String> {
    let Value::Object(mut data) = data else {
        return None;
    };

    let header = request_header();
    let cookie = header
        .as_object()?
        .iter()
        .map(|(key, value)| format!("{key}={}", value.as_str().unwrap_or_default()))
        .collect::<Vec<_>>()
        .join("; ");
    data.insert(
        "header".to_string(),
        Value::String(serde_json::to_string(&header).ok()?),
    );

    let params = lyrics_crypto::decrypter::netease::eapi::encrypt_params(
        url,
        &serde_json::to_string(&data).ok()?,
    );

    base_api::post_form_raw_with_headers(
        &EAPI_URL_RE.replace(url, "eapi"),
        &[("params", params.as_str())],
        &[
            ("User-Agent", USER_AGENT),
            ("Referer", REFERER),
            ("Cookie", cookie.as_str()),
        ],
    )
    .await
}

/// 构造 eapi 请求的公共参数与同名 Cookie，对应 C# `EapiHelper.PostAsync` 中的 `header`。
fn request_header() -> Value {
    json!({
        "__csrf": "",
        "appver": "8.0.0",
        "buildver": base_api::unix_seconds().to_string(),
        "channel": "",
        "deviceId": "",
        "mobilename": "",
        "resolution": "1920x1080",
        "os": "android",
        "osver": "",
        "requestId": format!("{}_{:04}", base_api::unix_millis(), rand::rng().random_range(0..1000u32)),
        "versioncode": "140",
        "MUSIC_U": "",
    })
}
