use std::fmt;

/// 歌词解密过程中可能产生的错误。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecryptError {
    /// 输入数据格式无效（如 Base64/十六进制解码失败、长度不足）。
    InvalidInput,
    /// 解压缩失败。
    DecompressionFailed,
    /// 编码格式无效（如 UTF-8 转换失败）。
    InvalidEncoding,
}

impl fmt::Display for DecryptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidInput => "输入数据格式无效",
            Self::DecompressionFailed => "解压缩失败",
            Self::InvalidEncoding => "解密结果不是合法的 UTF-8",
        })
    }
}

impl std::error::Error for DecryptError {}

/// 歌词解密器 trait，用于解密加密的歌词内容（如 QRC、KRC）。
pub trait LyricsDecrypter {
    /// 解密字符串形式的加密歌词，返回解密后的明文。
    fn decrypt(&self, input: &str) -> Result<String, DecryptError>;

    /// 解密字节形式的加密歌词，返回解密后的字节数据。
    fn decrypt_bytes(&self, input: &[u8]) -> Result<Vec<u8>, DecryptError>;
}
