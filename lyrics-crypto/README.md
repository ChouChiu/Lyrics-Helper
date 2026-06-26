# lyrics-crypto

歌词解密库，提供 QRC 和 KRC 格式加密歌词的解密功能。

## 支持的解密

- **QRC**：AES/ECB 解密 + gzip 解压
- **KRC**：DES/ECB 解密 + zlib 解压

## 依赖

```toml
[dependencies]
lyrics-crypto = "0.1"
```

## 使用

通常不需要直接依赖此 crate，建议使用门面库 `lyrics-helper`：

```rust
use lyrics_helper::decrypt_qrc;

let decrypted = decrypt_qrc(&encrypted_bytes).unwrap();
```

## 许可证

Apache-2.0
