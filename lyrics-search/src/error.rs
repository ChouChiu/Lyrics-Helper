//! 搜索与歌词获取的 crate 级错误类型。
//!
//! 各平台 Provider 与 [`Searcher`](crate::searchers::searcher::Searcher) 一律返回本类型，
//! 调用方据此区分「没有这首歌」「被 captcha」「被限流」「网络不通」等失败原因，
//! 而不必像 0.2 那样只能从一个 `None` 里猜。

use std::fmt;

/// 搜索或歌词获取过程中的错误。
#[derive(Debug)]
pub enum SearchError {
    /// 网络请求失败：连接、超时、TLS，或响应体无法按目标类型解码（`reqwest::Error`）。
    ///
    /// 可用 `reqwest::Error::is_timeout` / `is_connect` / `is_decode` 进一步区分。
    Http(reqwest::Error),
    /// 响应文本不是合法 JSON（对响应文本调用 `serde_json` 时失败）。
    Json(serde_json::Error),
    /// 服务端返回了非成功状态码（4xx/5xx），例如 429 限流、401 凭证失效。
    Status(u16),
    /// 平台在 HTTP 成功响应里返回了业务层错误（如 QQ 音乐的 `code != 0`）。
    Api(String),
    /// 命中验证码风控（Musixmatch 401 + `captcha` hint），需要人工处理或更换配置。
    Captcha,
    /// 响应内容不符合预期格式（如 base64 / UTF-8 解码失败）。
    Payload(String),
    /// 传入的配置非法。
    InvalidConfig(String),
}

impl fmt::Display for SearchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Http(error) => write!(f, "HTTP 请求失败：{error}"),
            Self::Json(error) => write!(f, "响应 JSON 解析失败：{error}"),
            Self::Status(status) => write!(f, "服务端返回 HTTP {status}"),
            Self::Api(message) => write!(f, "平台返回错误：{message}"),
            Self::Captcha => write!(f, "命中验证码风控，需要人工处理"),
            Self::Payload(message) => write!(f, "响应内容解析失败：{message}"),
            Self::InvalidConfig(message) => write!(f, "配置非法：{message}"),
        }
    }
}

impl std::error::Error for SearchError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Http(error) => Some(error),
            Self::Json(error) => Some(error),
            _ => None,
        }
    }
}

impl From<reqwest::Error> for SearchError {
    fn from(error: reqwest::Error) -> Self {
        Self::Http(error)
    }
}

impl From<serde_json::Error> for SearchError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}
