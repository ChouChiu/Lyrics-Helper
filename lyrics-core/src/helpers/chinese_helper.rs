use ferrous_opencc::config::BuiltinConfig;
use ferrous_opencc::OpenCC;
use once_cell::sync::Lazy;

static S2T_CONVERTER: Lazy<OpenCC> =
    Lazy::new(|| OpenCC::from_config(BuiltinConfig::S2t).expect("failed to load S2t config"));

static T2S_CONVERTER: Lazy<OpenCC> =
    Lazy::new(|| OpenCC::from_config(BuiltinConfig::T2s).expect("failed to load T2s config"));

/// 将繁体中文文本转换为简体中文。
pub fn to_simplified(text: &str) -> String {
    T2S_CONVERTER.convert(text)
}

/// 将简体中文文本转换为繁体中文。
pub fn to_traditional(text: &str) -> String {
    S2T_CONVERTER.convert(text)
}
