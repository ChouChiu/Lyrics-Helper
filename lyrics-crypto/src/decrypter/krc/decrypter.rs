use base64::Engine;
use flate2::read::ZlibDecoder;
use lyrics_core::traits::decrypter::{DecryptError, LyricsDecrypter};
use std::io::Read;

const KRC_KEY: [u8; 16] = [
    0x40, 0x47, 0x61, 0x77, 0x5e, 0x32, 0x74, 0x47,
    0x51, 0x36, 0x31, 0x2d, 0xce, 0xd2, 0x6e, 0x69,
];

pub struct KrcDecrypter;

impl LyricsDecrypter for KrcDecrypter {
    fn decrypt(&self, input: &str) -> Result<String, DecryptError> {
        decrypt_lyrics(input).ok_or(DecryptError::DecryptionFailed)
    }

    fn decrypt_bytes(&self, input: &[u8]) -> Result<Vec<u8>, DecryptError> {
        if input.len() <= 4 {
            return Err(DecryptError::InvalidInput);
        }
        let encrypted = &input[4..];
        let mut decrypted = Vec::with_capacity(encrypted.len());
        for (i, &byte) in encrypted.iter().enumerate() {
            decrypted.push(byte ^ KRC_KEY[i % KRC_KEY.len()]);
        }
        let mut decoder = ZlibDecoder::new(&decrypted[..]);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).map_err(|_| DecryptError::DecompressionFailed)?;
        if decompressed.is_empty() {
            return Err(DecryptError::DecompressionFailed);
        }
        // KRC format has an extra leading byte before the actual UTF-8 content
        Ok(decompressed[1..].to_vec())
    }
}

pub fn decrypt_lyrics(encrypted_lyrics: &str) -> Option<String> {
    // Base64 decode
    let encrypted_bytes = base64::engine::general_purpose::STANDARD
        .decode(encrypted_lyrics)
        .ok()?;

    // Skip first 4 bytes (magic header)
    if encrypted_bytes.len() <= 4 {
        return None;
    }
    let encrypted = &encrypted_bytes[4..];

    // XOR decrypt
    let mut decrypted = Vec::with_capacity(encrypted.len());
    for (i, &byte) in encrypted.iter().enumerate() {
        decrypted.push(byte ^ KRC_KEY[i % KRC_KEY.len()]);
    }

    // Zlib decompress
    let mut decoder = ZlibDecoder::new(&decrypted[..]);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed).ok()?;

    // Remove first character and convert to string
    if decompressed.is_empty() {
        return None;
    }

    let result = &decompressed[1..];
    String::from_utf8(result.to_vec()).ok()
}

pub fn decrypt_lyrics_from_file(file_content: &[u8]) -> Option<String> {
    // Skip first 4 bytes (magic header)
    if file_content.len() <= 4 {
        return None;
    }
    let encrypted = &file_content[4..];

    // XOR decrypt
    let mut decrypted = Vec::with_capacity(encrypted.len());
    for (i, &byte) in encrypted.iter().enumerate() {
        decrypted.push(byte ^ KRC_KEY[i % KRC_KEY.len()]);
    }

    // Zlib decompress
    let mut decoder = ZlibDecoder::new(&decrypted[..]);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed).ok()?;

    // Remove first character and convert to string
    if decompressed.is_empty() {
        return None;
    }

    let result = &decompressed[1..];
    String::from_utf8(result.to_vec()).ok()
}
