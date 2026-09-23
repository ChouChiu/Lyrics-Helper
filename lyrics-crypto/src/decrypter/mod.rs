//! 解密器模块，包含各平台加密歌词的解密实现。

pub mod krc;
pub mod netease;
pub mod qrc;

use flate2::read::ZlibDecoder;
use lyrics_core::traits::decrypter::DecryptError;
use std::io::Read;

/// QRC 与 KRC 共用的收尾流程：zlib 解压，再去掉正文前可能存在的 UTF-8 BOM。
///
/// 只在真的匹配到 `EF BB BF` 时才去掉：按字节丢掉首字节会把 BOM 截成 `BB BF`，
/// 整段正文随即不是合法 UTF-8；而正文没有 BOM 时无条件丢弃又会吃掉第一个字符。
fn inflate(compressed: &[u8]) -> Result<Vec<u8>, DecryptError> {
    let mut decompressed = Vec::new();
    ZlibDecoder::new(compressed)
        .read_to_end(&mut decompressed)
        .map_err(|_| DecryptError::DecompressionFailed)?;

    if decompressed.starts_with(b"\xEF\xBB\xBF") {
        decompressed.drain(..3);
    }

    Ok(decompressed)
}

/// 把解密结果解码为 UTF-8 文本。
fn into_text(bytes: Vec<u8>) -> Result<String, DecryptError> {
    String::from_utf8(bytes).map_err(|_| DecryptError::InvalidEncoding)
}
