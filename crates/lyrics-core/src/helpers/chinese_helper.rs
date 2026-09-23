use ferrous_opencc::OpenCC;
use ferrous_opencc::config::BuiltinConfig;
use std::sync::LazyLock;

static S2T_CONVERTER: LazyLock<OpenCC> =
    LazyLock::new(|| OpenCC::from_config(BuiltinConfig::S2t).expect("failed to load S2t config"));

static T2S_CONVERTER: LazyLock<OpenCC> =
    LazyLock::new(|| OpenCC::from_config(BuiltinConfig::T2s).expect("failed to load T2s config"));

/// 将繁体中文文本转换为简体中文。
pub fn to_simplified(text: &str) -> String {
    T2S_CONVERTER.convert(text)
}

/// 将简体中文文本转换为繁体中文。
pub fn to_traditional(text: &str) -> String {
    S2T_CONVERTER.convert(text)
}

/// 强制转换为简体：转换后替换 OpenCC 未覆盖的 4 个字符，再转换一次。
///
/// 对应 C# `ChineseHelper.ToSC(this string text, bool force = true)`。
pub fn to_simplified_forced(text: &str) -> String {
    let converted = to_simplified(text)
        .replace('藉', "借")
        .replace('咀', "嘴")
        .replace('昇', "升")
        .replace('髒', "脏");
    to_simplified(&converted)
}
