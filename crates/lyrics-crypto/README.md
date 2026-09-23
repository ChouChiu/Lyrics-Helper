# lyrics-crypto

歌词加解密库，提供 QRC、KRC 格式加密歌词的解密与网易云音乐 eapi 接口参数加密。

## 支持的功能

- **QRC**：十六进制密文的 Triple DES/ECB 解密 + zlib 解压
- **KRC**：Base64 密文跳过魔数头后的 XOR 解密 + zlib 解压
- **网易云音乐 eapi**：`params` 参数加密（MD5 摘要 + AES-128-ECB）

QRC、KRC 均提供实现 `LyricsDecrypter` 的 `QrcDecrypter` / `KrcDecrypter`，失败时通过 `DecryptError` 区分输入无效、解压失败与编码错误；返回 `Option` 的 `decrypt_lyrics` 是它们的简写

QRC 的 XML 信封处理（`xml_utils`）自 0.5.0 起位于 `lyrics_parsers::xml_utils`

## 依赖

```toml
[dependencies]
lyrics-crypto = "0.5.0"
```

## 使用

通常不需要直接依赖此 crate，建议使用门面库 `lyrics-helper`：

```rust
use lyrics_helper::decrypt_qrc;

// 一段加密的 QRC 歌词：十六进制密文（Triple DES/ECB + zlib）
let encrypted = "61EA2D770702AE2B2B52DA9EDDEC07BB35F01431C529E8AE46B70CD635C127867\
E1AB832ABFF18CF7AABF1313EF7EF537021F03A5E957206";

match decrypt_qrc(encrypted) {
    Some(decrypted) => println!("{decrypted}"),
    None => println!("解密失败"),
}
```

上面的示例通过门面库调用，因此还需要依赖 `lyrics-helper`；只依赖 `lyrics-crypto`
时可调用 `lyrics_crypto::decrypter::qrc::decrypter::decrypt_lyrics`
（KRC 对应 `lyrics_crypto::decrypter::krc::decrypter::decrypt_lyrics`）。

## 许可证

Apache-2.0
