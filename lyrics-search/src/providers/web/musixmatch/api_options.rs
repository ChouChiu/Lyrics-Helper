use crate::providers::web::base_api;
use rand::RngExt;
use std::time::Duration;

/// Musixmatch 请求 `t` 参数的生成方式，对应 C# `ApiOptions.RequestIdFactory`。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestIdFactory {
    /// Android API：32 位十六进制随机串（对应 C# `Guid.NewGuid().ToString("N")`）。
    Guid,
    /// 桌面 API：毫秒时间戳字符串（对应 C# `DateTimeOffset.UtcNow.ToUnixTimeMilliseconds()`）。
    UnixMilliseconds,
}

impl RequestIdFactory {
    /// 生成一次请求的 `t` 参数。
    pub fn create(self) -> String {
        match self {
            Self::Guid => random_hex_id(),
            Self::UnixMilliseconds => base_api::unix_millis().to_string(),
        }
    }
}

fn random_hex_id() -> String {
    use std::fmt::Write;

    let mut rng = rand::rng();
    let mut id = String::with_capacity(32);
    for _ in 0..16 {
        let _ = write!(id, "{:02x}", rng.random_range(0u8..=255));
    }
    id
}

/// Musixmatch API 请求配置，对应 C# `ApiOptions`。
#[derive(Debug, Clone)]
pub struct ApiOptions {
    /// API 基础地址（必须以 `/` 结尾）。
    pub api_base_url: String,
    /// API `app_id`。
    pub app_id: String,
    /// 请求使用的 User-Agent，为空时不携带该请求头。
    pub user_agent: Option<String>,
    /// 请求使用的 Cookie，为空时不携带该请求头。
    pub cookie: Option<String>,
    /// 单次请求超时时间。
    pub timeout: Duration,
    /// 请求 `t` 参数的生成方式。
    pub request_id_factory: RequestIdFactory,
}

impl Default for ApiOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl ApiOptions {
    /// 创建默认配置（等价于 C# 构造函数调用 `UseAndroid`）。
    pub fn new() -> Self {
        Self::android()
    }

    /// Android API 的默认配置。
    pub fn android() -> Self {
        Self {
            api_base_url: "https://apic.musixmatch.com/ws/1.1/".to_string(),
            app_id: "android-player-v1.0".to_string(),
            user_agent: Some("Dalvik/2.1.0 (Linux; U; Android 13)".to_string()),
            cookie: Some("AWSELB=0; AWSELBCORS=0".to_string()),
            timeout: Duration::from_secs(4),
            request_id_factory: RequestIdFactory::Guid,
        }
    }

    /// 桌面 API 的默认配置。
    pub fn desktop() -> Self {
        Self {
            api_base_url: "https://apic-desktop.musixmatch.com/ws/1.1/".to_string(),
            app_id: "web-desktop-app-v1.0".to_string(),
            user_agent: Some(base_api::USER_AGENT.to_string()),
            cookie: Some("AWSELB=0; AWSELBCORS=0".to_string()),
            timeout: Duration::from_secs(4),
            request_id_factory: RequestIdFactory::UnixMilliseconds,
        }
    }

    /// 使用 Android API 配置。
    pub fn use_android(&mut self) {
        *self = Self::android();
    }

    /// 使用桌面 API 配置。
    pub fn use_desktop(&mut self) {
        *self = Self::desktop();
    }

    /// Android API 的别名（对应 C# `UseMobile`）。
    pub fn use_mobile(&mut self) {
        self.use_android();
    }
}
