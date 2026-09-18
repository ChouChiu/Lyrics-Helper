# lyrics-generators

歌词生成器库，支持将 `LyricsData` 模型导出为多种格式字符串。

## 支持的格式

- LRC
- QRC
- KRC
- YRC
- Lyricify Syllable
- Lyricify Lines

## 依赖

```toml
[dependencies]
lyrics-generators = "0.3"
```

## 使用

通常不需要直接依赖此 crate，建议使用门面库 `lyrics-helper`：

```rust
use lyrics_helper::{parse, generate_string, LyricsRawTypes, LyricsTypes};

// QRC / KRC / YRC 生成器只输出带音节的行，输入必须是逐字（音节级）歌词
let qrc_input = "[0,1500]Hello(0,500) (500,500)World(1000,500)";
let data = parse(qrc_input, LyricsRawTypes::Qrc).unwrap();

let qrc = generate_string(&data, LyricsTypes::Qrc).unwrap();
println!("{qrc}");
```

注意方向性：QRC、KRC、YRC 与 Lyricify Syllable 的生成器只输出带音节的行，逐行（行级）
歌词无法生成这些格式——用 LRC 的解析结果去生成 QRC 只会得到空字符串，此时
`generate_string` 返回 `None`。Lyricify Lines 需要每行同时有起止时间，普通 LRC 没有
结束时间，同样会得到 `None`；只有 LRC 生成器接受任何带开始时间的行。反向转换需要先用
QRC / KRC / YRC / TTML 等含音节的格式解析。

上面的示例通过门面库调用，因此还需要依赖 `lyrics-helper`；只依赖
`lyrics-generators` 时可调用 `lyrics_generators::generate_string`。

## 许可证

Apache-2.0
