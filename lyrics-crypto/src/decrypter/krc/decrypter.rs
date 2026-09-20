use base64::Engine;
use flate2::read::ZlibDecoder;
use lyrics_core::traits::decrypter::{DecryptError, LyricsDecrypter};
use std::io::Read;

const KRC_KEY: [u8; 16] = [
    0x40, 0x47, 0x61, 0x77, 0x5e, 0x32, 0x74, 0x47, 0x51, 0x36, 0x31, 0x2d, 0xce, 0xd2, 0x6e, 0x69,
];

/// KRC 格式歌词解密器，实现 `LyricsDecrypter` trait。
pub struct KrcDecrypter;

/// KRC 文件头的魔数长度（`krc1`）。
const MAGIC_LEN: usize = 4;

/// KRC 密文的公共解密流程：跳过魔数 → XOR → Zlib 解压 → 去掉正文前的一个字节。
///
/// 输入过短、解压失败或解压结果为空时返回 `None`。
fn decrypt_payload(content: &[u8]) -> Option<Vec<u8>> {
    if content.len() <= MAGIC_LEN {
        return None;
    }

    let decrypted: Vec<u8> = content[MAGIC_LEN..]
        .iter()
        .zip(KRC_KEY.iter().cycle())
        .map(|(byte, key)| byte ^ key)
        .collect();

    let mut decompressed = Vec::new();
    ZlibDecoder::new(&decrypted[..])
        .read_to_end(&mut decompressed)
        .ok()?;

    // 解压结果的首字节不属于 UTF-8 正文。
    if decompressed.is_empty() {
        return None;
    }
    decompressed.remove(0);

    Some(decompressed)
}

impl LyricsDecrypter for KrcDecrypter {
    fn decrypt(&self, input: &str) -> Result<String, DecryptError> {
        decrypt_lyrics(input).ok_or(DecryptError::DecryptionFailed)
    }

    fn decrypt_bytes(&self, input: &[u8]) -> Result<Vec<u8>, DecryptError> {
        if input.len() <= MAGIC_LEN {
            return Err(DecryptError::InvalidInput);
        }
        decrypt_payload(input).ok_or(DecryptError::DecompressionFailed)
    }
}

/// 解密 KRC 加密歌词字符串（Base64 编码），返回解密后的明文歌词。
///
/// 解密流程：Base64 解码 → 跳过 4 字节魔数头 → XOR 解密 → Zlib 解压 → 去除首字节 → UTF-8 解码。
pub fn decrypt_lyrics(encrypted_lyrics: &str) -> Option<String> {
    let encrypted = base64::engine::general_purpose::STANDARD
        .decode(encrypted_lyrics)
        .ok()?;

    decrypt_lyrics_from_file(&encrypted)
}

/// 从 KRC 文件的原始字节内容解密歌词，返回解密后的明文歌词。
///
/// 与 [`decrypt_lyrics`] 不同，此函数直接接收文件字节（非 Base64 编码）。
pub fn decrypt_lyrics_from_file(file_content: &[u8]) -> Option<String> {
    String::from_utf8(decrypt_payload(file_content)?).ok()
}
