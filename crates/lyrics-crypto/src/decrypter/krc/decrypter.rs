use crate::decrypter::{inflate, into_text};
use base64::Engine;
use lyrics_core::traits::decrypter::{DecryptError, LyricsDecrypter};

const KRC_KEY: [u8; 16] = [
    0x40, 0x47, 0x61, 0x77, 0x5e, 0x32, 0x74, 0x47, 0x51, 0x36, 0x31, 0x2d, 0xce, 0xd2, 0x6e, 0x69,
];

/// KRC 格式歌词解密器，实现 `LyricsDecrypter` trait。
///
/// [`decrypt`](LyricsDecrypter::decrypt) 接收 Base64 文本，
/// [`decrypt_bytes`](LyricsDecrypter::decrypt_bytes) 接收 `.krc` 文件的原始字节。
pub struct KrcDecrypter;

/// KRC 文件头的魔数长度（`krc1`）。
const MAGIC_LEN: usize = 4;

/// KRC 密文的公共解密流程：跳过魔数 → XOR → zlib 解压 → 去掉可能存在的 UTF-8 BOM。
///
/// 酷狗以 `charset=utf8` 下发 KRC，正文带 BOM。上游 C# 的写法是
/// `Encoding.UTF8.GetString(...)[1..]`——先解码成字符串再丢掉第一个**字符**，
/// 在有 BOM 时正好等于去掉 BOM；这里改为只在真的匹配到 BOM 时才去掉。
fn decrypt_payload(content: &[u8]) -> Result<Vec<u8>, DecryptError> {
    let body = content
        .get(MAGIC_LEN..)
        .filter(|body| !body.is_empty())
        .ok_or(DecryptError::InvalidInput)?;

    let decrypted: Vec<u8> = body
        .iter()
        .zip(KRC_KEY.iter().cycle())
        .map(|(byte, key)| byte ^ key)
        .collect();

    let decompressed = inflate(&decrypted)?;
    if decompressed.is_empty() {
        return Err(DecryptError::DecompressionFailed);
    }
    Ok(decompressed)
}

impl LyricsDecrypter for KrcDecrypter {
    fn decrypt(&self, input: &str) -> Result<String, DecryptError> {
        let encrypted = base64::engine::general_purpose::STANDARD
            .decode(input)
            .map_err(|_| DecryptError::InvalidInput)?;
        into_text(decrypt_payload(&encrypted)?)
    }

    fn decrypt_bytes(&self, input: &[u8]) -> Result<Vec<u8>, DecryptError> {
        decrypt_payload(input)
    }
}

/// 解密 KRC 加密歌词字符串（Base64 编码），返回解密后的明文歌词。
///
/// 解密流程：Base64 解码 → 跳过 4 字节魔数头 → XOR 解密 → zlib 解压 → 去除可选的 BOM → UTF-8 解码。
/// 需要区分失败原因时使用 [`KrcDecrypter`]。
pub fn decrypt_lyrics(encrypted_lyrics: &str) -> Option<String> {
    KrcDecrypter.decrypt(encrypted_lyrics).ok()
}

/// 从 KRC 文件的原始字节内容解密歌词，返回解密后的明文歌词。
///
/// 与 [`decrypt_lyrics`] 不同，此函数直接接收文件字节（非 Base64 编码）。
pub fn decrypt_lyrics_from_file(file_content: &[u8]) -> Option<String> {
    decrypt_payload(file_content).and_then(into_text).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::Compression;
    use flate2::write::ZlibEncoder;
    use std::io::Write;

    const LYRICS: &str = "[id:$00000000]\n[0,1000]<0,1000,0>Hello";

    /// 按 KRC 的格式封装一段正文：BOM（可选）→ zlib 压缩 → XOR → 补上 `krc1` 魔数。
    fn encode(body: &str, with_bom: bool) -> Vec<u8> {
        let mut plain = Vec::new();
        if with_bom {
            plain.extend_from_slice(b"\xEF\xBB\xBF");
        }
        plain.extend_from_slice(body.as_bytes());

        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&plain).unwrap();
        let compressed = encoder.finish().unwrap();

        let mut out = b"krc1".to_vec();
        out.extend(
            compressed
                .iter()
                .zip(KRC_KEY.iter().cycle())
                .map(|(byte, key)| byte ^ key),
        );
        out
    }

    fn encode_base64(body: &str, with_bom: bool) -> String {
        base64::engine::general_purpose::STANDARD.encode(encode(body, with_bom))
    }

    /// 酷狗以 `charset=utf8` 下发 KRC，正文带 UTF-8 BOM：BOM 要去掉，正文一个字节都不能少。
    #[test]
    fn strips_utf8_bom_from_body() {
        assert_eq!(
            decrypt_lyrics(&encode_base64(LYRICS, true)).as_deref(),
            Some(LYRICS)
        );
        assert_eq!(
            decrypt_lyrics_from_file(&encode(LYRICS, true)).as_deref(),
            Some(LYRICS)
        );
    }

    /// 没有 BOM 时正文首字符同样要保留。
    ///
    /// 上游 C# 是 `Encoding.UTF8.GetString(...)[1..]`，即无条件丢掉第一个字符；
    /// 那只有在有 BOM 时才恰好正确，这里有意不跟随该行为。
    #[test]
    fn keeps_first_character_without_bom() {
        assert_eq!(
            decrypt_lyrics(&encode_base64(LYRICS, false)).as_deref(),
            Some(LYRICS)
        );
    }

    /// `LyricsDecrypter` 走的是同一条流程。
    #[test]
    fn decrypter_trait_matches_free_functions() {
        let encoded = encode(LYRICS, true);
        assert_eq!(
            KrcDecrypter.decrypt_bytes(&encoded).unwrap(),
            LYRICS.as_bytes()
        );
        assert_eq!(
            KrcDecrypter
                .decrypt(&base64::engine::general_purpose::STANDARD.encode(&encoded))
                .unwrap(),
            LYRICS
        );
    }

    /// 只有魔数、没有正文的输入不是可解密内容。
    #[test]
    fn rejects_truncated_input() {
        assert_eq!(decrypt_lyrics_from_file(b"krc1"), None);
        assert_eq!(
            KrcDecrypter.decrypt_bytes(b"krc1"),
            Err(DecryptError::InvalidInput)
        );
    }
}
