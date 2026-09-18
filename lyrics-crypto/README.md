# lyrics-crypto

歌词加解密库，提供 QRC、KRC 格式加密歌词的解密与网易云音乐 eapi 接口参数加密。

## 支持的功能

- **QRC**：AES/ECB 解密 + gzip 解压
- **KRC**：DES/ECB 解密 + zlib 解压
- **网易云音乐 eapi**：`params` 参数加密（MD5 摘要 + AES-128-ECB）

## 依赖

```toml
[dependencies]
lyrics-crypto = "0.3"
```

## 使用

通常不需要直接依赖此 crate，建议使用门面库 `lyrics-helper`：

```rust
use lyrics_helper::decrypt_qrc;

let decrypted = decrypt_qrc(&encrypted_bytes).unwrap();
```

## 许可证

Apache-2.0
