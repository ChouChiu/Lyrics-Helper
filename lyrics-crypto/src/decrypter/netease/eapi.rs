//! 网易云音乐 eapi 接口参数加密，对应 C# `EapiHelper`。
//!
//! `params` 字段的明文为 `nobody{url}use{json}md5forencrypt` 的 MD5 摘要与请求地址、
//! 请求正文拼接而成的字符串，再经 AES-128-ECB（PKCS7 填充，密钥 `e82ckenh8dichen8`）
//! 加密得到大写十六进制密文。

use std::fmt::Write as _;

use aes::Aes128;
use aes::cipher::{BlockCipherEncrypt, KeyInit};
use md5::{Digest, Md5};

/// eapi 加密密钥，对应 C# `EapiHelper.eapiKey`。
const EAPI_KEY: &[u8; 16] = b"e82ckenh8dichen8";

/// eapi 明文分隔符，对应 C# `EapiHelper.EApi` 中的字面量。
const SEPARATOR: &str = "-36cd479b6b5-";

/// 生成摘要与密文前需要剥离的地址前缀，对应 C# `EApi` 中的两次 `Replace`。
const EAPI_PREFIXES: [&str; 2] = [
    "https://interface3.music.163.com/e",
    "https://interface.music.163.com/e",
];

/// AES 分组长度（字节）。
const AES_BLOCK_SIZE: usize = 16;

/// 计算 eapi 请求的 `params` 字段值（十六进制大写）。
///
/// `data_json` 必须是请求正文的 JSON 文本，摘要与密文均基于该文本本身生成。
pub fn encrypt_params(url: &str, data_json: &str) -> String {
    let url = strip_eapi_prefix(url);
    let digest = Md5::digest(format!("nobody{url}use{data_json}md5forencrypt").as_bytes());
    let digest = hex(&digest, false);
    let mut buffer = format!("{url}{SEPARATOR}{data_json}{SEPARATOR}{digest}").into_bytes();
    pkcs7_pad(&mut buffer);

    let cipher = Aes128::new(&(*EAPI_KEY).into());
    for block in buffer.chunks_exact_mut(AES_BLOCK_SIZE) {
        // `pkcs7_pad` 保证了长度是分组长度的整数倍，每块恰好 16 字节。
        cipher.encrypt_block(block.try_into().expect("AES 分组长度为 16 字节"));
    }
    hex(&buffer, true)
}

/// 十六进制编码。
fn hex(bytes: &[u8], upper: bool) -> String {
    let mut text = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        // 写入 String 不会失败。
        let _ = if upper {
            write!(text, "{byte:02X}")
        } else {
            write!(text, "{byte:02x}")
        };
    }
    text
}

/// 剥离 eapi 接口前缀，对应 C# `EApi` 中的 `Replace`。
fn strip_eapi_prefix(url: &str) -> String {
    EAPI_PREFIXES
        .iter()
        .fold(url.to_string(), |url, prefix| url.replace(prefix, "/"))
}

/// 按 PKCS7 补足到 AES 分组长度，对应 .NET `Aes` 的默认填充方式（始终补齐）。
fn pkcs7_pad(buffer: &mut Vec<u8>) {
    let padding = AES_BLOCK_SIZE - buffer.len() % AES_BLOCK_SIZE;
    buffer.resize(buffer.len() + padding, padding as u8);
}

#[cfg(test)]
mod tests {
    use super::*;

    // 以下期望值由上游 C# `EapiHelper.EApi` 在同一地址与同一 JSON 文本下生成，
    // 用于校验前缀剥离、MD5 摘要、PKCS7 填充与十六进制大小写。

    #[test]
    fn test_encrypt_params_matches_csharp_reference() {
        let url = "https://interface3.music.163.com/eapi/song/lyric/v1";
        let json = r#"{"cp":"false","csrf_token":"","id":"423997333","kv":"0","lv":"0","rv":"0","tv":"0","yrv":"0","ytv":"0","yv":"0"}"#;
        assert_eq!(
            encrypt_params(url, json),
            "04AE33D34A93FE3EC22DA8FA305D290AB337D0FE5F36D211DE0D338CC6AA89D0\
             35DF0DF6670E9141B8FEB05134B45228856D603755651D56F4AD80DF0AE37F4E\
             325FEC7848021CACAC97B0297AA242CA41BB225BD74C9B97F4F2BA482D108F70F\
             173EDB093AC1C5B0B390F4ADA3666786894C034F63137522CE605164CFBBA7B8D\
             40BA661477B9A5EC6EA773131F4474A7422BFFFC535166212FC3FACA86691B937C\
             DDA03BF9BE46BFA91A2DE2847A3DB90146F2B8340B1A7AAF3925799EEA4C"
        );
    }

    #[test]
    fn test_encrypt_params_without_known_prefix() {
        // 地址不含可剥离前缀，且 JSON 文本含转义字符。
        let url = "https://interface.music.163.com/api/song/lyric/v1";
        let json = r#"{"id":"66842","n":"十年\"x\"\\y"}"#;
        assert_eq!(
            encrypt_params(url, json),
            "2DCD3F9164274BE9684609A30CF01C888C84C172B0F68EFE4690642DB0746EE4\
             9E393EDD74A95BA61D0CC64B4D24054E9D40620AAF9159BB43C875891B4A20153\
             71E072A9E33406D30BDDFDDD46A129C543CD723AE5311F0534DF375B59E70AC07\
             4C9F592CB92EBE4204ED6A7599C5DD34F76CDC3A0EF0590674B165B8BEF5A7060\
             39749B348C4F6C9FAC3397C2CFC50"
        );
    }
}
