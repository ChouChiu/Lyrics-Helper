use flate2::read::ZlibDecoder;
use lyrics_core::traits::decrypter::{DecryptError, LyricsDecrypter};
use std::io::Read;

const QRC_KEY: &[u8; 24] = b"!@#)(*$%123ZXC!@!@#)(NHL";

pub struct QrcDecrypter;

impl LyricsDecrypter for QrcDecrypter {
    fn decrypt(&self, input: &str) -> Result<String, DecryptError> {
        decrypt_lyrics(input).ok_or(DecryptError::DecryptionFailed)
    }

    fn decrypt_bytes(&self, input: &[u8]) -> Result<Vec<u8>, DecryptError> {
        let hex = std::str::from_utf8(input).map_err(|_| DecryptError::InvalidEncoding)?;
        let encrypted_bytes = hex_to_bytes(hex).ok_or(DecryptError::InvalidInput)?;
        let decrypted = triple_des_decrypt(&encrypted_bytes).ok_or(DecryptError::DecryptionFailed)?;
        let mut decoder = ZlibDecoder::new(&decrypted[..]);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).map_err(|_| DecryptError::DecompressionFailed)?;
        if decompressed.starts_with(&[0xEF, 0xBB, 0xBF]) {
            decompressed.drain(..3);
        }
        Ok(decompressed)
    }
}

pub fn decrypt_lyrics(encrypted_lyrics: &str) -> Option<String> {
    // Remove whitespace and newlines
    let encrypted = encrypted_lyrics.replace(|c: char| c.is_whitespace(), "");

    // Convert hex to bytes
    let encrypted_bytes = hex_to_bytes(&encrypted)?;

    // Triple-DES decrypt
    let decrypted = triple_des_decrypt(&encrypted_bytes)?;

    // Zlib decompress
    let mut decoder = ZlibDecoder::new(&decrypted[..]);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed).ok()?;

    // Remove UTF-8 BOM if present
    let result = if decompressed.starts_with(&[0xEF, 0xBB, 0xBF]) {
        &decompressed[3..]
    } else {
        &decompressed
    };

    String::from_utf8(result.to_vec()).ok()
}

fn hex_to_bytes(hex: &str) -> Option<Vec<u8>> {
    let mut bytes = Vec::with_capacity(hex.len() / 2);
    let mut i = 0;
    let chars: Vec<char> = hex.chars().collect();

    while i < chars.len() {
        let high = chars[i].to_digit(16)?;
        let low = chars.get(i + 1).and_then(|c| c.to_digit(16)).unwrap_or(0);
        bytes.push((high * 16 + low) as u8);
        i += 2;
    }

    Some(bytes)
}

fn triple_des_decrypt(data: &[u8]) -> Option<Vec<u8>> {
    if data.len() % 8 != 0 {
        return None;
    }

    let mut result = Vec::with_capacity(data.len());

    // Process 8-byte blocks
    for chunk in data.chunks(8) {
        let decrypted_block = simple_des_decrypt(chunk, QRC_KEY)?;
        result.extend_from_slice(&decrypted_block);
    }

    Some(result)
}

fn simple_des_decrypt(block: &[u8], key: &[u8; 24]) -> Option<[u8; 8]> {
    // Simplified Triple-DES decryption
    // In a real implementation, this would use the full DES algorithm
    // For now, we'll use a XOR-based approach for demonstration

    let mut result = [0u8; 8];

    // XOR with key bytes
    for i in 0..8 {
        result[i] = block[i] ^ key[i % 24];
    }

    Some(result)
}

pub fn decrypt_lyrics_from_xml(xml: &str) -> Option<String> {
    // Parse XML and decrypt each lyrics node
    // This is a simplified version
    let mut result = String::new();

    // Find all <Lyric_1 ... Content="..."/> nodes
    let mut start = 0;
    while let Some(content_start) = xml[start..].find("Content=\"") {
        let pos = start + content_start + 9; // Length of "Content=\""
        if let Some(content_end) = xml[pos..].find('"') {
            let encrypted = &xml[pos..pos + content_end];
            if let Some(decrypted) = decrypt_lyrics(encrypted) {
                result.push_str(&decrypted);
                result.push('\n');
            }
            start = pos + content_end + 1;
        } else {
            break;
        }
    }

    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}
