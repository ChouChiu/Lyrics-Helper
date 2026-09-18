//! HTTP 请求辅助。
//!
//! 对应 C# `BaseApi`：每个请求都单独构造请求头（`CreateRequest`），
//! 请求头不会在 Provider 之间共享或残留。

use reqwest::Client;
use serde::{Serialize, de::DeserializeOwned};
use std::sync::LazyLock;
use std::time::{SystemTime, UNIX_EPOCH};

/// 默认 User-Agent，对应 C# `BaseApi.UserAgent`。
///
/// 该值作为 HTTP 客户端的默认请求头，Provider 可在单次请求中自行覆盖。
pub const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/63.0.3239.132 Safari/537.36";

/// 网易云音乐默认 Cookie，对应 C# `BaseApi.Cookie`。
pub const COOKIE: &str = "os=pc;osver=Microsoft-Windows-10-Professional-build-16299.125-64bit;appver=2.0.3.131777;channel=netease;__remember_me=true";

static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .expect("Failed to create HTTP client")
});

/// 当前 Unix 时间戳（秒），取不到系统时间时返回 0。
pub fn unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// 当前 Unix 时间戳（毫秒），取不到系统时间时返回 0。
pub fn unix_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

/// 发送 GET 请求并将响应体反序列化为指定类型，失败时返回 `None`。
pub async fn get_json<T: DeserializeOwned>(url: &str) -> Option<T> {
    let response = HTTP_CLIENT.get(url).send().await.ok()?;
    response.json::<T>().await.ok()
}

/// 发送带自定义请求头的 GET 请求并将响应体反序列化为指定类型。
pub async fn get_json_with_headers<T: DeserializeOwned>(
    url: &str,
    headers: &[(&str, &str)],
) -> Option<T> {
    let mut request = HTTP_CLIENT.get(url);
    for (key, value) in headers {
        request = request.header(*key, *value);
    }
    let response = request.send().await.ok()?;
    response.json::<T>().await.ok()
}

/// 发送 JSON POST 请求并将响应体反序列化为指定类型。
pub async fn post_json<T: DeserializeOwned>(url: &str, body: &impl Serialize) -> Option<T> {
    let response = HTTP_CLIENT.post(url).json(body).send().await.ok()?;
    response.json::<T>().await.ok()
}

/// 发送带自定义请求头的 JSON POST 请求并将响应体反序列化为指定类型。
pub async fn post_json_with_headers<T: DeserializeOwned>(
    url: &str,
    body: &impl Serialize,
    headers: &[(&str, &str)],
) -> Option<T> {
    let mut request = HTTP_CLIENT.post(url).json(body);
    for (key, value) in headers {
        request = request.header(*key, *value);
    }
    let response = request.send().await.ok()?;
    response.json::<T>().await.ok()
}

/// 发送带自定义请求头的 JSON POST 请求，返回原始响应文本。
pub async fn post_json_raw_with_headers(
    url: &str,
    body: &impl Serialize,
    headers: &[(&str, &str)],
) -> Option<String> {
    let mut request = HTTP_CLIENT.post(url).json(body);
    for (key, value) in headers {
        request = request.header(*key, *value);
    }
    let response = request.send().await.ok()?;
    response.text().await.ok()
}

/// 发送表单 POST 请求并将响应体反序列化为指定类型。
pub async fn post_form<T: DeserializeOwned>(
    url: &str,
    form: &[(&str, &str)],
    headers: &[(&str, &str)],
) -> Option<T> {
    let mut request = HTTP_CLIENT.post(url).form(form);
    for (key, value) in headers {
        request = request.header(*key, *value);
    }
    let response = request.send().await.ok()?;
    response.json::<T>().await.ok()
}

/// 发送带自定义请求头的表单 POST 请求，返回原始响应文本。
pub async fn post_form_raw_with_headers(
    url: &str,
    form: &[(&str, &str)],
    headers: &[(&str, &str)],
) -> Option<String> {
    let mut request = HTTP_CLIENT.post(url).form(form);
    for (key, value) in headers {
        request = request.header(*key, *value);
    }
    let response = request.send().await.ok()?;
    response.text().await.ok()
}
