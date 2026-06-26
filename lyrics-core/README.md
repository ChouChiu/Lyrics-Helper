# lyrics-core

Lyricify 歌词核心库，提供歌词处理的基础类型、traits 和辅助工具。

## 概述

本 crate 是 Lyricify 歌词工具链的基础层，定义了：

- **模型**：`LyricsData`、`LineInfo`、`SyllableInfo`、`TrackMetadata` 及相关枚举
- **Traits**：`LyricsParser`、`LyricsGenerator`、`LyricsDecrypter`
- **辅助工具**：中文处理、字符串操作、数学计算、时间偏移、格式检测、优化等

## 依赖

```toml
[dependencies]
lyrics-core = "0.1"
```

## 使用

通常不需要直接依赖此 crate，建议使用门面库 `lyrics-helper`：

```toml
[dependencies]
lyrics-helper = "0.1"
```

## 许可证

Apache-2.0
