#[derive(Debug, Clone)]
pub enum DecryptError {
    InvalidInput,
    DecryptionFailed,
    DecompressionFailed,
    InvalidEncoding,
}

pub trait LyricsDecrypter {
    fn decrypt(&self, input: &str) -> Result<String, DecryptError>;
    fn decrypt_bytes(&self, input: &[u8]) -> Result<Vec<u8>, DecryptError>;
}
