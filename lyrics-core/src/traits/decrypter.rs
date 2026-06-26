/// 歌词解密过程中可能产生的错误。
#[derive(Debug, Clone)]
pub enum DecryptError {
    /// 输入数据格式无效。
    InvalidInput,
    /// 解密操作失败（如密钥错误或数据损坏）。
    DecryptionFailed,
    /// 解压缩失败。
    DecompressionFailed,
    /// 编码格式无效（如 UTF-8 转换失败）。
    InvalidEncoding,
}

/// 歌词解密器 trait，用于解密加密的歌词内容（如 QRC、KRC）。
pub trait LyricsDecrypter {
    /// 解密字符串形式的加密歌词，返回解密后的明文。
    fn decrypt(&self, input: &str) -> Result<String, DecryptError>;

    /// 解密字节形式的加密歌词，返回解密后的字节数据。
    fn decrypt_bytes(&self, input: &[u8]) -> Result<Vec<u8>, DecryptError>;
}
