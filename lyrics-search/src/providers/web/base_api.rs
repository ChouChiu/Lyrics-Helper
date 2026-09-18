//! HTTP 请求辅助。
//!
//! 对应 C# `BaseApi`：每个请求都单独构造请求头（`CreateRequest`），
//! 请求头不会在 Provider 之间共享或残留。
//!
//! 请求分两步：先由 [`send`] / [`send_json`] / [`send_form`] 发送并拿回原始响应，
//! 再由 [`json`] / [`text`] 解码响应体。所有失败都返回 [`SearchError`]，
//! 不再像 0.2 那样把 reqwest / serde 的错误压成 `None`。

use reqwest::{Client, RequestBuilder};
use serde::{Serialize, de::DeserializeOwned};
use std::sync::LazyLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::SearchError;

// 公开签名里出现的 reqwest 类型一并再导出，调用方不必自己依赖 reqwest。
pub use reqwest::{Method, Response, StatusCode};

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

/// 发送不带请求体的请求，返回原始响应。
pub async fn send(
    method: Method,
    url: &str,
    headers: &[(&str, &str)],
) -> Result<Response, SearchError> {
    send_request(method, url, headers, |request| request).await
}

/// 发送 JSON POST 请求，返回原始响应。
pub async fn send_json(
    url: &str,
    body: &impl Serialize,
    headers: &[(&str, &str)],
) -> Result<Response, SearchError> {
    send_request(Method::POST, url, headers, |request| request.json(body)).await
}

/// 发送表单 POST 请求，返回原始响应。
pub async fn send_form(
    url: &str,
    form: &[(&str, &str)],
    headers: &[(&str, &str)],
) -> Result<Response, SearchError> {
    send_request(Method::POST, url, headers, |request| request.form(form)).await
}

/// 将响应体反序列化为指定类型。
///
/// 状态码非 2xx 时返回 [`SearchError::Status`]，不会去解析错误页正文。
pub async fn json<T: DeserializeOwned>(response: Response) -> Result<T, SearchError> {
    ensure_success(&response)?;
    response.json::<T>().await.map_err(SearchError::Http)
}

/// 读取响应体文本。
///
/// 状态码非 2xx 时返回 [`SearchError::Status`]；部分平台（如 QQ 音乐）会在 2xx 里
/// 返回带业务错误码的 JSON，这类正文仍会原样交给调用方。
pub async fn text(response: Response) -> Result<String, SearchError> {
    ensure_success(&response)?;
    response.text().await.map_err(SearchError::Http)
}

/// 构造请求、附加请求头并发送。
async fn send_request(
    method: Method,
    url: &str,
    headers: &[(&str, &str)],
    body: impl FnOnce(RequestBuilder) -> RequestBuilder,
) -> Result<Response, SearchError> {
    let mut request = HTTP_CLIENT.request(method, url);
    for (key, value) in headers {
        request = request.header(*key, *value);
    }
    body(request).send().await.map_err(SearchError::Http)
}

/// 状态码非 2xx 时构造 [`SearchError::Status`]。
fn ensure_success(response: &Response) -> Result<(), SearchError> {
    let status = response.status();
    if status.is_success() {
        Ok(())
    } else {
        Err(SearchError::Status(status.as_u16()))
    }
}
